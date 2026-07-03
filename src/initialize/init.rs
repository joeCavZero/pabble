use std::fs;
use std::path::Path;

pub fn init() {
    let project_path = Path::new("pebble.toml");
    let src_path = Path::new("src");
    let main_path = Path::new("src/main.peng");

    if project_path.exists() {
        eprintln!("Project already initialized: pebble.toml already exists");
        return;
    }

    match fs::create_dir_all(src_path) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to create src directory:");
            eprintln!("{e:#?}");
            return;
        }
    }

    match fs::write(project_path, default_pebble_toml("my_project")) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to create pebble.toml:");
            eprintln!("{e:#?}");
            return;
        }
    }

    if !main_path.exists() {
        match fs::write(main_path, default_main_peng()) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to create src/main.peng:");
                eprintln!("{e:#?}");
                return;
            }
        }
    }

    println!("Initialized Pebble project");
}

fn default_pebble_toml(name: &str) -> String {
    format!(
        "\
[project]
name = \"{name}\"
version = \"0.1.0\"
entry = \"src/main.peng\"

[dependencies]
"
    )
}

fn default_main_peng() -> String {
    "\
import(\"io\") as io

func main() {
    io:println(\"Hello, Penguin!\")
}
"
    .to_string()
}