use crate::scorer::{Priority, ScoredEntry};
use chrono::{Datelike, NaiveDate};
use regex::Regex;
use serde_json::{json, Value};
use std::error::Error;
use std::io::Read;
use std::sync::LazyLock;
use std::thread;
use std::time::Duration;
use ureq::Agent;

const NOTION_VERSION: &str = "2022-06-28";

/// Notion 同步结果——区分"无 P0/P1 内容"与"当日页面已存在"两种跳过语义(2026-08-23 审计 #7)
pub enum SyncOutcome {
    /// 已创建当日摘要页
    Created,
    /// 无 P0/P1 条目可同步
    NoHighPriority,
    /// 当日页面已存在,幂等跳过
    PageExists,
}
const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 500;
const MAX_BLOCKS_PER_REQUEST: usize = 100;
const MAX_DESC_CHARS: usize = 300;

/// HTML 标签清洗正则,进程内只编译一次(与 report.rs 的 LazyLock 约定一致)
static TAG_RE: LazyLock<Regex> = LazyLock::new(|| match Regex::new(r"<[^>]+>") {
    Ok(r) => r,
    Err(e) => {
        eprintln!("[FATAL] TAG_RE regex compile failed: {e}");
        std::process::exit(1);
    }
});

pub struct NotionClient {
    agent: Agent,
    api_key: String,
    database_id: String,
}

impl NotionClient {
    pub fn new(api_key: String, database_id: String) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(10))
            .timeout_read(Duration::from_secs(30))
            .timeout_write(Duration::from_secs(10))
            .build();
        Self {
            agent,
            api_key,
            database_id,
        }
    }

    pub fn sync_daily_summary(
        &self,
        entries: &[ScoredEntry],
        today: NaiveDate,
    ) -> Result<SyncOutcome, Box<dyn Error>> {
        let selected: Vec<&ScoredEntry> = entries
            .iter()
            .filter(|entry| matches!(entry.priority, Priority::P0 | Priority::P1))
            .collect();

        if selected.is_empty() {
            return Ok(SyncOutcome::NoHighPriority);
        }

        if self.page_exists_for_date(today)? {
            return Ok(SyncOutcome::PageExists);
        }

        let title = build_page_title(today);
        let cover_url = first_cover_url(&selected);
        let blocks = build_blocks(&selected);
        if blocks.is_empty() {
            return Ok(SyncOutcome::NoHighPriority);
        }

        let mut batches = split_blocks(&blocks, MAX_BLOCKS_PER_REQUEST);
        let first_batch = batches.remove(0);
        let page_id = self.create_page(today, &title, cover_url, &first_batch)?;

        for batch in batches {
            self.append_blocks(&page_id, &batch)?;
        }

        Ok(SyncOutcome::Created)
    }

    fn page_exists_for_date(&self, today: NaiveDate) -> Result<bool, Box<dyn Error>> {
        let url = format!(
            "https://api.notion.com/v1/databases/{}/query",
            self.database_id
        );
        let body = json!({
            "filter": {
                "property": "Date",
                "date": { "equals": today.format("%Y-%m-%d").to_string() }
            },
            "page_size": 1
        });
        let value = self.send_json("POST", &url, &body)?;
        let results = value
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or("Notion query 响应缺少 results")?;
        Ok(!results.is_empty())
    }

    fn create_page(
        &self,
        today: NaiveDate,
        title: &str,
        cover_url: Option<&str>,
        children: &[Value],
    ) -> Result<String, Box<dyn Error>> {
        let mut body = json!({
            "parent": { "database_id": self.database_id },
            "properties": {
                "Title": {
                    "title": [{ "text": { "content": title } }]
                },
                "Date": {
                    "date": { "start": today.format("%Y-%m-%d").to_string() }
                }
            },
            "children": children,
        });

        if let Some(url) = cover_url {
            body["cover"] = json!({
                "type": "external",
                "external": { "url": url }
            });
        }

        let value = self.send_json("POST", "https://api.notion.com/v1/pages", &body)?;
        let page_id = value
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("Notion create page 响应缺少 id")?;
        Ok(page_id.to_string())
    }

    fn append_blocks(&self, page_id: &str, children: &[Value]) -> Result<(), Box<dyn Error>> {
        let url = format!("https://api.notion.com/v1/blocks/{page_id}/children");
        let body = json!({ "children": children });
        let _ = self.send_json("PATCH", &url, &body)?;
        Ok(())
    }

    fn send_json(&self, method: &str, url: &str, body: &Value) -> Result<Value, Box<dyn Error>> {
        let body_str = body.to_string();
        let mut last_err: Box<dyn Error> = "no attempts made".to_string().into();

        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                let backoff = INITIAL_BACKOFF_MS * 2u64.pow(attempt - 1);
                thread::sleep(Duration::from_millis(backoff));
            }

            let request = self
                .agent
                .request(method, url)
                .set("Authorization", &format!("Bearer {}", self.api_key))
                .set("Content-Type", "application/json")
                .set("Notion-Version", NOTION_VERSION);

            match request.send_string(&body_str) {
                Ok(resp) => {
                    let mut body = String::new();
                    resp.into_reader().read_to_string(&mut body)?;
                    let value: Value = serde_json::from_str(&body)?;
                    return Ok(value);
                }
                Err(ureq::Error::Status(code, resp)) => {
                    if (code == 429 || code >= 500) && attempt + 1 < MAX_RETRIES {
                        last_err = format!(
                            "Notion API {} {} failed with status {} (will retry)",
                            method, url, code
                        )
                        .into();
                        drop(resp);
                        continue;
                    }

                    let status_text = resp.status_text().to_string();
                    last_err = format!(
                        "Notion API {} {} failed with status {} {}",
                        method, url, code, status_text
                    )
                    .into();
                }
                Err(ureq::Error::Transport(t)) => {
                    last_err = format!("Notion API {} {} transport error: {t}", method, url).into();
                    if attempt + 1 < MAX_RETRIES {
                        continue;
                    }
                }
            }
        }

        Err(last_err)
    }
}

