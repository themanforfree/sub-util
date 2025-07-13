use std::collections::HashMap;

use serde::{Deserialize, Serialize, Serializer, ser::SerializeSeq};

use crate::*;
type Port = u16;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// The HTTP proxy port
    #[serde(alias = "http_port")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<Port>,
    /// The SOCKS5 proxy port
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socks_port: Option<Port>,
    /// The HTTP/SOCKS5 mixed proxy port
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mixed_port: Option<Port>,
    /// The redir port
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redir_port: Option<Port>,
    /// The tproxy port
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tproxy_port: Option<Port>,
    // /// HTTP and SOCKS5 proxy authentication
    // pub authentication: Vec<String>,
    /// Allow connections to the local-end server from other LAN IP addresses
    /// Deprecated see `bind_address`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_lan: Option<bool>,
    /// Clash router working mode
    /// Either `rule`, `global` or `direct`
    pub mode: RunMode,
    /// Log level
    /// Either `debug`, `info`, `warning`, `error` or `off`
    pub log_level: LogLevel,
    // /// DNS client/server settings
    // pub dns: DNS,
    /// Proxy settings
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "proxies_serialize")]
    pub proxies: Option<Vec<Proxy>>,
    /// proxy provider settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_providers: Option<HashMap<String, ProxyProvider>>,
    /// Proxy group settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_groups: Option<Vec<ProxyGroup>>,
    /// rule provider settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_providers: Option<HashMap<String, RuleProvider>>,
    /// Rule settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<Rule>>,
    // /// Hosts
    // pub hosts: HashMap<String, String>,
    // /// Country database path relative to the $CWD
    // #[educe(Default = "Country.mmdb")]
    // pub mmdb: String,
    // /// Country database download url
    // // TODO not compatiable with clash-meta
    // #[educe(Default = Some("https://github.com/Loyalsoldier/geoip/releases/download/202307271745/Country.mmdb".into()))]
    // pub mmdb_download_url: Option<String>,
    // /// Optional ASN database path relative to the working dir
    // #[educe(Default = "Country-asn.mmdb")]
    // pub asn_mmdb: String,
    // /// Optional ASN database download url
    // pub asn_mmdb_download_url: Option<String>,
    // /// Geosite database path relative to the $CWD
    // #[educe(Default = "geosite.dat")]
    // pub geosite: String,
    // /// Geosite database download url
    // #[educe(Default = Some("https://github.com/Loyalsoldier/v2ray-rules-dat/releases/download/202406182210/geosite.dat".into()))]
    // pub geosite_download_url: Option<String>,

    // // these options has default vals,
    // // and needs extra processing
    // /// whether your network environment supports IPv6
    // /// this will affect the DNS server response to AAAA questions
    // /// default is `false`
    // pub ipv6: bool,
    // /// external controller address
    // pub external_controller: Option<String>,
    // /// dashboard folder path relative to the $CWD
    // pub external_ui: Option<String>,
    // /// external controller secret
    // pub secret: Option<String>,
    // /// outbound interface name
    // pub interface_name: Option<String>,
    // /// fwmark on Linux only
    // pub routing_mask: Option<u32>,

    // /// experimental settings, if any
    // pub experimental: Option<Experimental>,

    // /// tun settings
    // /// # Example
    // /// ```yaml
    // /// tun:
    // ///   enable: true
    // ///   device-id: "dev://utun1989"
    // /// ```
    // pub tun: Option<TunConfig>,

    // pub listeners: Option<Vec<HashMap<String, Value>>>,
}

/// Custom serializer for proxies
/// Skip `Other` proxies
fn proxies_serialize<S>(proxy: &Option<Vec<Proxy>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(proxies) = proxy {
        let mut seq = serializer.serialize_seq(Some(proxies.len()))?;
        for proxy in proxies {
            if matches!(proxy, Proxy::Other(_)) {
                continue;
            }
            seq.serialize_element(proxy)?;
        }
        seq.end()
    } else {
        serializer.serialize_none()
    }
}
