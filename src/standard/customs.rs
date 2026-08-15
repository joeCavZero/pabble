use penguin::prelude::*;

use crate::standard::utils::*;

pub fn register_customs(
    peng: &mut PengEnv,
    unit: &mut PengUnit,
) -> Result<(), PengError> {
    // CUSTOM ACCESSES
    match unit.register_custom_access(peng, "len", len) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_access(peng, "sum", sum) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_access(peng, "push", push) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_access(peng, "keys", keys) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_access(peng, "resume", resume) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_access(peng, "pause", pause) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_access(peng, "cancel", cancel) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_access(peng, "state", state) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_access(peng, "join", join) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_access(peng, "get", get) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_access(peng, "is_finished", is_finished) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    // CUSTOM CALL
    match unit.register_custom_call(__self) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    // CUSTOM OPERATIONS
    match unit.register_custom_add(__add) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_subtract(__sub) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_multiply(__mul) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_divide(__div) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_power(__pow) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_remainder(__rem) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_negate(__neg) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_concat(__concat) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_and(__and) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_or(__or) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_not(__not) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_equals(__eq) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_not_equals(__ne) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_greater_than(__gt) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_greater_equals_than(__ge) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_less_than(__lt) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match unit.register_custom_less_equals_than(__le) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    Ok(())
}

// CUSTOM ACCESSES

pub fn len(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    if let Some(value) = ctx.get_arg_value(0) {
        let len = match value.value() {
            PengValue::Box(PengBox::Vector(v)) => v.values.len(),

            PengValue::Box(PengBox::Object(o)) => o.fields.len(),

            PengValue::Box(PengBox::Module(m)) => m.members.len(),

            PengValue::Box(PengBox::Type(PengType::Custom(t))) => t.fields.len(),

            PengValue::Box(PengBox::String(s)) => s.chars().count(),

            _ => {
                return Err(PengError::CannotCallValue(
                    "len() not supported for this value".into(),
                ));
            }
        };

        return Ok(PengBinded::Mutable(PengCell::Uint(len)));
    } else {
    }
    Err(PengError::CannotCallValue(
        "len() not supported for this value".into(),
    ))
}

pub fn sum(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let value = match ctx.get_arg_value(0) {
        Some(value) => value,
        None => {
            return Err(PengError::CannotCallValue("sum() expected receiver".into()));
        }
    };

    let vector = match value.value() {
        PengValue::Box(PengBox::Vector(v)) => v,
        _ => {
            return Err(PengError::CannotCallValue("sum() expected vector".into()));
        }
    };

    let mut sum = PengCell::Int(0);

    for item in &vector.values {
        sum = match (&sum, item.value()) {
            (PengCell::Int(left), PengCell::Int(right)) => PengCell::Int(left + right),

            (PengCell::Uint(left), PengCell::Uint(right)) => PengCell::Uint(left + right),

            (PengCell::Byte(left), PengCell::Byte(right)) => PengCell::Byte(left + right),

            (PengCell::Float32(left), PengCell::Float32(right)) => PengCell::Float32(left + right),

            (PengCell::Float64(left), PengCell::Float64(right)) => PengCell::Float64(left + right),

            // primeiro item define o tipo real da soma
            (PengCell::Int(0), PengCell::Uint(right)) => PengCell::Uint(*right),

            (PengCell::Int(0), PengCell::Byte(right)) => PengCell::Byte(*right),

            (PengCell::Int(0), PengCell::Float32(right)) => PengCell::Float32(*right),

            (PengCell::Int(0), PengCell::Float64(right)) => PengCell::Float64(*right),

            _ => {
                return Err(PengError::CannotCallValue(
                    "sum() expected vector with numbers of the same type".into(),
                ));
            }
        };
    }

    Ok(PengBinded::Mutable(sum))
}

