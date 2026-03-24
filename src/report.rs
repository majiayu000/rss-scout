use crate::parser::Entry;
use regex::Regex;
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

    pub fn write_section(
        &mut self,
        name: &str,
        entries: &[Entry],
    ) -> Result<(), Box<dyn Error>> {
        writeln!(self.file, "## {name} ({} 条新发现)", entries.len())?;
        writeln!(self.file)?;

        for entry in entries {
            writeln!(self.file, "- **{}**", entry.title)?;
            writeln!(self.file, "  {}", entry.link)?;
            if !entry.desc.is_empty() {
                let clean: String = HTML_TAG_RE
                    .replace_all(&entry.desc, "")
                    .chars()
                    .take(150)
                    .collect();
                if !clean.is_empty() {
                    writeln!(self.file, "  _{clean}..._")?;
                }
            }
            if !entry.date.is_empty() {
                writeln!(self.file, "  `{}`", entry.date)?;
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
