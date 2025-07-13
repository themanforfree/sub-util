use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize, Default, Copy, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warning,
    Error,
    #[serde(alias = "off")]
    Silent,
}
