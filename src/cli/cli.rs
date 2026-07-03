use crate::dependencies;
use crate::initialize;
use crate::execute::*;
use crate::compile::*;

#[derive(Debug)]
pub enum PebbleCLI {
    Run {
        entry: Option<String>,
    },
    Compile {
        entry: Option<String>,
        output: Option<String>,
    },
    Fetch,
    Install,
    Update,
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

            Some("fetch") => PebbleCLI::Fetch,

            Some("install") => PebbleCLI::Install,

            Some("update") => PebbleCLI::Update,

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
            Self::Run { entry } => {
                execute_from_entry(entry.as_ref());
            }

            Self::Compile { entry, output } => {
                compile(entry, output);
            }

            Self::Fetch => {
                dependencies::fetch();
            }

            Self::Install => {
                dependencies::install();
            }

            Self::Update => {
                dependencies::update();
            }

            Self::Init => {
                initialize::init();
            }

            Self::New { name } => {
                initialize::new(name);
            }

            Self::Version => {
                print_version();
            }

            Self::Help => {
                print_help();
            }
        }
    }
}

fn print_version() {
    println!("Pebble {}", env!("CARGO_PKG_VERSION"));
}

fn print_help() {
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
    compile                       Compile the current project
    compile <file>                Compile a specific source file to .penb
    compile <file> <output>       Compile a specific source file to a custom output
    fetch                         Fetch missing dependencies
    install                       Install dependencies
    update                        Update dependencies
    version                       Show Pebble version
    help                          Show this help
"
    );
}
