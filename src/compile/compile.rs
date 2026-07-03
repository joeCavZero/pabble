use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use penguin::prelude::*;

use crate::project::*;
use crate::standard::standard;

pub fn compile(entry: &Option<String>, output: &Option<String>) {
    let input = match resolve_entry(entry.as_ref(), "main.peng") {
        Ok(input) => input,

        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    let output = resolve_compile_output(&input, output.as_ref());

    let mut peng = PengEnv::new();
    let mut using_unit = PengUnit::library();

    let std_registry = match standard::setup(&mut peng, &mut using_unit) {
        Ok(std_registry) => std_registry,

        Err(e) => {
            eprintln!("Failed to setup standard library:");
            eprintln!("{e:#?}");
            return;
        }
    };

    let import_cache = Rc::new(RefCell::new(HashMap::new()));

    match crate::standard::import::setup(&mut peng, &mut using_unit, std_registry, import_cache) {
        Ok(()) => {}

        Err(e) => {
            eprintln!("Failed to setup import:");
            eprintln!("{e:#?}");
            return;
        }
    }

    match peng.compile_program_to_binary_file_using(
        &input,
        &output,
        &using_unit,
        &PengBinaryBuildOptions::default(),
        0,
    ) {
        Ok(()) => {
            println!("Compiled '{input}' -> '{output}'");
        }

        Err(e) => {
            eprintln!("Failed to compile '{input}':");
            eprintln!("{e:#?}");
        }
    }
}