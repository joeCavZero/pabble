use std::io::{self, Read, Write};

use penguin::prelude::*;

pub fn setup(peng: &mut PengEnv) -> PengUnit {
    let mut module = PengUnit::library();

    module
        .register_immutable_native_function(peng, "print", print)
        .unwrap();
    module
        .register_immutable_native_function(peng, "println", println)
        .unwrap();

    module
        .register_immutable_native_function(peng, "eprint", eprint)
        .unwrap();
    module
        .register_immutable_native_function(peng, "eprintln", eprintln)
        .unwrap();

    module
        .register_immutable_native_function(peng, "flush", flush)
        .unwrap();

    module
        .register_immutable_native_function(peng, "read_line", read_line)
        .unwrap();
    module
        .register_immutable_native_function(peng, "read_all", read_all)
        .unwrap();
    module
        .register_immutable_native_function(peng, "read_byte", read_byte)
        .unwrap();

    module
        .register_immutable_native_function(peng, "clear_screen", clear_screen)
        .unwrap();

    module
}

fn print(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    match print_args(ctx) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    match io::stdout().flush() {
        Ok(_) => Ok(PengBindedCell::Mutable(PengCell::Nil)),
        Err(_) => Err(PengError::CannotCallValue(
            "io:print() failed to flush stdout".into(),
        )),
    }
}

fn println(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    match print_args(ctx) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    println!();

    Ok(PengBindedCell::Mutable(PengCell::Nil))
}

fn print_args(ctx: &mut PengNativeFunctionCallContext) -> Result<(), PengError> {
    let mut index = 0usize;

    loop {
        let arg = match ctx.get_arg_cell(index) {
            Some(arg) => arg.clone(),
            None => break,
        };

        if index > 0 {
            print!(" ");
        }

        match print_binded_cell(ctx, &arg) {
            Ok(()) => {}
            Err(e) => return Err(e),
        }

        index += 1;
    }

    Ok(())
}

fn eprint(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    eprint_args(&ctx);

    match io::stderr().flush() {
        Ok(_) => Ok(PengBindedCell::Mutable(PengCell::Nil)),
        Err(_) => Err(PengError::CannotCallValue(
            "io:eprint() failed to flush stderr".into(),
        )),
    }
}

fn eprintln(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    eprint_args(&ctx);
    eprintln!();

    Ok(PengBindedCell::Mutable(PengCell::Nil))
}

fn flush(_: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    match io::stdout().flush() {
        Ok(_) => Ok(PengBindedCell::Mutable(PengCell::Nil)),
        Err(_) => Err(PengError::CannotCallValue(
            "io:flush() failed to flush stdout".into(),
        )),
    }
}

fn read_line(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let mut buffer = String::new();

    match io::stdin().read_line(&mut buffer) {
        Ok(_) => {}
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "io:read_line() failed to read stdin".into(),
            ));
        }
    }

    while buffer.ends_with('\n') || buffer.ends_with('\r') {
        buffer.pop();
    }

    let ptr = ctx.create_box(PengBox::String(buffer));

    Ok(PengBindedCell::Mutable(PengCell::Reference(ptr)))
}

fn read_all(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let mut buffer = String::new();

    match io::stdin().read_to_string(&mut buffer) {
        Ok(_) => {}
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "io:read_all() failed to read stdin".into(),
            ));
        }
    }

    let ptr = ctx.create_box(PengBox::String(buffer));

    Ok(PengBindedCell::Mutable(PengCell::Reference(ptr)))
}

fn read_byte(_: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let mut buffer = [0u8; 1];

    match io::stdin().read(&mut buffer) {
        Ok(0) => Ok(PengBindedCell::Mutable(PengCell::Nil)),
        Ok(_) => Ok(PengBindedCell::Mutable(PengCell::Byte(buffer[0]))),

        Err(_) => Err(PengError::CannotCallValue(
            "io:read_byte() failed to read stdin".into(),
        )),
    }
}

fn clear_screen(_: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    print!("\x1B[2J\x1B[1;1H");

    match io::stdout().flush() {
        Ok(_) => Ok(PengBindedCell::Mutable(PengCell::Nil)),
        Err(_) => Err(PengError::CannotCallValue(
            "io:clear_screen() failed to flush stdout".into(),
        )),
    }
}

