use std::fs;

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

    match standard::setup(&mut peng, &mut core) {
        Ok(_) => {}

        Err(e) => {
            println!("{:#?}", e);
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