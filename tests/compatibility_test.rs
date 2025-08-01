use std::io::Write;
use tempfile::NamedTempFile;
use sub_util::{generate_clash_config_with_validation, AppConfig};

#[test]
fn test_clash_config_structure_compatibility() {
    let config_content = r#"
[default-config]
mixed-port = 7890
allow-lan = true
mode = "rule"
log-level = "info"

[region-groups]
enabled = true

[proxies]
test = "https://example.com/clash"

[[groups]]
name = "Proxies"
type = "select"
proxies = ["HK", "DIRECT"]

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
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    
    let app_config = AppConfig::load_from_file(temp_file.path()).unwrap();
    let clash_config = generate_clash_config_with_validation(app_config).unwrap();
    
    // 验证必需的顶级字段存在
    assert!(clash_config.proxy_providers.is_some());
    assert!(clash_config.proxy_groups.is_some());
    assert!(clash_config.rules.is_some());
    
    // 验证 proxy-providers 结构
    let providers = clash_config.proxy_providers.unwrap();
    for (name, provider) in &providers {
        match provider {
            sub_util::ProxyProvider::Http(http_provider) => {
                // 验证必需字段
                assert!(!http_provider.url.is_empty(), "Provider {name} missing URL");
                assert!(http_provider.path.is_some(), "Provider {name} missing path");
                
                // 验证健康检查配置
                if let Some(health_check) = &http_provider.common.health_check {
                    assert!(!health_check.url.is_empty(), "Provider {name} health check missing URL");
                    assert!(health_check.interval > 0, "Provider {name} health check interval must be positive");
                }
                
                // 验证更新间隔
                if let Some(interval) = http_provider.common.interval {
                    assert!(interval > 0, "Provider {name} update interval must be positive");
                }
            }
            _ => panic!("Only HTTP providers are currently supported"),
        }
    }
    
    // 验证 proxy-groups 结构
    let groups = clash_config.proxy_groups.unwrap();
    for group in &groups {
        match group {
            sub_util::ProxyGroup::Select(select) => {
                assert!(!select.common.name.is_empty(), "Select group missing name");
                // Select 组应该有 proxies 或 use 字段之一
                assert!(
                    select.common.proxies.is_some() || select.common.use_provider.is_some(),
                    "Select group {} must have either proxies or use field", 
                    select.common.name
                );
            }
            sub_util::ProxyGroup::UrlTest(url_test) => {
                assert!(!url_test.common.name.is_empty(), "UrlTest group missing name");
                assert!(url_test.common.url.is_some(), "UrlTest group {} missing test URL", url_test.common.name);
                if let Some(interval) = url_test.common.interval {
                    assert!(interval > 0, "UrlTest group {} interval must be positive", url_test.common.name);
                }
            }
            sub_util::ProxyGroup::Fallback(fallback) => {
                assert!(!fallback.common.name.is_empty(), "Fallback group missing name");
            }
            sub_util::ProxyGroup::LoadBalance(load_balance) => {
                assert!(!load_balance.common.name.is_empty(), "LoadBalance group missing name");
            }
            sub_util::ProxyGroup::Relay(relay) => {
                assert!(!relay.common.name.is_empty(), "Relay group missing name");
            }
        }
    }
    
    // 验证 rules 结构
    let rules = clash_config.rules.unwrap();
    for rule in &rules {
        assert!(!rule.target.is_empty(), "Rule missing target");
        
        // MATCH 规则不应该有 value
        if rule.tag == sub_util::RuleTag::Match {
            assert!(rule.value.is_empty(), "MATCH rule should not have value");
        } else {
            // 其他规则应该有 value（除了某些特殊情况）
            if rule.tag != sub_util::RuleTag::RuleSet {
                assert!(!rule.value.is_empty(), "Rule {:?} missing value", rule.tag);
            }
        }
    }
}

