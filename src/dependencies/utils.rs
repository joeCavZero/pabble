use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::project::*;

#[derive(Debug, Clone, Copy)]
pub enum PebbleDependencyCommandMode {
    Fetch,
    Install,
    Update,
}

pub fn install_project_dependencies(mode: PebbleDependencyCommandMode) -> Result<(), String> {
    let project = match load_project_from_current_dir() {
        Ok(Some(project)) => project,

        Ok(None) => {
            return Err("No pebble.toml found in current directory".to_string());
        }

        Err(e) => {
            return Err(e);
        }
    };

    if project.dependencies.is_empty() {
        println!("No dependencies found");
        return Ok(());
    }

    match create_dependency_cache_dirs() {
        Ok(()) => {}

        Err(e) => {
            return Err(e);
        }
    }

    for (name, dependency) in project.dependencies.iter() {
        match install_dependency(name, dependency, mode) {
            Ok(path) => {
                println!(
                    "Resolved dependency '{}' -> '{}'",
                    name,
                    path.to_string_lossy()
                );
            }

            Err(e) => {
                return Err(e);
            }
        }
    }

    match mode {
        PebbleDependencyCommandMode::Fetch => {
            println!("Dependencies fetched");
        }

        PebbleDependencyCommandMode::Install => {
            println!("Dependencies installed");
        }

        PebbleDependencyCommandMode::Update => {
            println!("Dependencies updated");
        }
    }

    Ok(())
}

fn install_dependency(
    name: &str,
    dependency: &PebbleDependency,
    mode: PebbleDependencyCommandMode,
) -> Result<PathBuf, String> {
    match dependency {
        PebbleDependency::Version(version) => {
            install_version_dependency(name, version, mode)
        }

        PebbleDependency::Detailed(info) => {
            if let Some(path) = &info.path {
                return install_path_dependency(name, path);
            }

            if let Some(git) = &info.git {
                return install_git_dependency(name, git, info.version.as_ref(), mode);
            }

            if let Some(version) = &info.version {
                return install_version_dependency(name, version, mode);
            }

            Err(format!(
                "dependency '{}' has no path, git or version",
                name
            ))
        }
    }
}

fn install_path_dependency(name: &str, path: &str) -> Result<PathBuf, String> {
    let path_buf = PathBuf::from(path);

    if path_buf.is_file() {
        return Ok(path_buf);
    }

    if path_buf.is_dir() {
        return resolve_project_entry_from_root(name, &path_buf);
    }

    Err(format!(
        "dependency '{}' path '{}' not found",
        name, path
    ))
}

fn install_version_dependency(
    name: &str,
    version: &str,
    mode: PebbleDependencyCommandMode,
) -> Result<PathBuf, String> {
    let cache_dir = match pebble_dependency_cache_dir() {
        Ok(cache_dir) => cache_dir,
        Err(e) => return Err(e),
    };

    let dependency_root = cache_dir
        .join("registry")
        .join(sanitize_dependency_segment(name))
        .join(sanitize_dependency_segment(version));

    let should_update = matches!(mode, PebbleDependencyCommandMode::Update);

    if dependency_root.exists() && !should_update {
        return resolve_project_entry_from_root(name, &dependency_root);
    }

    let registry_root = match pebble_registry_root() {
        Ok(registry_root) => registry_root,
        Err(e) => return Err(e),
    };

    let registry_dependency_root = registry_root
        .join(sanitize_dependency_segment(name))
        .join(sanitize_dependency_segment(version));

    if !registry_dependency_root.exists() {
        return Err(format!(
            "dependency '{}' version '{}' not found in registry '{}'",
            name,
            version,
            registry_dependency_root.to_string_lossy()
        ));
    }

    if dependency_root.exists() {
        match fs::remove_dir_all(&dependency_root) {
            Ok(()) => {}

            Err(e) => {
                return Err(format!(
                    "failed to remove old cached dependency '{}': {}",
                    name, e
                ));
            }
        }
    }

    match copy_dir_all(&registry_dependency_root, &dependency_root) {
        Ok(()) => {}

        Err(e) => {
            return Err(format!(
                "failed to install dependency '{}' version '{}': {}",
                name, version, e
            ));
        }
    }

    resolve_project_entry_from_root(name, &dependency_root)
}

