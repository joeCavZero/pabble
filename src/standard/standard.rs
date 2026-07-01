use penguin::prelude::*;
use crate::standard::{custom_access::register_custom_accesses, *};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

pub type PebbleStandardRegistry = HashMap<String, PengUnit>;

pub fn setup(peng: &mut PengEnv, unit: &mut PengUnit) -> Result<PebbleStandardRegistry, PengError> {
    let mut registry = HashMap::new();

    registry.insert("io".to_string(), io::setup(peng));

    registry.insert("threads".to_string(), thread::setup(peng));

    registry.insert("random".to_string(), random::setup(peng));

    registry.insert("convert".to_string(), convert::setup(peng));
    
    registry.insert("fs".to_string(), fs::setup(peng));

    registry.insert("http".to_string(), http::setup(peng));

    registry.insert("os".to_string(), os::setup(peng));

    registry.insert("time".to_string(), time::setup(peng));

    registry.insert("sync".to_string(), sync::setup(peng));

    registry.insert("tcp".to_string(), tcp::setup(peng));

    registry.insert("udp".to_string(), udp::setup(peng));


    let import_cache = Rc::new(RefCell::new(HashMap::new()));

    unit.register_immutable_native_operation(peng, "impls", operations::implements).unwrap();

    register_custom_accesses(peng, unit);

    import::setup(
        peng,
        unit,
        registry.clone(),
        import_cache,
    ).unwrap();

    Ok(registry)
}