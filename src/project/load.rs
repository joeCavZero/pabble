use std::fs;
use std::path::{Path, PathBuf};

use crate::project::config::PabbleProjectFile;

pub const PEBBLE_PROJECT_FILE: &str = "pabble.toml";

pub fn load_project_from_current_dir() -> Result<Option<PabbleProjectFile>, String> {
    let path = Path::new(PEBBLE_PROJECT_FILE);

    if !path.exists() {
        return Ok(None);
    }

    match load_project_from_path(path) {
        Ok(project) => Ok(Some(project)),
        Err(e) => Err(e),
    }
}

pub fn load_project_from_path(path: &Path) -> Result<PabbleProjectFile, String> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,

        Err(e) => {
            return Err(format!(
                "Failed to read '{}': {}",
                path.to_string_lossy(),
                e
            ));
        }
    };

    match toml::from_str::<PabbleProjectFile>(&content) {
        Ok(project) => Ok(project),

        Err(e) => Err(format!(
            "Failed to parse '{}': {}",
            path.to_string_lossy(),
            e
        )),
    }
}

pub fn resolve_entry(entry: Option<&String>) -> Result<PathBuf, String> {
    let current_dir = match std::env::current_dir() {
        Ok(current_dir) => current_dir,

        Err(e) => {
            return Err(format!(
                "Failed to get current directory: {}",
                e
            ));
        }
    };

    let entry_path = match entry {
        Some(entry) => {
            let path = PathBuf::from(entry);

            if path.is_absolute() {
                path
            } else {
                current_dir.join(path)
            }
        }

        None => {
            let project = match load_project_from_current_dir() {
                Ok(Some(project)) => project,

                Ok(None) => {
                    return Err(
                        "No pabble.toml found in current directory".to_string()
                    );
                }

                Err(e) => {
                    return Err(e);
                }
            };

            if project.project.entry.trim().is_empty() {
                return Err(
                    "pabble.toml has empty project.entry".to_string()
                );
            }

            current_dir.join(project.project.entry)
        }
    };

    if !entry_path.exists() {
        return Err(format!(
            "Entry '{}' not found",
            entry_path.to_string_lossy()
        ));
    }

    if !entry_path.is_file() {
        return Err(format!(
            "Entry '{}' is not a file",
            entry_path.to_string_lossy()
        ));
    }

    match fs::canonicalize(&entry_path) {
        Ok(path) => Ok(path),

        Err(e) => Err(format!(
            "Failed to resolve entry '{}': {}",
            entry_path.to_string_lossy(),
            e
        )),
    }
}

pub fn resolve_compile_output(input: &str, output: Option<&String>) -> String {
    match output {
        Some(output) => output.clone(),

        None => {
            let path = Path::new(input);

            match path.file_stem() {
                Some(stem) => match stem.to_str() {
                    Some(stem) => format!("{stem}.penb"),
                    None => "main.penb".to_string(),
                },

                None => "main.penb".to_string(),
            }
        }
    }
}