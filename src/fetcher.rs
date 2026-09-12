use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use ureq::Agent;
use url::Url;

const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 500;
/// 单响应体积上限,防止超大/异常响应撑爆内存
const MAX_BODY_BYTES: usize = 10_000_000;
/// 单请求整体超时(含重定向与 body 读取),防止慢源占住 worker 拖尾整轮
const OVERALL_TIMEOUT_SECS: u64 = 60;

/// Real browser User-Agent to avoid 403 from sites that block bot UA strings.
const BROWSER_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

pub fn new_agent() -> Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(30))
        .timeout_write(Duration::from_secs(10))
        .timeout(Duration::from_secs(OVERALL_TIMEOUT_SECS))
        .user_agent(BROWSER_UA)
        .build()
}

type NowFn = Arc<dyn Fn() -> Instant + Send + Sync>;
type SleepFn = Arc<dyn Fn(Duration) + Send + Sync>;

/// Process-wide per-host request spacing. Callers consult this before each HTTP
/// GET so same-host feeds honor `host_min_interval_seconds` under rayon.
pub struct HostIntervalGate {
    last: Mutex<HashMap<String, Instant>>,
    now: NowFn,
    sleep: SleepFn,
}

impl HostIntervalGate {
    pub fn new() -> Self {
        Self {
            last: Mutex::new(HashMap::new()),
            now: Arc::new(Instant::now),
            sleep: Arc::new(std::thread::sleep),
        }
    }

    /// Test/injection hook: control time and record sleeps without wall-clock waits.
    #[cfg(test)]
    pub fn with_clock(now: NowFn, sleep: SleepFn) -> Self {
        Self {
            last: Mutex::new(HashMap::new()),
            now,
            sleep,
        }
    }

    fn record_now(&self, url: &str) {
        let Some(host) = host_of(url) else {
            return;
        };
        let mut map = self.last.lock().expect("HostIntervalGate poisoned");
        map.insert(host, (self.now)());
    }

    /// Sleep only the remaining gap since the last request to this URL's host,
    /// then record the request-start timestamp.
    pub fn wait_before(&self, url: &str, interval_secs: Option<u64>) {
        let Some(interval) = interval_secs.filter(|&s| s > 0) else {
            return;
        };
        let Some(host) = host_of(url) else {
            return;
        };
        let min = Duration::from_secs(interval);

        loop {
            let sleep_for = {
                let mut map = self.last.lock().expect("HostIntervalGate poisoned");
                let now = (self.now)();
                match map.get(&host) {
                    Some(last) => {
                        let elapsed = now.saturating_duration_since(*last);
                        if elapsed < min {
                            Some(min - elapsed)
                        } else {
                            map.insert(host.clone(), now);
                            None
                        }
                    }
                    None => {
                        map.insert(host.clone(), now);
                        None
                    }
                }
            };
            match sleep_for {
                Some(wait) => (self.sleep)(wait),
                None => return,
            }
        }
    }
}

impl Default for HostIntervalGate {
    fn default() -> Self {
        Self::new()
    }
}

/// Max configured `host_min_interval_seconds` among feeds that share `url`'s host.
pub fn max_interval_for_host(feeds: &[crate::config::Feed], url: &str) -> Option<u64> {
    let host = host_of(url)?;
    feeds
        .iter()
        .filter_map(|feed| {
            let feed_host = host_of(&feed.url)?;
            if feed_host == host {
                feed.host_min_interval_seconds
            } else {
                None
            }
        })
        .max()
}

fn host_of(url: &str) -> Option<String> {
    Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_lowercase()))
}