fn eprint_args(ctx: &PengNativeFunctionCallContext) {
    let mut index = 0usize;

    loop {
        let arg = match ctx.get_arg_cell(index) {
            Some(arg) => arg,
            None => break,
        };

        if index > 0 {
            eprint!(" ");
        }

        eprint_binded_cell(ctx, arg);
        index += 1;
    }
}

fn print_cell(ctx: &mut PengNativeFunctionCallContext, cell: &PengCell) -> Result<(), PengError> {
    match cell {
        PengCell::Bool(v) => print!("{}", v),
        PengCell::Byte(v) => print!("{}", v),
        PengCell::Float32(v) => print!("{}", v),
        PengCell::Float64(v) => print!("{}", v),
        PengCell::Int(v) => print!("{}", v),
        PengCell::Nil => print!("nil"),
        PengCell::Uint(v) => print!("{}", v),

        PengCell::Reference(ptr) => {
            return print_value_ref(ctx, *ptr);
        }
    }

    Ok(())
}

fn eprint_cell(ctx: &PengNativeFunctionCallContext, cell: &PengCell) {
    match cell {
        PengCell::Bool(v) => eprint!("{}", v),
        PengCell::Byte(v) => eprint!("{}", v),
        PengCell::Float32(v) => eprint!("{}", v),
        PengCell::Float64(v) => eprint!("{}", v),
        PengCell::Int(v) => eprint!("{}", v),
        PengCell::Nil => eprint!("nil"),
        PengCell::Uint(v) => eprint!("{}", v),

        PengCell::Reference(ptr) => eprint_value_ref(ctx, *ptr),
    }
}

fn print_binded_cell(
    ctx: &mut PengNativeFunctionCallContext,
    cell: &PengBindedCell,
) -> Result<(), PengError> {
    print_cell(ctx, cell.value())
}

fn eprint_binded_cell(ctx: &PengNativeFunctionCallContext, cell: &PengBindedCell) {
    eprint_cell(ctx, cell.value());
}

fn print_value_ref(
    ctx: &mut PengNativeFunctionCallContext,
    ptr: PengHeapPtr,
) -> Result<(), PengError> {
    let is_object = match ctx.get_value(ptr) {
        Some(PengValue::Box(PengBox::Object(_))) => true,
        Some(_) => false,
        None => {
            print!("<missing heap value>");
            return Ok(());
        }
    };

    if is_object {
        match try_object_str(ctx, ptr) {
            Ok(Some(value)) => {
                print!("{}", value);
                return Ok(());
            }

            Ok(None) => {}

            Err(e) => return Err(e),
        }
    }

    let value = match ctx.get_value(ptr) {
        Some(value) => value.clone(),
        None => {
            print!("<missing heap value>");
            return Ok(());
        }
    };

    print_value(ctx, &value)
}

fn try_object_str(
    ctx: &mut PengNativeFunctionCallContext,
    object_ptr: PengHeapPtr,
) -> Result<Option<String>, PengError> {
    let str_name = ctx.env_mut().ensure_pooled_name_ptr("__str".to_string());

    let str_method = match ctx.get_value(object_ptr) {
        Some(PengValue::Box(PengBox::Object(object))) => match object.fields.get(&str_name) {
            Some(method) => method.clone(),
            None => return Ok(None),
        },

        Some(_) => return Ok(None),

        None => return Err(PengError::HeapValueNotFound(object_ptr)),
    };

    let callable_ptr = match str_method.value() {
        PengCell::Reference(ptr) => *ptr,
        _ => {
            return Err(PengError::CannotCallValue(
                "__str must be a function".to_string(),
            ));
        }
    };

    match ctx.get_value(callable_ptr) {
        Some(PengValue::Box(PengBox::Function(_))) => {}

        Some(_) => {
            return Err(PengError::CannotCallValue(
                "__str must be a function".to_string(),
            ));
        }

        None => return Err(PengError::HeapValueNotFound(callable_ptr)),
    }

    let self_cell = PengBindedCell::Mutable(PengCell::Reference(object_ptr));

    let thread = ctx.thread();
    let unit = ctx.unit().clone();

    let result = match call_function_sync(ctx.env_mut(), thread, &unit, str_method, vec![self_cell])
    {
        Ok(result) => result,
        Err(e) => return Err(e),
    };

    let result_ptr = match result.value() {
        PengCell::Reference(ptr) => *ptr,

        _ => {
            return Err(PengError::CannotCallValue(
                "__str must return a string".to_string(),
            ));
        }
    };

    match ctx.get_value(result_ptr) {
        Some(PengValue::Box(PengBox::String(value))) => Ok(Some(value.clone())),

        Some(_) => Err(PengError::CannotCallValue(
            "__str must return a string".to_string(),
        )),

        None => Err(PengError::HeapValueNotFound(result_ptr)),
    }
}