fn install_git_dependency(
    name: &str,
    git: &str,
    version: Option<&String>,
    mode: PebbleDependencyCommandMode,
) -> Result<PathBuf, String> {
    let cache_dir = match pebble_dependency_cache_dir() {
        Ok(cache_dir) => cache_dir,
        Err(e) => return Err(e),
    };

    let dependency_root = cache_dir
        .join("git")
        .join(sanitize_dependency_segment(name));

    let should_update = matches!(mode, PebbleDependencyCommandMode::Update);

    if dependency_root.exists() {
        if should_update {
            match git_pull_dependency(name, &dependency_root) {
                Ok(()) => {}

                Err(e) => {
                    return Err(e);
                }
            }
        }
    } else {
        match fs::create_dir_all(cache_dir.join("git")) {
            Ok(()) => {}

            Err(e) => {
                return Err(format!(
                    "failed to create git dependency cache: {}",
                    e
                ));
            }
        }

        match git_clone_dependency(name, git, &dependency_root) {
            Ok(()) => {}

            Err(e) => {
                return Err(e);
            }
        }
    }

    if let Some(version) = version {
        match git_checkout_dependency(name, &dependency_root, version) {
            Ok(()) => {}

            Err(e) => {
                return Err(e);
            }
        }
    }

    resolve_project_entry_from_root(name, &dependency_root)
}

fn create_dependency_cache_dirs() -> Result<(), String> {
    let cache_dir = match pebble_dependency_cache_dir() {
        Ok(cache_dir) => cache_dir,
        Err(e) => return Err(e),
    };

    match fs::create_dir_all(cache_dir.join("git")) {
        Ok(()) => {}

        Err(e) => {
            return Err(format!(
                "failed to create git dependency cache: {}",
                e
            ));
        }
    }

    match fs::create_dir_all(cache_dir.join("registry")) {
        Ok(()) => {}

        Err(e) => {
            return Err(format!(
                "failed to create registry dependency cache: {}",
                e
            ));
        }
    }

    Ok(())
}

fn pebble_dependency_cache_dir() -> Result<PathBuf, String> {
    let current_dir = match std::env::current_dir() {
        Ok(current_dir) => current_dir,

        Err(e) => {
            return Err(format!(
                "failed to get current directory: {}",
                e
            ));
        }
    };

    Ok(current_dir.join(".pebble").join("deps"))
}

fn pebble_registry_root() -> Result<PathBuf, String> {
    match std::env::var("PEBBLE_REGISTRY_PATH") {
        Ok(path) => {
            let path_buf = PathBuf::from(path);

            if path_buf.exists() {
                return Ok(path_buf);
            }

            return Err(format!(
                "PEBBLE_REGISTRY_PATH points to missing directory '{}'",
                path_buf.to_string_lossy()
            ));
        }

        Err(_) => {}
    }

    let current_dir = match std::env::current_dir() {
        Ok(current_dir) => current_dir,

        Err(e) => {
            return Err(format!(
                "failed to get current directory: {}",
                e
            ));
        }
    };

    let local_registry = current_dir.join(".pebble").join("registry");

    if local_registry.exists() {
        return Ok(local_registry);
    }

    Err(
        "No registry found. Set PEBBLE_REGISTRY_PATH or create .pebble/registry".to_string(),
    )
}

