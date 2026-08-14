use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

use crate::project::*;
use crate::standard::*;
use penguin::prelude::*;

pub fn execute_from_entry(entry: Option<&String>) {
    let root = match resolve_entry(entry, "main.peng") {
        Ok(root) => root,

        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    let mut peng = PengEnv::new();
    let mut core = PengUnit::library();

    let std_registry = match standard::setup(&mut peng, &mut core) {
        Ok(std_registry) => std_registry,

        Err(e) => {
            println!("{:#?}", e);
            return;
        }
    };

    match crate::standard::raise::setup(&mut peng, &mut core) {
        Ok(()) => {}

        Err(e) => {
            eprintln!("Failed to setup raise:");
            eprintln!("{e:#?}");
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
        dependencies,
    ) {
        Ok(()) => {}

        Err(e) => {
            eprintln!("Failed to setup import:");
            eprintln!("{e:#?}");
            return;
        }
    }

    let unit = if root.ends_with(".penb") {
        let bytes = match fs::read(&root) {
            Ok(bytes) => bytes,

            Err(e) => {
                eprintln!("Failed to read '{root}':");
                eprintln!("{e}");
                return;
            }
        };

        match peng.load_program_from_binary_using(&bytes, &core) {
            Ok(unit) => unit,

            Err(e) => {
                println!("{:#?}", e);
                return;
            }
        }
    } else {
        let source = match fs::read_to_string(&root) {
            Ok(source) => source,

            Err(e) => {
                eprintln!("Failed to read '{root}':");
                eprintln!("{e}");
                return;
            }
        };

        match peng.load_program_from_source_using(&source, &core, 0) {
            Ok(unit) => unit,

            Err(e) => {
                println!("{:#?}", e);
                return;
            }
        }
    };

    let init = match unit.require_init() {
        Ok(init) => init,

        Err(e) => {
            println!("{:#?}", e);
            return;
        }
    };

    match peng.run(init, &unit) {
        Ok(_) => {}

        Err(e) => {
            println!("{:#?}", e);
            return;
        }
    }

    match peng.run_global_function("main", &unit, Vec::new()) {
        Ok(_) => {}

        Err(e) => println!("{:#?}", e),
    }
}