#[test]
fn test_yaml_output_format_compatibility() {
    let config_content = r#"
[default-config]
mixed-port = 7890
mode = "rule"

[region-groups]
enabled = true

[proxies]
test = "https://example.com/clash"

[[groups]]
name = "Proxies"
type = "select"
proxies = ["HK", "DIRECT"]

[[rules]]
type = "single"
tag = "MATCH"
target = "Proxies"
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    
    let app_config = AppConfig::load_from_file(temp_file.path()).unwrap();
    let clash_config = generate_clash_config_with_validation(app_config).unwrap();
    
    let yaml_content = serde_yaml::to_string(&clash_config).unwrap();
    
    // 验证 YAML 格式符合 Clash 规范
    
    // 1. 检查基本配置字段使用正确的命名
    assert!(yaml_content.contains("mixed-port:"), "Should use kebab-case for mixed-port");
    assert!(!yaml_content.contains("mixed_port:"), "Should not use snake_case");
    
    // 2. 检查 proxy-providers 字段
    assert!(yaml_content.contains("proxy-providers:"), "Should have proxy-providers section");
    assert!(yaml_content.contains("type: http"), "Provider should have type field");
    assert!(yaml_content.contains("url:"), "Provider should have url field");
    assert!(yaml_content.contains("path:"), "Provider should have path field");
    assert!(yaml_content.contains("health-check:"), "Provider should have health-check section");
    
    // 3. 检查 proxy-groups 字段
    assert!(yaml_content.contains("proxy-groups:"), "Should have proxy-groups section");
    assert!(yaml_content.contains("name:"), "Groups should have name field");
    assert!(yaml_content.contains("type:"), "Groups should have type field");
    
    // 4. 检查地区代理组的过滤器
    assert!(yaml_content.contains("filter:"), "Region groups should have filter field");
    assert!(yaml_content.contains("use:"), "Region groups should have use field");
    
    // 5. 检查规则格式
    assert!(yaml_content.contains("rules:"), "Should have rules section");
    assert!(yaml_content.contains("- MATCH,Proxies"), "Rules should be in correct format");
    
    // 6. 验证 YAML 可以被重新解析
    let reparsed: serde_yaml::Value = serde_yaml::from_str(&yaml_content).unwrap();
    assert!(reparsed.is_mapping(), "Root should be a mapping");
    
    let root_map = reparsed.as_mapping().unwrap();
    assert!(root_map.contains_key(serde_yaml::Value::String("mixed-port".to_string())));
    assert!(root_map.contains_key(serde_yaml::Value::String("proxy-providers".to_string())));
    assert!(root_map.contains_key(serde_yaml::Value::String("proxy-groups".to_string())));
    assert!(root_map.contains_key(serde_yaml::Value::String("rules".to_string())));
}

