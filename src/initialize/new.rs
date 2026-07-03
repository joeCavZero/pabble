use std::fs;
use std::path::Path;

use crate::initialize::init::init;

pub fn new(name: &str) {
    let root = Path::new(name);

    if root.exists() {
        eprintln!("Failed to create project '{name}': path already exists");
        return;
    }

    match fs::create_dir(root) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to create project directory '{name}':");
            eprintln!("{e:#?}");
            return;
        }
    }

    let current = match std::env::current_dir() {
        Ok(current) => current,
        Err(e) => {
            eprintln!("Failed to get current directory:");
            eprintln!("{e:#?}");
            return;
        }
    };

    match std::env::set_current_dir(root) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to enter project directory '{name}':");
            eprintln!("{e:#?}");
            return;
        }
    }

    init();

    match std::env::set_current_dir(current) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to restore current directory:");
            eprintln!("{e:#?}");
            return;
        }
    }

    println!("Created Pebble project '{name}'");
}