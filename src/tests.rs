// main.rs 流水线单元测试(自 main.rs 内联模块迁出,2026-08-23 重构)

use super::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn make_entry(link: &str) -> parser::Entry {
    parser::Entry {
        title: "Post".to_string(),
        desc: String::new(),
        link: link.to_string(),
        date: String::new(),
        image: None,
    }
}

fn make_feed() -> config::Feed {
    config::Feed {
        name: "Blog".to_string(),
        url: String::new(),
        skip_filter: false,
        tier: Some("aggregator".to_string()),
        kind: None,
        adapter: None,
        adapter_params: None,
        max_items: None,
        host_min_interval_seconds: None,
    }
}

fn make_scoring() -> config::ScoringConfig {
    config::ScoringConfig {
        keywords_high: Vec::new(),
        keywords_mid: Vec::new(),
    }
}

fn seen_with_links(links: &[&str]) -> dedup::SeenDb {
    let mut tmp = NamedTempFile::new().unwrap();
    write!(tmp, "").unwrap();

    let mut seen = dedup::SeenDb::load(tmp.path(), 90).unwrap();
    for link in links {
        seen.mark_seen(link);
    }
    seen
}

#[test]
fn current_run_links_do_not_affect_uniqueness_scoring() {
    let mut seen = seen_with_links(&[
        "https://example.com/history-1",
        "https://example.com/history-2",
    ]);
    let first_feed_entries = vec![make_entry("https://example.com/new-1")];
    let second_feed_entries = vec![make_entry("https://example.com/new-2")];
    let feed = make_feed();
    let scoring = make_scoring();
    let mut current_run_seen = HashSet::new();
    let mut pending_seen_links = Vec::new();

    let first_new = collect_new_entries(
        &first_feed_entries,
        &seen,
        &mut current_run_seen,
        &mut pending_seen_links,
    );
    let first_scored = scorer::score_entry(first_new[0], &feed, &scoring, &seen);

    let second_new = collect_new_entries(
        &second_feed_entries,
        &seen,
        &mut current_run_seen,
        &mut pending_seen_links,
    );
    let second_scored = scorer::score_entry(second_new[0], &feed, &scoring, &seen);

    assert_eq!(first_scored.breakdown[3], 3);
    assert_eq!(second_scored.breakdown[3], 3);
    assert_eq!(seen.domain_count("example.com"), 2);
    assert_eq!(pending_seen_links.len(), 2);

    for link in &pending_seen_links {
        seen.mark_seen(link);
    }

    assert_eq!(seen.domain_count("example.com"), 4);
}

#[test]
fn current_run_dedup_uses_normalized_urls_without_marking_seen() {
    let seen = seen_with_links(&[]);
    let entries = vec![
        make_entry("http://example.com/post?utm_source=rss"),
        make_entry("https://example.com/post"),
    ];
    let mut current_run_seen = HashSet::new();
    let mut pending_seen_links = Vec::new();

    let new_entries = collect_new_entries(
        &entries,
        &seen,
        &mut current_run_seen,
        &mut pending_seen_links,
    );

    assert_eq!(new_entries.len(), 1);
    assert_eq!(
        pending_seen_links,
        vec!["http://example.com/post?utm_source=rss"]
    );
    assert_eq!(seen.len(), 0);
}

#[test]
fn import_requires_existing_readable_config() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let opml_path = temp_dir.path().join("feeds.opml");
    let feeds_path = temp_dir.path().join("feeds.toml");
    fs::write(
        &opml_path,
        r#"<opml><body><outline text="Example" xmlUrl="https://example.com/feed.xml"/></body></opml>"#,
    )
    .expect("write opml");

    let error = run_import(&opml_path, true, &feeds_path).expect_err("missing config fails");

    assert!(error.to_string().contains("无法读取"));
    assert!(!feeds_path.exists());
}

#[test]
fn import_snippet_escapes_toml_strings() {
    let feed = opml::OpmlFeed {
        name: "Quote \"Feed\"".to_string(),
        url: "https://example.com/a?x=\"y\"".to_string(),
    };

    let snippet = format_import_snippet(&[(&feed, 1)]).expect("snippet");
    let parsed = toml::from_str::<config::Config>(&format!(
        r#"
[settings]
keywords = "rust"

{snippet}
"#
    ))
    .expect("escaped snippet parses");

    assert_eq!(parsed.feeds[0].name, feed.name);
    assert_eq!(parsed.feeds[0].url, feed.url);
}
