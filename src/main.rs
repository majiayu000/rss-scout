mod config;
mod dedup;
mod discover;
mod fetcher;
mod filter;
mod notion;
mod opml;
mod parser;
mod report;
mod scorer;

use clap::{Parser, Subcommand};
use rayon::prelude::*;
use regex::Regex;
use scorer::{Priority, ScoredEntry};
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "rss-scout", about = "Zero-API knowledge discovery for AI dev")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch, filter, and generate report
    Run {
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        feeds: Option<PathBuf>,
        #[arg(long)]
        data_dir: Option<PathBuf>,
    },
    /// Check for unread reports
    Check {
        #[arg(long)]
        data_dir: Option<PathBuf>,
    },
    /// List all configured feeds
    Feeds {
        #[arg(long)]
        feeds: Option<PathBuf>,
        #[arg(long)]
        data_dir: Option<PathBuf>,
    },
    /// Auto-discover new RSS feeds from recent report links
    Discover {
        /// Single URL to check for RSS feed
        #[arg(long)]
        url: Option<String>,
        /// Scan last N days of reports (default: 7)
        #[arg(long, default_value = "7")]
        days: usize,
        #[arg(long)]
        feeds: Option<PathBuf>,
        #[arg(long)]
        data_dir: Option<PathBuf>,
    },
    /// Import feeds from OPML file
    Import {
        /// Path to OPML file
        file: PathBuf,
        /// Only print, don't write to feeds.toml
        #[arg(long)]
        dry_run: bool,
        /// Path to feeds.toml
        #[arg(long)]
        feeds: Option<PathBuf>,
        #[arg(long)]
        data_dir: Option<PathBuf>,
    },
}

fn default_data_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".rss-scout")
}

fn resolve_feeds_path(feeds_arg: Option<&Path>, data_dir: &Path) -> PathBuf {
    feeds_arg
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| data_dir.join("feeds.toml"))
}

fn log(msg: &str) {
    eprintln!("[{}] {msg}", chrono::Local::now().format("%H:%M:%S"));
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run {
            dry_run,
            feeds,
            data_dir,
        } => {
            let data_dir = data_dir.unwrap_or_else(default_data_dir);
            let feeds_path = resolve_feeds_path(feeds.as_deref(), &data_dir);
            if let Err(e) = run(dry_run, &feeds_path, &data_dir) {
                eprintln!("[ERROR] {e}");
                std::process::exit(1);
            }
        }
        Commands::Check { data_dir } => {
            let data_dir = data_dir.unwrap_or_else(default_data_dir);
            check(&data_dir);
        }
        Commands::Feeds { feeds, data_dir } => {
            let data_dir = data_dir.unwrap_or_else(default_data_dir);
            let feeds_path = resolve_feeds_path(feeds.as_deref(), &data_dir);
            if let Err(e) = list_feeds(&feeds_path) {
                eprintln!("[ERROR] 加载 feeds 失败: {e}");
                std::process::exit(1);
            }
        }
        Commands::Discover {
            url,
            days,
            feeds,
            data_dir,
        } => {
            let data_dir = data_dir.unwrap_or_else(default_data_dir);
            let feeds_path = resolve_feeds_path(feeds.as_deref(), &data_dir);
            if let Some(url) = url {
                run_discover_url(&url);
            } else {
                run_discover_from_reports(&data_dir, &feeds_path, days);
            }
        }
        Commands::Import {
            file,
            dry_run,
            feeds,
            data_dir,
        } => {
            let data_dir = data_dir.unwrap_or_else(default_data_dir);
            let feeds_path = resolve_feeds_path(feeds.as_deref(), &data_dir);
            if let Err(e) = run_import(&file, dry_run, &feeds_path) {
                eprintln!("[ERROR] {e}");
                std::process::exit(1);
            }
        }
    }
}

/// Parallel fetch result — carries original index for deterministic ordering
struct FetchResult {
    index: usize,
    name: String,
    tier: Option<String>,
    kind: Option<String>,
    skip_filter: bool,
    raw_count: usize,
    entries: Vec<parser::Entry>,
}

