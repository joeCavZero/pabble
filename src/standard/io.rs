use std::io::{self, Read, Write};

use penguin::prelude::*;

pub fn setup(peng: &mut PengEnv) -> PengUnit {
    let mut module = PengUnit::library();

    module.register_immutable_native_function(peng, "print", print).unwrap();
    module.register_immutable_native_function(peng, "println", println).unwrap();

    module.register_immutable_native_function(peng, "eprint", eprint).unwrap();
    module.register_immutable_native_function(peng, "eprintln", eprintln).unwrap();

    module.register_immutable_native_function(peng, "flush", flush).unwrap();

    module.register_immutable_native_function(peng, "read_line", read_line).unwrap();
    module.register_immutable_native_function(peng, "read_all", read_all).unwrap();
    module.register_immutable_native_function(peng, "read_byte", read_byte).unwrap();

    module
        .register_immutable_native_function(peng, "clear_screen", clear_screen)
        .unwrap();

    module
}

fn print(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    print_args(&ctx);

    match io::stdout().flush() {
        Ok(_) => Ok(PengBindedCell::Mutable(PengCell::Nil)),
        Err(_) => Err(PengError::CannotCallValue(
            "io:print() failed to flush stdout".into(),
        )),
    }
}

fn println(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    print_args(&ctx);
    println!();

    Ok(PengBindedCell::Mutable(PengCell::Nil))
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

fn print_args(ctx: &PengNativeFunctionCallContext) {
    let mut index = 0usize;

    loop {
        let arg = match ctx.get_arg_cell(index) {
            Some(arg) => arg,
            None => break,
        };

        if index > 0 {
            print!(" ");
        }

        print_binded_cell(ctx, arg);
        index += 1;
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

fn print_cell(ctx: &PengNativeFunctionCallContext, cell: &PengCell) {
    match cell {
        PengCell::Bool(v) => print!("{}", v),
        PengCell::Byte(v) => print!("{}", v),
        PengCell::Float32(v) => print!("{}", v),
        PengCell::Float64(v) => print!("{}", v),
        PengCell::Int(v) => print!("{}", v),
        PengCell::Nil => print!("nil"),
        PengCell::Uint(v) => print!("{}", v),

        PengCell::Reference(ptr) => print_value_ref(ctx, *ptr),
    }
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

fn print_binded_cell(ctx: &PengNativeFunctionCallContext, cell: &PengBindedCell) {
    print_cell(ctx, cell.value());
}

fn eprint_binded_cell(ctx: &PengNativeFunctionCallContext, cell: &PengBindedCell) {
    eprint_cell(ctx, cell.value());
}

fn print_value_ref(ctx: &PengNativeFunctionCallContext, ptr: PengHeapPtr) {
    match ctx.get_value(ptr) {
        Some(value) => print_value(ctx, value),
        None => print!("<missing heap value>"),
    }
}

fn eprint_value_ref(ctx: &PengNativeFunctionCallContext, ptr: PengHeapPtr) {
    match ctx.get_value(ptr) {
        Some(value) => eprint_value(ctx, value),
        None => eprint!("<missing heap value>"),
    }
}

fn print_value(ctx: &PengNativeFunctionCallContext, value: &PengValue) {
    match value {
        PengValue::Cell(cell) => print_cell(ctx, cell),

        PengValue::Box(PengBox::Function(_)) => print!("<function>"),
        PengValue::Box(PengBox::Operation(_)) => print!("<operation>"),
        PengValue::Box(PengBox::Thread(_)) => print!("<thread>"),
        PengValue::Box(PengBox::Type(_)) => print!("<type>"),
        PengValue::Box(PengBox::Union(_)) => print!("<union>"),

        PengValue::Box(PengBox::String(s)) => print!("{}", s),

        PengValue::Box(PengBox::Vector(v)) => {
            print!("[");
            for (i, value) in v.values.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }

                print_binded_cell(ctx, value);
            }
            print!("]");
        }

        PengValue::Box(PengBox::Object(o)) => {
            print!("{{");
            for (i, (name, value)) in o.fields.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }

                match ctx.env().get_pooled_name(*name) {
                    Some(name) => print!("{}: ", name),
                    None => print!("<name {:?}>: ", name),
                }

                print_binded_cell(ctx, value);
            }
            print!("}}");
        }

        PengValue::Box(PengBox::Module(m)) => {
            print!("module {{");
            for (i, (name, value)) in m.members.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }

                match ctx.env().get_pooled_name(*name) {
                    Some(name) => print!("{}: ", name),
                    None => print!("<name {:?}>: ", name),
                }

                print_binded_cell(ctx, value);
            }
            print!("}}");
        }
    }
}

fn eprint_value(ctx: &PengNativeFunctionCallContext, value: &PengValue) {
    match value {
        PengValue::Cell(cell) => eprint_cell(ctx, cell),

        PengValue::Box(PengBox::Function(_)) => eprint!("<function>"),
        PengValue::Box(PengBox::Operation(_)) => eprint!("<operation>"),
        PengValue::Box(PengBox::Thread(_)) => eprint!("<thread>"),
        PengValue::Box(PengBox::Type(_)) => eprint!("<type>"),
        PengValue::Box(PengBox::Union(_)) => eprint!("<union>"),

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