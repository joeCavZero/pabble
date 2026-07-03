use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::project::dependency::PebbleDependency;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PebbleProjectFile {
    pub project: PebbleProjectConfig,

    #[serde(default)]
    pub dependencies: HashMap<String, PebbleDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PebbleProjectConfig {
    pub name: String,
    pub version: String,
    pub entry: String,
}

impl PebbleProjectFile {
    pub fn new_default(name: &str) -> Self {
        Self {
            project: PebbleProjectConfig {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                entry: "src/main.peng".to_string(),
            },

            dependencies: HashMap::new(),
        }
    }
}