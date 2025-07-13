use serde::{Deserialize, Deserializer, Serialize, Serializer};

// TODO: Use meaningful types instead of strings
#[derive(Debug, Clone)]
pub enum Rule {
    Domain(String, String),
    DomainSuffix(String, String),
    DomainRegex(String, String),
    DomainKeyword(String, String),
    IpCIDR(String, String),
    IpCIDR6(String, String),
    IpAsn(String, String),
    RuleSet(String, String),
    GeoIp(String, String),
    Match(String),
}

impl std::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rule::Domain(domain, proxy) => write!(f, "DOMAIN,{},{}", domain, proxy),
            Rule::DomainSuffix(domain_suffix, proxy) => {
                write!(f, "DOMAIN-SUFFIX,{},{}", domain_suffix, proxy)
            }
            Rule::DomainRegex(domain_regex, proxy) => {
                write!(f, "DOMAIN-REGEX,{},{}", domain_regex, proxy)
            }
            Rule::DomainKeyword(domain_keyword, proxy) => {
                write!(f, "DOMAIN-KEYWORD,{},{}", domain_keyword, proxy)
            }
            Rule::IpCIDR(ip_cidr, proxy) => {
                write!(f, "IP-CIDR,{},{}", ip_cidr, proxy)
            }
            Rule::IpCIDR6(ip_cidr6, proxy) => {
                write!(f, "IP-CIDR6,{},{}", ip_cidr6, proxy)
            }
            Rule::RuleSet(rule_set, proxy) => {
                write!(f, "RULE-SET,{},{}", rule_set, proxy)
            }
            Rule::IpAsn(ip_asn, proxy) => {
                write!(f, "IP-ASN,{},{}", ip_asn, proxy)
            }
            Rule::GeoIp(geo_ip, proxy) => {
                write!(f, "GEOIP,{},{}", geo_ip, proxy)
            }
            Rule::Match(proxy) => write!(f, "MATCH,{}", proxy),
        }
    }
}

impl<'de> Deserialize<'de> for Rule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts = s.split(',').collect::<Vec<&str>>();
        match parts[0] {
            "DOMAIN" => Ok(Rule::Domain(parts[1].to_string(), parts[2].to_string())),
            "DOMAIN-SUFFIX" => Ok(Rule::DomainSuffix(
                parts[1].to_string(),
                parts[2].to_string(),
            )),
            "RULE-SET" => Ok(Rule::RuleSet(parts[1].to_string(), parts[2].to_string())),
            "IP-ASN" => Ok(Rule::IpAsn(parts[1].to_string(), parts[2].to_string())),
            "DOMAIN-REGEX" => Ok(Rule::DomainRegex(
                parts[1].to_string(),
                parts[2].to_string(),
            )),
            "DOMAIN-KEYWORD" => Ok(Rule::DomainKeyword(
                parts[1].to_string(),
                parts[2].to_string(),
            )),
            "IP-CIDR" => Ok(Rule::IpCIDR(parts[1].to_string(), parts[2].to_string())),
            "IP-CIDR6" => Ok(Rule::IpCIDR6(parts[1].to_string(), parts[2].to_string())),
            "GEOIP" => Ok(Rule::GeoIp(parts[1].to_string(), parts[2].to_string())),
            "MATCH" => Ok(Rule::Match(parts[1].to_string())),
            _ => Err(serde::de::Error::custom(format!("invalid rule: {}", s))),
        }
    }
}

impl Serialize for Rule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
