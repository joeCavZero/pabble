use penguin::prelude::*;

pub fn register_custom_accesses(peng: &mut PengEnv, unit: &mut PengUnit) {
    unit.register_custom_access(peng, "len", len).unwrap();
    unit.register_custom_access(peng, "sum", sum).unwrap();
    unit.register_custom_access(peng, "push", push).unwrap();
    unit.register_custom_access(peng, "keys", keys).unwrap();
    unit.register_custom_access(peng, "resume", resume).unwrap();
    unit.register_custom_access(peng, "pause", pause).unwrap();
    unit.register_custom_access(peng, "cancel", cancel).unwrap();
    unit.register_custom_access(peng, "state", state).unwrap();
    unit.register_custom_access(peng, "join", join).unwrap();
}

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
