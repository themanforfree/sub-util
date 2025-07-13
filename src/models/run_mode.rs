use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RunMode {
    Global,
    #[default]
    Rule,
    Direct,
}