fn collect_new_entries<'a>(
    entries: &'a [parser::Entry],
    seen: &dedup::SeenDb,
    current_run_seen: &mut HashSet<String>,
    pending_seen_links: &mut Vec<String>,
) -> Vec<&'a parser::Entry> {
    let mut new_entries = Vec::new();

    for entry in entries {
        if seen.is_seen(&entry.link) {
            continue;
        }

        let normalized = dedup::normalize_url(&entry.link);
        if !current_run_seen.insert(normalized) {
            continue;
        }

        pending_seen_links.push(entry.link.clone());
        new_entries.push(entry);
    }

    new_entries
}

fn run(
    dry_run: bool,
    feeds_path: &Path,
    data_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = data_dir.join("output");
    fs::create_dir_all(&output_dir)?;

    let cfg = config::load(feeds_path)?;
    let keywords_re = Regex::new(&cfg.settings.keywords)?;

    let seen_path = data_dir.join("seen.txt");
    let mut seen = dedup::SeenDb::load(&seen_path, cfg.settings.seen_expire_days)?;

    let agent = fetcher::new_agent();
    let max_items = cfg.settings.max_items;

    // Phase 1 (parallel): fetch + parse + keyword filter
    log("开始并行采集...");
    // 失败显式化:并行阶段收集每源失败记录,报告尾部显式列出(2026-08-23 审计主线)
    let failures_mutex = std::sync::Mutex::new(Vec::<report::FetchFailure>::new());
    let mut results: Vec<FetchResult> = cfg
        .feeds
        .par_iter()
        .enumerate()
        .filter_map(|(index, feed)| {
            eprintln!(
                "[{}] 采集: {}",
                chrono::Local::now().format("%H:%M:%S"),
                feed.name
            );

            let body = match fetcher::fetch(&agent, &feed.url) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!(
                        "[{}]   SKIP (下载失败): {} — {e}",
                        chrono::Local::now().format("%H:%M:%S"),
                        feed.name
                    );
                    failures_mutex
                        .lock()
                        .expect("failures_mutex poisoned")
                        .push(report::FetchFailure {
                            index,
                            name: feed.name.clone(),
                            reason: format!("下载失败: {e}"),
                        });
                    return None;
                }
            };

            if body.is_empty() {
                eprintln!(
                    "[{}]   SKIP (空响应): {}",
                    chrono::Local::now().format("%H:%M:%S"),
                    feed.name
                );
                failures_mutex
                    .lock()
                    .expect("failures_mutex poisoned")
                    .push(report::FetchFailure {
                        index,
                        name: feed.name.clone(),
                        reason: "空响应".to_string(),
                    });
                return None;
            }

            let entries = parser::parse(&body, max_items);
            let raw_count = entries.len();

            // Keyword filter (no SeenDb access — dedup deferred to serial phase)
            let filtered: Vec<parser::Entry> = entries
                .into_iter()
                .filter(|e| {
                    if feed.skip_filter {
                        return true;
                    }
                    if filter::is_arxiv_source(&feed.name) {
                        filter::passes_arxiv_filter(e)
                    } else {
                        filter::passes_keyword_filter(e, &keywords_re)
                    }
                })
                .collect();

            Some(FetchResult {
                index,
                name: feed.name.clone(),
                tier: feed.tier.clone(),
                kind: feed.kind.clone(),
                skip_filter: feed.skip_filter,
                raw_count,
                entries: filtered,
            })
        })
        .collect();

    // Restore deterministic order by original index
    results.sort_by_key(|r| r.index);
    failures_mutex
        .lock()
        .expect("failures_mutex poisoned")
        .sort_by_key(|f| f.index);

    // 守卫:全部源失败时不得生成空报告覆盖当日已有文件,显式失败(审计 #5)
    if results.is_empty() && !cfg.feeds.is_empty() {
        let locked = failures_mutex.lock().expect("failures_mutex poisoned");
        let names: Vec<&str> = locked.iter().map(|f| f.name.as_str()).collect();
        return Err(format!(
            "全部 {} 个源采集失败，放弃生成报告(不覆盖当日已有文件)。失败源: {}",
            cfg.feeds.len(),
            names.join("、")
        )
        .into());
    }

    // 取出失败清单供报告小节与汇总日志使用
    let failures = failures_mutex
        .into_inner()
        .expect("failures_mutex poisoned");
    if !failures.is_empty() {
        log(&format!(
            "⚠ 采集失败 {}/{} 源",
            failures.len(),
            cfg.feeds.len()
        ));
    }

    // Phase 2 (serial): dedup + score + report write
    let mut all_scored: Vec<ScoredEntry> = Vec::new();
    let mut total_count: usize = 0;
    let mut current_run_seen: HashSet<String> = HashSet::new();
    let mut pending_seen_links: Vec<String> = Vec::new();

    for result in &results {
        total_count += result.raw_count;
        eprintln!(
            "[{}]   解析 {} 条 ({})",
            chrono::Local::now().format("%H:%M:%S"),
            result.raw_count,
            result.name
        );

        let new_entries = collect_new_entries(
            &result.entries,
            &seen,
            &mut current_run_seen,
            &mut pending_seen_links,
        );

        if new_entries.is_empty() {
            continue;
        }

        // Build a temporary Feed-like struct for scoring
        let feed_ref = config::Feed {
            name: result.name.clone(),
            url: String::new(),
            skip_filter: result.skip_filter,
            tier: result.tier.clone(),
            kind: result.kind.clone(),
            adapter: None,
            adapter_params: None,
            max_items: None,
            host_min_interval_seconds: None,
        };

        for entry in &new_entries {
            all_scored.push(scorer::score_entry(
                entry,
                &feed_ref,
                &cfg.settings.scoring,
                &seen,
            ));
        }
    }

    for link in &pending_seen_links {
        seen.mark_seen(link);
    }

    // Sort by score descending
    all_scored.sort_by_key(|entry| std::cmp::Reverse(entry.score));

    let new_count = all_scored.len();

    // Render report
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let report_path = output_dir.join(format!("scout-{today}.md"));
    let mut rpt = report::Report::new(&report_path, &cfg.settings.keywords, dry_run)?;

    let p0_items: Vec<&ScoredEntry> = all_scored
        .iter()
        .filter(|e| e.priority == Priority::P0)
        .collect();
    rpt.write_summary(&p0_items)?;
    rpt.write_priority_sections(&all_scored)?;
    rpt.write_changelog_compact(&all_scored)?;
    rpt.write_failures(cfg.feeds.len(), &failures)?;
    rpt.write_footer(total_count, new_count, seen.len())?;

    if !dry_run {
        // 先同步 Notion 再落盘 seen:Notion 失败时 seen 尚未写盘,重跑即可补发,
        // 旧顺序下失败条目已被去重库挡住会永久漏发(2026-08-23 审计 #7)
        maybe_sync_notion(&cfg, &all_scored)?;
        seen.save(&seen_path)?;
    }

    log(&format!("完成: {new_count} 新 / {total_count} 总"));
    log(&format!("报告: {}", report_path.display()));
    println!("{}", report_path.display());

    Ok(())
}

