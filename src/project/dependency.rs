use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PebbleDependency {
    Version(String),
    Detailed(PebbleDependencyInfo),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PebbleDependencyInfo {
    pub version: Option<String>,
    pub path: Option<String>,
    pub git: Option<String>,
}