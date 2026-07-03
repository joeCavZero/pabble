use crate::dependencies;
use crate::initialize;
use crate::execute::*;
use crate::compile::*;

#[derive(Debug)]
pub enum PabbleCLI {
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

impl PabbleCLI {
    pub fn new(os_args: Vec<String>) -> Self {
        let mut args = os_args.into_iter();

        args.next();

        match args.next().as_deref() {
            Some("run") => PabbleCLI::Run { entry: args.next() },

            Some("compile") => PabbleCLI::Compile {
                entry: args.next(),
                output: args.next(),
            },

            Some("fetch") => PabbleCLI::Fetch,

            Some("install") => PabbleCLI::Install,

            Some("update") => PabbleCLI::Update,

            Some("init") => PabbleCLI::Init,

            Some("new") => match args.next() {
                Some(name) => PabbleCLI::New { name },
                None => PabbleCLI::Help,
            },

            Some("version") | Some("-v") | Some("--version") => PabbleCLI::Version,

            Some("help") | Some("-h") | Some("--help") => PabbleCLI::Help,

            Some(_) | None => PabbleCLI::Help,
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
    println!("Pabble {}", env!("CARGO_PKG_VERSION"));
}

fn print_help() {
    println!(
        "\
Pabble - Penguin Package Manager

USAGE:
    pabble <COMMAND>

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
    version                       Show Pabble version
    help                          Show this help
"
    );
}
