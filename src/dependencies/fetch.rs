use crate::dependencies::utils::*;

pub fn fetch() {
    match install_project_dependencies(PebbleDependencyCommandMode::Fetch) {
        Ok(()) => {}

        Err(e) => {
            eprintln!("{e}");
        }
    }
}