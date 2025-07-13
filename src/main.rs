use std::collections::HashMap;

use sub_util::{
    AppConfig, Config, HealthCheck, HttpProxyProvider, HttpRuleProvider, ProxyProvider,
    ProxyProviderCommon, Rule, RuleCfg, RuleProvider, RuleProviderCommon, RuleTag,
};
fn main() {
    let arg1 = std::env::args().nth(1);
    let cfg_path = arg1.as_deref().unwrap_or("config.toml");
    let app_config = AppConfig::load_from_file(cfg_path).unwrap();
    println!("{:#?}", app_config);

    let mut proxy_providers = HashMap::new();
    for (name, url) in app_config.proxies {
        proxy_providers.insert(
            name.clone(),
            ProxyProvider::Http(HttpProxyProvider {
                url: url,
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

    let clash_config = Config {
        proxy_providers: Some(proxy_providers),
        proxy_groups: Some(app_config.groups),
        rule_providers: Some(rule_providers),
        rules: Some(rules),
        ..Default::default()
    };

    let s = serde_yaml::to_string(&clash_config).unwrap();
    println!("{}", s);
}