#[cfg(test)]
mod tests;

fn maybe_sync_notion(
    cfg: &config::Config,
    all_scored: &[ScoredEntry],
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(notion_cfg) = cfg.notion.as_ref() else {
        return Ok(());
    };
    if !notion_cfg.enabled {
        return Ok(());
    }
    if notion_cfg.database_id.trim().is_empty() {
        return Err("Notion 已启用，但 database_id 为空".into());
    }

    let api_key = std::env::var("NOTION_API_KEY")
        .map_err(|_| "Notion 已启用，但环境变量 NOTION_API_KEY 缺失")?;
    if api_key.trim().is_empty() {
        return Err("Notion 已启用，但环境变量 NOTION_API_KEY 为空".into());
    }

    let notion_client = notion::NotionClient::new(api_key, notion_cfg.database_id.clone());
    let today = chrono::Local::now().date_naive();
    match notion_client.sync_daily_summary(all_scored, today)? {
        notion::SyncOutcome::Created => log("Notion: 已创建当日摘要页"),
        notion::SyncOutcome::PageExists => log("Notion: 当日页面已存在，幂等跳过"),
        notion::SyncOutcome::NoHighPriority => log("Notion: 无 P0/P1 条目，跳过"),
    }
    Ok(())
}

fn check(data_dir: &Path) {
    let output_dir = data_dir.join("output");
    let read_marker = data_dir.join(".last-read");

    let latest = match find_latest_report(&output_dir) {
        Some(p) => p,
        None => return,
    };

    if let Ok(last_read) = fs::read_to_string(&read_marker) {
        if last_read.trim() == latest.to_string_lossy().as_ref() {
            return;
        }
    }

    let content = fs::read_to_string(&latest).unwrap_or_default();
    let new_count = report::extract_new_count(&content);
    if new_count == 0 {
        return;
    }

    let date_part = latest
        .file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| s.strip_prefix("scout-"))
        .unwrap_or("");

    eprintln!(
        "Knowledge Scout: {date_part} 发现 {new_count} 条新内容，运行 /knowledge-scout 查看分析"
    );
    if let Err(e) = fs::write(&read_marker, latest.to_string_lossy().as_ref()) {
        eprintln!("[WARN] 写入 .last-read 失败: {e}");
    }
}

