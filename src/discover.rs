use crate::config;
use crate::fetcher;
use rayon::prelude::*;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;
use url::Url;

/// Common RSS feed paths to probe on a domain
const PROBE_PATHS: &[&str] = &[
    "/feed",
    "/feed.xml",
    "/feed/rss",
    "/rss",
    "/rss.xml",
    "/atom.xml",
    "/blog/feed",
    "/blog/rss",
    "/blog/rss.xml",
    "/blog/feed.xml",
    "/blog/atom.xml",
    "/index.xml",
    "/feeds",
    "/articles.rss",
    "/posts.rss",
    "/news/rss",
];

/// Domains to skip (not useful as individual feed sources)
const SKIP_DOMAINS: &[&str] = &[
    "github.com",
    "arxiv.org",
    "news.ycombinator.com",
    "youtube.com",
    "twitter.com",
    "x.com",
    "reddit.com",
    "linkedin.com",
    "medium.com",
    "wikipedia.org",
    "docs.google.com",
    "drive.google.com",
    "t.co",
    "bit.ly",
    "doi.org",
    "substack.com",
    "facebook.com",
    "instagram.com",
    "open.spotify.com",
    "podcasts.apple.com",
    "notion.so",
    "slack.com",
    "discord.com",
    "telegram.org",
    "tiktok.com",
];

/// Regex for extracting URLs from report text
static URL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https?://[^\s\)>\]`]+").expect("URL_RE: valid regex"));

/// Regex for extracting complete <link> tags from HTML
static LINK_TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)<link[^>]*>").expect("LINK_TAG_RE: valid regex"));

/// Regex for checking rel="alternate" attribute inside a <link> tag
static REL_ALTERNATE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)rel\s*=\s*["']alternate["']"#).expect("REL_ALTERNATE_RE: valid regex")
});

/// Regex for checking type="application/(rss|atom)+xml" attribute
static TYPE_FEED_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)type\s*=\s*["']application/(rss|atom)\+xml["']"#)
        .expect("TYPE_FEED_RE: valid regex")
});

/// Regex for extracting href attribute value
static HREF_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)href\s*=\s*["']([^"']+)["']"#).expect("HREF_RE: valid regex")
});

pub struct Candidate {
    pub domain: String,
    pub feed_url: String,
    pub title: String,
    pub entry_count: usize,
}

/// Scan recent reports, extract domains, probe for RSS feeds
pub fn discover(
    data_dir: &Path,
    feeds_path: &Path,
    days: usize,
) -> Result<Vec<Candidate>, Box<dyn Error>> {
    let known_domains = load_known_domains(feeds_path)?;
    let report_urls = extract_urls_from_reports(data_dir, days)?;

    let mut domain_freq: HashMap<String, usize> = HashMap::new();
    for url in &report_urls {
        if let Some(domain) = extract_domain(url) {
            if !known_domains.contains(&domain) && !is_skip_domain(&domain) {
                *domain_freq.entry(domain).or_insert(0) += 1;
            }
        }
    }

    let mut domains: Vec<(String, usize)> = domain_freq.into_iter().collect();
    domains.sort_by(|a, b| b.1.cmp(&a.1));
    domains.truncate(50);

    let agent = fetcher::new_agent();

    let candidates: Vec<Candidate> = domains
        .par_iter()
        .filter_map(|(domain, _freq)| {
            if let Some(c) = try_autodiscovery(&agent, domain) {
                eprintln!("  [discover] {domain} → {}", c.feed_url);
                return Some(c);
            }
            if let Some(c) = try_probe_paths(&agent, domain) {
                eprintln!("  [discover] {domain} → {}", c.feed_url);
                return Some(c);
            }
            eprintln!("  [discover] {domain} — 未找到");
            None
        })
        .collect();

    Ok(candidates)
}

/// Discover RSS feed for a single URL
pub fn discover_url(url: &str) -> Result<Option<Candidate>, Box<dyn Error>> {
    let domain = extract_domain(url).ok_or("invalid URL")?;
    let agent = fetcher::new_agent();

    if let Some(c) = try_autodiscovery(&agent, &domain) {
        return Ok(Some(c));
    }
    if let Some(c) = try_probe_paths(&agent, &domain) {
        return Ok(Some(c));
    }
    Ok(None)
}

fn load_known_domains(feeds_path: &Path) -> Result<HashSet<String>, Box<dyn Error>> {
    let cfg = config::load(feeds_path)?;
    let mut domains = HashSet::new();
    for feed in &cfg.feeds {
        if let Some(d) = extract_domain(&feed.url) {
            domains.insert(d);
        }
    }
    Ok(domains)
}

fn extract_urls_from_reports(data_dir: &Path, days: usize) -> Result<Vec<String>, Box<dyn Error>> {
    let output_dir = data_dir.join("output");

    let mut urls = Vec::new();
    let mut reports: Vec<_> = fs::read_dir(&output_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("scout-") && n.ends_with(".md"))
                .unwrap_or(false)
        })
        .collect();
    reports.sort();

    let start = reports.len().saturating_sub(days);
    for report_path in &reports[start..] {
        if let Ok(content) = fs::read_to_string(report_path) {
            for cap in URL_RE.find_iter(&content) {
                let cleaned = cap
                    .as_str()
                    .trim_end_matches(&['.', ',', ';', '"', '\''][..]);
                urls.push(cleaned.to_string());
            }
        }
    }

    Ok(urls)
}

