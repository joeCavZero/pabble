use std::io::{self, Write};

use penguin::prelude::*;

pub fn setup(peng: &mut PengEnv) -> PengUnit {
    let mut module = PengUnit::library();

    module.register_native_function(peng, "print", move |ctx| {
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
        io::stdout().flush().unwrap();
        Ok(PengBindedCell::Mutable(PengCell::Nil))
    })
    .unwrap();


    module.register_native_function(peng, "println", move |ctx| {
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

        println!();

        Ok(PengBindedCell::Mutable(PengCell::Nil))
    })
    .unwrap();


    module.register_native_function(peng, "input", move |ctx| {
        let mut buffer = String::new();

        match std::io::stdin().read_line(&mut buffer) {
            Ok(_) => {}

            Err(_) => {
                return Err(PengError::CannotCallValue(
                    "io:input() failed to read stdin".into(),
                ));
            }
        }

        while buffer.ends_with('\n') || buffer.ends_with('\r') {
            buffer.pop();
        }

        let ptr = ctx.create_box(PengBox::String(buffer));

        Ok(PengBindedCell::Mutable(PengCell::Reference(ptr)))
    }).unwrap();

    module
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

fn print_binded_cell(ctx: &PengNativeFunctionCallContext, cell: &PengBindedCell) {
    print_cell(ctx, cell.value());
}

fn print_value_ref(ctx: &PengNativeFunctionCallContext, ptr: PengHeapPtr) {
    match ctx.get_value(ptr) {
        Some(value) => print_value(ctx, value),
        None => print!("<missing heap value>"),
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
