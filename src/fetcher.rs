use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Duration;
use ureq::Agent;
use url::Url;

const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 500;
/// 单响应体积上限,防止超大/异常响应撑爆内存
const MAX_BODY_BYTES: usize = 10_000_000;
/// 单请求整体超时(含重定向与 body 读取),防止慢源占住 worker 拖尾整轮
const OVERALL_TIMEOUT_SECS: u64 = 60;
/// Manual redirect budget; auto-follow is disabled so each hop can be blocklisted.
const MAX_REDIRECTS: u32 = 5;

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
        // Disable ureq auto-follow so we can reject private redirect targets.
        .redirects(0)
        .build()
}

/// Returns true when `url` targets a local/private/link-local/metadata host that
/// must not be requested or propagated into reports.
///
/// Covers literal IPs (RFC1918, loopback, link-local incl. 169.254.169.254, IPv6
/// ULA `fc00::/7`, IPv6 link-local) plus `localhost` / `*.local` hostnames.
pub fn is_blocked_destination(url: &str) -> bool {
    let parsed = match Url::parse(url) {
        Ok(u) => u,
        Err(_) => match Url::parse(&format!("http://{url}")) {
            Ok(u) => u,
            Err(_) => return true,
        },
    };

    match parsed.scheme() {
        "http" | "https" => {}
        // Non-HTTP schemes are not fetchable by this client; treat as blocked.
        _ => return true,
    }

    match parsed.host() {
        Some(url::Host::Domain(domain)) => {
            let lower = domain.to_ascii_lowercase();
            lower == "localhost" || lower.ends_with(".local")
        }
        Some(url::Host::Ipv4(ip)) => is_blocked_ip(IpAddr::V4(ip)),
        Some(url::Host::Ipv6(ip)) => is_blocked_ip(IpAddr::V6(ip)),
        None => true,
    }
}

fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_blocked_ipv4(v4),
        IpAddr::V6(v6) => {
            // Check native IPv6 specials first: `::1`.to_ipv4() is `0.0.0.1`, which
            // is not treated as IPv4 loopback and would otherwise slip through.
            if is_blocked_ipv6(v6) {
                return true;
            }
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_blocked_ipv4(v4);
            }
            // Deprecated IPv4-compatible form `::a.b.c.d` (excludes :: and ::1 above).
            if let Some(v4) = v6.to_ipv4() {
                return is_blocked_ipv4(v4);
            }
            false
        }
    }
}

fn is_blocked_ipv4(ip: Ipv4Addr) -> bool {
    ip.is_unspecified() // 0.0.0.0/8-ish: 0.0.0.0
        || ip.is_loopback() // 127.0.0.0/8
        || ip.is_private() // 10/8, 172.16/12, 192.168/16
        || ip.is_link_local() // 169.254.0.0/16 (incl. 169.254.169.254)
        || ip.is_broadcast()
}

fn is_blocked_ipv6(ip: Ipv6Addr) -> bool {
    if ip.is_unspecified() || ip.is_loopback() {
        return true;
    }
    let segments = ip.segments();
    // Unique local addresses fc00::/7
    if segments[0] & 0xfe00 == 0xfc00 {
        return true;
    }
    // Link-local fe80::/10
    if segments[0] & 0xffc0 == 0xfe80 {
        return true;
    }
    false
}

fn resolve_redirect(current: &str, location: &str) -> Result<String, Box<dyn std::error::Error>> {
    let base = Url::parse(current)?;
    Ok(base.join(location)?.to_string())
}