pub fn push(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let item = match ctx.get_arg_cell(1) {
        Some(item) => item.clone(),
        None => {
            return Err(PengError::CannotCallValue(
                "push() expected one argument".into(),
            ));
        }
    };

    let mut value = match ctx.get_arg_value_mut(0) {
        Some(value) => value,
        None => {
            return Err(PengError::CannotCallValue(
                "push() expected mutable vector".into(),
            ));
        }
    };

    let vector = match value.value_mut() {
        PengValue::Box(PengBox::Vector(v)) => v,
        _ => {
            return Err(PengError::CannotCallValue(
                "push() expected mutable vector".into(),
            ));
        }
    };

    vector.values.push(item);

    Ok(PengBinded::Mutable(PengCell::Nil))
}

pub fn keys(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let names: Vec<String> = match ctx.get_arg_value(0) {
        Some(value) => match value.value() {
            PengValue::Box(PengBox::Object(obj)) => obj
                .fields
                .keys()
                .filter_map(|name| ctx.env().get_pooled_name(*name).cloned())
                .collect(),

            PengValue::Box(PengBox::Module(module)) => module
                .members
                .keys()
                .filter_map(|name| ctx.env().get_pooled_name(*name).cloned())
                .collect(),

            PengValue::Box(PengBox::Type(PengType::Custom(ty))) => ty
                .fields
                .keys()
                .filter_map(|name| ctx.env().get_pooled_name(*name).cloned())
                .collect(),

            _ => {
                return Err(PengError::CannotCallValue(
                    "keys() expected object, module or type".into(),
                ));
            }
        },

        None => {
            return Err(PengError::CannotCallValue(
                "keys() expected receiver".into(),
            ));
        }
    };

    let values = names
        .into_iter()
        .map(|name| {
            let ptr = ctx
                .env_mut()
                .create_heap_value(PengValue::Box(PengBox::String(name)));

            PengBinded::Mutable(PengCell::Reference(ptr))
        })
        .collect();

    let ptr = ctx
        .env_mut()
        .create_heap_value(PengValue::Box(PengBox::Vector(PengVector::new(values))));

    Ok(PengBinded::Mutable(PengCell::Reference(ptr)))
}

pub fn resume(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let mut value = match ctx.get_arg_value_mut(0) {
        Some(value) => value,
        None => {
            return Err(PengError::CannotCallValue(
                "resume() expected thread".into(),
            ));
        }
    };

    match value.value_mut() {
        PengValue::Box(PengBox::Thread(t)) => {
            t.state = PengThreadState::Running;
            Ok(PengBinded::Mutable(PengCell::Nil))
        }

        _ => Err(PengError::CannotCallValue(
            "resume() expected thread".into(),
        )),
    }
}

pub fn pause(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let mut value = match ctx.get_arg_value_mut(0) {
        Some(value) => value,
        None => {
            return Err(PengError::CannotCallValue("pause() expected thread".into()));
        }
    };

    match value.value_mut() {
        PengValue::Box(PengBox::Thread(t)) => {
            t.state = PengThreadState::Paused;
            Ok(PengBinded::Mutable(PengCell::Nil))
        }

        _ => Err(PengError::CannotCallValue("pause() expected thread".into())),
    }
}

pub fn cancel(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let mut value = match ctx.get_arg_value_mut(0) {
        Some(value) => value,
        None => {
            return Err(PengError::CannotCallValue(
                "cancel() expected thread".into(),
            ));
        }
    };

    match value.value_mut() {
        PengValue::Box(PengBox::Thread(t)) => {
            t.state = PengThreadState::Cancelled;
            Ok(PengBinded::Mutable(PengCell::Nil))
        }

        _ => Err(PengError::CannotCallValue(
            "cancel() expected thread".into(),
        )),
    }
}

pub fn state(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let state = match ctx.get_arg_value(0) {
        Some(value) => match value.value() {
            PengValue::Box(PengBox::Thread(t)) => match t.state {
                PengThreadState::Running => "running",
                PengThreadState::Finished => "finished",
                PengThreadState::Paused => "paused",
                PengThreadState::Waiting(_) => "waiting",
                PengThreadState::Cancelled => "cancelled",
                PengThreadState::Failed => "failed",
                PengThreadState::Sleeping(_) => "sleeping",
            },

            _ => {
                return Err(PengError::CannotCallValue("state() expected thread".into()));
            }
        },

        None => {
            return Err(PengError::CannotCallValue("state() expected thread".into()));
        }
    };

    let ptr = ctx
        .env_mut()
        .create_heap_value(PengValue::Box(PengBox::String(state.to_string())));

    Ok(PengBinded::Mutable(PengCell::Reference(ptr)))
}

