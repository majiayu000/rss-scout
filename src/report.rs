use crate::scorer::{Priority, ScoredEntry};
use regex::Regex;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::LazyLock;

static HTML_TAG_RE: LazyLock<Regex> = LazyLock::new(|| match Regex::new(r"<[^>]+>") {
    Ok(r) => r,
    Err(e) => {
        eprintln!("[FATAL] HTML_TAG_RE regex compile failed: {e}");
        std::process::exit(1);
    }
});

pub struct Report {
    file: File,
}

impl Report {
    pub fn new(path: &Path, keywords: &str, dry_run: bool) -> Result<Self, Box<dyn Error>> {
        let mut file = File::create(path)?;
        let today = chrono::Local::now().format("%Y-%m-%d");
        let mode = if dry_run { "dry-run" } else { "production" };
        let kw_display = keywords.replace('|', " | ");

        writeln!(file, "# Knowledge Scout — {today}")?;
        writeln!(file)?;
        writeln!(file, "> 关键词: `{kw_display}`")?;
        writeln!(file, "> 模式: {mode}")?;
        writeln!(file)?;

        Ok(Report { file })
    }

    pub fn write_summary(&mut self, entries: &[&ScoredEntry]) -> Result<(), Box<dyn Error>> {
        if entries.is_empty() {
            return Ok(());
        }
        writeln!(self.file, "## 今日亮点")?;
        for (i, se) in entries.iter().enumerate() {
            writeln!(
                self.file,
                "{}. [{}] {}",
                i + 1,
                se.feed_name,
                se.entry.title
            )?;
        }
        writeln!(self.file)?;
        Ok(())
    }

    pub fn write_priority_sections(&mut self, all: &[ScoredEntry]) -> Result<(), Box<dyn Error>> {
        for &prio in &[Priority::P0, Priority::P1, Priority::P2] {
            let items: Vec<&ScoredEntry> = all.iter().filter(|e| e.priority == prio).collect();
            if items.is_empty() {
                continue;
            }
            let (label, is_folded) = match prio {
                Priority::P0 => ("P0 高价值", false),
                Priority::P1 => ("P1 值得关注", false),
                Priority::P2 => ("P2 低优先级", true),
            };

            if is_folded {
                writeln!(
                    self.file,
                    "<details><summary>{label} ({} 条)</summary>\n",
                    items.len()
                )?;
            } else {
                writeln!(self.file, "## {label} ({} 条)", items.len())?;
                writeln!(self.file)?;
            }

            // Group by feed_name
            let mut groups: BTreeMap<&str, Vec<&ScoredEntry>> = BTreeMap::new();
            for item in &items {
                groups.entry(&item.feed_name).or_default().push(item);
            }

            for (feed_name, feed_items) in &groups {
                writeln!(self.file, "### {feed_name} ({} 条)", feed_items.len())?;
                for se in feed_items {
                    let domain = extract_domain(&se.entry.link);
                    writeln!(
                        self.file,
                        "- **[{}] {}** — {domain}",
                        se.score, se.entry.title
                    )?;
                    writeln!(self.file, "  {}", se.entry.link)?;
                    write_desc(&mut self.file, &se.entry.desc)?;
                    writeln!(
                        self.file,
                        "  源:{} 词:{} 鲜:{} 独:{}",
                        se.breakdown[0], se.breakdown[1], se.breakdown[2], se.breakdown[3]
                    )?;
                    if let Some(ref img) = se.entry.image {
                        writeln!(self.file, "  img:{img}")?;
                    }
                }
            }
            writeln!(self.file)?;

            if is_folded {
                writeln!(self.file, "</details>\n")?;
            }
        }
        Ok(())
    }

    pub fn write_changelog_compact(&mut self, all: &[ScoredEntry]) -> Result<(), Box<dyn Error>> {
        let changelogs: BTreeMap<&str, Vec<&ScoredEntry>> = all
            .iter()
            .filter(|e| e.feed_kind.as_deref() == Some("changelog"))
            .fold(BTreeMap::new(), |mut acc, e| {
                acc.entry(e.feed_name.as_str()).or_default().push(e);
                acc
            });

        if changelogs.is_empty() {
            return Ok(());
        }

        writeln!(self.file, "## 版本追踪")?;
        for (name, items) in &changelogs {
            if items.len() == 1 {
                writeln!(self.file, "- **{name}**: {}", items[0].entry.title)?;
            } else {
                let first = &items
                    .last()
                    .map(|e| &e.entry.title)
                    .unwrap_or(&items[0].entry.title);
                let last = &items
                    .first()
                    .map(|e| &e.entry.title)
                    .unwrap_or(&items[0].entry.title);
                writeln!(
                    self.file,
                    "- **{name}**: {first}~{last} ({} releases)",
                    items.len()
                )?;
            }
        }
        writeln!(self.file)?;
        Ok(())
    }

    pub fn write_footer(
        &mut self,
        total: usize,
        new: usize,
        seen_count: usize,
    ) -> Result<(), Box<dyn Error>> {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(self.file, "---")?;
        writeln!(self.file)?;
        writeln!(
            self.file,
            "**统计**: 扫描 {total} 条 / 新发现 {new} 条 / 去重库 {seen_count} 条"
        )?;
        writeln!(self.file)?;
        writeln!(self.file, "_生成时间: {now}_")?;
        Ok(())
    }
}

fn write_desc(file: &mut File, desc: &str) -> std::io::Result<()> {
    if desc.is_empty() {
        return Ok(());
    }
    let clean: String = HTML_TAG_RE
        .replace_all(desc, "")
        .chars()
        .take(150)
        .collect();
    if !clean.is_empty() {
        writeln!(file, "  _{clean}..._")?;
    }
    Ok(())
}

fn extract_domain(link: &str) -> String {
    url::Url::parse(link)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_default()
}

pub fn extract_new_count(content: &str) -> usize {
    for line in content.lines() {
        if line.starts_with("**统计**") {
            if let Some(s) = line.split("新发现 ").nth(1) {
                if let Some(n) = s.split(' ').next() {
                    return n.parse().unwrap_or(0);
                }
            }
        }
    }
    0
}
