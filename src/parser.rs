use regex::Regex;
use std::sync::LazyLock;

static IMG_SRC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<img[^>]+src=["']([^"']+)["']"#).unwrap_or_else(|e| {
        eprintln!("[FATAL] IMG_SRC_RE regex compile failed: {e}");
        std::process::exit(1);
    })
});

#[derive(Clone)]
pub struct Entry {
    pub title: String,
    pub link: String,
    pub date: String,
    pub desc: String,
    pub image: Option<String>,
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

            let raw_link = e.links.first()?.href.clone();
            if raw_link.is_empty() {
                return None;
            }
            let link = normalize_link(&raw_link);

            let date = e
                .published
                .or(e.updated)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default();

            let raw_desc = e.summary.map(|s| s.content).unwrap_or_default();

            // Extract image: priority media thumbnail > desc <img> tag
            let image = extract_media_thumbnail(&e.media)
                .or_else(|| extract_img_from_html(&raw_desc));

            let desc: String = raw_desc.chars().take(200).collect();

            Some(Entry {
                title,
                link,
                date,
                desc,
                image,
            })
        })
        .collect()
}

fn extract_media_thumbnail(media: &[feed_rs::model::MediaObject]) -> Option<String> {
    for m in media {
        for thumb in &m.thumbnails {
            let uri = thumb.image.uri.as_str();
            if !uri.is_empty() {
                return Some(uri.to_string());
            }
        }
        // Fallback: media content with image MIME
        for content in &m.content {
            if let Some(ref ct) = content.content_type {
                if ct.as_ref().starts_with("image/") {
                    if let Some(ref url) = content.url {
                        return Some(url.to_string());
                    }
                }
            }
        }
    }
    None
}

/// www.reddit.com 屏蔽 bot 抓取，转换为 old.reddit.com 以便 WebFetch 读取全文
fn normalize_link(link: &str) -> String {
    if let Some(rest) = link.strip_prefix("https://www.reddit.com") {
        return format!("https://old.reddit.com{rest}");
    }
    if let Some(rest) = link.strip_prefix("http://www.reddit.com") {
        return format!("http://old.reddit.com{rest}");
    }
    if let Some(rest) = link.strip_prefix("https://reddit.com") {
        return format!("https://old.reddit.com{rest}");
    }
    if let Some(rest) = link.strip_prefix("http://reddit.com") {
        return format!("http://old.reddit.com{rest}");
    }
    link.to_string()
}

fn extract_img_from_html(html: &str) -> Option<String> {
    IMG_SRC_RE
        .captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .filter(|url| url.starts_with("http"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rss() {
        let xml = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Test Feed</title>
    <item>
      <title>First Post</title>
      <link>https://example.com/first</link>
      <pubDate>Mon, 01 Jan 2024 00:00:00 GMT</pubDate>
      <description>Hello world</description>
    </item>
    <item>
      <title>Second Post</title>
      <link>https://example.com/second</link>
      <description>Another post</description>
    </item>
  </channel>
</rss>"#;

        let entries = parse(xml.as_bytes(), 10);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].title, "First Post");
        assert_eq!(entries[0].link, "https://example.com/first");
        assert!(!entries[0].date.is_empty());
        assert_eq!(entries[1].title, "Second Post");
        assert_eq!(entries[1].link, "https://example.com/second");
    }

    #[test]
    fn test_parse_atom() {
        let xml = r#"<?xml version="1.0"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Atom Feed</title>
  <entry>
    <title>Atom Entry</title>
    <link href="https://example.com/atom-entry"/>
    <updated>2024-01-01T00:00:00Z</updated>
    <summary>An atom summary</summary>
  </entry>
</feed>"#;

        let entries = parse(xml.as_bytes(), 10);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Atom Entry");
        assert_eq!(entries[0].link, "https://example.com/atom-entry");
        assert!(!entries[0].date.is_empty());
    }

    #[test]
    fn test_empty_feed() {
        let xml = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Empty Feed</title>
  </channel>
</rss>"#;

        let entries = parse(xml.as_bytes(), 10);
        assert!(entries.is_empty());
    }

    #[test]
    fn test_max_items_limit() {
        let xml = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Test</title>
    <item><title>A</title><link>https://example.com/a</link></item>
    <item><title>B</title><link>https://example.com/b</link></item>
    <item><title>C</title><link>https://example.com/c</link></item>
  </channel>
</rss>"#;

        let entries = parse(xml.as_bytes(), 2);
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_invalid_xml() {
        let entries = parse(b"not xml at all", 10);
        assert!(entries.is_empty());
    }

    #[test]
    fn test_desc_truncated() {
        let long_desc = "x".repeat(500);
        let xml = format!(
            r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Test</title>
    <item>
      <title>Post</title>
      <link>https://example.com/post</link>
      <description>{long_desc}</description>
    </item>
  </channel>
</rss>"#
        );

        let entries = parse(xml.as_bytes(), 10);
        assert_eq!(entries.len(), 1);
        assert!(entries[0].desc.len() <= 200);
    }

    #[test]
    fn test_image_from_desc_img_tag() {
        let xml = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Test</title>
    <item>
      <title>With Image</title>
      <link>https://example.com/img</link>
      <description>&lt;img src="https://cdn.example.com/photo.jpg"&gt; Some text</description>
    </item>
  </channel>
</rss>"#;

        let entries = parse(xml.as_bytes(), 10);
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].image.as_deref(),
            Some("https://cdn.example.com/photo.jpg")
        );
    }

    #[test]
    fn test_reddit_link_normalized_to_old() {
        assert_eq!(
            normalize_link("https://www.reddit.com/r/rust/comments/abc"),
            "https://old.reddit.com/r/rust/comments/abc"
        );
        assert_eq!(
            normalize_link("https://reddit.com/r/MachineLearning/comments/xyz"),
            "https://old.reddit.com/r/MachineLearning/comments/xyz"
        );
    }

    #[test]
    fn test_non_reddit_link_unchanged() {
        let link = "https://simonwillison.net/2024/post";
        assert_eq!(normalize_link(link), link);
    }

    #[test]
    fn test_old_reddit_link_unchanged() {
        let link = "https://old.reddit.com/r/rust/comments/abc";
        assert_eq!(normalize_link(link), link);
    }

    #[test]
    fn test_no_image() {
        let xml = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Test</title>
    <item>
      <title>No Image</title>
      <link>https://example.com/noimg</link>
      <description>Plain text only</description>
    </item>
  </channel>
</rss>"#;

        let entries = parse(xml.as_bytes(), 10);
        assert_eq!(entries.len(), 1);
        assert!(entries[0].image.is_none());
    }
}
