use penguin::prelude::*;
use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    path::PathBuf,
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
        PebbleDependency::Version(_) => Err(format!(
            "dependency '{}' uses version registry, but registry resolution is not implemented yet",
            name
        )),

        PebbleDependency::Detailed(info) => {
            if let Some(path) = &info.path {
                let path_buf = PathBuf::from(path);

                if path_buf.is_file() {
                    return Ok(path_buf);
                }

                if path_buf.is_dir() {
                    let project_file = path_buf.join("pebble.toml");

                    let project = match load_project_from_path(&project_file) {
                        Ok(project) => project,

                        Err(e) => {
                            return Err(format!(
                                "failed to load dependency '{}': {}",
                                name, e
                            ));
                        }
                    };

                    return Ok(path_buf.join(project.project.entry));
                }

                return Err(format!(
                    "dependency '{}' path '{}' not found",
                    name, path
                ));
            }

            if info.git.is_some() {
                return Err(format!(
                    "dependency '{}' uses git, but git resolution is not implemented yet",
                    name
                ));
            }

            Err(format!(
                "dependency '{}' has no path, git or supported source",
                name
            ))
        }
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