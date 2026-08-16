use std::{collections::HashMap, io::ErrorKind, time::Duration};

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

pub fn object(
    ctx: &mut PengNativeFunctionCallContext,
    values: Vec<(&str, PengBindedCell)>,
) -> Result<PengBindedCell, PengError> {
    new_object(ctx, values)
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

    let ptr = ctx.create_box(PengBox::Type(PengType::Custom(PengCustomType { fields })));

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
                "expected string value".to_string(),
            )),

            None => Err(PengError::CannotCallValue(
                "got missing heap string value".to_string(),
            )),
        },

        _ => Err(PengError::CannotCallValue(
            "expected string value".to_string(),
        )),
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
            "expected unsigned integer value".to_string(),
        )),
    }
}

pub fn cell_to_int(cell: &PengBindedCell) -> Result<isize, PengError> {
    match cell.value() {
        PengCell::Int(value) => Ok(*value),

        PengCell::Uint(value) => {
            if *value > isize::MAX as usize {
                return Err(PengError::CannotCallValue(
                    "expected int argument".to_string(),
                ));
            }

            Ok(*value as isize)
        }

        PengCell::Byte(value) => Ok(*value as isize),

        _ => Err(PengError::CannotCallValue(
            "expected int argument".to_string(),
        )),
    }
}

pub fn cell_to_bool(cell: &PengBindedCell) -> Result<bool, PengError> {
    match cell.value() {
        PengCell::Bool(value) => Ok(*value),

        _ => Err(PengError::CannotCallValue(
            "expected bool value".to_string(),
        )),
    }
}

pub fn cell_to_number(
    ctx: &PengNativeFunctionCallContext,
    cell: &PengBindedCell,
) -> Result<f64, PengError> {
    match cell.value() {
        PengCell::Int(v) => Ok(*v as f64),
        PengCell::Uint(v) => Ok(*v as f64),
        PengCell::Byte(v) => Ok(*v as f64),
        PengCell::Float32(v) => Ok(*v as f64),
        PengCell::Float64(v) => Ok(*v),

        PengCell::Reference(ptr) => match ctx.get_value(*ptr) {
            Some(PengValue::Cell(PengCell::Int(v))) => Ok(*v as f64),
            Some(PengValue::Cell(PengCell::Uint(v))) => Ok(*v as f64),
            Some(PengValue::Cell(PengCell::Byte(v))) => Ok(*v as f64),
            Some(PengValue::Cell(PengCell::Float32(v))) => Ok(*v as f64),
            Some(PengValue::Cell(PengCell::Float64(v))) => Ok(*v),

            Some(_) => Err(PengError::CannotCallValue(
                "expected number argument".to_string(),
            )),

            None => Err(PengError::CannotCallValue(
                "heap value not found".to_string(),
            )),
        },

        _ => Err(PengError::CannotCallValue(
            "expected number argument".to_string(),
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
                "expected object value".to_string(),
            )),

            None => Err(PengError::CannotCallValue(
                "got missing heap object value".to_string(),
            )),
        },

        _ => Err(PengError::CannotCallValue(
            "expected object value".to_string(),
        )),
    }
}

pub fn nil() -> Result<PengBindedCell, PengError> {
    Ok(PengBindedCell::Mutable(PengCell::Nil))
}

pub fn bool_cell(value: bool) -> Result<PengBindedCell, PengError> {
    Ok(PengBindedCell::Mutable(PengCell::Bool(value)))
}

pub fn int_cell(value: isize) -> Result<PengBindedCell, PengError> {
    Ok(PengBindedCell::Mutable(PengCell::Int(value)))
}

pub fn uint_cell(value: usize) -> Result<PengBindedCell, PengError> {
    Ok(PengBindedCell::Mutable(PengCell::Uint(value)))
}

pub fn byte_cell(value: u8) -> Result<PengBindedCell, PengError> {
    Ok(PengBindedCell::Mutable(PengCell::Byte(value)))
}

pub fn f32_cell(value: f32) -> Result<PengBindedCell, PengError> {
    Ok(PengBindedCell::Mutable(PengCell::Float32(value)))
}

