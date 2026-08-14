use penguin::prelude::*;

use crate::standard::{customs::register_customs, *};

use std::collections::HashMap;

pub type PabbleStandardRegistry = HashMap<String, PengUnit>;

pub fn setup(peng: &mut PengEnv, unit: &mut PengUnit) -> Result<PabbleStandardRegistry, PengError> {
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

    registry.insert("math".to_string(), math::setup(peng));

    registry.insert("net".to_string(), net::setup(peng));

    registry.insert("ffi".to_string(), ffi::setup(peng));
    
    registry.insert("crypto".to_string(), crypto::setup(peng));

    unit.register_immutable_native_operation(peng, "impls", operations::implements).unwrap();

    register_customs(peng, unit);

    Ok(registry)
}