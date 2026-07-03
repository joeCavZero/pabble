use std::fs;
use std::path::Path;

use crate::project::config::PebbleProjectFile;

pub const PEBBLE_PROJECT_FILE: &str = "pebble.toml";

pub fn load_project_from_current_dir() -> Result<Option<PebbleProjectFile>, String> {
    let path = Path::new(PEBBLE_PROJECT_FILE);

    if !path.exists() {
        return Ok(None);
    }

    match load_project_from_path(path) {
        Ok(project) => Ok(Some(project)),
        Err(e) => Err(e),
    }
}

pub fn load_project_from_path(path: &Path) -> Result<PebbleProjectFile, String> {
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

    match toml::from_str::<PebbleProjectFile>(&content) {
        Ok(project) => Ok(project),

        Err(e) => Err(format!(
            "Failed to parse '{}': {}",
            path.to_string_lossy(),
            e
        )),
    }
}

pub fn resolve_entry(entry: Option<&String>, default_entry: &str) -> Result<String, String> {
    match entry {
        Some(entry) => Ok(entry.clone()),

        None => {
            let project = match load_project_from_current_dir() {
                Ok(project) => project,
                Err(e) => return Err(e),
            };

            match project {
                Some(project) => {
                    if project.project.entry.trim().is_empty() {
                        return Err("pebble.toml has empty project.entry".to_string());
                    }

                    Ok(project.project.entry)
                }

                None => Ok(default_entry.to_string()),
            }
        }
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