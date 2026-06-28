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

        let expected_type = match right_value {
            PengValue::Box(PengBox::Type(t)) => t,
            _ => {
                return Err(PengError::CannotCallValue(
                    "impl expected type on right side".into(),
                ));
            }
        };

        let result = value_impl_type(&left_value, &expected_type);

        Ok(PengBinded::Mutable(PengCell::Bool(result)))
    }


fn value_impl_type(value: &PengValue, expected: &PengType) -> bool {
    match expected {
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
        PengType::Type => matches!(value, PengValue::Box(PengBox::Type(_))),
        PengType::Union => matches!(value, PengValue::Box(PengBox::Union(_))),

        PengType::Vector(_) => {
            matches!(value, PengValue::Box(PengBox::Vector(_)))
        }

        PengType::Custom(custom_type) => match value {
            PengValue::Box(PengBox::Object(obj)) => {
                custom_type
                    .fields
                    .keys()
                    .all(|name| obj.fields.contains_key(name))
            }

            _ => false,
        },
    }
}