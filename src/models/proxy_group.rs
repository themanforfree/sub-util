use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
pub enum ProxyGroup {
    Relay(RelayGroup), // TODO: Relay will be deprecated in the future, use dialer-proxy instead
    UrlTest(UrlTestGroup),
    Fallback(FallbackGroup),
    LoadBalance(LoadBalanceGroup),
    Select(SelectGroup),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct RelayGroup {
    #[serde(flatten)]
    pub common: ProxyGroupCommon,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum Strategy {
    RoundRobin,
    ConsistentHashing,
    StickySession,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct LoadBalanceGroup {
    #[serde(flatten)]
    pub common: ProxyGroupCommon,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<Strategy>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct FallbackGroup {
    #[serde(flatten)]
    pub common: ProxyGroupCommon,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct UrlTestGroup {
    #[serde(flatten)]
    pub common: ProxyGroupCommon,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tolerance: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct SelectGroup {
    #[serde(flatten)]
    pub common: ProxyGroupCommon,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ProxyGroupCommon {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxies: Option<Vec<String>>,
    #[serde(rename = "use")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_provider: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lazy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_failed_times: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_udp: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
}
