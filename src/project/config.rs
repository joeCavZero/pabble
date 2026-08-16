use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::project::dependency::PabbleDependency;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PabbleProjectFile {
    pub project: PabbleProjectConfig,

    #[serde(default)]
    pub dependencies: HashMap<String, PabbleDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PabbleProjectConfig {
    pub name: String,
    pub version: String,
    pub entry: String,
}

impl PabbleProjectFile {
    pub fn new_default(name: &str) -> Self {
        Self {
            project: PabbleProjectConfig {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                entry: "src/main.peng".to_string(),
            },

            dependencies: HashMap::new(),
        }
    }
}