#[test]
fn test_filter_regex_compatibility() {
    let config_content = r#"
[region-groups]
enabled = true

[[region-groups.regions]]
name = "HK"
filter = "(?i)(hk|hong kong|香港|港)"

[[region-groups.regions]]
name = "US"
filter = "(?i)(us|usa|united states|美国|美)"

[proxies]
test = "https://example.com/clash"
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    
    let app_config = AppConfig::load_from_file(temp_file.path()).unwrap();
    let clash_config = generate_clash_config_with_validation(app_config).unwrap();
    
    let groups = clash_config.proxy_groups.unwrap();
    
    // 验证过滤器格式
    for group in &groups {
        match group {
            sub_util::ProxyGroup::Select(select) => {
                if let Some(filter) = &select.common.filter {
                    // 验证过滤器是有效的正则表达式格式
                    assert!(filter.contains("(?i)"), "Filter should be case-insensitive");
                    assert!(filter.contains("(") && filter.contains(")"), "Filter should have grouping");
                    assert!(filter.contains("|"), "Filter should have alternatives");
                    
                    // 验证过滤器不包含可能导致问题的字符
                    assert!(!filter.contains("\\\\"), "Filter should not have double backslashes");
                    assert!(!filter.starts_with("^") || !filter.ends_with("$"), "Filter should not be anchored unless necessary");
                }
            }
            sub_util::ProxyGroup::UrlTest(url_test) => {
                if let Some(filter) = &url_test.common.filter {
                    // 同样的验证逻辑
                    assert!(filter.contains("(?i)"), "Filter should be case-insensitive");
                    assert!(filter.contains("(") && filter.contains(")"), "Filter should have grouping");
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test_rule_format_compatibility() {
    let config_content = r#"
[proxies]
test = "https://example.com/clash"

[[groups]]
name = "Proxies"
type = "select"
proxies = ["DIRECT"]

[[rules]]
type = "single"
tag = "DOMAIN"
value = "example.com"
target = "DIRECT"

[[rules]]
type = "single"
tag = "DOMAIN-SUFFIX"
value = "google.com"
target = "Proxies"

[[rules]]
type = "single"
tag = "IP-CIDR"
value = "192.168.1.0/24"
target = "DIRECT"

[[rules]]
type = "single"
tag = "GEOIP"
value = "CN"
target = "DIRECT"

[[rules]]
name = "test-rule-set"
type = "set"
url = "https://example.com/rules.yaml"
behavior = "domain"
target = "Proxies"

[[rules]]
type = "single"
tag = "MATCH"
target = "Proxies"
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    
    let app_config = AppConfig::load_from_file(temp_file.path()).unwrap();
    let clash_config = generate_clash_config_with_validation(app_config).unwrap();
    
    let yaml_content = serde_yaml::to_string(&clash_config).unwrap();
    
    // 验证规则格式符合 Clash 规范
    assert!(yaml_content.contains("- DOMAIN,example.com,DIRECT"), "DOMAIN rule format");
    assert!(yaml_content.contains("- DOMAIN-SUFFIX,google.com,Proxies"), "DOMAIN-SUFFIX rule format");
    assert!(yaml_content.contains("- IP-CIDR,192.168.1.0/24,DIRECT"), "IP-CIDR rule format");
    assert!(yaml_content.contains("- GEOIP,CN,DIRECT"), "GEOIP rule format");
    assert!(yaml_content.contains("- RULE-SET,test-rule-set,Proxies"), "RULE-SET rule format");
    assert!(yaml_content.contains("- MATCH,Proxies"), "MATCH rule format");
    
    // 验证 MATCH 规则在最后
    let rules_section = yaml_content.split("rules:").nth(1).unwrap();
    let rules_lines: Vec<&str> = rules_section.lines().filter(|line| line.trim().starts_with("- ")).collect();
    let last_rule = rules_lines.last().unwrap();
    assert!(last_rule.contains("MATCH"), "MATCH rule should be last");
}

#[test]
fn test_provider_health_check_compatibility() {
    let config_content = r#"
[provider-config]
health-check-url = "http://cp.cloudflare.com/generate_204"
health-check-interval = 600
lazy = false

[proxies]
test = "https://example.com/clash"
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    
    let app_config = AppConfig::load_from_file(temp_file.path()).unwrap();
    let clash_config = generate_clash_config_with_validation(app_config).unwrap();
    
    let yaml_content = serde_yaml::to_string(&clash_config).unwrap();
    
    // 验证健康检查配置格式
    assert!(yaml_content.contains("health-check:"), "Should have health-check section");
    assert!(yaml_content.contains("enable: true"), "Health check should be enabled");
    assert!(yaml_content.contains("url: http://cp.cloudflare.com/generate_204"), "Custom health check URL");
    assert!(yaml_content.contains("interval: 600"), "Custom health check interval");
    assert!(yaml_content.contains("lazy: false"), "Custom lazy setting");
    
    // 验证健康检查 URL 是有效的
    let providers = clash_config.proxy_providers.unwrap();
    for provider in providers.values() {
        if let sub_util::ProxyProvider::Http(http_provider) = provider {
            if let Some(health_check) = &http_provider.common.health_check {
                assert!(health_check.url.starts_with("http://") || health_check.url.starts_with("https://"));
                assert!(health_check.interval >= 60, "Health check interval should be at least 60 seconds");
            }
        }
    }
}