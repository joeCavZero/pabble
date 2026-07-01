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

pub fn new_type(
    ctx: &mut PengNativeFunctionCallContext,
    values: Vec<(&str, PengBindedCell)>,
) -> Result<PengBindedCell, PengError> {
    let mut fields = HashMap::new();

    for (name, value) in values {
        let name_ptr = ctx.env_mut().ensure_pooled_name_ptr(name.to_string());
        fields.insert(name_ptr, value);
    }

    let ptr = ctx.create_box(PengBox::Type(PengType::Custom(PengCustomType {
        fields,
    })));

    Ok(PengBindedCell::Mutable(PengCell::Reference(ptr)))
}
pub fn cell_to_string(
    ctx: &PengNativeFunctionCallContext,
    cell: &PengBindedCell,
) -> Result<String, PengError> {
    match cell.value() {
        PengCell::Reference(ptr) => match ctx.get_value(*ptr) {
            Some(PengValue::Box(PengBox::String(value))) => Ok(value.clone()),

            Some(_) => Err(PengError::CannotCallValue(
                "expected string value".to_string())),

            None => Err(PengError::CannotCallValue(
                "got missing heap string value".to_string())),
        },

        _ => Err(PengError::CannotCallValue(
            "expected string value".to_string())),
    }
}

pub fn cell_to_uint(cell: &PengBindedCell) -> Result<usize, PengError> {
    match cell.value() {
        PengCell::Uint(value) => Ok(*value),

        PengCell::Int(value) => {
            if *value < 0 {
                return Err(PengError::CannotCallValue(
                    "expected non-negative integer value".to_string(),
                    ));
            }

            Ok(*value as usize)
        }

        PengCell::Byte(value) => Ok(*value as usize),

        _ => Err(PengError::CannotCallValue(
            "expected unsigned integer value".to_string()
        )),
    }
}

pub fn cell_to_bool(cell: &PengBindedCell) -> Result<bool, PengError> {
    match cell.value() {
        PengCell::Bool(value) => Ok(*value),

        _ => Err(PengError::CannotCallValue(
            "expected bool value".to_string()
        )),
    }
}

pub fn get_object_fields_from_cell(
    ctx: &PengNativeFunctionCallContext,
    cell: &PengBindedCell,
) -> Result<HashMap<PengNamePoolPtr, PengBindedCell>, PengError> {
    match cell.value() {
        PengCell::Reference(ptr) => match ctx.get_value(*ptr) {
            Some(PengValue::Box(PengBox::Object(object))) => Ok(object.fields.clone()),

            Some(_) => Err(PengError::CannotCallValue(
                "expected object value".to_string())),

            None => Err(PengError::CannotCallValue(
                "got missing heap object value".to_string(),
                )),
        },

        _ => Err(PengError::CannotCallValue(
            "expected object value".to_string())),
    }
}

pub fn nil() -> Result<PengBindedCell, PengError> {
    Ok(PengBindedCell::Mutable(PengCell::Nil))
}

pub fn string(ctx: &mut PengNativeFunctionCallContext, value: String) -> Result<PengBindedCell, PengError> {
    Ok(string_cell(ctx, value))
}

pub fn string_cell(ctx: &mut PengNativeFunctionCallContext, value: String) -> PengBindedCell {
    let ptr = ctx.create_box(PengBox::String(value));

    PengBindedCell::Mutable(PengCell::Reference(ptr))
}

pub fn object_from_fields(
    ctx: &mut PengNativeFunctionCallContext,
    fields: HashMap<PengNamePoolPtr, PengBindedCell>,
) -> PengBindedCell {
    let ptr = ctx.create_box(PengBox::Object(PengObject { fields }));

    PengBindedCell::Mutable(PengCell::Reference(ptr))
}

pub fn object_from_string_pairs(
    ctx: &mut PengNativeFunctionCallContext,
    values: Vec<(String, String)>,
) -> PengBindedCell {
    let mut fields = HashMap::new();

    for (name, value) in values {
        let name_ptr = ctx.env_mut().ensure_pooled_name_ptr(name);
        fields.insert(name_ptr, string_cell(ctx, value));
    }

    object_from_fields(ctx, fields)
}
