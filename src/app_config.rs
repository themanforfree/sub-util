use serde::Deserialize;
use std::{collections::HashMap, fmt, io, path::Path};

use crate::{LogLevel, ProxyGroup, Rule, RuleSetBehavior, RuleTag, RunMode};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Toml(toml::de::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "io error: {err}"),
            Error::Toml(err) => write!(f, "toml error: {err}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Self::Toml(value)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
pub enum RuleCfg {
    Single(RuleSingleCfg),
    Set(RuleSetCfg),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuleSingleCfg {
    pub tag: RuleTag,
    #[serde(default)]
    pub value: String,
    pub target: String,
}

impl From<RuleSingleCfg> for Rule {
    fn from(cfg: RuleSingleCfg) -> Self {
        Rule {
            tag: cfg.tag,
            value: cfg.value,
            target: cfg.target,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuleSetCfg {
    pub name: String,
    pub url: String,
    pub behavior: RuleSetBehavior,
    pub target: String,
    #[serde(default)]
    pub interval: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    #[serde(default)]
    pub proxies: HashMap<String, String>,
    #[serde(default)]
    pub groups: Vec<ProxyGroup>,
    #[serde(default)]
    pub rules: Vec<RuleCfg>,
    
    // 新增字段
    #[serde(default)]
    pub region_groups: Option<RegionGroupConfig>,
    #[serde(default)]
    pub default_config: Option<DefaultConfig>,
    #[serde(default)]
    pub provider_config: Option<ProviderConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RegionGroupConfig {
    pub enabled: bool,
    #[serde(default)]
    pub regions: Vec<RegionTemplate>,
    #[serde(default = "default_true")]
    pub create_auto_groups: bool,
    #[serde(default)]
    pub global_filter: Option<String>,
}

impl Default for RegionGroupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            regions: get_default_region_templates(),
            create_auto_groups: true,
            global_filter: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RegionTemplate {
    pub name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    pub filter: String,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DefaultConfig {
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub socks_port: Option<u16>,
    #[serde(default)]
    pub mixed_port: Option<u16>,
    #[serde(default)]
    pub mode: Option<RunMode>,
    #[serde(default)]
    pub log_level: Option<LogLevel>,
    #[serde(default)]
    pub allow_lan: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProviderConfig {
    #[serde(default)]
    pub health_check_url: Option<String>,
    #[serde(default)]
    pub health_check_interval: Option<u64>,
    #[serde(default)]
    pub update_interval: Option<u64>,
    #[serde(default)]
    pub lazy: Option<bool>,
}

fn default_true() -> bool {
    true
}

pub fn get_default_region_templates() -> Vec<RegionTemplate> {
    vec![
        RegionTemplate {
            name: "HK".to_string(),
            display_name: Some("香港".to_string()),
            filter: "(?i)(hk|hong kong|香港|港)".to_string(),
            icon: Some("🇭🇰".to_string()),
        },
        RegionTemplate {
            name: "US".to_string(),
            display_name: Some("美国".to_string()),
            filter: "(?i)(us|usa|united states|美国|美)".to_string(),
            icon: Some("🇺🇸".to_string()),
        },
        RegionTemplate {
            name: "JP".to_string(),
            display_name: Some("日本".to_string()),
            filter: "(?i)(jp|japan|日本|日)".to_string(),
            icon: Some("🇯🇵".to_string()),
        },
        RegionTemplate {
            name: "SG".to_string(),
            display_name: Some("新加坡".to_string()),
            filter: "(?i)(sg|singapore|新加坡|新)".to_string(),
            icon: Some("🇸🇬".to_string()),
        },
        RegionTemplate {
            name: "TW".to_string(),
            display_name: Some("台湾".to_string()),
            filter: "(?i)(tw|taiwan|台湾|台)".to_string(),
            icon: Some("🇹🇼".to_string()),
        },
        RegionTemplate {
            name: "KR".to_string(),
            display_name: Some("韩国".to_string()),
            filter: "(?i)(kr|korea|韩国|韩)".to_string(),
            icon: Some("🇰🇷".to_string()),
        },
    ]
}

impl AppConfig {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let data = std::fs::read(path)?;
        let cfg = toml::from_slice(&data)?;
        Ok(cfg)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_region_group_config_default() {
        let config = RegionGroupConfig::default();
        assert!(config.enabled);
        assert!(config.create_auto_groups);
        assert_eq!(config.regions.len(), 6); // 默认有 6 个地区
        assert!(config.global_filter.is_none());
    }

    #[test]
    fn test_get_default_region_templates() {
        let templates = get_default_region_templates();
        assert_eq!(templates.len(), 6);
        
        // 检查香港模板
        let hk_template = templates.iter().find(|t| t.name == "HK").unwrap();
        assert_eq!(hk_template.display_name, Some("香港".to_string()));
        assert_eq!(hk_template.filter, "(?i)(hk|hong kong|香港|港)");
        assert_eq!(hk_template.icon, Some("🇭🇰".to_string()));
        
        // 检查美国模板
        let us_template = templates.iter().find(|t| t.name == "US").unwrap();
        assert_eq!(us_template.display_name, Some("美国".to_string()));
        assert_eq!(us_template.filter, "(?i)(us|usa|united states|美国|美)");
        assert_eq!(us_template.icon, Some("🇺🇸".to_string()));
    }

    #[test]
    fn test_app_config_load_from_file() {
        let config_content = r#"
[default-config]
mixed-port = 7890
allow-lan = true
mode = "rule"
log-level = "info"

[region-groups]
enabled = true
create-auto-groups = true

[[region-groups.regions]]
name = "HK"
display-name = "香港"
filter = "(?i)(hk|hong kong)"
icon = "🇭🇰"

[provider-config]
health-check-url = "http://test.com/generate_204"
health-check-interval = 600
update-interval = 7200
lazy = false

[proxies]
test-provider = "https://example.com/clash"

[[groups]]
name = "Proxies"
type = "select"
proxies = ["Auto", "DIRECT"]

[[rules]]
type = "single"
tag = "DOMAIN"
value = "example.com"
target = "DIRECT"

[[rules]]
name = "test-rule-set"
type = "set"
url = "https://example.com/rules.yaml"
behavior = "domain"
target = "Proxies"
interval = 3600
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();
        
        let config = AppConfig::load_from_file(temp_file.path()).unwrap();
        
        // 检查默认配置
        let default_config = config.default_config.unwrap();
        assert_eq!(default_config.mixed_port, Some(7890));
        assert_eq!(default_config.allow_lan, Some(true));
        assert_eq!(default_config.mode, Some(RunMode::Rule));
        assert_eq!(default_config.log_level, Some(LogLevel::Info));
        
        // 检查地区组配置
        let region_config = config.region_groups.unwrap();
        assert!(region_config.enabled);
        assert!(region_config.create_auto_groups);
        assert_eq!(region_config.regions.len(), 1);
        assert_eq!(region_config.regions[0].name, "HK");
        
        // 检查 provider 配置
        let provider_config = config.provider_config.unwrap();
        assert_eq!(provider_config.health_check_url, Some("http://test.com/generate_204".to_string()));
        assert_eq!(provider_config.health_check_interval, Some(600));
        assert_eq!(provider_config.update_interval, Some(7200));
        assert_eq!(provider_config.lazy, Some(false));
        
        // 检查订阅源
        assert_eq!(config.proxies.len(), 1);
        assert_eq!(config.proxies.get("test-provider"), Some(&"https://example.com/clash".to_string()));
        
        // 检查代理组
        assert_eq!(config.groups.len(), 1);
        match &config.groups[0] {
            ProxyGroup::Select(select) => {
                assert_eq!(select.common.name, "Proxies");
            }
            _ => panic!("Expected Select group"),
        }
        
        // 检查规则
        assert_eq!(config.rules.len(), 2);
        match &config.rules[0] {
            RuleCfg::Single(rule) => {
                assert_eq!(rule.tag, RuleTag::Domain);
                assert_eq!(rule.value, "example.com");
                assert_eq!(rule.target, "DIRECT");
            }
            _ => panic!("Expected Single rule"),
        }
        
        match &config.rules[1] {
            RuleCfg::Set(rule_set) => {
                assert_eq!(rule_set.name, "test-rule-set");
                assert_eq!(rule_set.url, "https://example.com/rules.yaml");
                assert_eq!(rule_set.behavior, RuleSetBehavior::Domain);
                assert_eq!(rule_set.target, "Proxies");
                assert_eq!(rule_set.interval, Some(3600));
            }
            _ => panic!("Expected Set rule"),
        }
    }

    #[test]
    fn test_app_config_minimal() {
        let config_content = r#"
[proxies]
test = "https://example.com/clash"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();
        
        let config = AppConfig::load_from_file(temp_file.path()).unwrap();
        
        // 检查默认值
        assert!(config.default_config.is_none());
        assert!(config.region_groups.is_none());
        assert!(config.provider_config.is_none());
        assert_eq!(config.groups.len(), 0);
        assert_eq!(config.rules.len(), 0);
        assert_eq!(config.proxies.len(), 1);
    }

    #[test]
    fn test_app_config_load_invalid_file() {
        let result = AppConfig::load_from_file("non_existent_file.toml");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Io(_) => {},
            _ => panic!("Expected IO error"),
        }
    }

    #[test]
    fn test_app_config_load_invalid_toml() {
        let invalid_content = "invalid toml content [[[";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(invalid_content.as_bytes()).unwrap();
        
        let result = AppConfig::load_from_file(temp_file.path());
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Toml(_) => {},
            _ => panic!("Expected TOML error"),
        }
    }

    #[test]
    fn test_rule_single_cfg_into_rule() {
        let rule_cfg = RuleSingleCfg {
            tag: RuleTag::DomainSuffix,
            value: "google.com".to_string(),
            target: "Proxies".to_string(),
        };
        
        let rule: Rule = rule_cfg.into();
        assert_eq!(rule.tag, RuleTag::DomainSuffix);
        assert_eq!(rule.value, "google.com");
        assert_eq!(rule.target, "Proxies");
    }

    #[test]
    fn test_default_true() {
        assert_eq!(default_true(), true);
    }

    #[test]
    fn test_error_display() {
        let io_error = Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));
        assert!(io_error.to_string().contains("io error"));
        
        // 测试 TOML 错误通过实际的解析错误
        let invalid_toml = "invalid [[[";
        let toml_parse_result: Result<AppConfig, toml::de::Error> = toml::from_str(invalid_toml);
        let toml_error = Error::Toml(toml_parse_result.unwrap_err());
        assert!(toml_error.to_string().contains("toml error"));
    }

    #[test]
    fn test_region_template_deserialization() {
        let toml_content = r#"
name = "TEST"
display-name = "测试"
filter = "(?i)(test)"
icon = "🧪"
"#;
        
        let template: RegionTemplate = toml::from_str(toml_content).unwrap();
        assert_eq!(template.name, "TEST");
        assert_eq!(template.display_name, Some("测试".to_string()));
        assert_eq!(template.filter, "(?i)(test)");
        assert_eq!(template.icon, Some("🧪".to_string()));
    }

    #[test]
    fn test_default_config_partial() {
        let toml_content = r#"
mixed-port = 7890
mode = "global"
"#;
        
        let config: DefaultConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.mixed_port, Some(7890));
        assert_eq!(config.mode, Some(RunMode::Global));
        assert_eq!(config.port, None);
        assert_eq!(config.socks_port, None);
        assert_eq!(config.log_level, None);
        assert_eq!(config.allow_lan, None);
    }

    #[test]
    fn test_provider_config_partial() {
        let toml_content = r#"
health-check-interval = 600
lazy = true
"#;
        
        let config: ProviderConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.health_check_interval, Some(600));
        assert_eq!(config.lazy, Some(true));
        assert_eq!(config.health_check_url, None);
        assert_eq!(config.update_interval, None);
    }
}