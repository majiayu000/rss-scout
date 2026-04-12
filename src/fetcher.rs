use std::io::Read;
use std::time::Duration;
use ureq::Agent;

const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 500;

/// Real browser User-Agent to avoid 403 from sites that block bot UA strings.
const BROWSER_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

pub fn new_agent() -> Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(30))
        .timeout_write(Duration::from_secs(10))
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
                resp.into_reader().take(10_000_000).read_to_end(&mut body)?;
                return Ok(body);
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
