use penguin::prelude::*;
use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    path::PathBuf,
    process::Command,
    rc::Rc,
    time::SystemTime,
};

use crate::project::*;
use crate::standard::standard::*;

#[derive(Clone)]
pub enum PebbleImportCacheEntry {
    Loading {
        unit: PengUnit,
        modified: Option<SystemTime>,
    },

    Loaded {
        unit: PengUnit,
        modified: Option<SystemTime>,
    },
}

pub type PebbleImportCache = Rc<RefCell<HashMap<PathBuf, PebbleImportCacheEntry>>>;

pub fn setup(
    peng: &mut PengEnv,
    unit: &mut PengUnit,
    std_registry: PebbleStandardRegistry,
    import_cache: PebbleImportCache,
    dependencies: HashMap<String, PebbleDependency>,
) -> Result<(), PengError> {
    let prelude_for_import = Rc::new(RefCell::new(unit.clone()));
    let prelude_for_import_ref = prelude_for_import.clone();

    match unit.register_immutable_native_function(peng, "import", move |ctx| {
        let path = match ctx.get_arg_value(0) {
            Some(value) => match value.value() {
                PengValue::Box(PengBox::String(s)) => s.clone(),

                _ => {
                    return Err(PengError::CannotCallValue(
                        "import() expected string path".into(),
                    ));
                }
            },

            None => {
                return Err(PengError::CannotCallValue("import() expected path".into()));
            }
        };

        if let Some(std_unit) = std_registry.get(&path) {
            let module_ptr = std_unit.create_module_heap(ctx.env_mut());

            return Ok(PengBinded::Immutable(PengCell::Reference(module_ptr)));
        }

        if let Some(dependency) = dependencies.get(&path) {
            let dependency_path = match resolve_dependency_entry(&path, dependency) {
                Ok(path) => path,

                Err(e) => {
                    return Err(PengError::CannotCallValue(e));
                }
            };

            return import_local_module(
                ctx,
                &dependency_path,
                &prelude_for_import_ref,
                &import_cache,
            );
        }

        let is_local_import = path.starts_with("./")
            || path.starts_with("../")
            || path.ends_with(".peng")
            || path.ends_with(".penb");

        if !is_local_import {
            return Err(PengError::CannotCallValue(format!(
                "module '{}' not found",
                path
            )));
        }

        import_local_module(
            ctx,
            &PathBuf::from(&path),
            &prelude_for_import_ref,
            &import_cache,
        )
    }) {
        Ok(_) => {
            *prelude_for_import.borrow_mut() = unit.clone();

            Ok(())
        }

        Err(e) => Err(e),
    }
}

fn resolve_dependency_entry(
    name: &str,
    dependency: &PebbleDependency,
) -> Result<PathBuf, String> {
    match dependency {
        PebbleDependency::Version(version) => resolve_version_dependency_entry(name, version),

        PebbleDependency::Detailed(info) => {
            if let Some(path) = &info.path {
                return resolve_path_dependency_entry(name, path);
            }

            if let Some(git) = &info.git {
                return resolve_git_dependency_entry(name, git, info.version.as_ref());
            }

            if let Some(version) = &info.version {
                return resolve_version_dependency_entry(name, version);
            }

            Err(format!(
                "dependency '{}' has no path, git or version",
                name
            ))
        }
    }
}

