mod app_config;
mod models;

use std::collections::HashMap;

pub use app_config::*;
pub use models::*;

pub fn generate_clash_config(app_config: AppConfig) -> Config {
    let mut proxy_providers = HashMap::new();
    for (name, url) in app_config.proxies {
        // TODO: 自动生成 name 并自动添加到 groups
        proxy_providers.insert(
            name.clone(),
            ProxyProvider::Http(HttpProxyProvider {
                url,
                path: Some(format!("./proxies/{}.yaml", name)),
                common: ProxyProviderCommon {
                    interval: Some(3600),
                    health_check: Some(HealthCheck {
                        enable: true,
                        url: "http://www.gstatic.com/generate_204".to_string(),
                        interval: 300,
                        lazy: Some(true),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }),
        );
    }

    let mut rule_providers = HashMap::new();
    let mut rules = Vec::new();
    for rule_cfg in app_config.rules {
        match rule_cfg {
            RuleCfg::Single(rule) => rules.push(rule.into()),
            RuleCfg::Set(rule_set) => {
                rule_providers.insert(
                    rule_set.name.clone(),
                    RuleProvider::Http(HttpRuleProvider {
                        url: rule_set.url,
                        path: Some(format!("./rules/{}.yaml", rule_set.name.clone())),
                        common: RuleProviderCommon {
                            behavior: rule_set.behavior,
                            interval: Some(86400),
                            format: None,
                        },
                        ..Default::default()
                    }),
                );
                rules.push(Rule {
                    tag: RuleTag::RuleSet,
                    value: rule_set.name,
                    target: rule_set.target,
                });
            }
        }
    }

    Config {
        proxy_providers: Some(proxy_providers),
        proxy_groups: Some(app_config.groups),
        rule_providers: Some(rule_providers),
        rules: Some(rules),
        ..Default::default()
    }
}
