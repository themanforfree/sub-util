use serde::Deserialize;
use std::{collections::HashMap, io, path::Path};

use crate::{ProxyGroup, Rule, RuleSetBehavior, RuleTag};

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct RuleSetCfg {
    pub name: String,
    pub url: String,
    pub behavior: RuleSetBehavior,
    pub target: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    #[serde(default)]
    pub proxies: HashMap<String, String>,
    #[serde(default)]
    pub groups: Vec<ProxyGroup>,
    #[serde(default)]
    pub rules: Vec<RuleCfg>,
}

impl AppConfig {
    pub fn load_from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let data = std::fs::read(path)?;
        toml::from_slice(&data).map_err(io::Error::other)
    }
}