fn resolve_path_dependency_entry(name: &str, path: &str) -> Result<PathBuf, String> {
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

fn resolve_version_dependency_entry(name: &str, version: &str) -> Result<PathBuf, String> {
    let cache_dir = match pebble_dependency_cache_dir() {
        Ok(cache_dir) => cache_dir,
        Err(e) => return Err(e),
    };

    let dependency_root = cache_dir
        .join("registry")
        .join(sanitize_dependency_segment(name))
        .join(sanitize_dependency_segment(version));

    if dependency_root.is_file() {
        return Ok(dependency_root);
    }

    if dependency_root.is_dir() {
        return resolve_project_entry_from_root(name, &dependency_root);
    }

    Err(format!(
        "dependency '{}' version '{}' not found in local registry cache '{}'",
        name,
        version,
        dependency_root.to_string_lossy()
    ))
}

fn resolve_git_dependency_entry(
    name: &str,
    git: &str,
    version: Option<&String>,
) -> Result<PathBuf, String> {
    let cache_dir = match pebble_dependency_cache_dir() {
        Ok(cache_dir) => cache_dir,
        Err(e) => return Err(e),
    };

    let dependency_root = cache_dir
        .join("git")
        .join(sanitize_dependency_segment(name));

    if dependency_root.exists() {
        match git_pull_dependency(name, &dependency_root) {
            Ok(()) => {}

            Err(e) => {
                return Err(e);
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

    let default_source = root.join("src/main.peng");

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

fn import_local_module(
    ctx: &mut PengNativeFunctionCallContext,
    path: &PathBuf,
    prelude_for_import_ref: &Rc<RefCell<PengUnit>>,
    import_cache: &PebbleImportCache,
) -> Result<PengBindedCell, PengError> {
    let canonical_path = match fs::canonicalize(path) {
        Ok(path) => path,

        Err(_) => {
            return Err(PengError::CannotCallValue(format!(
                "local module '{}' not found",
                path.to_string_lossy()
            )));
        }
    };

    let modified = match fs::metadata(&canonical_path) {
        Ok(metadata) => match metadata.modified() {
            Ok(time) => Some(time),
            Err(_) => None,
        },

        Err(_) => None,
    };

    let cached_unit = {
        let cache = import_cache.borrow();

        match cache.get(&canonical_path) {
            Some(PebbleImportCacheEntry::Loaded {
                unit,
                modified: cached_modified,
            }) => {
                if *cached_modified == modified {
                    Some(unit.clone())
                } else {
                    None
                }
            }

            Some(PebbleImportCacheEntry::Loading { unit, .. }) => Some(unit.clone()),

            None => None,
        }
    };

    if let Some(cached_unit) = cached_unit {
        let module_ptr = cached_unit.create_module_heap(ctx.env_mut());

        return Ok(PengBinded::Immutable(PengCell::Reference(module_ptr)));
    }

    let prelude = prelude_for_import_ref.borrow().clone();

    let imported_unit = if canonical_path.to_string_lossy().ends_with(".penb") {
        let bytes = match fs::read(&canonical_path) {
            Ok(bytes) => bytes,

            Err(e) => {
                return Err(PengError::CannotCallValue(format!(
                    "failed to read binary module '{}': {}",
                    canonical_path.to_string_lossy(),
                    e
                )));
            }
        };

        match ctx.env_mut().load_program_from_binary_using(&bytes, &prelude) {
            Ok(unit) => unit,

            Err(e) => {
                return Err(e);
            }
        }
    } else {
        let source = match fs::read_to_string(&canonical_path) {
            Ok(source) => source,

            Err(e) => {
                return Err(PengError::CannotCallValue(format!(
                    "failed to read source module '{}': {}",
                    canonical_path.to_string_lossy(),
                    e
                )));
            }
        };

        match ctx.env_mut().load_program_from_source_using(&source, &prelude, 0) {
            Ok(unit) => unit,

            Err(e) => {
                return Err(e);
            }
        }
    };

    let init = match imported_unit.require_init() {
        Ok(init) => init,

        Err(e) => {
            return Err(e);
        }
    };

    {
        let mut cache = import_cache.borrow_mut();

        cache.insert(
            canonical_path.clone(),
            PebbleImportCacheEntry::Loading {
                unit: imported_unit.clone(),
                modified,
            },
        );
    }

    match ctx.env_mut().run_isolated(init, &imported_unit) {
        Ok(_) => {}

        Err(e) => {
            let mut cache = import_cache.borrow_mut();

            cache.remove(&canonical_path);

            return Err(e);
        }
    }

    {
        let mut cache = import_cache.borrow_mut();

        cache.insert(
            canonical_path.clone(),
            PebbleImportCacheEntry::Loaded {
                unit: imported_unit.clone(),
                modified,
            },
        );
    }

    let module_ptr = imported_unit.create_module_heap(ctx.env_mut());

    Ok(PengBinded::Immutable(PengCell::Reference(module_ptr)))
}