use penguin::prelude::*;
use std::{
    cell::RefCell, collections::HashMap, fs, path::PathBuf, process::Command, rc::Rc,
    time::SystemTime,
};

use crate::debug::error::PengErrorSources;
use crate::project::*;
use crate::standard::standard::*;

#[derive(Clone)]
pub enum PabbleImportCacheEntry {
    Loading {
        unit: PengUnit,
        modified: Option<SystemTime>,
    },

    Loaded {
        unit: PengUnit,
        modified: Option<SystemTime>,
    },
}

pub type PabbleImportCache = Rc<RefCell<HashMap<PathBuf, PabbleImportCacheEntry>>>;

pub type PabbleImportBaseStack = Rc<RefCell<Vec<PathBuf>>>;

pub fn setup(
    peng: &mut PengEnv,
    unit: &mut PengUnit,
    std_registry: PabbleStandardRegistry,
    import_cache: PabbleImportCache,
    error_sources: PengErrorSources,
    dependencies: HashMap<String, PabbleDependency>,
    entry_path: PathBuf,
) -> Result<(), PengError> {
    let prelude_for_import = Rc::new(RefCell::new(unit.clone()));

    let prelude_for_import_ref = prelude_for_import.clone();

    let entry_base = match entry_path.parent() {
        Some(parent) => parent.to_path_buf(),

        None => {
            return Err(PengError::CannotCallValue(
                "project entry has no parent directory".into(),
            ));
        }
    };

    let import_base_stack: PabbleImportBaseStack = Rc::new(RefCell::new(vec![entry_base]));

    let import_base_stack_ref = import_base_stack.clone();
    let error_sources_ref = error_sources.clone();

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
                &import_base_stack_ref,
                &error_sources_ref,
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

        let requested_path = PathBuf::from(&path);

        let local_path = if requested_path.is_absolute() {
            requested_path
        } else {
            let stack = import_base_stack_ref.borrow();

            match stack.last() {
                Some(base) => base.join(requested_path),

                None => {
                    return Err(PengError::CannotCallValue(
                        "import() has no base directory".into(),
                    ));
                }
            }
        };

        import_local_module(
            ctx,
            &local_path,
            &prelude_for_import_ref,
            &import_cache,
            &import_base_stack_ref,
            &error_sources_ref,
        )
    }) {
        Ok(_) => {
            *prelude_for_import.borrow_mut() = unit.clone();

            Ok(())
        }

        Err(e) => Err(e),
    }
}

fn resolve_dependency_entry(name: &str, dependency: &PabbleDependency) -> Result<PathBuf, String> {
    match dependency {
        PabbleDependency::Version(version) => resolve_version_dependency_entry(name, version),

        PabbleDependency::Detailed(info) => {
            if let Some(path) = &info.path {
                return resolve_path_dependency_entry(name, path);
            }

            if let Some(git) = &info.git {
                return resolve_git_dependency_entry(name, git, info.version.as_ref());
            }

            if let Some(version) = &info.version {
                return resolve_version_dependency_entry(name, version);
            }

            Err(format!("dependency '{}' has no path, git or version", name))
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

    Err(format!("dependency '{}' path '{}' not found", name, path))
}

fn resolve_version_dependency_entry(name: &str, version: &str) -> Result<PathBuf, String> {
    let cache_dir = match pabble_dependency_cache_dir() {
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
    let cache_dir = match pabble_dependency_cache_dir() {
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
                return Err(format!("failed to create git dependency cache: {}", e));
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
    let project_file = root.join("pabble.toml");

    if !project_file.exists() {
        return Err(format!(
            "dependency '{}' does not contain pabble.toml",
            name
        ));
    }

    let project = match load_project_from_path(&project_file) {
        Ok(project) => project,

        Err(e) => {
            return Err(format!("failed to load dependency '{}': {}", name, e));
        }
    };

    if project.project.entry.trim().is_empty() {
        return Err(format!("dependency '{}' has empty project.entry", name));
    }

    let entry = root.join(project.project.entry);

    if !entry.exists() {
        return Err(format!(
            "dependency '{}' entry '{}' not found",
            name,
            entry.to_string_lossy()
        ));
    }

    if !entry.is_file() {
        return Err(format!(
            "dependency '{}' entry '{}' is not a file",
            name,
            entry.to_string_lossy()
        ));
    }

    match fs::canonicalize(&entry) {
        Ok(entry) => Ok(entry),

        Err(e) => Err(format!(
            "failed to resolve dependency '{}' entry '{}': {}",
            name,
            entry.to_string_lossy(),
            e
        )),
    }
}

fn pabble_dependency_cache_dir() -> Result<PathBuf, String> {
    let current_dir = match std::env::current_dir() {
        Ok(current_dir) => current_dir,

        Err(e) => {
            return Err(format!("failed to get current directory: {}", e));
        }
    };

    Ok(current_dir.join(".pabble").join("deps"))
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

fn git_clone_dependency(name: &str, git: &str, dependency_root: &PathBuf) -> Result<(), String> {
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

fn git_pull_dependency(name: &str, dependency_root: &PathBuf) -> Result<(), String> {
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
    import_cache: &PabbleImportCache,
    import_base_stack: &PabbleImportBaseStack,
    error_sources: &PengErrorSources,
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
            Some(PabbleImportCacheEntry::Loaded {
                unit,
                modified: cached_modified,
            }) => {
                if *cached_modified == modified {
                    Some(unit.clone())
                } else {
                    None
                }
            }

            Some(PabbleImportCacheEntry::Loading { unit, .. }) => Some(unit.clone()),

            None => None,
        }
    };

    if let Some(cached_unit) = cached_unit {
        let module_ptr = cached_unit.create_module_heap(ctx.env_mut());

        return Ok(PengBinded::Immutable(PengCell::Reference(module_ptr)));
    }

    let prelude = prelude_for_import_ref.borrow().clone();

    let imported_unit = match canonical_path.extension() {
        Some(extension) if extension == "penb" => {
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

            match ctx
                .env_mut()
                .load_program_from_binary_using(&bytes, &prelude)
            {
                Ok(unit) => unit,

                Err(e) => {
                    return Err(e);
                }
            }
        }

        _ => {
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

            let source_id = error_sources.get_or_insert_path(&canonical_path);

            match ctx
                .env_mut()
                .load_program_from_source_using(&source, &prelude, source_id)
            {
                Ok(unit) => unit,

                Err(e) => {
                    return Err(e);
                }
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
            PabbleImportCacheEntry::Loading {
                unit: imported_unit.clone(),
                modified,
            },
        );
    }

    let module_base = match canonical_path.parent() {
        Some(parent) => parent.to_path_buf(),

        None => {
            let mut cache = import_cache.borrow_mut();

            cache.remove(&canonical_path);

            return Err(PengError::CannotCallValue(
                "imported module has no parent directory".into(),
            ));
        }
    };

    import_base_stack.borrow_mut().push(module_base);

    let init_result = ctx.env_mut().run_isolated(init, &imported_unit);

    import_base_stack.borrow_mut().pop();

    match init_result {
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
            PabbleImportCacheEntry::Loaded {
                unit: imported_unit.clone(),
                modified,
            },
        );
    }

    let module_ptr = imported_unit.create_module_heap(ctx.env_mut());

    Ok(PengBinded::Immutable(PengCell::Reference(module_ptr)))
}
