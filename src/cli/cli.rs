#[derive(Debug)]
pub enum PebbleCLI {
    Run { entry: Option<String> },
    Init,
    New { name: String },
    Version,
    Help,
}

impl PebbleCLI {
    pub fn new(os_args: Vec<String>) -> Self {
        let mut args = os_args.into_iter();

        args.next();

        match args.next().as_deref() {
            Some("run") => PebbleCLI::Run { entry: args.next() },

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
                match entry {
                    Some(ent) => self.run_run(Some(ent)),
                    None => self.run_run(None),
                }
            },
            Self::Init => self.run_init(),
            Self::New { name } => self.run_new(name),
            Self::Version => self.run_version(),
            Self::Help => self.run_help(),
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
    new <name>      Create a new project
    init            Initialize a project in the current directory
    run             Run the current project
    run <file>      Run a specific source file
    version         Show Pebble version
    help            Show this help
"
        );
    }
}
