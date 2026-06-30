use std::collections::HashMap;

use penguin::prelude::*;

pub fn new_object(
    ctx: &mut PengNativeFunctionCallContext,
    values: Vec<(&str, PengBindedCell)>,
) -> Result<PengBindedCell, PengError> {
    let mut fields = HashMap::new();

    for (name, value) in values {
        let name_ptr = ctx.env_mut().ensure_pooled_name_ptr(name.to_string());
        fields.insert(name_ptr, value);
    }

    let ptr = ctx.create_box(PengBox::Object(PengObject { fields }));

    Ok(PengBindedCell::Mutable(PengCell::Reference(ptr)))
}
