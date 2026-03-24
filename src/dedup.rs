use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use url::Url;

const STRIP_PARAMS: &[&str] = &[
    "utm_source",
    "utm_medium",
    "utm_campaign",
    "utm_term",
    "utm_content",
    "ref",
    "source",
    "via",
    "fbclid",
    "gclid",
    "mc_cid",
    "mc_eid",
];

pub struct SeenDb {
    entries: HashMap<String, String>,
}

impl SeenDb {
    pub fn load(path: &Path, expire_days: u64) -> Result<Self, Box<dyn Error>> {
        let mut entries = HashMap::new();
        let cutoff = Utc::now() - Duration::days(expire_days as i64);
        let mut expired = 0usize;
        let mut migrated = 0usize;

        if let Ok(file) = fs::File::open(path) {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                let line = line.trim().to_string();
                if line.is_empty() {
                    continue;
                }

                let (url_raw, ts) = if let Some(idx) = line.find('|') {
                    (line[..idx].to_string(), line[idx + 1..].to_string())
                } else {
                    migrated += 1;
                    (line, String::new())
                };

                if !ts.is_empty() {
                    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&ts) {
                        if dt < cutoff {
                            expired += 1;
                            continue;
                        }
                    } else if let Ok(dt) =
                        chrono::NaiveDateTime::parse_from_str(&ts, "%Y-%m-%dT%H:%M:%SZ")
                    {
                        if dt.and_utc() < cutoff {
                            expired += 1;
                            continue;
                        }
                    }
                }

                entries.insert(normalize_url(&url_raw), ts);
            }
        }

        if expired > 0 {
            eprintln!("[dedup] 过期移除 {expired} 条 (>{expire_days}天)");
        }
        if migrated > 0 {
            eprintln!("[dedup] 旧格式迁移 {migrated} 条");
        }

        Ok(SeenDb { entries })
    }

    pub fn is_seen(&self, url: &str) -> bool {
        self.entries.contains_key(&normalize_url(url))
    }

    pub fn mark_seen(&mut self, url: &str) {
        let norm = normalize_url(url);
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        self.entries.insert(norm, now);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let now_iso = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let mut urls: Vec<&String> = self.entries.keys().collect();
        urls.sort();

        let mut content = String::new();
        for url in urls {
            let ts = match self.entries.get(url) {
                Some(v) => v,
                None => continue,
            };
            let ts = if ts.is_empty() { &now_iso } else { ts };
            content.push_str(url);
            content.push('|');
            content.push_str(ts);
            content.push('\n');
        }
        fs::write(path, content)?;
        eprintln!(
            "[dedup] 库大小 {} 条",
            self.entries.len()
        );
        Ok(())
    }
}

pub fn normalize_url(raw: &str) -> String {
    let Ok(parsed) = Url::parse(raw) else {
        return raw.to_lowercase();
    };

    let scheme = match parsed.scheme() {
        "http" | "https" => "https",
        s => s,
    };

    let host = parsed.host_str().unwrap_or("");
    let port = parsed.port().map(|p| format!(":{p}")).unwrap_or_default();

    let mut path = parsed.path().to_string();
    if path.len() > 1 && path.ends_with('/') {
        path.pop();
    }

    let mut params: Vec<(String, String)> = parsed
        .query_pairs()
        .filter(|(k, _)| !STRIP_PARAMS.contains(&k.to_lowercase().as_str()))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    params.sort();

    let query = if params.is_empty() {
        String::new()
    } else {
        let q: String = params
            .iter()
            .map(|(k, v)| {
                if v.is_empty() {
                    k.clone()
                } else {
                    format!("{k}={v}")
                }
            })
            .collect::<Vec<_>>()
            .join("&");
        format!("?{q}")
    };

    format!("{scheme}://{host}{port}{path}{query}")
}