pub fn join(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let target_ptr = match ctx.get_arg_cell(0) {
        Some(arg) => match arg.value() {
            PengCell::Reference(ptr) => *ptr,

            _ => {
                return Err(PengError::CannotCallValue(
                    "threads:join() expected thread".into(),
                ));
            }
        },

        None => {
            return Err(PengError::CannotCallValue(
                "threads:join() expected thread".into(),
            ));
        }
    };

    match ctx.get_value(target_ptr) {
        Some(PengValue::Box(PengBox::Thread(thread))) => {
            match &thread.result {
                PengThreadResult::Returned(value) => {
                    return Ok(value.clone());
                }

                PengThreadResult::Failed(e) => {
                    return Err((**e).clone());
                }

                PengThreadResult::Pending => {}
            }
        }

        Some(_) => {
            return Err(PengError::CannotCallValue(
                "threads:join() expected thread".into(),
            ));
        }

        None => {
            return Err(PengError::ThreadNotFound(target_ptr));
        }
    }

    match ctx.set_current_thread_state(PengThreadState::Waiting(target_ptr)) {
        Ok(_) => {}

        Err(e) => {
            return Err(e);
        }
    }

    match ctx.yield_now() {
        Ok(_) => {}

        Err(e) => {
            return Err(e);
        }
    }

    Ok(PengBinded::Mutable(PengCell::Nil))
}

pub fn get(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let receiver = match ctx.get_arg_cell(0) {
        Some(cell) => cell.clone(),
        None => {
            return Err(PengError::CannotCallValue(
                "get() expected receiver".into(),
            ));
        }
    };

    let default = match ctx.get_arg_cell(2) {
        Some(cell) => cell.clone(),
        None => PengBinded::Mutable(PengCell::Nil),
    };

    let key_name_ptr = match ctx.get_arg_cell(1) {
        Some(cell) => {
            match cell.value() {
                PengCell::Reference(ptr) => {
                    let string = match ctx.get_value(*ptr) {
                        Some(PengValue::Box(PengBox::String(s))) => s.clone(),
                        Some(_) => String::new(),
                        None => return Err(PengError::HeapValueNotFound(*ptr)),
                    };

                    if string.is_empty() {
                        None
                    } else {
                        Some(ctx.env_mut().ensure_pooled_name_ptr(string))
                    }
                }

                _ => None,
            }
        }

        None => None,
    };

    let key_index = match ctx.get_arg_cell(1) {
        Some(cell) => {
            match cell.value() {
                PengCell::Int(v) => {
                    if *v < 0 {
                        None
                    } else {
                        Some(*v as usize)
                    }
                }

                PengCell::Uint(v) => Some(*v),

                _ => None,
            }
        }

        None => None,
    };

    let receiver_ptr = match receiver.value() {
        PengCell::Reference(ptr) => *ptr,

        _ => {
            return Ok(receiver);
        }
    };

    match ctx.get_value(receiver_ptr) {
        Some(PengValue::Box(PengBox::Thread(thread))) => {
            match &thread.result {
                PengThreadResult::Returned(cell) => Ok(cell.clone()),
                PengThreadResult::Pending => Ok(default),
                PengThreadResult::Failed(e) => Err((**e).clone()),
            }
        }

        Some(PengValue::Box(PengBox::Vector(vector))) => {
            let index = match key_index {
                Some(i) => i,
                None => return Ok(default),
            };

            match vector.values.get(index) {
                Some(value) => Ok(value.clone()),
                None => Ok(default),
            }
        }

        Some(PengValue::Box(PengBox::Object(object))) => {
            let name = match key_name_ptr {
                Some(name) => name,
                None => return Ok(default),
            };

            match object.fields.get(&name) {
                Some(value) => Ok(value.clone()),
                None => Ok(default),
            }
        }

        Some(PengValue::Box(PengBox::Type(PengType::Custom(custom_type)))) => {
            let name = match key_name_ptr {
                Some(name) => name,
                None => return Ok(default),
            };

            match custom_type.fields.get(&name) {
                Some(value) => Ok(value.clone()),
                None => Ok(default),
            }
        }

        Some(_) => Ok(default),

        None => Err(PengError::HeapValueNotFound(receiver_ptr)),
    }
}

