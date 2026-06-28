use penguin::prelude::*;
pub fn setup(peng: &mut PengEnv) -> PengUnit {
    let mut module = PengUnit::library();

    
    module
        .register_native_function(peng, "spawn", |ctx| {
            let function_ptr = match ctx.get_arg_cell(0) {
                Some(arg) => match arg.value() {
                    PengCell::Reference(ptr) => *ptr,
                    _ => {
                        return Err(PengError::CannotCallValue(
                            "Thread:spawn() expected function as first argument".into(),
                        ));
                    }
                },

                None => {
                    return Err(PengError::CannotCallValue(
                        "Thread:spawn() expected function".into(),
                    ));
                }
            };

            match ctx.get_value(function_ptr) {
                Some(PengValue::Box(PengBox::Function(_))) => {}
                _ => {
                    return Err(PengError::CannotCallValue(
                        "Thread:spawn() expected function as first argument".into(),
                    ));
                }
            }

            let mut params = Vec::new();
            let mut index = 1usize;

            loop {
                match ctx.get_arg_cell(index) {
                    Some(arg) => params.push(arg.clone()),
                    None => break,
                }

                index += 1;
            }

            let thread = PengThread::new(function_ptr, 0, params, PengThreadState::Running);

            let ptr = ctx.create_box(PengBox::Thread(thread));

            ctx.env_mut().activate_thread(ptr);

            Ok(PengBinded::Mutable(PengCell::Reference(ptr)))
        })
        .unwrap();

    
    module
        .register_native_function(peng, "yield", |ctx| {
            match ctx.yield_now() {
                Ok(()) => {}
                Err(e) => return Err(e),
            }
            Ok(PengBinded::Mutable(PengCell::Nil))
        })
        .unwrap();

    module
}