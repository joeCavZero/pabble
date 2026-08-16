use crate::dependencies::utils::*;

pub fn install() {
    match install_project_dependencies(PabbleDependencyCommandMode::Install) {
        Ok(()) => {}

        Err(e) => {
            eprintln!("{e}");
        }
    }
}
