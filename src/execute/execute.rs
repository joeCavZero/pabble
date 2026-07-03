use crate::standard::*;
use crate::project::*;
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
        match peng.load_program_from_binary_file_using(&root, &core) {
            Ok(unit) => unit,

            Err(e) => {
                println!("{:#?}", e);
                return;
            }
        }
    } else {
        match peng.load_program_from_file_using(&root, &core, 0) {
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