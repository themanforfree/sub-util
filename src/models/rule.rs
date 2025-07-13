use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

// TODO: Use meaningful types instead of strings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum RuleTag {
    Domain,
    DomainSuffix,
    DomainRegex,
    DomainKeyword,
    IpCIDR,
    IpCIDR6,
    IpAsn,
    RuleSet,
    GeoIp,
    Match,
}

impl FromStr for RuleTag {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DOMAIN" => Ok(RuleTag::Domain),
            "DOMAIN-SUFFIX" => Ok(RuleTag::DomainSuffix),
            "DOMAIN-REGEX" => Ok(RuleTag::DomainRegex),
            "DOMAIN-KEYWORD" => Ok(RuleTag::DomainKeyword),
            "IP-CIDR" => Ok(RuleTag::IpCIDR),
            "IP-CIDR6" => Ok(RuleTag::IpCIDR6),
            "IP-ASN" => Ok(RuleTag::IpAsn),
            "RULE-SET" => Ok(RuleTag::RuleSet),
            "GEOIP" => Ok(RuleTag::GeoIp),
            "MATCH" => Ok(RuleTag::Match),
            _ => Err(format!("invalid rule tag: {}", s)),
        }
    }
}

impl std::fmt::Display for RuleTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleTag::Domain => write!(f, "DOMAIN"),
            RuleTag::DomainSuffix => write!(f, "DOMAIN-SUFFIX"),
            RuleTag::DomainRegex => write!(f, "DOMAIN-REGEX"),
            RuleTag::DomainKeyword => write!(f, "DOMAIN-KEYWORD"),
            RuleTag::IpCIDR => write!(f, "IP-CIDR"),
            RuleTag::IpCIDR6 => write!(f, "IP-CIDR6"),
            RuleTag::IpAsn => write!(f, "IP-ASN"),
            RuleTag::RuleSet => write!(f, "RULE-SET"),
            RuleTag::GeoIp => write!(f, "GEOIP"),
            RuleTag::Match => write!(f, "MATCH"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub tag: RuleTag,
    pub value: String,
    pub target: String,
}

impl std::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if matches!(self.tag, RuleTag::Match) {
            write!(f, "{},{}", self.tag, self.target)
        } else {
            write!(f, "{},{},{}", self.tag, self.value, self.target)
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
        let tag = RuleTag::from_str(parts[0]).map_err(serde::de::Error::custom)?;
        if matches!(tag, RuleTag::Match) {
            Ok(Rule {
                tag,
                value: String::new(),
                target: parts[1].to_string(),
            })
        } else {
            Ok(Rule {
                tag,
                value: parts[1].to_string(),
                target: parts[2].to_string(),
            })
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
