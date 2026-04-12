use crate::config::{Feed, ScoringConfig};
use crate::dedup::SeenDb;
use crate::parser::Entry;
use chrono::{DateTime, Utc};
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    P0,
    P1,
    P2,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::P0 => write!(f, "P0"),
            Priority::P1 => write!(f, "P1"),
            Priority::P2 => write!(f, "P2"),
        }
    }
}

#[derive(Clone)]
pub struct ScoredEntry {
    pub entry: Entry,
    pub feed_name: String,
    pub feed_kind: Option<String>,
    pub score: u8,
    pub priority: Priority,
    pub breakdown: [u8; 4],
}

pub fn score_entry(
    entry: &Entry,
    feed: &Feed,
    scoring: &ScoringConfig,
    seen_db: &SeenDb,
) -> ScoredEntry {
    let source = score_source(feed);
    let keyword = score_keywords(&entry.title, &entry.desc, scoring);
    let freshness = score_freshness(&entry.date);
    let uniqueness = score_uniqueness(&entry.link, seen_db);

    let total = source + keyword + freshness + uniqueness;
    let priority = if total >= 9 {
        Priority::P0
    } else if total >= 6 {
        Priority::P1
    } else {
        Priority::P2
    };

    ScoredEntry {
        entry: entry.clone(),
        feed_name: feed.name.clone(),
        feed_kind: feed.kind.clone(),
        score: total,
        priority,
        breakdown: [source, keyword, freshness, uniqueness],
    }
}

fn score_source(feed: &Feed) -> u8 {
    if let Some(ref tier) = feed.tier {
        return match tier.as_str() {
            "core" => 3,
            "official" => 2,
            "aggregator" => 1,
            "release" => 0,
            _ => 1,
        };
    }
    // Infer from name prefix
    let name = &feed.name;
    if name.starts_with("GH:") {
        0
    } else if name.starts_with("HN:") || name.starts_with("YT:") {
        1
    } else {
        1
    }
}

fn score_keywords(title: &str, desc: &str, scoring: &ScoringConfig) -> u8 {
    let text = format!("{} {}", title, desc).to_lowercase();

    let high_hits = scoring
        .keywords_high
        .iter()
        .filter(|kw| text.contains(&kw.to_lowercase()))
        .count();
    let mid_hits = scoring
        .keywords_mid
        .iter()
        .filter(|kw| text.contains(&kw.to_lowercase()))
        .count();

    if high_hits >= 2 {
        3
    } else if high_hits == 1 {
        2
    } else if mid_hits >= 2 {
        2
    } else if mid_hits >= 1 {
        1
    } else {
        0
    }
}

fn score_freshness(date_str: &str) -> u8 {
    if date_str.is_empty() {
        return 1;
    }
    let parsed = DateTime::parse_from_rfc3339(date_str)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%SZ")
                .map(|dt| dt.and_utc())
        });

    let Ok(dt) = parsed else {
        return 1;
    };

    let age = Utc::now().signed_duration_since(dt);
    if age.num_hours() <= 24 {
        3
    } else if age.num_hours() <= 48 {
        2
    } else if age.num_hours() <= 72 {
        1
    } else {
        0
    }
}