pub fn f64_cell(value: f64) -> Result<PengBindedCell, PengError> {
    Ok(PengBindedCell::Mutable(PengCell::Float64(value)))
}

pub fn string(
    ctx: &mut PengNativeFunctionCallContext,
    value: String,
) -> Result<PengBindedCell, PengError> {
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

pub fn vector(
    ctx: &mut PengNativeFunctionCallContext,
    values: Vec<PengBindedCell>,
) -> Result<PengBindedCell, PengError> {
    let ptr = ctx.create_box(PengBox::Vector(PengVector { values }));

    Ok(PengBindedCell::Mutable(PengCell::Reference(ptr)))
}

pub fn string_binded_cell_from_env(env: &mut PengEnv, value: String) -> PengBindedCell {
    let ptr = env.create_heap_value(PengValue::Box(PengBox::String(value)));

    PengBindedCell::Mutable(PengCell::Reference(ptr))
}

pub fn usize_to_u64_saturating(value: usize) -> u64 {
    if value > u64::MAX as usize {
        u64::MAX
    } else {
        value as u64
    }
}

pub fn is_temporary_read_error(e: &std::io::Error) -> bool {
    e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut
}

pub fn get_map_field(
    ctx: &mut PengNativeFunctionCallContext,
    fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    name: &str,
) -> Option<PengBindedCell> {
    let name_ptr = ctx.env_mut().ensure_pooled_name_ptr(name.to_string());

    fields.get(&name_ptr).cloned()
}

pub fn get_object_field(
    ctx: &mut PengNativeFunctionCallContext,
    object_ptr: PengHeapPtr,
    name: &str,
) -> Result<Option<PengBindedCell>, PengError> {
    let name_ptr = ctx.env_mut().ensure_pooled_name_ptr(name.to_string());

    match ctx.env_mut().get_heap_mut(object_ptr) {
        Some(PengValue::Box(PengBox::Object(object))) => Ok(object.fields.get(&name_ptr).cloned()),
        Some(_) => Err(PengError::CannotCallValue(format!(
            "object field access expected object for '{}'",
            name
        ))),
        None => Err(PengError::HeapValueNotFound(object_ptr)),
    }
}

pub fn timeout_to_duration(value: Option<usize>) -> Option<Duration> {
    value.map(|value| Duration::from_millis(usize_to_u64_saturating(value)))
}

pub fn get_arg_cell(
    ctx: &PengNativeFunctionCallContext,
    index: usize,
) -> Result<PengBindedCell, PengError> {
    match ctx.get_arg_cell(index) {
        Some(arg) => Ok(arg.clone()),
        None => Err(PengError::CannotCallValue("missing argument".to_string())),
    }
}

pub fn get_string_arg(
    ctx: &PengNativeFunctionCallContext,
    index: usize,
) -> Result<String, PengError> {
    let arg = match get_arg_cell(ctx, index) {
        Ok(arg) => arg,
        Err(e) => return Err(e),
    };

    cell_to_string(ctx, &arg)
}

pub fn get_uint_arg(ctx: &PengNativeFunctionCallContext, index: usize) -> Result<usize, PengError> {
    let arg = match get_arg_cell(ctx, index) {
        Ok(arg) => arg,
        Err(e) => return Err(e),
    };

    cell_to_uint(&arg)
}

pub fn get_int_arg(ctx: &PengNativeFunctionCallContext, index: usize) -> Result<isize, PengError> {
    let arg = match get_arg_cell(ctx, index) {
        Ok(arg) => arg,
        Err(e) => return Err(e),
    };

    cell_to_int(&arg)
}

pub fn get_bool_arg(ctx: &PengNativeFunctionCallContext, index: usize) -> Result<bool, PengError> {
    let arg = match get_arg_cell(ctx, index) {
        Ok(arg) => arg,
        Err(e) => return Err(e),
    };

    cell_to_bool(&arg)
}

pub fn unary_f64(
    ctx: &mut PengNativeFunctionCallContext,
    op: fn(f64) -> f64,
) -> Result<PengBindedCell, PengError> {
    let value = match get_number_arg(ctx, 0) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(PengBindedCell::Mutable(PengCell::Float64(op(value))))
}

pub fn binary_f64(
    ctx: &mut PengNativeFunctionCallContext,
    op: fn(f64, f64) -> f64,
) -> Result<PengBindedCell, PengError> {
    let left = match get_number_arg(ctx, 0) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let right = match get_number_arg(ctx, 1) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(PengBindedCell::Mutable(PengCell::Float64(op(left, right))))
}

pub fn get_number_arg(ctx: &PengNativeFunctionCallContext, index: usize) -> Result<f64, PengError> {
    let arg = match get_arg_cell(ctx, index) {
        Ok(arg) => arg,
        Err(e) => return Err(e),
    };

    cell_to_number(ctx, &arg)
}

pub fn custom_binary_method(
    ctx: &mut PengNativeFunctionCallContext,
    method_name: &str,
) -> Result<PengBindedCell, PengError> {
    let receiver = match ctx.get_arg_cell(0) {
        Some(receiver) => receiver.clone(),

        None => {
            return Err(PengError::CannotCallValue(format!(
                "{} expected left operand",
                method_name
            )));
        }
    };

    let _right = match ctx.get_arg_cell(1) {
        Some(right) => right.clone(),

        None => {
            return Err(PengError::CannotCallValue(format!(
                "{} expected right operand",
                method_name
            )));
        }
    };

    let receiver_ptr = match receiver.value() {
        PengCell::Reference(ptr) => *ptr,

        _ => {
            return Err(PengError::CannotCallValue(format!(
                "{} not supported for this value",
                method_name
            )));
        }
    };

    let method_name_ptr = ctx
        .env_mut()
        .ensure_pooled_name_ptr(method_name.to_string());

    let method = match ctx.get_value(receiver_ptr) {
        Some(PengValue::Box(PengBox::Object(object))) => {
            match object.fields.get(&method_name_ptr) {
                Some(method) => method.clone(),

                None => {
                    return Err(PengError::CannotCallValue(format!(
                        "{} not implemented",
                        method_name
                    )));
                }
            }
        }

        Some(PengValue::Box(PengBox::Type(PengType::Custom(custom_type)))) => {
            match custom_type.fields.get(&method_name_ptr) {
                Some(method) => method.clone(),

                None => {
                    return Err(PengError::CannotCallValue(format!(
                        "{} not implemented",
                        method_name
                    )));
                }
            }
        }

        Some(_) => {
            return Err(PengError::CannotCallValue(format!(
                "{} not supported for this value",
                method_name
            )));
        }

        None => {
            return Err(PengError::HeapValueNotFound(receiver_ptr));
        }
    };

    Ok(method)
}

pub fn custom_unary_method(
    ctx: &mut PengNativeFunctionCallContext,
    method_name: &str,
) -> Result<PengBindedCell, PengError> {
    let receiver = match ctx.get_arg_cell(0) {
        Some(receiver) => receiver.clone(),

        None => {
            return Err(PengError::CannotCallValue(format!(
                "{} expected operand",
                method_name
            )));
        }
    };

    let receiver_ptr = match receiver.value() {
        PengCell::Reference(ptr) => *ptr,

        _ => {
            return Err(PengError::CannotCallValue(format!(
                "{} not supported for this value",
                method_name
            )));
        }
    };

    let method_name_ptr = ctx
        .env_mut()
        .ensure_pooled_name_ptr(method_name.to_string());

    match ctx.get_value(receiver_ptr) {
        Some(PengValue::Box(PengBox::Object(object))) => {
            match object.fields.get(&method_name_ptr) {
                Some(method) => Ok(method.clone()),

                None => Err(PengError::CannotCallValue(format!(
                    "{} not implemented",
                    method_name
                ))),
            }
        }

        Some(PengValue::Box(PengBox::Type(PengType::Custom(custom_type)))) => {
            match custom_type.fields.get(&method_name_ptr) {
                Some(method) => Ok(method.clone()),

                None => Err(PengError::CannotCallValue(format!(
                    "{} not implemented",
                    method_name
                ))),
            }
        }

        Some(_) => Err(PengError::CannotCallValue(format!(
            "{} not supported for this value",
            method_name
        ))),

        None => Err(PengError::HeapValueNotFound(receiver_ptr)),
    }
}
