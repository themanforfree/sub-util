use std::io::Write;
use tempfile::NamedTempFile;
use sub_util::{generate_clash_config_with_validation, AppConfig};

#[test]
fn test_end_to_end_config_generation() {
    let config_content = r#"
# 完整的端到端测试配置
[default-config]
mixed-port = 7890
allow-lan = true
mode = "rule"
log-level = "info"

[provider-config]
health-check-url = "http://www.gstatic.com/generate_204"
health-check-interval = 300
update-interval = 3600
lazy = true

[region-groups]
enabled = true
create-auto-groups = true

[[region-groups.regions]]
name = "HK"
display-name = "香港"
filter = "(?i)(hk|hong kong|香港|港)"
icon = "🇭🇰"

[[region-groups.regions]]
name = "US"
display-name = "美国"
filter = "(?i)(us|usa|united states|美国|美)"
icon = "🇺🇸"

[proxies]
test-provider1 = "https://example1.com/clash"
test-provider2 = "https://example2.com/clash"

[[groups]]
name = "Proxies"
type = "select"
proxies = ["Auto", "HK", "US", "DIRECT"]

[[groups]]
name = "Auto"
type = "url-test"
use = ["test-provider1", "test-provider2"]
url = "http://www.gstatic.com/generate_204"
interval = 300

[[groups]]
name = "OpenAI"
type = "select"
proxies = ["US", "US-Auto", "Proxies"]

[[rules]]
type = "single"
tag = "DOMAIN"
value = "example.com"
target = "DIRECT"

[[rules]]
name = "openai"
type = "set"
url = "https://example.com/openai.yaml"
behavior = "classical"
target = "OpenAI"

[[rules]]
name = "direct"
type = "set"
url = "https://example.com/direct.yaml"
behavior = "domain"
target = "DIRECT"
interval = 86400

[[rules]]
type = "single"
tag = "GEOIP"
value = "CN"
target = "DIRECT"

[[rules]]
type = "single"
tag = "MATCH"
target = "Proxies"
"#;

    // 创建临时配置文件
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    
    // 加载配置
    let app_config = AppConfig::load_from_file(temp_file.path()).unwrap();
    
    // 生成 Clash 配置
    let clash_config = generate_clash_config_with_validation(app_config).unwrap();
    
    // 验证生成的配置
    
    // 1. 检查默认配置
    assert_eq!(clash_config.mixed_port, Some(7890));
    assert_eq!(clash_config.allow_lan, Some(true));
    assert_eq!(clash_config.mode, sub_util::RunMode::Rule);
    assert_eq!(clash_config.log_level, sub_util::LogLevel::Info);
    
    // 2. 检查 proxy providers
    let providers = clash_config.proxy_providers.unwrap();
    assert_eq!(providers.len(), 2);
    assert!(providers.contains_key("test-provider1"));
    assert!(providers.contains_key("test-provider2"));
    
    // 验证 provider 配置
    match providers.get("test-provider1").unwrap() {
        sub_util::ProxyProvider::Http(http_provider) => {
            assert_eq!(http_provider.url, "https://example1.com/clash");
            assert_eq!(http_provider.path, Some("./proxies/test-provider1.yaml".to_string()));
            assert_eq!(http_provider.common.interval, Some(3600));
            
            let health_check = http_provider.common.health_check.as_ref().unwrap();
            assert_eq!(health_check.url, "http://www.gstatic.com/generate_204");
            assert_eq!(health_check.interval, 300);
            assert_eq!(health_check.lazy, Some(true));
        }
        _ => panic!("Expected HTTP provider"),
    }
    
    // 3. 检查代理组
    let groups = clash_config.proxy_groups.unwrap();
    // 2 个地区 * 2 (select + auto) + 3 个用户组 = 7 个组
    assert_eq!(groups.len(), 7);
    
    // 检查地区代理组
    let hk_group = groups.iter().find(|g| match g {
        sub_util::ProxyGroup::Select(s) => s.common.name == "HK",
        _ => false,
    }).unwrap();
    
    match hk_group {
        sub_util::ProxyGroup::Select(select) => {
            assert_eq!(select.common.filter, Some("(?i)(hk|hong kong|香港|港)".to_string()));
            assert_eq!(select.common.use_provider, Some(vec!["test-provider1".to_string(), "test-provider2".to_string()]));
            assert_eq!(select.common.icon, Some("🇭🇰".to_string()));
        }
        _ => panic!("Expected Select group"),
    }
    
    // 检查地区自动测试组
    let hk_auto_group = groups.iter().find(|g| match g {
        sub_util::ProxyGroup::UrlTest(u) => u.common.name == "HK-Auto",
        _ => false,
    }).unwrap();
    
    match hk_auto_group {
        sub_util::ProxyGroup::UrlTest(url_test) => {
            assert_eq!(url_test.common.filter, Some("(?i)(hk|hong kong|香港|港)".to_string()));
            assert_eq!(url_test.common.url, Some("http://www.gstatic.com/generate_204".to_string()));
            assert_eq!(url_test.common.interval, Some(300));
        }
        _ => panic!("Expected UrlTest group"),
    }
    
    // 检查用户定义的代理组
    let proxies_group = groups.iter().find(|g| match g {
        sub_util::ProxyGroup::Select(s) => s.common.name == "Proxies",
        _ => false,
    }).unwrap();
    
    match proxies_group {
        sub_util::ProxyGroup::Select(select) => {
            assert_eq!(select.common.proxies, Some(vec!["Auto".to_string(), "HK".to_string(), "US".to_string(), "DIRECT".to_string()]));
        }
        _ => panic!("Expected Select group"),
    }
    
    // 4. 检查规则提供者
    let rule_providers = clash_config.rule_providers.unwrap();
    assert_eq!(rule_providers.len(), 2);
    assert!(rule_providers.contains_key("openai"));
    assert!(rule_providers.contains_key("direct"));
    
    // 验证规则提供者配置
    match rule_providers.get("openai").unwrap() {
        sub_util::RuleProvider::Http(http_provider) => {
            assert_eq!(http_provider.url, "https://example.com/openai.yaml");
            assert_eq!(http_provider.path, Some("./rules/openai.yaml".to_string()));
            assert_eq!(http_provider.common.behavior, sub_util::RuleSetBehavior::Classical);
        }
        _ => panic!("Expected HTTP rule provider"),
    }
    
    match rule_providers.get("direct").unwrap() {
        sub_util::RuleProvider::Http(http_provider) => {
            assert_eq!(http_provider.common.interval, Some(86400));
        }
        _ => panic!("Expected HTTP rule provider"),
    }
    
    // 5. 检查规则
    let rules = clash_config.rules.unwrap();
    assert_eq!(rules.len(), 5);
    
    // 检查单个规则
    assert_eq!(rules[0].tag, sub_util::RuleTag::Domain);
    assert_eq!(rules[0].value, "example.com");
    assert_eq!(rules[0].target, "DIRECT");
    
    // 检查规则集规则
    assert_eq!(rules[1].tag, sub_util::RuleTag::RuleSet);
    assert_eq!(rules[1].value, "openai");
    assert_eq!(rules[1].target, "OpenAI");
    
    // 检查最终匹配规则
    assert_eq!(rules[4].tag, sub_util::RuleTag::Match);
    assert_eq!(rules[4].target, "Proxies");
}