fn score_uniqueness(link: &str, seen_db: &SeenDb) -> u8 {
    let domain = Url::parse(link)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_lowercase()));

    let Some(domain) = domain else {
        return 1;
    };

    let count = seen_db.domain_count(&domain);
    if count <= 2 {
        3
    } else if count <= 10 {
        2
    } else if count <= 30 {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn make_feed(name: &str, tier: Option<&str>, kind: Option<&str>) -> Feed {
        Feed {
            name: name.to_string(),
            url: String::new(),
            skip_filter: false,
            tier: tier.map(|s| s.to_string()),
            kind: kind.map(|s| s.to_string()),
        }
    }

    fn make_scoring(high: &[&str], mid: &[&str]) -> ScoringConfig {
        ScoringConfig {
            keywords_high: high.iter().map(|s| s.to_string()).collect(),
            keywords_mid: mid.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn make_entry(title: &str, desc: &str, link: &str, date: &str) -> Entry {
        Entry {
            title: title.to_string(),
            desc: desc.to_string(),
            link: link.to_string(),
            date: date.to_string(),
            image: None,
        }
    }

    fn empty_seen_db() -> SeenDb {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "").unwrap();
        SeenDb::load(f.path(), 90).unwrap()
    }

    // --- score_source ---

    #[test]
    fn test_tier_score_core() {
        assert_eq!(score_source(&make_feed("Blog", Some("core"), None)), 3);
    }

    #[test]
    fn test_tier_score_official() {
        assert_eq!(score_source(&make_feed("Blog", Some("official"), None)), 2);
    }

    #[test]
    fn test_tier_score_aggregator() {
        assert_eq!(
            score_source(&make_feed("Blog", Some("aggregator"), None)),
            1
        );
    }

    #[test]
    fn test_tier_score_release() {
        assert_eq!(score_source(&make_feed("Blog", Some("release"), None)), 0);
    }

    #[test]
    fn test_tier_score_infer_gh() {
        assert_eq!(score_source(&make_feed("GH:repo", None, None)), 0);
    }

    #[test]
    fn test_tier_score_infer_hn() {
        assert_eq!(score_source(&make_feed("HN:frontpage", None, None)), 1);
    }

    #[test]
    fn test_tier_score_infer_default() {
        assert_eq!(score_source(&make_feed("SomeBlog", None, None)), 1);
    }

    // --- score_keywords ---

    #[test]
    fn test_keyword_high_two_hits() {
        let scoring = make_scoring(&["claude code", "mcp"], &[]);
        assert_eq!(
            score_keywords("Claude Code adds MCP support", "", &scoring),
            3
        );
    }

    #[test]
    fn test_keyword_high_one_hit() {
        let scoring = make_scoring(&["claude code", "mcp"], &[]);
        assert_eq!(score_keywords("Claude Code update", "", &scoring), 2);
    }

    #[test]
    fn test_keyword_mid_two_hits() {
        let scoring = make_scoring(&[], &["agent", "benchmark"]);
        assert_eq!(
            score_keywords("Agent benchmark results", "", &scoring),
            2
        );
    }

    #[test]
    fn test_keyword_mid_one_hit() {
        let scoring = make_scoring(&[], &["agent", "benchmark"]);
        assert_eq!(score_keywords("New agent framework", "", &scoring), 1);
    }

    #[test]
    fn test_keyword_no_hits() {
        let scoring = make_scoring(&["claude code"], &["agent"]);
        assert_eq!(score_keywords("Weather forecast today", "", &scoring), 0);
    }

    // --- score_freshness ---

    #[test]
    fn test_freshness_today() {
        let now = Utc::now().to_rfc3339();
        assert_eq!(score_freshness(&now), 3);
    }

    #[test]
    fn test_freshness_yesterday() {
        let yesterday = (Utc::now() - chrono::Duration::hours(30)).to_rfc3339();
        assert_eq!(score_freshness(&yesterday), 2);
    }

    #[test]
    fn test_freshness_three_days() {
        let three_days = (Utc::now() - chrono::Duration::hours(60)).to_rfc3339();
        assert_eq!(score_freshness(&three_days), 1);
    }

    #[test]
    fn test_freshness_old() {
        let old = (Utc::now() - chrono::Duration::days(10)).to_rfc3339();
        assert_eq!(score_freshness(&old), 0);
    }

    #[test]
    fn test_freshness_empty_date() {
        assert_eq!(score_freshness(""), 1);
    }

    #[test]
    fn test_freshness_invalid_date() {
        assert_eq!(score_freshness("not-a-date"), 1);
    }

    // --- priority ---

    #[test]
    fn test_priority_p0() {
        let scoring = make_scoring(&["claude code", "mcp"], &[]);
        let feed = make_feed("Blog", Some("core"), None);
        let entry = make_entry(
            "Claude Code MCP filtering",
            "",
            "https://unique-domain.com/post",
            &Utc::now().to_rfc3339(),
        );
        let seen = empty_seen_db();
        let scored = score_entry(&entry, &feed, &scoring, &seen);
        // source=3, keyword=3, freshness=3, uniqueness=3 = 12 >= 9
        assert_eq!(scored.priority, Priority::P0);
        assert!(scored.score >= 9);
    }

    #[test]
    fn test_priority_p1() {
        let scoring = make_scoring(&["claude code"], &[]);
        let feed = make_feed("Blog", Some("aggregator"), None);
        let entry = make_entry(
            "Claude Code update",
            "",
            "https://unique.com/post",
            &(Utc::now() - chrono::Duration::hours(30)).to_rfc3339(),
        );
        let seen = empty_seen_db();
        let scored = score_entry(&entry, &feed, &scoring, &seen);
        // source=1, keyword=2, freshness=2, uniqueness=3 = 8 -> P1
        assert_eq!(scored.priority, Priority::P1);
        assert!(scored.score >= 6 && scored.score < 9);
    }

    #[test]
    fn test_priority_p2() {
        let scoring = make_scoring(&["claude code"], &[]);
        let feed = make_feed("GH:repo", None, None);
        let entry = make_entry(
            "Version bump",
            "",
            "https://github.com/repo/release",
            &(Utc::now() - chrono::Duration::days(10)).to_rfc3339(),
        );
        let seen = empty_seen_db();
        let scored = score_entry(&entry, &feed, &scoring, &seen);
        // source=0, keyword=0, freshness=0, uniqueness=3 = 3 -> P2
        assert_eq!(scored.priority, Priority::P2);
        assert!(scored.score < 6);
    }
}
