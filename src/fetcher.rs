use std::io::Read;
use std::time::Duration;
use ureq::Agent;

pub fn new_agent() -> Agent {
    ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(20))
        .timeout_write(Duration::from_secs(10))
        .user_agent("rss-scout/1.0 (knowledge-discovery)")
        .build()
}

pub fn fetch(agent: &Agent, url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let resp = agent.get(url).call()?;
    let mut body = Vec::new();
    resp.into_reader().take(10_000_000).read_to_end(&mut body)?;
    Ok(body)
}
