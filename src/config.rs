use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub settings: Settings,
    pub feeds: Vec<Feed>,
    #[serde(default, rename = "notion")]
    _legacy_notion: Option<LegacyNotionConfig>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyNotionConfig {
    #[serde(rename = "enabled")]
    _enabled: bool,
    #[serde(rename = "database_id")]
    _database_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct ScoringConfig {
    #[serde(default)]
    pub keywords_high: Vec<String>,
    #[serde(default)]
    pub keywords_mid: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
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
    let content =
        fs::read_to_string(path).map_err(|e| format!("无法读取 {}: {e}", path.display()))?;
    let config: Config =
        toml::from_str(&content).map_err(|e| format!("解析 {}: {e}", path.display()))?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn parses_valid_config() {
        let config = toml::from_str::<Config>(
            r#"
[settings]
keywords = "claude"

[[feeds]]
name = "Example"
url = "https://example.com/feed.xml"
"#,
        )
        .expect("valid config should parse");

        assert_eq!(config.settings.keywords, "claude");
        assert_eq!(config.feeds.len(), 1);
    }

    #[test]
    fn accepts_legacy_notion_section() {
        let config = toml::from_str::<Config>(
            r#"
[settings]
keywords = "claude"

[notion]
enabled = true
database_id = "db_123"

[[feeds]]
name = "Example"
url = "https://example.com/feed.xml"
"#,
        )
        .expect("legacy notion config should still parse");

        assert_eq!(config.settings.keywords, "claude");
        assert_eq!(config.feeds.len(), 1);
    }
}