#[test]
fn test_yaml_serialization() {
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
    
    // 测试 YAML 序列化
    let yaml_result = serde_yaml::to_string(&clash_config);
    assert!(yaml_result.is_ok());
    
    let yaml_content = yaml_result.unwrap();
    
    // 验证 YAML 内容包含预期的字段
    assert!(yaml_content.contains("mixed-port: 7890"));
    assert!(yaml_content.contains("mode: rule"));
    assert!(yaml_content.contains("proxy-providers:"));
    assert!(yaml_content.contains("proxy-groups:"));
    assert!(yaml_content.contains("rules:"));
    
    // 验证地区代理组在 YAML 中的格式
    assert!(yaml_content.contains("name: HK"));
    assert!(yaml_content.contains("filter:"));
    assert!(yaml_content.contains("use:"));
    
    println!("Generated YAML:\n{}", yaml_content);
}

#[test]
fn test_minimal_config() {
    let config_content = r#"
[proxies]
minimal = "https://example.com/clash"
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    
    let app_config = AppConfig::load_from_file(temp_file.path()).unwrap();
    let clash_config = generate_clash_config_with_validation(app_config).unwrap();
    
    // 验证最小配置也能正常工作
    assert!(clash_config.proxy_providers.is_some());
    assert!(clash_config.proxy_groups.is_some());
    assert!(clash_config.rules.is_some());
    
    // 默认情况下不应该有地区代理组
    let groups = clash_config.proxy_groups.unwrap();
    assert_eq!(groups.len(), 0); // 没有用户组，也没有启用地区组
}

#[test]
fn test_config_validation_errors() {
    // 测试无效的订阅 URL
    let invalid_url_config = r#"
[proxies]
invalid = "not-a-url"
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(invalid_url_config.as_bytes()).unwrap();
    
    let app_config = AppConfig::load_from_file(temp_file.path()).unwrap();
    let result = generate_clash_config_with_validation(app_config);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        sub_util::ConfigError::InvalidSubscriptionUrl(_) => {},
        _ => panic!("Expected InvalidSubscriptionUrl error"),
    }
}

#[test]
fn test_region_groups_disabled() {
    let config_content = r#"
[region-groups]
enabled = false

[proxies]
test = "https://example.com/clash"

[[groups]]
name = "Proxies"
type = "select"
proxies = ["DIRECT"]
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    
    let app_config = AppConfig::load_from_file(temp_file.path()).unwrap();
    let clash_config = generate_clash_config_with_validation(app_config).unwrap();
    
    // 验证地区代理组被禁用时不会生成
    let groups = clash_config.proxy_groups.unwrap();
    assert_eq!(groups.len(), 1); // 只有用户定义的 Proxies 组
    
    match &groups[0] {
        sub_util::ProxyGroup::Select(select) => {
            assert_eq!(select.common.name, "Proxies");
        }
        _ => panic!("Expected Proxies group"),
    }
}