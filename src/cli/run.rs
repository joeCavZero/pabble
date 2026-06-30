use crate::cli::*;
use crate::standard::*;
use penguin::prelude::*;

impl PebbleCLI {
    pub fn run_program(&self, entry: Option<&String>) {
        let root = match entry {
            Some(path) => path,

            None => &".".to_string(),
        };

        let mut peng = PengEnv::new();
        let mut core = PengUnit::library();

        match standard::setup(&mut peng, &mut core) {
            Ok(_) => {}
            Err(e) => println!("{:#?}", e),
        }

        let unit = match peng.load_program_from_file_using(root, &core, 0) {
            Ok(unit) => unit,
            Err(e) => {
                println!("{:#?}", e);
                return;
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
}