pub fn is_finished(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let receiver = match ctx.get_arg_cell(0) {
        Some(cell) => cell,

        None => {
            return Err(PengError::CannotCallValue(
                "is_finished() expected receiver".into(),
            ));
        }
    };
    let receiver_ptr = match receiver.value() {
        PengCell::Reference(ptr) => *ptr,

        _ => {
            return Err(PengError::ExpectedThread);
        }
    };

    match ctx.get_value(receiver_ptr) {
        Some(PengValue::Box(PengBox::Thread(thread))) => {
            Ok(PengBinded::Mutable(PengCell::Bool(matches!(
                thread.state,
                PengThreadState::Finished
            ))))
        }

        Some(_) => Err(PengError::ExpectedThread),

        None => Err(PengError::ThreadNotFound(receiver_ptr)),
    }
}

// CUSTOM OPERATORS

pub fn __add(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    custom_binary_method(ctx, "__add")
}

pub fn __sub(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    custom_binary_method(ctx, "__sub")
}

pub fn __mul(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    custom_binary_method(ctx, "__mul")
}

pub fn __div(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    custom_binary_method(ctx, "__div")
}

pub fn __pow(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    custom_binary_method(ctx, "__pow")
}

pub fn __rem(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    custom_binary_method(ctx, "__rem")
}

pub fn __neg(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    custom_unary_method(ctx, "__neg")
}

pub fn __concat(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    custom_binary_method(ctx, "__concat")
}

pub fn __and(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    custom_binary_method(ctx, "__and")
}

pub fn __or(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    custom_binary_method(ctx, "__or")
}

pub fn __not(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    custom_unary_method(ctx, "__not")
}

pub fn __eq(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    custom_binary_method(ctx, "__eq")
}

pub fn __ne(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    custom_binary_method(ctx, "__ne")
}

pub fn __gt(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    custom_binary_method(ctx, "__gt")
}

pub fn __ge(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    custom_binary_method(ctx, "__ge")
}

pub fn __lt(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    custom_binary_method(ctx, "__lt")
}

pub fn __le(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    custom_binary_method(ctx, "__le")
}

pub fn __self(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    let receiver = match ctx.get_arg_cell(0) {
        Some(receiver) => receiver.clone(),

        None => {
            return Err(PengError::CannotCallValue(
                "__self expected receiver".into(),
            ));
        }
    };

    let receiver_ptr = match receiver.value() {
        PengCell::Reference(ptr) => *ptr,

        _ => {
            return Err(PengError::CannotCallValue(
                "__self not supported for this value".into(),
            ));
        }
    };

    let name_ptr = ctx
        .env_mut()
        .ensure_pooled_name_ptr("__self".to_string());

    let method = match ctx.get_value(receiver_ptr) {
        Some(PengValue::Box(PengBox::Type(PengType::Custom(custom_type)))) => {
            match custom_type.fields.get(&name_ptr) {
                Some(method) => method.clone(),

                None => {
                    return Err(PengError::CannotCallValue(
                        "custom type does not implement __self".into(),
                    ));
                }
            }
        }

        Some(_) => {
            return Err(PengError::CannotCallValue(
                "__self not supported for this value".into(),
            ));
        }

        None => {
            return Err(PengError::HeapValueNotFound(receiver_ptr));
        }
    };

    let method_ptr = match method.value() {
        PengCell::Reference(ptr) => *ptr,

        _ => {
            return Err(PengError::CannotCallValue(
                "__self must be a function".into(),
            ));
        }
    };

    match ctx.get_value(method_ptr) {
        Some(PengValue::Box(PengBox::Function(_))) => {}

        Some(_) => {
            return Err(PengError::CannotCallValue(
                "__self must be a function".into(),
            ));
        }

        None => {
            return Err(PengError::HeapValueNotFound(method_ptr));
        }
    }

    Ok(method)
}