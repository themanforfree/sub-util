use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "type")]
pub enum Proxy {
    Direct {
        #[serde(flatten)]
        common: ProxyCommon,
    },
    Tuic {
        #[serde(flatten)]
        common: ProxyCommon,
        #[serde(skip_serializing_if = "Option::is_none")]
        uuid: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        password: Option<String>,
        #[serde(flatten)]
        extra: Option<HashMap<String, Value>>,
    },
    #[serde(untagged)]
    Other(HashMap<String, Value>),
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum IpVersion {
    #[default]
    Dual,
    Ipv4,
    Ipv6,
    Ipv4Prefer,
    Ipv6Prefer,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ProxyCommon {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_version: Option<IpVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub udp: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing_mark: Option<u32>,
}
