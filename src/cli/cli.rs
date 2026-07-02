use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use penguin::prelude::*;

use crate::standard::standard;

#[derive(Debug)]
pub enum PebbleCLI {
    Run {
        entry: Option<String>,
    },
    Compile {
        entry: Option<String>,
        output: Option<String>,
    },
    Init,
    New {
        name: String,
    },
    Version,
    Help,
}

impl PebbleCLI {
    pub fn new(os_args: Vec<String>) -> Self {
        let mut args = os_args.into_iter();

        args.next();

        match args.next().as_deref() {
            Some("run") => PebbleCLI::Run { entry: args.next() },

            Some("compile") => PebbleCLI::Compile {
                entry: args.next(),
                output: args.next(),
            },

            Some("init") => PebbleCLI::Init,

            Some("new") => match args.next() {
                Some(name) => PebbleCLI::New { name },
                None => PebbleCLI::Help,
            },

            Some("version") | Some("-v") | Some("--version") => PebbleCLI::Version,

            Some("help") | Some("-h") | Some("--help") => PebbleCLI::Help,

            Some(_) | None => PebbleCLI::Help,
        }
    }

    pub fn run(&self) {
        match self {
            Self::Run { entry } => match entry {
                Some(ent) => self.run_program(Some(ent)),
                None => self.run_program(None),
            },

            Self::Compile { entry, output } => {
                self.run_compile(entry.as_deref(), output.as_deref())
            }

            Self::Init => self.run_init(),
            Self::New { name } => self.run_new(name),
            Self::Version => self.run_version(),
            Self::Help => self.run_help(),
        }
    }

    fn run_compile(&self, entry: Option<&str>, output: Option<&str>) {
        let input = match entry {
            Some(entry) => entry.to_string(),
            None => "main.peng".to_string(),
        };

        let output = match output {
            Some(output) => output.to_string(),

            None => {
                let path = std::path::Path::new(&input);

                match path.file_stem() {
                    Some(stem) => match stem.to_str() {
                        Some(stem) => format!("{stem}.penb"),
                        None => "main.penb".to_string(),
                    },

                    None => "main.penb".to_string(),
                }
            }
        };

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

        match crate::standard::import::setup(&mut peng, &mut using_unit, std_registry, import_cache)
        {
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

    fn run_init(&self) {
        println!("Initializing project...");
        // TODO:
        // - create pebble.toml if it doesn't exist
        // - create src/
        // - create src/main.peng
    }

    fn run_new(&self, name: &str) {
        println!("Creating project '{name}'...");
        // TODO:
        // - create directory
        // - generate project skeleton
    }

    fn run_version(&self) {
        println!("Pebble {}", env!("CARGO_PKG_VERSION"));
    }

    fn run_help(&self) {
        println!(
            "\
Pebble - Penguin Package Manager

USAGE:
    pebble <COMMAND>

COMMANDS:
    new <name>                    Create a new project
    init                          Initialize a project in the current directory
    run                           Run the current project
    run <file>                    Run a specific source file
    compile                       Compile main.peng to main.penb
    compile <file>                Compile a specific source file to .penb
    compile <file> <output>       Compile a specific source file to a custom output
    version                       Show Pebble version
    help                          Show this help
"
        );
    }
}
