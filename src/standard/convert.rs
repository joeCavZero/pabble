use penguin::prelude::*;

pub fn setup(peng: &mut PengEnv) -> PengUnit {
    let mut module = PengUnit::library();

    module
        .register_native_function(peng, "integer", |ctx| {
            let arg = match ctx.get_arg_cell(0) {
                Some(arg) => arg,
                None => {
                    return Err(PengError::CannotCallValue(
                        "convert:integer() expected value".into(),
                    ));
                }
            };

            let value = match arg.value() {
                PengCell::Int(v) => *v,

                PengCell::Uint(v) => *v as isize,

                PengCell::Byte(v) => *v as isize,

                PengCell::Bool(v) => {
                    if *v {
                        1
                    } else {
                        0
                    }
                }

                PengCell::Float32(v) => *v as isize,

                PengCell::Float64(v) => *v as isize,

                PengCell::Reference(ptr) => {
                    match ctx.get_value(*ptr) {
                        Some(PengValue::Box(PengBox::String(s))) => {
                            match s.trim().parse::<isize>() {
                                Ok(v) => v,

                                Err(_) => {
                                    return Err(PengError::CannotCallValue(format!(
                                        "convert:integer() cannot convert '{}' to integer",
                                        s
                                    )));
                                }
                            }
                        }

                        Some(_) => {
                            return Err(PengError::CannotCallValue(
                                "convert:integer() expected string or primitive value".into(),
                            ));
                        }

                        None => {
                            return Err(PengError::HeapValueNotFound(*ptr));
                        }
                    }
                }

                PengCell::Nil => {
                    return Err(PengError::CannotCallValue(
                        "convert:integer() cannot convert nil to integer".into(),
                    ));
                }
            };

            Ok(PengBinded::Mutable(PengCell::Int(value)))
        })
        .unwrap();

    module
}