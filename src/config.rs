use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub settings: Settings,
    #[serde(default)]
    pub notion: Option<NotionConfig>,
    pub feeds: Vec<Feed>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct NotionConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub database_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    // feeds-tools.toml 无 keywords(工具源全量保留),default 为空正则=不过滤
    #[serde(default)]
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
// 透传字段由下游管线消费,本仓二进制不读取——dead_code 是声明本身的目的
#[allow(dead_code)]
pub struct Feed {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub skip_filter: bool,
    #[serde(default)]
    pub tier: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    // --- 透传字段:由下游 knowledge-scout 管线消费,本仓二进制暂不实现,
    //     仅声明以保证 feeds.toml 可解析(2026-08-23 审计:schema 漂移导致自家配置解析失败) ---
    /// API 适配器名(hn_algolia / arxiv_api / github_api / openalex)
    #[serde(default)]
    pub adapter: Option<String>,
    /// 适配器自由参数(query/min_points/mode/search 等)
    #[serde(default)]
    pub adapter_params: Option<toml::Value>,
    /// 每源条目上限(覆盖 settings.max_items)
    #[serde(default)]
    pub max_items: Option<usize>,
    /// 同 host 最小请求间隔(秒),限速用
    #[serde(default)]
    pub host_min_interval_seconds: Option<u64>,
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
    use super::*;

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
        assert!(config.notion.is_none());
        assert_eq!(config.feeds.len(), 1);
    }

    #[test]
    fn parses_enabled_notion_section() {
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
        .expect("notion config should parse");

        let notion = config.notion.as_ref().unwrap();
        assert!(notion.enabled);
        assert_eq!(notion.database_id, "db_123");
    }

    /// 契约测试:仓库内两份真实配置必须始终能被 config::load 解析。
    /// 2026-08-23 审计发现 adapter 等字段曾使本仓二进制无法解析自家配置而无人察觉。
    #[test]
    fn repo_feed_configs_parse() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        for file in ["feeds.toml", "feeds-tools.toml"] {
            let cfg = load(&root.join(file))
                .unwrap_or_else(|e| panic!("{file} 应能被 config::load 解析: {e}"));
            assert!(!cfg.feeds.is_empty(), "{file} 应至少包含一个源");
        }
    }
}
