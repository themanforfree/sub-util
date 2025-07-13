use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::Proxy;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
pub enum ProxyProvider {
    Http(HttpProxyProvider),
    File(FileProxyProvider),
    Inline(InlineProxyProvider),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HealthCheck {
    pub enable: bool,
    pub url: String,
    pub interval: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lazy: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ProxyName {
    pub pattern: String,
    pub target: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Override {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_name: Option<Vec<ProxyName>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_cert_verify: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub udp: Option<bool>,
    #[serde(flatten)]
    pub extra: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ProxyProviderCommon {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_check: Option<HealthCheck>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#override: Option<Override>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct HttpProxyProvider {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<HashMap<String, Vec<String>>>,
    #[serde(flatten)]
    pub common: ProxyProviderCommon,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct InlineProxyProvider {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Vec<Proxy>>,
    #[serde(flatten)]
    pub common: ProxyProviderCommon,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct FileProxyProvider {
    pub path: String,
    #[serde(flatten)]
    pub common: ProxyProviderCommon,
}
