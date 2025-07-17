use serde::Deserialize;
use std::{collections::HashMap, fmt, io, path::Path};

use crate::{ProxyGroup, Rule, RuleSetBehavior, RuleTag};

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
}

impl AppConfig {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let data = std::fs::read(path)?;
        let cfg = toml::from_slice(&data)?;
        Ok(cfg)
    }
}