pub fn fetch(agent: &Agent, url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if is_blocked_destination(url) {
        return Err(format!("blocked destination (private/local host): {url}").into());
    }

    let mut last_err: Box<dyn std::error::Error> = "no attempts made".to_string().into();
    for attempt in 0..MAX_RETRIES {
        if attempt > 0 {
            let backoff = INITIAL_BACKOFF_MS * 2u64.pow(attempt - 1);
            std::thread::sleep(Duration::from_millis(backoff));
        }

        let mut current_url = url.to_string();
        let mut redirects: u32 = 0;

        loop {
            if is_blocked_destination(&current_url) {
                return Err(
                    format!("blocked destination (private/local host): {current_url}").into(),
                );
            }

            match agent
                .get(&current_url)
                .set(
                    "Accept",
                    "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
                )
                .set("Accept-Language", "en-US,en;q=0.9")
                .call()
            {
                Ok(resp) => {
                    let status = resp.status();
                    if (300..400).contains(&status) {
                        if redirects >= MAX_REDIRECTS {
                            return Err(
                                format!("too many redirects ({MAX_REDIRECTS}) from {url}").into()
                            );
                        }
                        let Some(location) = resp.header("Location").map(str::to_string) else {
                            return Err(format!(
                                "HTTP {status} redirect without Location from {current_url}"
                            )
                            .into());
                        };
                        let next = resolve_redirect(&current_url, &location)?;
                        if is_blocked_destination(&next) {
                            return Err(format!(
                                "blocked redirect destination (private/local host): {next}"
                            )
                            .into());
                        }
                        redirects += 1;
                        current_url = next;
                        continue;
                    }

                    let mut body = Vec::new();
                    resp.into_reader()
                        .take(MAX_BODY_BYTES as u64)
                        .read_to_end(&mut body)?;
                    if body.len() >= MAX_BODY_BYTES {
                        return Err(format!(
                            "响应达到 {} 字节上限，疑似截断（{}）",
                            MAX_BODY_BYTES, current_url
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
                            "[retry] {current_url} 第 {} 次失败(HTTP {code})，{}ms 后重试",
                            attempt + 1,
                            INITIAL_BACKOFF_MS * 2u64.pow(attempt)
                        );
                    }
                    break;
                }
                Err(e) => {
                    last_err = e.into();
                    if attempt + 1 < MAX_RETRIES {
                        eprintln!(
                            "[retry] {current_url} 第 {} 次失败，{}ms 后重试",
                            attempt + 1,
                            INITIAL_BACKOFF_MS * 2u64.pow(attempt)
                        );
                    }
                    break;
                }
            }
        }
    }
    Err(last_err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocked_rfc1918_and_metadata() {
        assert!(is_blocked_destination("http://192.168.1.1/"));
        assert!(is_blocked_destination("https://10.0.0.1/feed.xml"));
        assert!(is_blocked_destination("http://172.16.5.5/"));
        assert!(is_blocked_destination(
            "http://169.254.169.254/latest/meta-data/"
        ));
    }

    #[test]
    fn blocked_ipv6_ula_and_loopback() {
        assert!(is_blocked_destination("http://[fd12:3456:789a::1]/"));
        assert!(is_blocked_destination("http://[fc00::1]/"));
        assert!(is_blocked_destination("http://[::1]/"));
        assert!(is_blocked_destination("http://[fe80::1]/"));
    }

    #[test]
    fn blocked_localhost_variants() {
        assert!(is_blocked_destination("http://localhost:5174/blog/x"));
        assert!(is_blocked_destination("https://localhost/foo"));
        assert!(is_blocked_destination("http://127.0.0.1:8080/path"));
        assert!(is_blocked_destination("http://0.0.0.0/"));
        assert!(is_blocked_destination("http://myhost.local/feed"));
        assert!(is_blocked_destination("HTTP://LocalHost/foo"));
    }

    #[test]
    fn allows_public_urls() {
        assert!(!is_blocked_destination("https://sourcegraph.com/blog/x"));
        assert!(!is_blocked_destination("https://arxiv.org/abs/2509.22202"));
        assert!(!is_blocked_destination("https://localhost.example.com/"));
        assert!(!is_blocked_destination(
            "https://example.com/127.0.0.1/path"
        ));
        assert!(!is_blocked_destination(
            "https://example.com/192.168.1.1/path"
        ));
        assert!(!is_blocked_destination("https://8.8.8.8/"));
    }

    #[test]
    fn blocked_ipv4_mapped_ipv6() {
        assert!(is_blocked_destination("http://[::ffff:192.168.1.1]/"));
        assert!(is_blocked_destination("http://[::ffff:10.0.0.1]/"));
        assert!(is_blocked_destination("http://[::ffff:169.254.169.254]/"));
    }

    #[test]
    fn fetch_rejects_blocked_before_request() {
        let agent = new_agent();
        let err = fetch(&agent, "http://192.168.1.1/").expect_err("private IP must be rejected");
        assert!(
            err.to_string().contains("blocked destination"),
            "unexpected error: {err}"
        );
    }
}
