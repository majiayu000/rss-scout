pub struct Entry {
    pub title: String,
    pub link: String,
    pub date: String,
    pub desc: String,
}

pub fn parse(data: &[u8], max_items: usize) -> Vec<Entry> {
    let feed = match feed_rs::parser::parse(data) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[parser] feed 解析失败: {e}");
            return Vec::new();
        }
    };

    feed.entries
        .into_iter()
        .take(max_items)
        .filter_map(|e| {
            let title = e.title.map(|t| t.content)?.trim().to_string();
            if title.is_empty() {
                return None;
            }

            let link = e.links.first()?.href.clone();
            if link.is_empty() {
                return None;
            }

            let date = e
                .published
                .or(e.updated)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default();

            let desc = e.summary.map(|s| s.content).unwrap_or_default();
            let desc: String = desc.chars().take(200).collect();

            Some(Entry {
                title,
                link,
                date,
                desc,
            })
        })
        .collect()
}
