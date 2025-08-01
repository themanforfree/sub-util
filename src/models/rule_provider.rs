use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
pub enum RuleProvider {
    Http(HttpRuleProvider),
    File(FileRuleProvider),
    Inline(InlineRuleProvider),
}
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderFormat {
    #[default]
    Yaml,
    Text,
    Mrs,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RuleProviderCommon {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ProviderFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u64>,
    pub behavior: RuleSetBehavior,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HttpRuleProvider {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<HashMap<String, Vec<String>>>,
    #[serde(flatten)]
    pub common: RuleProviderCommon,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileRuleProvider {
    pub path: String,
    #[serde(flatten)]
    pub common: RuleProviderCommon,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InlineRuleProvider {
    pub payload: Vec<String>,
    #[serde(flatten)]
    pub common: RuleProviderCommon,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RuleSetBehavior {
    #[default]
    Domain,
    Ipcidr,
    Classical,
}
