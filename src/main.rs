mod config;
mod dedup;
mod fetcher;
mod filter;
mod parser;
mod report;

use clap::{Parser, Subcommand};
use regex::Regex;
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
            list_feeds(&feeds_path);
        }
    }
}

fn run(dry_run: bool, feeds_path: &Path, data_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = data_dir.join("output");
    fs::create_dir_all(&output_dir)?;

    let cfg = config::load(feeds_path)?;
    let keywords_re = Regex::new(&cfg.settings.keywords)?;

    let seen_path = data_dir.join("seen.txt");
    let mut seen = dedup::SeenDb::load(&seen_path, cfg.settings.seen_expire_days)?;

    let agent = fetcher::new_agent();

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let report_path = output_dir.join(format!("scout-{today}.md"));
    let mut rpt = report::Report::new(&report_path, &cfg.settings.keywords, dry_run)?;

    let mut total_count: usize = 0;
    let mut new_count: usize = 0;

    for feed in &cfg.feeds {
        log(&format!("采集: {}", feed.name));

        let body = match fetcher::fetch(&agent, &feed.url) {
            Ok(b) => b,
            Err(e) => {
                log(&format!("  SKIP (下载失败): {} — {e}", feed.name));
                continue;
            }
        };

        if body.is_empty() {
            log(&format!("  SKIP (空响应): {}", feed.name));
            continue;
        }

        let entries = parser::parse(&body, cfg.settings.max_items);
        let raw_count = entries.len();
        log(&format!("  解析 {raw_count} 条"));
        total_count += raw_count;

        let new_entries: Vec<_> = entries
            .into_iter()
            .filter(|e| !seen.is_seen(&e.link))
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

        if new_entries.is_empty() {
            continue;
        }

        for entry in &new_entries {
            seen.mark_seen(&entry.link);
        }
        new_count += new_entries.len();

        rpt.write_section(&feed.name, &new_entries)?;
    }

    rpt.write_footer(total_count, new_count, seen.len())?;

    if !dry_run {
        seen.save(&seen_path)?;
    }

    log(&format!("完成: {new_count} 新 / {total_count} 总"));
    log(&format!("报告: {}", report_path.display()));
    println!("{}", report_path.display());

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
        "📡 Knowledge Scout: {date_part} 发现 {new_count} 条新内容，运行 /knowledge-scout 查看分析"
    );
    let _ = fs::write(&read_marker, latest.to_string_lossy().as_ref());
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

fn list_feeds(feeds_path: &Path) {
    let cfg = match config::load(feeds_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[ERROR] 加载 feeds 失败: {e}");
            return;
        }
    };
    for (i, feed) in cfg.feeds.iter().enumerate() {
        let tag = if feed.skip_filter { "skip" } else { "filter" };
        println!("{:>3}. [{tag}] {} — {}", i + 1, feed.name, feed.url);
    }
    println!("\n共 {} 个源", cfg.feeds.len());
}
