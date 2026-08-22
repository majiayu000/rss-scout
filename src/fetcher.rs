use std::io::Read;
use std::time::Duration;
use ureq::Agent;

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

pub fn fetch(agent: &Agent, url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut last_err: Box<dyn std::error::Error> = "no attempts made".to_string().into();
    for attempt in 0..MAX_RETRIES {
        if attempt > 0 {
            let backoff = INITIAL_BACKOFF_MS * 2u64.pow(attempt - 1);
            std::thread::sleep(Duration::from_millis(backoff));
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
                // 仅 429 与 5xx 可重试;其余 4xx 为永久性失败立即返回,避免死链白耗退避时间
                if code != 429 && code < 500 {
                    return Err(last_err);
                }
                if attempt + 1 < MAX_RETRIES {
                    eprintln!(
                        "[retry] {url} 第 {} 次失败(HTTP {code})，{}ms 后重试",
                        attempt + 1,
                        INITIAL_BACKOFF_MS * 2u64.pow(attempt)
                    );
                }
            }
            Err(e) => {
                last_err = e.into();
                if attempt + 1 < MAX_RETRIES {
                    eprintln!(
                        "[retry] {url} 第 {} 次失败，{}ms 后重试",
                        attempt + 1,
                        INITIAL_BACKOFF_MS * 2u64.pow(attempt)
                    );
                }
            }
        }
    }
    Err(last_err)
}
