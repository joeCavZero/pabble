use penguin::prelude::*;
use crate::standard::{custom_access::register_custom_accesses, *};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

pub type PebbleStdRegistry = HashMap<String, PengUnit>;

pub fn setup(peng: &mut PengEnv, unit: &mut PengUnit) -> Result<PebbleStdRegistry, PengError> {
    let mut registry = HashMap::new();

    registry.insert("io".to_string(), io::setup(peng));

    registry.insert("threads".to_string(), thread::setup(peng));

    registry.insert("random".to_string(), random::setup(peng));

    registry.insert("convert".to_string(), convert::setup(peng));

    let import_cache = Rc::new(RefCell::new(HashMap::new()));

    import::setup(
        peng,
        unit,
        registry.clone(),
        import_cache,
    ).unwrap();

    unit.register_native_operation(peng, "impl", operations::implements).unwrap();
    register_custom_accesses(peng, unit);

Ok(registry)
}