fn find_latest_report(output_dir: &Path) -> Option<PathBuf> {
    let mut reports: Vec<PathBuf> = fs::read_dir(output_dir)
        .ok()?
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
    reports.last().cloned()
}

fn list_feeds(feeds_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let cfg = config::load(feeds_path)?;
    for (i, feed) in cfg.feeds.iter().enumerate() {
        let tag = if feed.skip_filter { "skip" } else { "filter" };
        let tier = feed.tier.as_deref().unwrap_or("-");
        let kind = feed.kind.as_deref().unwrap_or("-");
        println!(
            "{:>3}. [{tag}] [{tier}/{kind}] {} — {}",
            i + 1,
            feed.name,
            feed.url
        );
    }
    println!("\n共 {} 个源", cfg.feeds.len());
    Ok(())
}

fn run_discover_url(url: &str) {
    eprintln!("探测 {url} ...");
    match discover::discover_url(url) {
        Ok(Some(c)) => {
            println!("发现 RSS feed:");
            println!("   名称: {}", c.title);
            println!("   URL:  {}", c.feed_url);
            println!("   条目: {} 条", c.entry_count);
            println!("\n添加到 feeds.toml:");
            println!("[[feeds]]");
            println!("name = \"{}\"", c.domain);
            println!("url = \"{}\"", c.feed_url);
        }
        Ok(None) => {
            eprintln!("未找到 RSS feed");
        }
        Err(e) => {
            eprintln!("[ERROR] {e}");
            std::process::exit(1);
        }
    }
}

fn run_discover_from_reports(data_dir: &Path, feeds_path: &Path, days: usize) {
    eprintln!("扫描最近 {days} 天报告中的外部链接...");
    match discover::discover(data_dir, feeds_path, days) {
        Ok(candidates) => {
            if candidates.is_empty() {
                println!("未发现新的 RSS 源");
                return;
            }
            println!("\n发现 {} 个新 RSS 源:\n", candidates.len());
            for (i, c) in candidates.iter().enumerate() {
                println!(
                    "{:>3}. {} — {} ({} 条)",
                    i + 1,
                    c.domain,
                    c.feed_url,
                    c.entry_count
                );
                println!("     标题: {}", c.title);
            }
            println!("\n--- feeds.toml 片段 ---\n");
            for c in &candidates {
                println!("[[feeds]]");
                println!("name = \"{}\"", c.domain);
                println!("url = \"{}\"", c.feed_url);
                println!();
            }
        }
        Err(e) => {
            eprintln!("[ERROR] {e}");
            std::process::exit(1);
        }
    }
}

