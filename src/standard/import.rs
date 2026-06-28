use penguin::prelude::*;
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Clone)]
pub struct ImportCacheEntry {
    pub unit: PengUnit,
    pub modified: Option<SystemTime>,
}

pub type StdRegistry = HashMap<String, PengUnit>;

pub fn setup(
    peng: &mut PengEnv,
    unit: &mut PengUnit,
    std_registry: StdRegistry,
) {
    unit.register_native_function(peng, "import", move |ctx| {
        let path = match ctx.get_arg_value(0) {
            Some(value) => match value.value() {
                PengValue::Box(PengBox::String(s)) => s.clone(),
                _ => {
                    return Err(PengError::CannotCallValue(
                        "import() expected string path".into(),
                    ));
                }
            },

            None => {
                return Err(PengError::CannotCallValue(
                    "import() expected path".into(),
                ));
            }
        };

        if let Some(std_unit) = std_registry.get(&path) {
            let module_ptr = std_unit.create_module_heap(ctx.env_mut());

            return Ok(PengBinded::Immutable(PengCell::Reference(module_ptr)));
        }

        Err(PengError::CannotCallValue(
            format!("module '{}' not found", path),
        ))
    })
    .unwrap();
}