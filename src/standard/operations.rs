use penguin::prelude::*;

pub fn implements(ctx: &mut PengNativeOperationCallContext) -> Result<PengBindedCell, PengError> {
    let left_cell = ctx.get_left_cell().clone();
    let right_cell = ctx.get_right_cell().clone();

    let left_value = match left_cell.value() {
        PengCell::Reference(ptr) => match ctx.get_value(*ptr) {
            Some(value) => value.clone(),
            None => return Err(PengError::HeapValueNotFound(*ptr)),
        },

        cell => PengValue::Cell(cell.clone()),
    };

    let right_value = match right_cell.value() {
        PengCell::Reference(ptr) => match ctx.get_value(*ptr) {
            Some(value) => value.clone(),
            None => return Err(PengError::HeapValueNotFound(*ptr)),
        },

        cell => PengValue::Cell(cell.clone()),
    };

    let result = match &right_value {
        PengValue::Box(PengBox::Type(expected_type)) => {
            match value_impl_type(ctx, &left_value, expected_type) {
                Ok(result) => result,
                Err(e) => return Err(e),
            }
        }

        PengValue::Box(PengBox::Union(union)) => {
            match value_impl_union(ctx, &left_value, union) {
                Ok(result) => result,
                Err(e) => return Err(e),
            }
        }

        _ => {
            return Err(PengError::CannotCallValue(
                "impls expected type or union on right side".into(),
            ));
        }
    };

    Ok(PengBinded::Mutable(PengCell::Bool(result)))
}

fn value_impl_union(
    ctx: &PengNativeOperationCallContext,
    value: &PengValue,
    union: &PengUnion,
) -> Result<bool, PengError> {
    for expected in &union.unions {
        match value_impl_type(ctx, value, expected) {
            Ok(true) => return Ok(true),
            Ok(false) => {}
            Err(e) => return Err(e),
        }
    }

    Ok(false)
}

fn value_impl_type(
    ctx: &PengNativeOperationCallContext,
    value: &PengValue,
    expected: &PengType,
) -> Result<bool, PengError> {
    let result = match expected {
        PengType::Any => true,

        PengType::Nil => matches!(value, PengValue::Cell(PengCell::Nil)),
        PengType::Int => matches!(value, PengValue::Cell(PengCell::Int(_))),
        PengType::Uint => matches!(value, PengValue::Cell(PengCell::Uint(_))),
        PengType::Float32 => matches!(value, PengValue::Cell(PengCell::Float32(_))),
        PengType::Float64 => matches!(value, PengValue::Cell(PengCell::Float64(_))),
        PengType::Byte => matches!(value, PengValue::Cell(PengCell::Byte(_))),
        PengType::Bool => matches!(value, PengValue::Cell(PengCell::Bool(_))),

        PengType::String => matches!(value, PengValue::Box(PengBox::String(_))),
        PengType::Object => matches!(value, PengValue::Box(PengBox::Object(_))),
        PengType::Module => matches!(value, PengValue::Box(PengBox::Module(_))),
        PengType::Function => matches!(value, PengValue::Box(PengBox::Function(_))),
        PengType::Operator => matches!(value, PengValue::Box(PengBox::Operation(_))),
        PengType::Thread => matches!(value, PengValue::Box(PengBox::Thread(_))),

        PengType::Type => {
            matches!(
                value,
                PengValue::Box(PengBox::Type(_)) | PengValue::Box(PengBox::Union(_))
            )
        }

        PengType::Union => matches!(value, PengValue::Box(PengBox::Union(_))),

        PengType::Vector(expected_inner) => match value {
            PengValue::Box(PengBox::Vector(vector)) => {
                if matches!(expected_inner.as_ref(), PengType::Any) {
                    true
                } else {
                    for item in &vector.values {
                        let item_value = match item.value() {
                            PengCell::Reference(ptr) => match ctx.get_value(*ptr) {
                                Some(value) => value.clone(),
                                None => return Err(PengError::HeapValueNotFound(*ptr)),
                            },

                            cell => PengValue::Cell(cell.clone()),
                        };

                        match value_impl_type(ctx, &item_value, expected_inner.as_ref()) {
                            Ok(true) => {}
                            Ok(false) => return Ok(false),
                            Err(e) => return Err(e),
                        }
                    }

                    true
                }
            }

            _ => false,
        },

        PengType::Custom(custom_type) => match value {
            PengValue::Box(PengBox::Object(obj)) => {
                custom_type
                    .fields
                    .keys()
                    .all(|name| obj.fields.contains_key(name))
            }

            _ => false,
        },
    };

    Ok(result)
}