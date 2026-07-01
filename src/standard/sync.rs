use std::collections::HashMap;

use penguin::prelude::*;

pub fn setup(peng: &mut PengEnv) -> PengUnit {
    let mut module = PengUnit::library();

    let mutex_type = mutex_type_value(peng);
    match module.register_immutable_global(peng, "Mutex", mutex_type) {
        Ok(_) => {}
        Err(_) => {}
    }

    let atomic_bool_type = atomic_bool_type_value(peng);
    match module.register_immutable_global(peng, "AtomicBool", atomic_bool_type) {
        Ok(_) => {}
        Err(_) => {}
    }

    let atomic_int_type = atomic_int_type_value(peng);
    match module.register_immutable_global(peng, "AtomicInt", atomic_int_type) {
        Ok(_) => {}
        Err(_) => {}
    }

    let channel_type = channel_type_value(peng);
    match module.register_immutable_global(peng, "Channel", channel_type) {
        Ok(_) => {}
        Err(_) => {}
    }

    let once_type = once_type_value(peng);
    match module.register_immutable_global(peng, "Once", once_type) {
        Ok(_) => {}
        Err(_) => {}
    }

    module
}

fn mutex_type_value(peng: &mut PengEnv) -> PengValue {
    let mut fields = HashMap::new();

    let lock_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(mutex_lock),
    )));

    let try_lock_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(mutex_try_lock),
    )));

    let unlock_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(mutex_unlock),
    )));

    let is_locked_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(mutex_is_locked),
    )));

    fields.insert(
        peng.ensure_pooled_name_ptr("locked".to_string()),
        PengBinded::Mutable(PengCell::Bool(false)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("lock".to_string()),
        PengBinded::Immutable(PengCell::Reference(lock_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("try_lock".to_string()),
        PengBinded::Immutable(PengCell::Reference(try_lock_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("unlock".to_string()),
        PengBinded::Immutable(PengCell::Reference(unlock_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("is_locked".to_string()),
        PengBinded::Immutable(PengCell::Reference(is_locked_ptr)),
    );

    PengValue::Box(PengBox::Type(PengType::Custom(PengCustomType {
        fields,
    })))
}

fn atomic_bool_type_value(peng: &mut PengEnv) -> PengValue {
    let mut fields = HashMap::new();

    let get_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(atomic_bool_get),
    )));

    let set_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(atomic_bool_set),
    )));

    let swap_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(atomic_bool_swap),
    )));

    fields.insert(
        peng.ensure_pooled_name_ptr("value".to_string()),
        PengBinded::Mutable(PengCell::Bool(false)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("get".to_string()),
        PengBinded::Immutable(PengCell::Reference(get_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("set".to_string()),
        PengBinded::Immutable(PengCell::Reference(set_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("swap".to_string()),
        PengBinded::Immutable(PengCell::Reference(swap_ptr)),
    );

    PengValue::Box(PengBox::Type(PengType::Custom(PengCustomType {
        fields,
    })))
}

fn atomic_int_type_value(peng: &mut PengEnv) -> PengValue {
    let mut fields = HashMap::new();

    let get_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(atomic_int_get),
    )));

    let set_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(atomic_int_set),
    )));

    let add_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(atomic_int_add),
    )));

    let sub_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(atomic_int_sub),
    )));

    let swap_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(atomic_int_swap),
    )));

    fields.insert(
        peng.ensure_pooled_name_ptr("value".to_string()),
        PengBinded::Mutable(PengCell::Int(0)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("get".to_string()),
        PengBinded::Immutable(PengCell::Reference(get_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("set".to_string()),
        PengBinded::Immutable(PengCell::Reference(set_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("add".to_string()),
        PengBinded::Immutable(PengCell::Reference(add_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("sub".to_string()),
        PengBinded::Immutable(PengCell::Reference(sub_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("swap".to_string()),
        PengBinded::Immutable(PengCell::Reference(swap_ptr)),
    );

    PengValue::Box(PengBox::Type(PengType::Custom(PengCustomType {
        fields,
    })))
}

fn channel_type_value(peng: &mut PengEnv) -> PengValue {
    let mut fields = HashMap::new();

    let send_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(channel_send),
    )));

    let try_recv_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(channel_try_recv),
    )));

    let recv_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(channel_recv),
    )));

    let len_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(channel_len),
    )));

    fields.insert(
        peng.ensure_pooled_name_ptr("values".to_string()),
        PengBinded::Mutable(PengCell::Nil),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("send".to_string()),
        PengBinded::Immutable(PengCell::Reference(send_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("try_recv".to_string()),
        PengBinded::Immutable(PengCell::Reference(try_recv_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("recv".to_string()),
        PengBinded::Immutable(PengCell::Reference(recv_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("len".to_string()),
        PengBinded::Immutable(PengCell::Reference(len_ptr)),
    );

    PengValue::Box(PengBox::Type(PengType::Custom(PengCustomType {
        fields,
    })))
}

fn once_type_value(peng: &mut PengEnv) -> PengValue {
    let mut fields = HashMap::new();

    let try_begin_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(once_try_begin),
    )));

    let finish_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(once_finish),
    )));

    let is_done_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(once_is_done),
    )));

    fields.insert(
        peng.ensure_pooled_name_ptr("state".to_string()),
        PengBinded::Mutable(PengCell::Uint(0)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("try_begin".to_string()),
        PengBinded::Immutable(PengCell::Reference(try_begin_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("finish".to_string()),
        PengBinded::Immutable(PengCell::Reference(finish_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("is_done".to_string()),
        PengBinded::Immutable(PengCell::Reference(is_done_ptr)),
    );

    PengValue::Box(PengBox::Type(PengType::Custom(PengCustomType {
        fields,
    })))
}

fn mutex_lock(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let mutex_ptr = match get_object_ptr_arg(ctx, 0, "Mutex.lock") {
        Ok(ptr) => ptr,
        Err(e) => return Err(e),
    };

    let locked = match get_bool_field(ctx, mutex_ptr, "locked", false, "Mutex.lock") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    if !locked {
        match set_field(
            ctx,
            mutex_ptr,
            "locked",
            PengBinded::Mutable(PengCell::Bool(true)),
        ) {
            Ok(_) => {}
            Err(e) => return Err(e),
        }

        return Ok(PengBinded::Mutable(PengCell::Bool(true)));
    }

    match ctx.set_current_thread_state(PengThreadState::Waiting(mutex_ptr)) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    match ctx.yield_now() {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    Ok(PengBinded::Mutable(PengCell::Bool(false)))
}

fn mutex_try_lock(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let mutex_ptr = match get_object_ptr_arg(ctx, 0, "Mutex.try_lock") {
        Ok(ptr) => ptr,
        Err(e) => return Err(e),
    };

    let locked = match get_bool_field(ctx, mutex_ptr, "locked", false, "Mutex.try_lock") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    if locked {
        return Ok(PengBinded::Mutable(PengCell::Bool(false)));
    }

    match set_field(
        ctx,
        mutex_ptr,
        "locked",
        PengBinded::Mutable(PengCell::Bool(true)),
    ) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    Ok(PengBinded::Mutable(PengCell::Bool(true)))
}

fn mutex_unlock(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let mutex_ptr = match get_object_ptr_arg(ctx, 0, "Mutex.unlock") {
        Ok(ptr) => ptr,
        Err(e) => return Err(e),
    };

    let locked = match get_bool_field(ctx, mutex_ptr, "locked", false, "Mutex.unlock") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    if !locked {
        return Err(PengError::CannotCallValue(
            "sync:Mutex.unlock() called on unlocked mutex".to_string(),
        ));
    }

    match set_field(
        ctx,
        mutex_ptr,
        "locked",
        PengBinded::Mutable(PengCell::Bool(false)),
    ) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    match ctx.env_mut().wake_thread(mutex_ptr) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    Ok(PengBinded::Mutable(PengCell::Nil))
}

fn mutex_is_locked(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let mutex_ptr = match get_object_ptr_arg(ctx, 0, "Mutex.is_locked") {
        Ok(ptr) => ptr,
        Err(e) => return Err(e),
    };

    let locked = match get_bool_field(ctx, mutex_ptr, "locked", false, "Mutex.is_locked") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(PengBinded::Mutable(PengCell::Bool(locked)))
}

fn atomic_bool_get(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let ptr = match get_object_ptr_arg(ctx, 0, "AtomicBool.get") {
        Ok(ptr) => ptr,
        Err(e) => return Err(e),
    };

    let value = match get_bool_field(ctx, ptr, "value", false, "AtomicBool.get") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(PengBinded::Mutable(PengCell::Bool(value)))
}

fn atomic_bool_set(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let ptr = match get_object_ptr_arg(ctx, 0, "AtomicBool.set") {
        Ok(ptr) => ptr,
        Err(e) => return Err(e),
    };

    let value = match get_bool_arg(ctx, 1, "AtomicBool.set") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    match set_field(
        ctx,
        ptr,
        "value",
        PengBinded::Mutable(PengCell::Bool(value)),
    ) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    Ok(PengBinded::Mutable(PengCell::Nil))
}

fn atomic_bool_swap(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let ptr = match get_object_ptr_arg(ctx, 0, "AtomicBool.swap") {
        Ok(ptr) => ptr,
        Err(e) => return Err(e),
    };

    let new_value = match get_bool_arg(ctx, 1, "AtomicBool.swap") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let old_value = match get_bool_field(ctx, ptr, "value", false, "AtomicBool.swap") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    match set_field(
        ctx,
        ptr,
        "value",
        PengBinded::Mutable(PengCell::Bool(new_value)),
    ) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    Ok(PengBinded::Mutable(PengCell::Bool(old_value)))
}

fn atomic_int_get(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let ptr = match get_object_ptr_arg(ctx, 0, "AtomicInt.get") {
        Ok(ptr) => ptr,
        Err(e) => return Err(e),
    };

    let value = match get_int_field(ctx, ptr, "value", 0, "AtomicInt.get") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(PengBinded::Mutable(PengCell::Int(value)))
}

fn atomic_int_set(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let ptr = match get_object_ptr_arg(ctx, 0, "AtomicInt.set") {
        Ok(ptr) => ptr,
        Err(e) => return Err(e),
    };

    let value = match get_int_arg(ctx, 1, "AtomicInt.set") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    match set_field(
        ctx,
        ptr,
        "value",
        PengBinded::Mutable(PengCell::Int(value)),
    ) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    Ok(PengBinded::Mutable(PengCell::Nil))
}

fn atomic_int_add(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let ptr = match get_object_ptr_arg(ctx, 0, "AtomicInt.add") {
        Ok(ptr) => ptr,
        Err(e) => return Err(e),
    };

    let add_value = match get_int_arg(ctx, 1, "AtomicInt.add") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let current = match get_int_field(ctx, ptr, "value", 0, "AtomicInt.add") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let result = match current.checked_add(add_value) {
        Some(value) => value,
        None => {
            return Err(PengError::CannotCallValue(
                "sync:AtomicInt.add() overflow".to_string(),
            ));
        }
    };

    match set_field(
        ctx,
        ptr,
        "value",
        PengBinded::Mutable(PengCell::Int(result)),
    ) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    Ok(PengBinded::Mutable(PengCell::Int(result)))
}

fn atomic_int_sub(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let ptr = match get_object_ptr_arg(ctx, 0, "AtomicInt.sub") {
        Ok(ptr) => ptr,
        Err(e) => return Err(e),
    };

    let sub_value = match get_int_arg(ctx, 1, "AtomicInt.sub") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let current = match get_int_field(ctx, ptr, "value", 0, "AtomicInt.sub") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let result = match current.checked_sub(sub_value) {
        Some(value) => value,
        None => {
            return Err(PengError::CannotCallValue(
                "sync:AtomicInt.sub() overflow".to_string(),
            ));
        }
    };

    match set_field(
        ctx,
        ptr,
        "value",
        PengBinded::Mutable(PengCell::Int(result)),
    ) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    Ok(PengBinded::Mutable(PengCell::Int(result)))
}

fn atomic_int_swap(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let ptr = match get_object_ptr_arg(ctx, 0, "AtomicInt.swap") {
        Ok(ptr) => ptr,
        Err(e) => return Err(e),
    };

    let new_value = match get_int_arg(ctx, 1, "AtomicInt.swap") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let old_value = match get_int_field(ctx, ptr, "value", 0, "AtomicInt.swap") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    match set_field(
        ctx,
        ptr,
        "value",
        PengBinded::Mutable(PengCell::Int(new_value)),
    ) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    Ok(PengBinded::Mutable(PengCell::Int(old_value)))
}

fn channel_send(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let channel_ptr = match get_object_ptr_arg(ctx, 0, "Channel.send") {
        Ok(ptr) => ptr,
        Err(e) => return Err(e),
    };

    let value = match ctx.get_arg_cell(1) {
        Some(value) => value.clone(),
        None => {
            return Err(PengError::CannotCallValue(
                "sync:Channel.send() expected value".to_string(),
            ));
        }
    };

    let queue_ptr = match get_or_create_channel_queue(ctx, channel_ptr, "Channel.send") {
        Ok(ptr) => ptr,
        Err(e) => return Err(e),
    };

    match ctx.env_mut().get_heap_mut(queue_ptr) {
        Some(PengValue::Box(PengBox::Vector(vector))) => {
            vector.values.push(value);
        }

        Some(_) => {
            return Err(PengError::CannotCallValue(
                "sync:Channel.send() invalid channel queue".to_string(),
            ));
        }

        None => {
            return Err(PengError::HeapValueNotFound(queue_ptr));
        }
    }

    match ctx.env_mut().wake_thread(channel_ptr) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    Ok(PengBinded::Mutable(PengCell::Nil))
}

fn channel_try_recv(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let channel_ptr = match get_object_ptr_arg(ctx, 0, "Channel.try_recv") {
        Ok(ptr) => ptr,
        Err(e) => return Err(e),
    };

    let queue_ptr = match get_channel_queue(ctx, channel_ptr, "Channel.try_recv") {
        Ok(Some(ptr)) => ptr,
        Ok(None) => return Ok(PengBinded::Mutable(PengCell::Nil)),
        Err(e) => return Err(e),
    };

    pop_channel_queue(ctx, queue_ptr, "Channel.try_recv")
}

fn channel_recv(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let channel_ptr = match get_object_ptr_arg(ctx, 0, "Channel.recv") {
        Ok(ptr) => ptr,
        Err(e) => return Err(e),
    };

    let queue_ptr = match get_channel_queue(ctx, channel_ptr, "Channel.recv") {
        Ok(Some(ptr)) => ptr,
        Ok(None) => {
            match ctx.set_current_thread_state(PengThreadState::Waiting(channel_ptr)) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }

            match ctx.yield_now() {
                Ok(_) => {}
                Err(e) => return Err(e),
            }

            return Ok(PengBinded::Mutable(PengCell::Nil));
        }
        Err(e) => return Err(e),
    };

    let value = match pop_channel_queue(ctx, queue_ptr, "Channel.recv") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    match value.value() {
        PengCell::Nil => {
            match ctx.set_current_thread_state(PengThreadState::Waiting(channel_ptr)) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }

            match ctx.yield_now() {
                Ok(_) => {}
                Err(e) => return Err(e),
            }

            Ok(PengBinded::Mutable(PengCell::Nil))
        }

        _ => Ok(value),
    }
}

fn channel_len(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let channel_ptr = match get_object_ptr_arg(ctx, 0, "Channel.len") {
        Ok(ptr) => ptr,
        Err(e) => return Err(e),
    };

    let queue_ptr = match get_channel_queue(ctx, channel_ptr, "Channel.len") {
        Ok(Some(ptr)) => ptr,
        Ok(None) => return Ok(PengBinded::Mutable(PengCell::Uint(0))),
        Err(e) => return Err(e),
    };

    match ctx.env_mut().get_heap_mut(queue_ptr) {
        Some(PengValue::Box(PengBox::Vector(vector))) => {
            Ok(PengBinded::Mutable(PengCell::Uint(vector.values.len())))
        }

        Some(_) => Err(PengError::CannotCallValue(
            "sync:Channel.len() invalid channel queue".to_string(),
        )),

        None => Err(PengError::HeapValueNotFound(queue_ptr)),
    }
}

fn once_try_begin(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let once_ptr = match get_object_ptr_arg(ctx, 0, "Once.try_begin") {
        Ok(ptr) => ptr,
        Err(e) => return Err(e),
    };

    let state = match get_uint_field(ctx, once_ptr, "state", 0, "Once.try_begin") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    match state {
        0 => {
            match set_field(
                ctx,
                once_ptr,
                "state",
                PengBinded::Mutable(PengCell::Uint(1)),
            ) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }

            Ok(PengBinded::Mutable(PengCell::Bool(true)))
        }

        1 => {
            match ctx.set_current_thread_state(PengThreadState::Waiting(once_ptr)) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }

            match ctx.yield_now() {
                Ok(_) => {}
                Err(e) => return Err(e),
            }

            Ok(PengBinded::Mutable(PengCell::Bool(false)))
        }

        2 => Ok(PengBinded::Mutable(PengCell::Bool(false))),

        _ => Err(PengError::CannotCallValue(
            "sync:Once.try_begin() invalid state".to_string(),
        )),
    }
}

fn once_finish(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let once_ptr = match get_object_ptr_arg(ctx, 0, "Once.finish") {
        Ok(ptr) => ptr,
        Err(e) => return Err(e),
    };

    match set_field(
        ctx,
        once_ptr,
        "state",
        PengBinded::Mutable(PengCell::Uint(2)),
    ) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    match ctx.env_mut().wake_threads(once_ptr) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    Ok(PengBinded::Mutable(PengCell::Nil))
}

fn once_is_done(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let once_ptr = match get_object_ptr_arg(ctx, 0, "Once.is_done") {
        Ok(ptr) => ptr,
        Err(e) => return Err(e),
    };

    let state = match get_uint_field(ctx, once_ptr, "state", 0, "Once.is_done") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(PengBinded::Mutable(PengCell::Bool(state == 2)))
}

fn get_object_ptr_arg(
    ctx: &PengNativeFunctionCallContext,
    index: usize,
    function_name: &str,
) -> Result<PengHeapPtr, PengError> {
    let arg = match ctx.get_arg_cell(index) {
        Some(arg) => arg,
        None => {
            return Err(PengError::CannotCallValue(format!(
                "sync:{}() missing object argument",
                function_name
            )));
        }
    };

    let ptr = match arg.value() {
        PengCell::Reference(ptr) => *ptr,

        _ => {
            return Err(PengError::CannotCallValue(format!(
                "sync:{}() expected object reference",
                function_name
            )));
        }
    };

    match ctx.get_value(ptr) {
        Some(PengValue::Box(PengBox::Object(_))) => Ok(ptr),

        Some(_) => Err(PengError::CannotCallValue(format!(
            "sync:{}() expected object",
            function_name
        ))),

        None => Err(PengError::HeapValueNotFound(ptr)),
    }
}

fn get_bool_arg(
    ctx: &PengNativeFunctionCallContext,
    index: usize,
    function_name: &str,
) -> Result<bool, PengError> {
    let arg = match ctx.get_arg_cell(index) {
        Some(arg) => arg,
        None => {
            return Err(PengError::CannotCallValue(format!(
                "sync:{}() missing bool argument",
                function_name
            )));
        }
    };

    match arg.value() {
        PengCell::Bool(value) => Ok(*value),

        _ => Err(PengError::CannotCallValue(format!(
            "sync:{}() expected bool",
            function_name
        ))),
    }
}

fn get_int_arg(
    ctx: &PengNativeFunctionCallContext,
    index: usize,
    function_name: &str,
) -> Result<isize, PengError> {
    let arg = match ctx.get_arg_cell(index) {
        Some(arg) => arg,
        None => {
            return Err(PengError::CannotCallValue(format!(
                "sync:{}() missing int argument",
                function_name
            )));
        }
    };

    match arg.value() {
        PengCell::Int(value) => Ok(*value),

        PengCell::Uint(value) => {
            if *value > isize::MAX as usize {
                return Err(PengError::CannotCallValue(format!(
                    "sync:{}() uint argument is too large for int",
                    function_name
                )));
            }

            Ok(*value as isize)
        }

        _ => Err(PengError::CannotCallValue(format!(
            "sync:{}() expected int",
            function_name
        ))),
    }
}

fn get_bool_field(
    ctx: &mut PengNativeFunctionCallContext,
    object_ptr: PengHeapPtr,
    name: &str,
    default_value: bool,
    function_name: &str,
) -> Result<bool, PengError> {
    let cell = match get_field(ctx, object_ptr, name) {
        Ok(Some(cell)) => cell,
        Ok(None) => return Ok(default_value),
        Err(e) => return Err(e),
    };

    match cell.value() {
        PengCell::Bool(value) => Ok(*value),

        _ => Err(PengError::CannotCallValue(format!(
            "sync:{}() field '{}' must be bool",
            function_name, name
        ))),
    }
}

fn get_int_field(
    ctx: &mut PengNativeFunctionCallContext,
    object_ptr: PengHeapPtr,
    name: &str,
    default_value: isize,
    function_name: &str,
) -> Result<isize, PengError> {
    let cell = match get_field(ctx, object_ptr, name) {
        Ok(Some(cell)) => cell,
        Ok(None) => return Ok(default_value),
        Err(e) => return Err(e),
    };

    match cell.value() {
        PengCell::Int(value) => Ok(*value),

        PengCell::Uint(value) => {
            if *value > isize::MAX as usize {
                return Err(PengError::CannotCallValue(format!(
                    "sync:{}() field '{}' is too large for int",
                    function_name, name
                )));
            }

            Ok(*value as isize)
        }

        _ => Err(PengError::CannotCallValue(format!(
            "sync:{}() field '{}' must be int",
            function_name, name
        ))),
    }
}

fn get_uint_field(
    ctx: &mut PengNativeFunctionCallContext,
    object_ptr: PengHeapPtr,
    name: &str,
    default_value: usize,
    function_name: &str,
) -> Result<usize, PengError> {
    let cell = match get_field(ctx, object_ptr, name) {
        Ok(Some(cell)) => cell,
        Ok(None) => return Ok(default_value),
        Err(e) => return Err(e),
    };

    match cell.value() {
        PengCell::Uint(value) => Ok(*value),

        PengCell::Int(value) => {
            if *value < 0 {
                return Err(PengError::CannotCallValue(format!(
                    "sync:{}() field '{}' must be non-negative",
                    function_name, name
                )));
            }

            Ok(*value as usize)
        }

        _ => Err(PengError::CannotCallValue(format!(
            "sync:{}() field '{}' must be uint",
            function_name, name
        ))),
    }
}

fn get_field(
    ctx: &mut PengNativeFunctionCallContext,
    object_ptr: PengHeapPtr,
    name: &str,
) -> Result<Option<PengBindedCell>, PengError> {
    let name_ptr = ctx.env_mut().ensure_pooled_name_ptr(name.to_string());

    match ctx.env_mut().get_heap_mut(object_ptr) {
        Some(PengValue::Box(PengBox::Object(object))) => {
            match object.fields.get(&name_ptr) {
                Some(value) => Ok(Some(value.clone())),
                None => Ok(None),
            }
        }

        Some(_) => Err(PengError::CannotCallValue(format!(
            "sync object field access expected object for '{}'",
            name
        ))),

        None => Err(PengError::HeapValueNotFound(object_ptr)),
    }
}

fn set_field(
    ctx: &mut PengNativeFunctionCallContext,
    object_ptr: PengHeapPtr,
    name: &str,
    value: PengBindedCell,
) -> Result<(), PengError> {
    let name_ptr = ctx.env_mut().ensure_pooled_name_ptr(name.to_string());

    match ctx.env_mut().get_heap_mut(object_ptr) {
        Some(PengValue::Box(PengBox::Object(object))) => {
            object.fields.insert(name_ptr, value);
            Ok(())
        }

        Some(_) => Err(PengError::CannotCallValue(format!(
            "sync object field set expected object for '{}'",
            name
        ))),

        None => Err(PengError::HeapValueNotFound(object_ptr)),
    }
}

fn get_channel_queue(
    ctx: &mut PengNativeFunctionCallContext,
    channel_ptr: PengHeapPtr,
    function_name: &str,
) -> Result<Option<PengHeapPtr>, PengError> {
    let cell = match get_field(ctx, channel_ptr, "values") {
        Ok(Some(cell)) => cell,
        Ok(None) => return Ok(None),
        Err(e) => return Err(e),
    };

    match cell.value() {
        PengCell::Nil => Ok(None),

        PengCell::Reference(ptr) => {
            match ctx.get_value(*ptr) {
                Some(PengValue::Box(PengBox::Vector(_))) => Ok(Some(*ptr)),

                Some(_) => Err(PengError::CannotCallValue(format!(
                    "sync:{}() channel values must be vector",
                    function_name
                ))),

                None => Err(PengError::HeapValueNotFound(*ptr)),
            }
        }

        _ => Err(PengError::CannotCallValue(format!(
            "sync:{}() channel values must be vector or nil",
            function_name
        ))),
    }
}

fn get_or_create_channel_queue(
    ctx: &mut PengNativeFunctionCallContext,
    channel_ptr: PengHeapPtr,
    function_name: &str,
) -> Result<PengHeapPtr, PengError> {
    match get_channel_queue(ctx, channel_ptr, function_name) {
        Ok(Some(ptr)) => return Ok(ptr),
        Ok(None) => {}
        Err(e) => return Err(e),
    }

    let queue_ptr = ctx.create_box(PengBox::Vector(PengVector::new_empty()));

    match set_field(
        ctx,
        channel_ptr,
        "values",
        PengBinded::Mutable(PengCell::Reference(queue_ptr)),
    ) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    Ok(queue_ptr)
}

fn pop_channel_queue(
    ctx: &mut PengNativeFunctionCallContext,
    queue_ptr: PengHeapPtr,
    function_name: &str,
) -> Result<PengBindedCell, PengError> {
    match ctx.env_mut().get_heap_mut(queue_ptr) {
        Some(PengValue::Box(PengBox::Vector(vector))) => {
            if vector.values.len() == 0 {
                return Ok(PengBinded::Mutable(PengCell::Nil));
            }

            Ok(vector.values.remove(0))
        }

        Some(_) => Err(PengError::CannotCallValue(format!(
            "sync:{}() invalid channel queue",
            function_name
        ))),

        None => Err(PengError::HeapValueNotFound(queue_ptr)),
    }
}