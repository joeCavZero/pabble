use crate::dependencies::utils::*;

pub fn install() {
    match install_project_dependencies(PebbleDependencyCommandMode::Install) {
        Ok(()) => {}

        Err(e) => {
            eprintln!("{e}");
        }
    }
}