mod app_config;
mod models;
mod proxy_group_generator;

use std::collections::HashMap;

pub use app_config::*;
pub use models::*;
pub use proxy_group_generator::*;

// 默认配置常量
const DEFAULT_HEALTH_CHECK_URL: &str = "http://www.gstatic.com/generate_204";
const DEFAULT_HEALTH_CHECK_INTERVAL: u64 = 300;
const DEFAULT_UPDATE_INTERVAL: u64 = 3600;
const DEFAULT_RULE_UPDATE_INTERVAL: u64 = 86400;

#[derive(Debug)]
pub enum ConfigError {
    InvalidSubscriptionUrl(String),
    ProxyGroupGenerationFailed(String),
    RuleProcessingFailed(String),
    ConfigValidationFailed(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidSubscriptionUrl(url) => write!(f, "Invalid subscription URL: {}", url),
            ConfigError::ProxyGroupGenerationFailed(msg) => write!(f, "Proxy group generation failed: {}", msg),
            ConfigError::RuleProcessingFailed(msg) => write!(f, "Rule processing failed: {}", msg),
            ConfigError::ConfigValidationFailed(msg) => write!(f, "Config validation failed: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

/// 生成 proxy providers
fn generate_proxy_providers(
    proxies: &HashMap<String, String>,
    provider_config: &Option<ProviderConfig>
) -> HashMap<String, ProxyProvider> {
    let mut providers = HashMap::new();
    
    for (name, url) in proxies {
        let health_check = HealthCheck {
            enable: true,
            url: provider_config
                .as_ref()
                .and_then(|c| c.health_check_url.clone())
                .unwrap_or_else(|| DEFAULT_HEALTH_CHECK_URL.to_string()),
            interval: provider_config
                .as_ref()
                .and_then(|c| c.health_check_interval)
                .unwrap_or(DEFAULT_HEALTH_CHECK_INTERVAL),
            lazy: provider_config
                .as_ref()
                .and_then(|c| c.lazy),
        };
        
        providers.insert(
            name.clone(),
            ProxyProvider::Http(HttpProxyProvider {
                url: url.clone(),
                path: Some(format!("./proxies/{}.yaml", name)),
                common: ProxyProviderCommon {
                    interval: provider_config
                        .as_ref()
                        .and_then(|c| c.update_interval)
                        .or(Some(DEFAULT_UPDATE_INTERVAL)),
                    health_check: Some(health_check),
                    filter: None,
                    exclude_filter: None,
                    exclude_type: None,
                    r#override: None,
                },
                proxy: None,
                size_limit: None,
                header: None,
            }),
        );
    }
    
    providers
}

pub fn generate_clash_config(app_config: AppConfig) -> Config {
    let mut config = Config::default();
    
    // 应用默认配置
    apply_default_config(&mut config, &app_config.default_config);
    
    // 生成 proxy providers
    let proxy_providers = generate_proxy_providers(&app_config.proxies, &app_config.provider_config);
    
    // 生成地区代理组（如果启用）
    let region_groups = if let Some(region_config) = &app_config.region_groups {
        if region_config.enabled {
            ProxyGroupTemplateGenerator::generate_region_groups(
                &proxy_providers.keys().cloned().collect::<Vec<_>>(),
                region_config
            )
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    
    // 合并所有代理组
    let all_groups = ProxyGroupTemplateGenerator::merge_with_user_groups(
        region_groups,
        app_config.groups
    );
    
    // 生成规则和规则提供者
    let (rule_providers, rules) = generate_rules_and_providers(&app_config.rules);
    
    config.proxy_providers = Some(proxy_providers);
    config.proxy_groups = Some(all_groups);
    config.rule_providers = Some(rule_providers);
    config.rules = Some(rules);
    
    config
}

/// 应用默认配置
fn apply_default_config(config: &mut Config, default_config: &Option<DefaultConfig>) {
    if let Some(defaults) = default_config {
        if let Some(port) = defaults.port {
            config.port = Some(port);
        }
        if let Some(socks_port) = defaults.socks_port {
            config.socks_port = Some(socks_port);
        }
        if let Some(mixed_port) = defaults.mixed_port {
            config.mixed_port = Some(mixed_port);
        }
        if let Some(mode) = defaults.mode {
            config.mode = mode;
        }
        if let Some(log_level) = defaults.log_level {
            config.log_level = log_level;
        }
        if let Some(allow_lan) = defaults.allow_lan {
            config.allow_lan = Some(allow_lan);
        }
    }
}

/// 生成规则和规则提供者
fn generate_rules_and_providers(rules_config: &[RuleCfg]) -> (HashMap<String, RuleProvider>, Vec<Rule>) {
    let mut rule_providers = HashMap::new();
    let mut rules = Vec::new();
    
    for rule_cfg in rules_config {
        match rule_cfg {
            RuleCfg::Single(rule) => rules.push(rule.clone().into()),
            RuleCfg::Set(rule_set) => {
                rule_providers.insert(
                    rule_set.name.clone(),
                    RuleProvider::Http(HttpRuleProvider {
                        url: rule_set.url.clone(),
                        path: Some(format!("./rules/{}.yaml", rule_set.name)),
                        common: RuleProviderCommon {
                            behavior: rule_set.behavior,
                            interval: rule_set.interval.or(Some(DEFAULT_RULE_UPDATE_INTERVAL)),
                            format: None,
                        },
                        ..Default::default()
                    }),
                );
                rules.push(Rule {
                    tag: RuleTag::RuleSet,
                    value: rule_set.name.clone(),
                    target: rule_set.target.clone(),
                });
            }
        }
    }
    
    (rule_providers, rules)
}

/// 验证应用配置
pub fn validate_app_config(app_config: &AppConfig) -> Result<(), ConfigError> {
    // 验证订阅 URL
    for (name, url) in &app_config.proxies {
        if !is_valid_url(url) {
            return Err(ConfigError::InvalidSubscriptionUrl(format!("{}: {}", name, url)));
        }
    }
    
    // 验证地区代理组配置
    if let Some(region_config) = &app_config.region_groups {
        if region_config.enabled {
            for region in &region_config.regions {
                if let Err(e) = ProxyGroupTemplateGenerator::validate_filter(&region.filter) {
                    return Err(ConfigError::ProxyGroupGenerationFailed(format!("Invalid filter for region {}: {}", region.name, e)));
                }
            }
        }
    }
    
    // 获取所有可能的代理组名称（包括地区代理组）
    let available_groups = get_all_available_groups(app_config);
    
    // 验证规则配置
    for rule_cfg in &app_config.rules {
        match rule_cfg {
            RuleCfg::Single(rule) => {
                if rule.target.is_empty() {
                    return Err(ConfigError::RuleProcessingFailed("Rule target cannot be empty".to_string()));
                }
                validate_rule_target(&rule.target, &available_groups)?;
            }
            RuleCfg::Set(rule_set) => {
                if !is_valid_url(&rule_set.url) {
                    return Err(ConfigError::RuleProcessingFailed(format!("Invalid rule set URL: {}", rule_set.url)));
                }
                if rule_set.target.is_empty() {
                    return Err(ConfigError::RuleProcessingFailed("Rule set target cannot be empty".to_string()));
                }
                validate_rule_target(&rule_set.target, &available_groups)?;
            }
        }
    }
    
    Ok(())
}

/// 获取所有可用的代理组名称
fn get_all_available_groups(app_config: &AppConfig) -> Vec<String> {
    let mut groups = Vec::new();
    
    // 添加用户定义的代理组
    for group in &app_config.groups {
        match group {
            ProxyGroup::Select(select) => groups.push(select.common.name.clone()),
            ProxyGroup::UrlTest(url_test) => groups.push(url_test.common.name.clone()),
            ProxyGroup::Fallback(fallback) => groups.push(fallback.common.name.clone()),
            ProxyGroup::LoadBalance(load_balance) => groups.push(load_balance.common.name.clone()),
            ProxyGroup::Relay(relay) => groups.push(relay.common.name.clone()),
        }
    }
    
    // 添加地区代理组（如果启用）
    if let Some(region_config) = &app_config.region_groups {
        if region_config.enabled {
            let regions = ProxyGroupTemplateGenerator::get_merged_region_templates(region_config);
            for region in &regions {
                groups.push(region.name.clone());
                if region_config.create_auto_groups {
                    groups.push(format!("{}-Auto", region.name));
                }
            }
        }
    }
    
    // 添加内置的特殊目标
    groups.push("DIRECT".to_string());
    groups.push("REJECT".to_string());
    
    groups
}

/// 验证规则目标是否有效
fn validate_rule_target(target: &str, available_groups: &[String]) -> Result<(), ConfigError> {
    // 检查是否是内置的特殊目标
    if target == "DIRECT" || target == "REJECT" {
        return Ok(());
    }
    
    // 检查是否是可用的代理组
    if available_groups.contains(&target.to_string()) {
        return Ok(());
    }
    
    Err(ConfigError::RuleProcessingFailed(format!(
        "Rule target '{}' is not a valid proxy group. Available groups: {}", 
        target, 
        available_groups.join(", ")
    )))
}

/// 验证生成的配置
pub fn validate_generated_config(config: &Config) -> Result<(), ConfigError> {
    // 验证代理组
    if let Some(groups) = &config.proxy_groups {
        for group in groups {
            match group {
                ProxyGroup::Select(select) => {
                    if select.common.name.is_empty() {
                        return Err(ConfigError::ConfigValidationFailed("Proxy group name cannot be empty".to_string()));
                    }
                }
                ProxyGroup::UrlTest(url_test) => {
                    if url_test.common.name.is_empty() {
                        return Err(ConfigError::ConfigValidationFailed("Proxy group name cannot be empty".to_string()));
                    }
                    if url_test.common.url.is_none() {
                        return Err(ConfigError::ConfigValidationFailed(format!("URL test group {} must have a test URL", url_test.common.name)));
                    }
                }
                _ => {} // 其他类型的验证可以在这里添加
            }
        }
    }
    
    // 验证 proxy providers
    if let Some(providers) = &config.proxy_providers {
        for (name, provider) in providers {
            match provider {
                ProxyProvider::Http(http_provider) => {
                    if !is_valid_url(&http_provider.url) {
                        return Err(ConfigError::ConfigValidationFailed(format!("Invalid provider URL for {}: {}", name, http_provider.url)));
                    }
                }
                _ => {} // 其他类型的验证
            }
        }
    }
    
    Ok(())
}

/// 简单的 URL 验证
fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

/// 带验证的配置生成函数
pub fn generate_clash_config_with_validation(app_config: AppConfig) -> Result<Config, ConfigError> {
    // 验证输入配置
    validate_app_config(&app_config)?;
    
    // 生成配置
    let config = generate_clash_config(app_config);
    
    // 验证生成的配置
    validate_generated_config(&config)?;
    
    Ok(config)
}

/// 获取可用的地区代理组列表
pub fn get_available_region_groups(app_config: &AppConfig) -> Vec<String> {
    let mut region_groups = Vec::new();
    
    if let Some(region_config) = &app_config.region_groups {
        if region_config.enabled {
            let regions = ProxyGroupTemplateGenerator::get_merged_region_templates(region_config);
            for region in &regions {
                region_groups.push(region.name.clone());
                if region_config.create_auto_groups {
                    region_groups.push(format!("{}-Auto", region.name));
                }
            }
        }
    }
    
    region_groups
}

/// 检查规则目标是否引用了地区代理组
pub fn is_region_group_target(target: &str, app_config: &AppConfig) -> bool {
    let region_groups = get_available_region_groups(app_config);
    region_groups.contains(&target.to_string())
}

/// 验证规则集配置
pub fn validate_rule_set_config(rule_set: &RuleSetCfg) -> Result<(), ConfigError> {
    // 验证 URL
    if !is_valid_url(&rule_set.url) {
        return Err(ConfigError::RuleProcessingFailed(format!("Invalid rule set URL: {}", rule_set.url)));
    }
    
    // 验证名称
    if rule_set.name.is_empty() {
        return Err(ConfigError::RuleProcessingFailed("Rule set name cannot be empty".to_string()));
    }
    
    // 验证更新间隔
    if let Some(interval) = rule_set.interval {
        if interval < 60 {
            return Err(ConfigError::RuleProcessingFailed("Rule set update interval should be at least 60 seconds".to_string()));
        }
    }
    
    Ok(())
}

/// 获取规则集的有效更新间隔
pub fn get_rule_set_update_interval(rule_set: &RuleSetCfg) -> u64 {
    rule_set.interval.unwrap_or(DEFAULT_RULE_UPDATE_INTERVAL)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_app_config() -> AppConfig {
        let mut proxies = HashMap::new();
        proxies.insert("test-provider".to_string(), "https://example.com/clash".to_string());

        AppConfig {
            proxies,
            groups: vec![
                ProxyGroup::Select(SelectGroup {
                    common: ProxyGroupCommon {
                        name: "Proxies".to_string(),
                        proxies: Some(vec!["Auto".to_string(), "DIRECT".to_string()]),
                        use_provider: None,
                        url: None,
                        interval: None,
                        lazy: None,
                        timeout: None,
                        max_failed_times: None,
                        disable_udp: None,
                        icon: None,
                        filter: None,
                    },
                }),
            ],
            rules: vec![
                RuleCfg::Single(RuleSingleCfg {
                    tag: RuleTag::Domain,
                    value: "example.com".to_string(),
                    target: "DIRECT".to_string(),
                }),
                RuleCfg::Set(RuleSetCfg {
                    name: "test-rule-set".to_string(),
                    url: "https://example.com/rules.yaml".to_string(),
                    behavior: RuleSetBehavior::Domain,
                    target: "Proxies".to_string(),
                    interval: None,
                }),
            ],
            region_groups: Some(RegionGroupConfig {
                enabled: true,
                regions: vec![
                    RegionTemplate {
                        name: "HK".to_string(),
                        display_name: Some("香港".to_string()),
                        filter: "(?i)(hk|hong kong)".to_string(),
                        icon: Some("🇭🇰".to_string()),
                    },
                ],
                create_auto_groups: true,
                global_filter: None,
            }),
            default_config: Some(DefaultConfig {
                port: Some(7890),
                socks_port: None,
                mixed_port: Some(7891),
                mode: Some(RunMode::Rule),
                log_level: Some(LogLevel::Info),
                allow_lan: Some(true),
            }),
            provider_config: Some(ProviderConfig {
                health_check_url: Some("http://test.com/generate_204".to_string()),
                health_check_interval: Some(600),
                update_interval: Some(7200),
                lazy: Some(false),
            }),
        }
    }

    #[test]
    fn test_generate_proxy_providers() {
        let mut proxies = HashMap::new();
        proxies.insert("test".to_string(), "https://example.com/clash".to_string());
        
        let provider_config = Some(ProviderConfig {
            health_check_url: Some("http://test.com".to_string()),
            health_check_interval: Some(600),
            update_interval: Some(7200),
            lazy: Some(false),
        });

        let providers = generate_proxy_providers(&proxies, &provider_config);
        
        assert_eq!(providers.len(), 1);
        assert!(providers.contains_key("test"));
        
        match providers.get("test").unwrap() {
            ProxyProvider::Http(http_provider) => {
                assert_eq!(http_provider.url, "https://example.com/clash");
                assert_eq!(http_provider.path, Some("./proxies/test.yaml".to_string()));
                assert_eq!(http_provider.common.interval, Some(7200));
                
                let health_check = http_provider.common.health_check.as_ref().unwrap();
                assert_eq!(health_check.url, "http://test.com");
                assert_eq!(health_check.interval, 600);
                assert_eq!(health_check.lazy, Some(false));
            }
            _ => panic!("Expected HTTP provider"),
        }
    }

    #[test]
    fn test_generate_proxy_providers_with_defaults() {
        let mut proxies = HashMap::new();
        proxies.insert("test".to_string(), "https://example.com/clash".to_string());

        let providers = generate_proxy_providers(&proxies, &None);
        
        match providers.get("test").unwrap() {
            ProxyProvider::Http(http_provider) => {
                assert_eq!(http_provider.common.interval, Some(DEFAULT_UPDATE_INTERVAL));
                
                let health_check = http_provider.common.health_check.as_ref().unwrap();
                assert_eq!(health_check.url, DEFAULT_HEALTH_CHECK_URL);
                assert_eq!(health_check.interval, DEFAULT_HEALTH_CHECK_INTERVAL);
            }
            _ => panic!("Expected HTTP provider"),
        }
    }

    #[test]
    fn test_apply_default_config() {
        let mut config = Config::default();
        let default_config = Some(DefaultConfig {
            port: Some(8080),
            socks_port: Some(1080),
            mixed_port: Some(7890),
            mode: Some(RunMode::Global),
            log_level: Some(LogLevel::Debug),
            allow_lan: Some(false),
        });

        apply_default_config(&mut config, &default_config);

        assert_eq!(config.port, Some(8080));
        assert_eq!(config.socks_port, Some(1080));
        assert_eq!(config.mixed_port, Some(7890));
        assert_eq!(config.mode, RunMode::Global);
        assert_eq!(config.log_level, LogLevel::Debug);
        assert_eq!(config.allow_lan, Some(false));
    }

    #[test]
    fn test_generate_rules_and_providers() {
        let rules_config = vec![
            RuleCfg::Single(RuleSingleCfg {
                tag: RuleTag::Domain,
                value: "example.com".to_string(),
                target: "DIRECT".to_string(),
            }),
            RuleCfg::Set(RuleSetCfg {
                name: "test-set".to_string(),
                url: "https://example.com/rules.yaml".to_string(),
                behavior: RuleSetBehavior::Classical,
                target: "Proxies".to_string(),
                interval: Some(3600),
            }),
        ];

        let (rule_providers, rules) = generate_rules_and_providers(&rules_config);

        assert_eq!(rule_providers.len(), 1);
        assert_eq!(rules.len(), 2);

        // 检查规则提供者
        let provider = rule_providers.get("test-set").unwrap();
        match provider {
            RuleProvider::Http(http_provider) => {
                assert_eq!(http_provider.url, "https://example.com/rules.yaml");
                assert_eq!(http_provider.path, Some("./rules/test-set.yaml".to_string()));
                assert_eq!(http_provider.common.behavior, RuleSetBehavior::Classical);
                assert_eq!(http_provider.common.interval, Some(3600));
            }
            _ => panic!("Expected HTTP rule provider"),
        }

        // 检查规则
        assert_eq!(rules[0].tag, RuleTag::Domain);
        assert_eq!(rules[0].value, "example.com");
        assert_eq!(rules[0].target, "DIRECT");

        assert_eq!(rules[1].tag, RuleTag::RuleSet);
        assert_eq!(rules[1].value, "test-set");
        assert_eq!(rules[1].target, "Proxies");
    }

    #[test]
    fn test_generate_clash_config() {
        let app_config = create_test_app_config();
        let config = generate_clash_config(app_config);

        // 检查默认配置是否应用
        assert_eq!(config.port, Some(7890));
        assert_eq!(config.mixed_port, Some(7891));
        assert_eq!(config.mode, RunMode::Rule);
        assert_eq!(config.log_level, LogLevel::Info);
        assert_eq!(config.allow_lan, Some(true));

        // 检查 proxy providers
        assert!(config.proxy_providers.is_some());
        let providers = config.proxy_providers.unwrap();
        assert_eq!(providers.len(), 1);
        assert!(providers.contains_key("test-provider"));

        // 检查代理组（应该包含地区代理组和用户代理组）
        assert!(config.proxy_groups.is_some());
        let groups = config.proxy_groups.unwrap();
        // 1 个地区 * 2 (select + auto) + 1 个用户组 = 3 个组
        assert_eq!(groups.len(), 3);

        // 检查规则和规则提供者
        assert!(config.rule_providers.is_some());
        assert!(config.rules.is_some());
        let rule_providers = config.rule_providers.unwrap();
        let rules = config.rules.unwrap();
        assert_eq!(rule_providers.len(), 1);
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_validate_app_config() {
        let app_config = create_test_app_config();
        assert!(validate_app_config(&app_config).is_ok());
    }

    #[test]
    fn test_validate_app_config_invalid_url() {
        let mut app_config = create_test_app_config();
        app_config.proxies.insert("invalid".to_string(), "not-a-url".to_string());
        
        let result = validate_app_config(&app_config);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidSubscriptionUrl(_) => {},
            _ => panic!("Expected InvalidSubscriptionUrl error"),
        }
    }

    #[test]
    fn test_validate_app_config_invalid_rule_target() {
        let mut app_config = create_test_app_config();
        app_config.rules.push(RuleCfg::Single(RuleSingleCfg {
            tag: RuleTag::Domain,
            value: "test.com".to_string(),
            target: "NonExistentGroup".to_string(),
        }));
        
        let result = validate_app_config(&app_config);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::RuleProcessingFailed(_) => {},
            _ => panic!("Expected RuleProcessingFailed error"),
        }
    }

    #[test]
    fn test_generate_clash_config_with_validation() {
        let app_config = create_test_app_config();
        let result = generate_clash_config_with_validation(app_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_valid_url() {
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("http://example.com"));
        assert!(!is_valid_url("ftp://example.com"));
        assert!(!is_valid_url("example.com"));
        assert!(!is_valid_url(""));
    }

    #[test]
    fn test_get_available_region_groups() {
        let app_config = create_test_app_config();
        let region_groups = get_available_region_groups(&app_config);
        
        assert_eq!(region_groups.len(), 2); // HK 和 HK-Auto
        assert!(region_groups.contains(&"HK".to_string()));
        assert!(region_groups.contains(&"HK-Auto".to_string()));
    }

    #[test]
    fn test_is_region_group_target() {
        let app_config = create_test_app_config();
        
        assert!(is_region_group_target("HK", &app_config));
        assert!(is_region_group_target("HK-Auto", &app_config));
        assert!(!is_region_group_target("US", &app_config));
        assert!(!is_region_group_target("Proxies", &app_config));
    }

    #[test]
    fn test_validate_rule_set_config() {
        let valid_rule_set = RuleSetCfg {
            name: "test".to_string(),
            url: "https://example.com/rules.yaml".to_string(),
            behavior: RuleSetBehavior::Domain,
            target: "Proxies".to_string(),
            interval: Some(3600),
        };
        
        assert!(validate_rule_set_config(&valid_rule_set).is_ok());
        
        // 测试无效 URL
        let invalid_url_rule_set = RuleSetCfg {
            name: "test".to_string(),
            url: "not-a-url".to_string(),
            behavior: RuleSetBehavior::Domain,
            target: "Proxies".to_string(),
            interval: None,
        };
        
        assert!(validate_rule_set_config(&invalid_url_rule_set).is_err());
        
        // 测试空名称
        let empty_name_rule_set = RuleSetCfg {
            name: "".to_string(),
            url: "https://example.com/rules.yaml".to_string(),
            behavior: RuleSetBehavior::Domain,
            target: "Proxies".to_string(),
            interval: None,
        };
        
        assert!(validate_rule_set_config(&empty_name_rule_set).is_err());
        
        // 测试无效间隔
        let invalid_interval_rule_set = RuleSetCfg {
            name: "test".to_string(),
            url: "https://example.com/rules.yaml".to_string(),
            behavior: RuleSetBehavior::Domain,
            target: "Proxies".to_string(),
            interval: Some(30), // 小于 60 秒
        };
        
        assert!(validate_rule_set_config(&invalid_interval_rule_set).is_err());
    }

    #[test]
    fn test_get_rule_set_update_interval() {
        let rule_set_with_interval = RuleSetCfg {
            name: "test".to_string(),
            url: "https://example.com/rules.yaml".to_string(),
            behavior: RuleSetBehavior::Domain,
            target: "Proxies".to_string(),
            interval: Some(3600),
        };
        
        assert_eq!(get_rule_set_update_interval(&rule_set_with_interval), 3600);
        
        let rule_set_without_interval = RuleSetCfg {
            name: "test".to_string(),
            url: "https://example.com/rules.yaml".to_string(),
            behavior: RuleSetBehavior::Domain,
            target: "Proxies".to_string(),
            interval: None,
        };
        
        assert_eq!(get_rule_set_update_interval(&rule_set_without_interval), DEFAULT_RULE_UPDATE_INTERVAL);
    }
}