/// Fetch `url`, optionally spacing via `gate` and using `host_min_interval_secs`
/// as a floor for HTTP 429 retry backoff.
pub fn fetch(
    agent: &Agent,
    url: &str,
    gate: Option<&HostIntervalGate>,
    host_min_interval_secs: Option<u64>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if let Some(gate) = gate {
        gate.wait_before(url, host_min_interval_secs);
    }

    let mut last_err: Box<dyn std::error::Error> = "no attempts made".to_string().into();
    let mut last_was_429 = false;
    for attempt in 0..MAX_RETRIES {
        if attempt > 0 {
            let backoff = INITIAL_BACKOFF_MS * 2u64.pow(attempt - 1);
            // Floor 429 backoff at the configured host interval so short 500ms
            // retries do not fight a 60s Reddit-style bucket.
            let wait_ms = if last_was_429 {
                let host_ms = host_min_interval_secs
                    .filter(|&s| s > 0)
                    .map(|s| s.saturating_mul(1000))
                    .unwrap_or(0);
                backoff.max(host_ms)
            } else {
                backoff
            };
            std::thread::sleep(Duration::from_millis(wait_ms));
            // Re-arm the host gate after a retry wait so parallel peers see the cooldown.
            if last_was_429 {
                if let Some(gate) = gate {
                    gate.record_now(url);
                }
            }
        }
        match agent
            .get(url)
            .set(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            )
            .set("Accept-Language", "en-US,en;q=0.9")
            .call()
        {
            Ok(resp) => {
                let mut body = Vec::new();
                resp.into_reader()
                    .take(MAX_BODY_BYTES as u64)
                    .read_to_end(&mut body)?;
                if body.len() >= MAX_BODY_BYTES {
                    return Err(format!(
                        "响应达到 {} 字节上限，疑似截断（{}）",
                        MAX_BODY_BYTES, url
                    )
                    .into());
                }
                return Ok(body);
            }
            Err(ureq::Error::Status(code, resp)) => {
                last_err = format!("HTTP {code} {}", resp.status_text()).into();
                last_was_429 = code == 429;
                // 仅 429 与 5xx 可重试;其余 4xx 为永久性失败立即返回,避免死链白耗退避时间
                if code != 429 && code < 500 {
                    return Err(last_err);
                }
                if attempt + 1 < MAX_RETRIES {
                    let backoff = INITIAL_BACKOFF_MS * 2u64.pow(attempt);
                    let wait_ms = if code == 429 {
                        let host_ms = host_min_interval_secs
                            .filter(|&s| s > 0)
                            .map(|s| s.saturating_mul(1000))
                            .unwrap_or(0);
                        backoff.max(host_ms)
                    } else {
                        backoff
                    };
                    eprintln!(
                        "[retry] {url} 第 {} 次失败(HTTP {code})，{}ms 后重试",
                        attempt + 1,
                        wait_ms
                    );
                }
            }
            Err(e) => {
                last_err = e.into();
                last_was_429 = false;
                if attempt + 1 < MAX_RETRIES {
                    let backoff = INITIAL_BACKOFF_MS * 2u64.pow(attempt);
                    eprintln!(
                        "[retry] {url} 第 {} 次失败，{}ms 后重试",
                        attempt + 1,
                        backoff
                    );
                }
            }
        }
    }
    Err(last_err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn fake_gate(
        elapsed_ms: Arc<AtomicU64>,
        sleeps: Arc<Mutex<Vec<Duration>>>,
    ) -> HostIntervalGate {
        let base = Instant::now();
        let now: NowFn = {
            let elapsed_ms = Arc::clone(&elapsed_ms);
            Arc::new(move || base + Duration::from_millis(elapsed_ms.load(Ordering::SeqCst)))
        };
        let sleep: SleepFn = {
            let elapsed_ms = Arc::clone(&elapsed_ms);
            let sleeps = Arc::clone(&sleeps);
            Arc::new(move |d: Duration| {
                sleeps.lock().expect("sleeps").push(d);
                elapsed_ms.fetch_add(d.as_millis() as u64, Ordering::SeqCst);
            })
        };
        HostIntervalGate::with_clock(now, sleep)
    }

    #[test]
    fn same_host_requests_are_spaced_by_configured_interval() {
        let elapsed_ms = Arc::new(AtomicU64::new(0));
        let sleeps = Arc::new(Mutex::new(Vec::new()));
        let gate = fake_gate(Arc::clone(&elapsed_ms), Arc::clone(&sleeps));
        let interval = Some(60u64);

        let mut request_at_ms = Vec::new();

        gate.wait_before("https://www.reddit.com/r/a/.rss", interval);
        request_at_ms.push(elapsed_ms.load(Ordering::SeqCst));

        gate.wait_before("https://www.reddit.com/r/b/.rss", interval);
        request_at_ms.push(elapsed_ms.load(Ordering::SeqCst));

        let recorded = sleeps.lock().expect("sleeps");
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0], Duration::from_secs(60));
        assert!(
            request_at_ms[1].saturating_sub(request_at_ms[0]) >= 60_000,
            "second same-host request must be spaced by >= 60s, got {request_at_ms:?}"
        );
    }

    #[test]
    fn different_hosts_do_not_wait() {
        let elapsed_ms = Arc::new(AtomicU64::new(0));
        let sleeps = Arc::new(Mutex::new(Vec::new()));
        let gate = fake_gate(Arc::clone(&elapsed_ms), Arc::clone(&sleeps));

        gate.wait_before("https://www.reddit.com/r/a/.rss", Some(60));
        gate.wait_before("https://example.com/feed.xml", Some(60));

        assert!(sleeps.lock().expect("sleeps").is_empty());
    }

    #[test]
    fn none_or_zero_interval_skips_gating() {
        let elapsed_ms = Arc::new(AtomicU64::new(0));
        let sleeps = Arc::new(Mutex::new(Vec::new()));
        let gate = fake_gate(Arc::clone(&elapsed_ms), Arc::clone(&sleeps));

        gate.wait_before("https://www.reddit.com/r/a/.rss", None);
        gate.wait_before("https://www.reddit.com/r/b/.rss", Some(0));
        gate.wait_before("https://www.reddit.com/r/c/.rss", Some(30));

        assert!(sleeps.lock().expect("sleeps").is_empty());
    }

    #[test]
    fn max_interval_for_host_picks_largest_matching_feed() {
        let feeds = vec![
            crate::config::Feed {
                name: "a".into(),
                url: "https://www.reddit.com/r/a/.rss".into(),
                skip_filter: false,
                tier: None,
                kind: None,
                adapter: None,
                adapter_params: None,
                max_items: None,
                host_min_interval_seconds: Some(30),
            },
            crate::config::Feed {
                name: "b".into(),
                url: "https://www.reddit.com/r/b/.rss".into(),
                skip_filter: false,
                tier: None,
                kind: None,
                adapter: None,
                adapter_params: None,
                max_items: None,
                host_min_interval_seconds: Some(60),
            },
            crate::config::Feed {
                name: "other".into(),
                url: "https://example.com/feed.xml".into(),
                skip_filter: false,
                tier: None,
                kind: None,
                adapter: None,
                adapter_params: None,
                max_items: None,
                host_min_interval_seconds: Some(10),
            },
        ];
        assert_eq!(
            max_interval_for_host(&feeds, "https://www.reddit.com/r/new/.rss"),
            Some(60)
        );
        assert_eq!(max_interval_for_host(&feeds, "https://example.org/x"), None);
    }
}
