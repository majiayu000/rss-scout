use regex::Regex;
use std::sync::LazyLock;

pub struct OpmlFeed {
    pub name: String,
    pub url: String,
}

static OUTLINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<outline[^>]*>"#).expect("valid regex")
});

static XML_URL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"xmlUrl\s*=\s*"([^"]+)""#).expect("valid regex")
});

static TEXT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:text|title)\s*=\s*"([^"]+)""#).expect("valid regex")
});

/// Parse OPML XML content, extract all feed URLs with names.
pub fn parse_opml(content: &str) -> Vec<OpmlFeed> {
    let mut feeds = Vec::new();

    for outline in OUTLINE_RE.find_iter(content) {
        let tag = outline.as_str();

        let Some(url_cap) = XML_URL_RE.captures(tag) else {
            continue;
        };
        let url = url_cap[1].to_string();

        let name = TEXT_RE
            .captures(tag)
            .map(|c| c[1].to_string())
            .unwrap_or_else(|| {
                // Derive name from domain
                url::Url::parse(&url)
                    .ok()
                    .and_then(|u| u.host_str().map(|h| h.to_string()))
                    .unwrap_or_else(|| "Unknown".to_string())
            });

        feeds.push(OpmlFeed { name, url });
    }

    feeds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_opml_basic() {
        let opml = r#"<?xml version="1.0" encoding="UTF-8"?>
<opml version="2.0">
  <body>
    <outline text="Tech" title="Tech">
      <outline text="Simon Willison" title="Simon Willison" xmlUrl="https://simonwillison.net/atom/everything/" htmlUrl="https://simonwillison.net"/>
      <outline text="Hacker News" xmlUrl="https://news.ycombinator.com/rss"/>
    </outline>
  </body>
</opml>"#;

        let feeds = parse_opml(opml);
        assert_eq!(feeds.len(), 2);
        assert_eq!(feeds[0].name, "Simon Willison");
        assert_eq!(
            feeds[0].url,
            "https://simonwillison.net/atom/everything/"
        );
        assert_eq!(feeds[1].name, "Hacker News");
    }

    #[test]
    fn test_parse_opml_no_feeds() {
        let opml = r#"<opml><body><outline text="Folder"/></body></opml>"#;
        let feeds = parse_opml(opml);
        assert!(feeds.is_empty());
    }
}