fn build_page_title(today: NaiveDate) -> String {
    format!("{}月{}日AI资讯", today.month(), today.day())
}

fn first_cover_url<'a>(entries: &'a [&ScoredEntry]) -> Option<&'a str> {
    entries
        .iter()
        .find_map(|entry| entry.entry.image.as_deref())
}

fn build_blocks(entries: &[&ScoredEntry]) -> Vec<Value> {
    let mut blocks = Vec::new();

    for (index, entry) in entries.iter().enumerate() {
        let title = format!("{}、{}", index + 1, entry.entry.title);
        blocks.push(heading_block(&title));

        let desc = clean_description(&entry.entry.desc);
        if !desc.is_empty() {
            blocks.push(paragraph_block(&desc));
        }

        blocks.push(source_block(&entry.feed_name, &entry.entry.link));

        if let Some(image) = entry.entry.image.as_deref() {
            blocks.push(image_block(image));
        }

        if index + 1 < entries.len() {
            blocks.push(divider_block());
        }
    }

    blocks
}

fn clean_description(raw: &str) -> String {
    let cleaned = TAG_RE.replace_all(raw, " ");
    let cleaned = cleaned
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");
    let normalized = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    normalized.chars().take(MAX_DESC_CHARS).collect()
}

fn split_blocks(blocks: &[Value], chunk_size: usize) -> Vec<Vec<Value>> {
    blocks
        .chunks(chunk_size)
        .map(|chunk| chunk.to_vec())
        .collect()
}

fn heading_block(content: &str) -> Value {
    json!({
        "object": "block",
        "type": "heading_2",
        "heading_2": {
            "rich_text": [{ "type": "text", "text": { "content": content } }]
        }
    })
}

fn paragraph_block(content: &str) -> Value {
    json!({
        "object": "block",
        "type": "paragraph",
        "paragraph": {
            "rich_text": [{ "type": "text", "text": { "content": content } }]
        }
    })
}

fn source_block(feed_name: &str, link: &str) -> Value {
    json!({
        "object": "block",
        "type": "paragraph",
        "paragraph": {
            "rich_text": [
                { "type": "text", "text": { "content": "来源：" } },
                {
                    "type": "text",
                    "text": {
                        "content": feed_name,
                        "link": { "url": link }
                    },
                    "annotations": {
                        "underline": true
                    }
                }
            ]
        }
    })
}

fn image_block(url: &str) -> Value {
    json!({
        "object": "block",
        "type": "image",
        "image": {
            "type": "external",
            "external": { "url": url }
        }
    })
}

fn divider_block() -> Value {
    json!({
        "object": "block",
        "type": "divider",
        "divider": {}
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Entry;

    fn make_entry(title: &str, desc: &str, link: &str, image: Option<&str>) -> ScoredEntry {
        ScoredEntry {
            entry: Entry {
                title: title.to_string(),
                link: link.to_string(),
                date: String::new(),
                desc: desc.to_string(),
                image: image.map(|s| s.to_string()),
            },
            feed_name: "ExampleFeed".to_string(),
            feed_kind: Some("blog".to_string()),
            score: 9,
            priority: Priority::P0,
            breakdown: [3, 2, 2, 2],
        }
    }

    #[test]
    fn test_clean_description_strips_html_and_truncates() {
        let raw = format!("<p>Hello <b>world</b></p>{}", "x".repeat(400));
        let clean = clean_description(&raw);
        assert!(!clean.contains("<p>"));
        assert!(clean.starts_with("Hello world"));
        assert!(clean.len() <= MAX_DESC_CHARS);
    }

    #[test]
    fn test_build_page_title() {
        let date = NaiveDate::from_ymd_opt(2026, 4, 12).unwrap();
        assert_eq!(build_page_title(date), "4月12日AI资讯");
    }

    #[test]
    fn test_first_cover_url() {
        let a = make_entry("A", "", "https://a.com", None);
        let b = make_entry("B", "", "https://b.com", Some("https://img.com/1.png"));
        let entries = vec![&a, &b];
        assert_eq!(first_cover_url(&entries), Some("https://img.com/1.png"));
    }

    #[test]
    fn test_build_blocks_with_image_and_divider() {
        let a = make_entry(
            "A",
            "<p>Desc</p>",
            "https://a.com",
            Some("https://img.com/1.png"),
        );
        let b = make_entry("B", "", "https://b.com", None);
        let blocks = build_blocks(&[&a, &b]);
        assert_eq!(blocks.len(), 7);
        assert_eq!(blocks[0]["type"], "heading_2");
        assert_eq!(blocks[1]["type"], "paragraph");
        assert_eq!(blocks[2]["type"], "paragraph");
        assert_eq!(blocks[3]["type"], "image");
        assert_eq!(blocks[4]["type"], "divider");
        assert_eq!(blocks[5]["type"], "heading_2");
        assert_eq!(blocks[6]["type"], "paragraph");
    }

    #[test]
    fn test_split_blocks_batches() {
        let blocks = vec![paragraph_block("x"); 101];
        let batches = split_blocks(&blocks, 100);
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].len(), 100);
        assert_eq!(batches[1].len(), 1);
    }
}