fn eprint_value_ref(ctx: &PengNativeFunctionCallContext, ptr: PengHeapPtr) {
    match ctx.get_value(ptr) {
        Some(value) => eprint_value(ctx, value),
        None => eprint!("<missing heap value>"),
    }
}

fn print_value(
    ctx: &mut PengNativeFunctionCallContext,
    value: &PengValue,
) -> Result<(), PengError> {
    match value {
        PengValue::Cell(cell) => {
            return print_cell(ctx, cell);
        }

        PengValue::Box(PengBox::Function(_)) => print!("<function>"),
        PengValue::Box(PengBox::Operation(_)) => print!("<operation>"),
        PengValue::Box(PengBox::Thread(_)) => print!("<thread>"),
        PengValue::Box(PengBox::Type(_)) => print!("<type>"),

        PengValue::Box(PengBox::String(s)) => print!("{}", s),

        PengValue::Box(PengBox::Vector(v)) => {
            let values = v.values.clone();

            print!("[");

            for (i, value) in values.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }

                match print_binded_cell(ctx, value) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }

            print!("]");
        }

        PengValue::Box(PengBox::Object(o)) => {
            let fields = o.fields.clone();

            print!("{{");

            for (i, (name, value)) in fields.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }

                match ctx.env().get_pooled_name(*name) {
                    Some(name) => print!("{}: ", name),
                    None => print!("<name {:?}>: ", name),
                }

                match print_binded_cell(ctx, value) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }

            print!("}}");
        }

        PengValue::Box(PengBox::Module(m)) => {
            let members = m.members.clone();

            print!("module {{");

            for (i, (name, value)) in members.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }

                match ctx.env().get_pooled_name(*name) {
                    Some(name) => print!("{}: ", name),
                    None => print!("<name {:?}>: ", name),
                }

                match print_binded_cell(ctx, value) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }

            print!("}}");
        }
    }

    Ok(())
}

fn eprint_value(ctx: &PengNativeFunctionCallContext, value: &PengValue) {
    match value {
        PengValue::Cell(cell) => eprint_cell(ctx, cell),

        PengValue::Box(PengBox::Function(_)) => eprint!("<function>"),
        PengValue::Box(PengBox::Operation(_)) => eprint!("<operation>"),
        PengValue::Box(PengBox::Thread(_)) => eprint!("<thread>"),
        PengValue::Box(PengBox::Type(_)) => eprint!("<type>"),

        PengValue::Box(PengBox::String(s)) => eprint!("{}", s),

        PengValue::Box(PengBox::Vector(v)) => {
            eprint!("[");
            for (i, value) in v.values.iter().enumerate() {
                if i > 0 {
                    eprint!(", ");
                }

                eprint_binded_cell(ctx, value);
            }
            eprint!("]");
        }

        PengValue::Box(PengBox::Object(o)) => {
            eprint!("{{");
            for (i, (name, value)) in o.fields.iter().enumerate() {
                if i > 0 {
                    eprint!(", ");
                }

                match ctx.env().get_pooled_name(*name) {
                    Some(name) => eprint!("{}: ", name),
                    None => eprint!("<name {:?}>: ", name),
                }

                eprint_binded_cell(ctx, value);
            }
            eprint!("}}");
        }

        PengValue::Box(PengBox::Module(m)) => {
            eprint!("module {{");
            for (i, (name, value)) in m.members.iter().enumerate() {
                if i > 0 {
                    eprint!(", ");
                }

                match ctx.env().get_pooled_name(*name) {
                    Some(name) => eprint!("{}: ", name),
                    None => eprint!("<name {:?}>: ", name),
                }

                eprint_binded_cell(ctx, value);
            }
            eprint!("}}");
        }
    }
}