fn extract_domain(url: &str) -> Option<String> {
    Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_lowercase()))
}

fn is_skip_domain(domain: &str) -> bool {
    SKIP_DOMAINS
        .iter()
        .any(|skip| domain == *skip || domain.ends_with(&format!(".{skip}")))
}

/// Try HTML <link rel="alternate"> autodiscovery (order-independent attribute matching)
fn try_autodiscovery(agent: &ureq::Agent, domain: &str) -> Option<Candidate> {
    let url = format!("https://{domain}");
    let body_bytes = fetcher::fetch(agent, &url).ok()?;
    let body = String::from_utf8_lossy(&body_bytes);

    // Extract each <link ...> tag, then independently check for all three attributes
    for tag_match in LINK_TAG_RE.find_iter(&body) {
        let tag = tag_match.as_str();

        if !REL_ALTERNATE_RE.is_match(tag) {
            continue;
        }
        if !TYPE_FEED_RE.is_match(tag) {
            continue;
        }

        let feed_href = match HREF_RE.captures(tag) {
            Some(caps) => caps[1].to_string(),
            None => continue,
        };

        // Resolve relative URLs
        let feed_url = if feed_href.starts_with("http") {
            feed_href
        } else if feed_href.starts_with('/') {
            format!("https://{domain}{feed_href}")
        } else {
            format!("https://{domain}/{feed_href}")
        };

        // Validate it's actually a feed
        if let Some(c) = validate_feed(agent, domain, &feed_url) {
            return Some(c);
        }
    }

    None
}

/// Try common RSS paths
fn try_probe_paths(agent: &ureq::Agent, domain: &str) -> Option<Candidate> {
    for path in PROBE_PATHS {
        let url = format!("https://{domain}{path}");
        if let Some(c) = validate_feed(agent, domain, &url) {
            return Some(c);
        }
    }
    None
}

/// Fetch URL, check if it's valid RSS/Atom, return candidate
fn validate_feed(agent: &ureq::Agent, domain: &str, url: &str) -> Option<Candidate> {
    let body = fetcher::fetch(agent, url).ok()?;
    let feed = feed_rs::parser::parse(&body[..]).ok()?;

    if feed.entries.is_empty() {
        return None;
    }

    let title = feed
        .title
        .map(|t| t.content)
        .unwrap_or_else(|| domain.to_string());

    Some(Candidate {
        domain: domain.to_string(),
        feed_url: url.to_string(),
        title,
        entry_count: feed.entries.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            extract_domain("https://example.com/path?q=1"),
            Some("example.com".to_string())
        );
        assert_eq!(
            extract_domain("https://sub.example.com/"),
            Some("sub.example.com".to_string())
        );
        assert_eq!(extract_domain("not-a-url"), None);
        assert_eq!(extract_domain(""), None);
    }

    #[test]
    fn test_is_skip_domain() {
        assert!(is_skip_domain("github.com"));
        assert!(is_skip_domain("api.github.com"));
        assert!(is_skip_domain("substack.com"));
        assert!(is_skip_domain("facebook.com"));
        assert!(is_skip_domain("open.spotify.com"));
        assert!(!is_skip_domain("example.com"));
        assert!(!is_skip_domain("simonwillison.net"));
    }

    #[test]
    fn test_url_extraction_trims_punctuation() {
        let text = r#"Check https://example.com/page, and https://other.com/path. Also "https://quoted.com/x""#;
        let urls: Vec<String> = URL_RE
            .find_iter(text)
            .map(|m| {
                m.as_str()
                    .trim_end_matches(&['.', ',', ';', '"', '\''][..])
                    .to_string()
            })
            .collect();
        assert_eq!(urls[0], "https://example.com/page");
        assert_eq!(urls[1], "https://other.com/path");
        assert_eq!(urls[2], "https://quoted.com/x");
    }

    #[test]
    fn test_autodiscovery_regex_order_independent() {
        // rel before type before href
        let tag1 = r#"<link rel="alternate" type="application/rss+xml" href="/feed.xml">"#;
        // href before type before rel
        let tag2 = r#"<link href="/feed.xml" type="application/rss+xml" rel="alternate">"#;
        // type before rel before href
        let tag3 = r#"<link type="application/atom+xml" rel="alternate" href="/atom.xml">"#;
        // Mixed case
        let tag4 = r#"<link REL="alternate" TYPE="application/rss+xml" HREF="/rss">"#;

        for tag in [tag1, tag2, tag3, tag4] {
            assert!(LINK_TAG_RE.is_match(tag), "LINK_TAG_RE should match: {tag}");
            let tag_match = LINK_TAG_RE.find(tag).unwrap().as_str();
            assert!(
                REL_ALTERNATE_RE.is_match(tag_match),
                "REL_ALTERNATE_RE should match: {tag}"
            );
            assert!(
                TYPE_FEED_RE.is_match(tag_match),
                "TYPE_FEED_RE should match: {tag}"
            );
            assert!(HREF_RE.is_match(tag_match), "HREF_RE should match: {tag}");
        }

        // Non-feed link should NOT match type
        let non_feed = r#"<link rel="stylesheet" type="text/css" href="/style.css">"#;
        let tag_match = LINK_TAG_RE.find(non_feed).unwrap().as_str();
        assert!(!REL_ALTERNATE_RE.is_match(tag_match));
    }
}
