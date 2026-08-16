use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PabbleDependency {
    Version(String),
    Detailed(PabbleDependencyInfo),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PabbleDependencyInfo {
    pub version: Option<String>,
    pub path: Option<String>,
    pub git: Option<String>,
}
