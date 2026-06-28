use penguin::prelude::*;
use std::{
    cell::RefCell,
    collections::HashMap,
    path::PathBuf,
    rc::Rc,
    time::SystemTime,
};

use crate::standard::standard::*;

#[derive(Clone)]
pub struct PebbleImportCacheEntry {
    pub unit: PengUnit,
    pub modified: Option<SystemTime>,
}

pub type PebbleImportCache = Rc<RefCell<HashMap<PathBuf, PebbleImportCacheEntry>>>;

pub fn setup(
    peng: &mut PengEnv,
    unit: &mut PengUnit,
    std_registry: PebbleStdRegistry,
    import_cache: PebbleImportCache,
) -> Result<(), PengError> {
    let prelude_for_import = unit.clone();

    match unit.register_native_function(peng, "import", move |ctx| {
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
                return Err(PengError::CannotCallValue(
                    "import() expected path".into(),
                ));
            }
        };

        // 1. Standard library import
        if let Some(std_unit) = std_registry.get(&path) {
            let module_ptr = std_unit.create_module_heap(ctx.env_mut());

            return Ok(PengBinded::Immutable(PengCell::Reference(module_ptr)));
        }

        // 2. Local file import
        let is_local_import =
            path.starts_with("./") || path.starts_with("../") || path.ends_with(".peng");

        if !is_local_import {
            return Err(PengError::CannotCallValue(format!(
                "module '{}' not found",
                path
            )));
        }

        let path_buf = PathBuf::from(&path);

        let canonical_path = match std::fs::canonicalize(&path_buf) {
            Ok(path) => path,

            Err(_) => {
                return Err(PengError::CannotCallValue(format!(
                    "local module '{}' not found",
                    path
                )));
            }
        };

        let modified = match std::fs::metadata(&canonical_path) {
            Ok(metadata) => match metadata.modified() {
                Ok(time) => Some(time),
                Err(_) => None,
            },

            Err(_) => None,
        };

        {
            let cache = import_cache.borrow();

            if let Some(entry) = cache.get(&canonical_path) {
                if entry.modified == modified {
                    let module_ptr = entry.unit.create_module_heap(ctx.env_mut());

                    return Ok(PengBinded::Immutable(PengCell::Reference(module_ptr)));
                }
            }
        }

        let canonical_path_string = canonical_path.to_string_lossy().to_string();

        let imported_unit = match ctx.env_mut().load_program_from_file_using(
            &canonical_path_string,
            &prelude_for_import,
            0,
        ) {
            Ok(unit) => unit,

            Err(e) => {
                return Err(e);
            }
        };

        let init = match imported_unit.require_init() {
            Ok(init) => init,

            Err(e) => {
                return Err(e);
            }
        };

        match ctx.env_mut().run_isolated(init, &imported_unit) {
            Ok(_) => {}

            Err(e) => {
                return Err(e);
            }
        }

        {
            let mut cache = import_cache.borrow_mut();

            cache.insert(
                canonical_path,
                PebbleImportCacheEntry {
                    unit: imported_unit.clone(),
                    modified,
                },
            );
        }

        let module_ptr = imported_unit.create_module_heap(ctx.env_mut());

        Ok(PengBinded::Immutable(PengCell::Reference(module_ptr)))
    }) {
        Ok(_) => Ok(()),

        Err(e) => Err(e),
    }
}