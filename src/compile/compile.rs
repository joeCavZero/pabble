use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

use penguin::prelude::*;

use crate::project::*;
use crate::standard;

pub fn compile(
    entry: &Option<String>,
    output: &Option<String>,
) {
    let input = match resolve_entry(entry.as_ref()) {
        Ok(input) => input,

        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    let input_string = input.to_string_lossy().into_owned();

    let output = resolve_compile_output(
        &input_string,
        output.as_ref(),
    );

    let source = match fs::read_to_string(&input) {
        Ok(source) => source,

        Err(e) => {
            eprintln!("Failed to read '{}':", input.to_string_lossy());
            eprintln!("{e}");
            return;
        }
    };

    let mut peng = PengEnv::new();
    let mut using_unit = PengUnit::library();

    let std_registry =
        match standard::standard::setup(&mut peng, &mut using_unit) {
            Ok(std_registry) => std_registry,

            Err(e) => {
                eprintln!("Failed to setup standard library:");
                eprintln!("{e:#?}");
                return;
            }
        };

    match standard::raise::setup(
        &mut peng,
        &mut using_unit,
    ) {
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

    let import_cache =
        Rc::new(RefCell::new(HashMap::new()));

    match standard::import::setup(
        &mut peng,
        &mut using_unit,
        std_registry,
        import_cache,
        dependencies,
        input.clone(),
    ) {
        Ok(()) => {}

        Err(e) => {
            eprintln!("Failed to setup import:");
            eprintln!("{e:#?}");
            return;
        }
    }

    let bytes =
        match peng.compile_program_to_binary_using(
            &source,
            &using_unit,
            &PengBinaryBuildOptions::default(),
            0,
        ) {
            Ok(bytes) => bytes,

            Err(e) => {
                eprintln!(
                    "Failed to compile '{}':",
                    input.to_string_lossy()
                );
                eprintln!("{e:#?}");
                return;
            }
        };

    match fs::write(&output, bytes) {
        Ok(()) => {
            println!(
                "Compiled '{}' -> '{}'",
                input.to_string_lossy(),
                output
            );
        }

        Err(e) => {
            eprintln!("Failed to write '{output}':");
            eprintln!("{e}");
        }
    }
}