fn resolve_project_entry_from_root(name: &str, root: &PathBuf) -> Result<PathBuf, String> {
    let project_file = root.join("pebble.toml");

    if project_file.exists() {
        let project = match load_project_from_path(&project_file) {
            Ok(project) => project,

            Err(e) => {
                return Err(format!(
                    "failed to load dependency '{}': {}",
                    name, e
                ));
            }
        };

        let entry = root.join(project.project.entry);

        if entry.exists() {
            return Ok(entry);
        }

        return Err(format!(
            "dependency '{}' entry '{}' not found",
            name,
            entry.to_string_lossy()
        ));
    }

    let default_source = root.join("src").join("main.peng");

    if default_source.exists() {
        return Ok(default_source);
    }

    let default_binary = root.join("main.penb");

    if default_binary.exists() {
        return Ok(default_binary);
    }

    Err(format!(
        "dependency '{}' does not contain pebble.toml, src/main.peng or main.penb",
        name
    ))
}

fn sanitize_dependency_segment(value: &str) -> String {
    let mut output = String::new();

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.' {
            output.push(ch);
        } else {
            output.push('_');
        }
    }

    if output.is_empty() {
        "dependency".to_string()
    } else {
        output
    }
}

fn git_clone_dependency(
    name: &str,
    git: &str,
    dependency_root: &PathBuf,
) -> Result<(), String> {
    let status = match Command::new("git")
        .arg("clone")
        .arg(git)
        .arg(dependency_root)
        .status()
    {
        Ok(status) => status,

        Err(e) => {
            return Err(format!(
                "failed to run git clone for dependency '{}': {}",
                name, e
            ));
        }
    };

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "git clone failed for dependency '{}' from '{}'",
            name, git
        ))
    }
}

fn git_pull_dependency(
    name: &str,
    dependency_root: &PathBuf,
) -> Result<(), String> {
    let status = match Command::new("git")
        .arg("-C")
        .arg(dependency_root)
        .arg("pull")
        .arg("--ff-only")
        .status()
    {
        Ok(status) => status,

        Err(e) => {
            return Err(format!(
                "failed to run git pull for dependency '{}': {}",
                name, e
            ));
        }
    };

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "git pull failed for dependency '{}' in '{}'",
            name,
            dependency_root.to_string_lossy()
        ))
    }
}

fn git_checkout_dependency(
    name: &str,
    dependency_root: &PathBuf,
    version: &str,
) -> Result<(), String> {
    let status = match Command::new("git")
        .arg("-C")
        .arg(dependency_root)
        .arg("checkout")
        .arg(version)
        .status()
    {
        Ok(status) => status,

        Err(e) => {
            return Err(format!(
                "failed to run git checkout for dependency '{}': {}",
                name, e
            ));
        }
    };

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "git checkout '{}' failed for dependency '{}' in '{}'",
            version,
            name,
            dependency_root.to_string_lossy()
        ))
    }
}

fn copy_dir_all(from: &Path, to: &Path) -> Result<(), String> {
    match fs::create_dir_all(to) {
        Ok(()) => {}

        Err(e) => {
            return Err(format!(
                "failed to create directory '{}': {}",
                to.to_string_lossy(),
                e
            ));
        }
    }

    let entries = match fs::read_dir(from) {
        Ok(entries) => entries,

        Err(e) => {
            return Err(format!(
                "failed to read directory '{}': {}",
                from.to_string_lossy(),
                e
            ));
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,

            Err(e) => {
                return Err(format!(
                    "failed to read directory entry in '{}': {}",
                    from.to_string_lossy(),
                    e
                ));
            }
        };

        let source_path = entry.path();
        let target_path = to.join(entry.file_name());

        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,

            Err(e) => {
                return Err(format!(
                    "failed to read file type '{}': {}",
                    source_path.to_string_lossy(),
                    e
                ));
            }
        };

        if file_type.is_dir() {
            match copy_dir_all(&source_path, &target_path) {
                Ok(()) => {}

                Err(e) => {
                    return Err(e);
                }
            }
        } else {
            match fs::copy(&source_path, &target_path) {
                Ok(_) => {}

                Err(e) => {
                    return Err(format!(
                        "failed to copy '{}' to '{}': {}",
                        source_path.to_string_lossy(),
                        target_path.to_string_lossy(),
                        e
                    ));
                }
            }
        }
    }

    Ok(())
}