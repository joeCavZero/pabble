use penguin::prelude::*;
use crate::standard::*;
use std::collections::HashMap;

pub type StdRegistry = HashMap<String, PengUnit>;

pub fn setup(peng: &mut PengEnv, unit: &mut PengUnit) -> Result<StdRegistry, PengError> {
    let mut registry = HashMap::new();

    let io = io::setup(peng);
    registry.insert("io".to_string(), io);

    import::setup(peng, unit, registry.clone());

    match unit.register_native_operation(peng, "impl", implements::implements) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    Ok(registry)
}

