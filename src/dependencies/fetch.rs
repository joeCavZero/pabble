use crate::dependencies::utils::*;

pub fn fetch() {
    match install_project_dependencies(PabbleDependencyCommandMode::Fetch) {
        Ok(()) => {}

        Err(e) => {
            eprintln!("{e}");
        }
    }
}
