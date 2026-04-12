use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    pub settings: Settings,
    pub feeds: Vec<Feed>,
}

#[derive(Deserialize)]
pub struct Settings {
    pub keywords: String,
    #[serde(default = "default_max_items")]
    pub max_items: usize,
    #[serde(default = "default_expire_days")]
    pub seen_expire_days: u64,
    #[serde(default)]
    pub scoring: ScoringConfig,
}

#[derive(Deserialize, Default)]
pub struct ScoringConfig {
    #[serde(default)]
    pub keywords_high: Vec<String>,
    #[serde(default)]
    pub keywords_mid: Vec<String>,
}

#[derive(Deserialize)]
pub struct Feed {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub skip_filter: bool,
    #[serde(default)]
    pub tier: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
}

fn default_max_items() -> usize {
    30
}

fn default_expire_days() -> u64 {
    90
}

pub fn load(path: &Path) -> Result<Config, Box<dyn Error>> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("无法读取 {}: {e}", path.display()))?;
    let config: Config = toml::from_str(&content)
        .map_err(|e| format!("解析 {}: {e}", path.display()))?;
    Ok(config)
}