fn run_import(
    opml_path: &Path,
    dry_run: bool,
    feeds_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let opml_content = fs::read_to_string(opml_path)
        .map_err(|e| format!("无法读取 {}: {e}", opml_path.display()))?;

    let candidates = opml::parse_opml(&opml_content);
    if candidates.is_empty() {
        eprintln!("OPML 中未找到任何 feed");
        return Ok(());
    }
    log(&format!("OPML 解析到 {} 个 feed", candidates.len()));

    // Import appends to the full config file, so validate it before fetch work.
    let cfg = config::load(feeds_path)?;
    let existing_domains: std::collections::HashSet<String> = cfg
        .feeds
        .iter()
        .filter_map(|f| url::Url::parse(&f.url).ok())
        .filter_map(|u| u.host_str().map(|h| h.to_lowercase()))
        .collect();

    // Filter out already-known domains
    let new_candidates: Vec<&opml::OpmlFeed> = candidates
        .iter()
        .filter(|c| {
            url::Url::parse(&c.url)
                .ok()
                .and_then(|u| u.host_str().map(|h| h.to_lowercase()))
                .map(|d| !existing_domains.contains(&d))
                .unwrap_or(false)
        })
        .collect();

    if new_candidates.is_empty() {
        eprintln!("所有 feed 域名已存在于 feeds.toml 中");
        return Ok(());
    }

    log(&format!(
        "去重后 {} 个新 feed，开始验证...",
        new_candidates.len()
    ));

    // Validate each candidate
    let agent = fetcher::new_agent();
    let mut valid: Vec<(&opml::OpmlFeed, usize)> = Vec::new();

    for candidate in &new_candidates {
        match fetcher::fetch(&agent, &candidate.url) {
            Ok(body) => {
                let entries = parser::parse(&body, 10);
                if entries.is_empty() {
                    eprintln!("  SKIP (无条目): {} — {}", candidate.name, candidate.url);
                } else {
                    eprintln!(
                        "  OK ({} 条): {} — {}",
                        entries.len(),
                        candidate.name,
                        candidate.url
                    );
                    valid.push((candidate, entries.len()));
                }
            }
            Err(e) => {
                eprintln!(
                    "  SKIP (获取失败): {} — {} — {e}",
                    candidate.name, candidate.url
                );
            }
        }
    }

    if valid.is_empty() {
        eprintln!("没有可用的新 feed");
        return Ok(());
    }

    let snippet = format_import_snippet(&valid)?;

    if dry_run {
        println!(
            "--- dry-run: 以下内容将追加到 {} ---\n",
            feeds_path.display()
        );
        println!("{snippet}");
        println!("共 {} 个有效 feed", valid.len());
    } else {
        let mut existing = fs::read_to_string(feeds_path)
            .map_err(|e| format!("无法读取 {}: {e}", feeds_path.display()))?;
        if !existing.ends_with('\n') {
            existing.push('\n');
        }
        existing.push_str(&snippet);
        fs::write(feeds_path, existing)?;
        log(&format!(
            "已追加 {} 个 feed 到 {}",
            valid.len(),
            feeds_path.display()
        ));
    }

    Ok(())
}

#[derive(Serialize)]
struct ImportFeeds<'a> {
    feeds: Vec<ImportFeed<'a>>,
}

#[derive(Serialize)]
struct ImportFeed<'a> {
    name: &'a str,
    url: &'a str,
}

fn format_import_snippet(
    valid: &[(&opml::OpmlFeed, usize)],
) -> Result<String, Box<dyn std::error::Error>> {
    let feeds = valid
        .iter()
        .map(|(feed, _count)| ImportFeed {
            name: &feed.name,
            url: &feed.url,
        })
        .collect();
    let mut snippet = String::from("\n# OPML 导入\n");
    snippet.push_str(&toml::to_string(&ImportFeeds { feeds })?);
    Ok(snippet)
}
