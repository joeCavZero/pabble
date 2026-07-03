use crate::dependencies::utils::*;

pub fn update() {
    match install_project_dependencies(PebbleDependencyCommandMode::Update) {
        Ok(()) => {}

        Err(e) => {
            eprintln!("{e}");
        }
    }
}