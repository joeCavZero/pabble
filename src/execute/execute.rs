use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

use crate::debug::error::{format_peng_error_with_env, PengErrorSources};
use crate::project::*;
use crate::standard::*;
use penguin::prelude::*;

pub fn execute_from_entry(entry: Option<&String>) {
    let entry_path = match resolve_entry(entry) {
        Ok(entry_path) => entry_path,

        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    let mut peng = PengEnv::new();
    let mut core = PengUnit::library();
    let error_sources = PengErrorSources::new();
    error_sources.insert_path(0, &entry_path);

    let std_registry = match standard::setup(&mut peng, &mut core) {
        Ok(std_registry) => std_registry,

        Err(e) => {
            eprintln!("{}", format_peng_error_with_env(e, &error_sources, &peng));
            return;
        }
    };

    match crate::standard::raise::setup(&mut peng, &mut core) {
        Ok(()) => {}

        Err(e) => {
            eprintln!("Failed to setup raise:");
            eprintln!("{}", format_peng_error_with_env(e, &error_sources, &peng));
            return;
        }
    }

    let dependencies = match load_project_from_current_dir() {
        Ok(Some(project)) => project.dependencies,

        Ok(None) => HashMap::new(),

        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    let import_cache = Rc::new(RefCell::new(HashMap::new()));

    match crate::standard::import::setup(
        &mut peng,
        &mut core,
        std_registry,
        import_cache,
        error_sources.clone(),
        dependencies,
        entry_path.clone(),
    ) {
        Ok(()) => {}

        Err(e) => {
            eprintln!("Failed to setup import:");
            eprintln!("{}", format_peng_error_with_env(e, &error_sources, &peng));
            return;
        }
    }

    let unit = match entry_path.extension() {
        Some(extension) if extension == "penb" => {
            let bytes = match fs::read(&entry_path) {
                Ok(bytes) => bytes,

                Err(e) => {
                    eprintln!("Failed to read '{}':", entry_path.to_string_lossy());
                    eprintln!("{e}");
                    return;
                }
            };

            match peng.load_program_from_binary_using(&bytes, &core) {
                Ok(unit) => unit,

                Err(e) => {
                    eprintln!("{}", format_peng_error_with_env(e, &error_sources, &peng));
                    return;
                }
            }
        }

        _ => {
            let source = match fs::read_to_string(&entry_path) {
                Ok(source) => source,

                Err(e) => {
                    eprintln!("Failed to read '{}':", entry_path.to_string_lossy());
                    eprintln!("{e}");
                    return;
                }
            };

            match peng.load_program_from_source_using(&source, &core, 0) {
                Ok(unit) => unit,

                Err(e) => {
                    eprintln!("{}", format_peng_error_with_env(e, &error_sources, &peng));
                    return;
                }
            }
        }
    };

    let init = match unit.require_init() {
        Ok(init) => init,

        Err(e) => {
            eprintln!("{}", format_peng_error_with_env(e, &error_sources, &peng));
            return;
        }
    };

    match peng.run(init, &unit) {
        Ok(_) => {}

        Err(e) => {
            eprintln!("{}", format_peng_error_with_env(e, &error_sources, &peng));
            return;
        }
    }

    match peng.run_global_function("main", &unit, Vec::new()) {
        Ok(_) => {}

        Err(e) => {
            eprintln!("{}", format_peng_error_with_env(e, &error_sources, &peng));
        }
    }
}
