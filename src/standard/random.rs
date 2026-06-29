use penguin::prelude::*;
use rand::RngExt;

pub fn setup(peng: &mut PengEnv) -> PengUnit {
    let mut module = PengUnit::library();

    module
        .register_native_function(peng, "integer", |ctx| {
            let min = match ctx.get_arg_cell(0) {
                Some(arg) => match arg.value() {
                    PengCell::Int(v) => *v,

                    _ => {
                        return Err(PengError::CannotCallValue(
                            "random:int() expected int as first argument".into(),
                        ));
                    }
                },

                None => {
                    return Err(PengError::CannotCallValue(
                        "random:int() expected min".into(),
                    ));
                }
            };

            let max = match ctx.get_arg_cell(1) {
                Some(arg) => match arg.value() {
                    PengCell::Int(v) => *v,

                    _ => {
                        return Err(PengError::CannotCallValue(
                            "random:int() expected int as second argument".into(),
                        ));
                    }
                },

                None => {
                    return Err(PengError::CannotCallValue(
                        "random:int() expected max".into(),
                    ));
                }
            };

            let mut rng = rand::rng();

            let min_i64 = min as i64;
            let max_i64 = max as i64;

            let value_i64 = rng.random_range(min_i64..=max_i64);

            let value = value_i64 as isize;

            Ok(PengBinded::Mutable(PengCell::Int(value)))
        })
        .unwrap();

    module
        .register_native_function(peng, "float64", |ctx| {
            let min = match ctx.get_arg_cell(0) {
                Some(arg) => match arg.value() {
                    PengCell::Float64(v) => *v,

                    PengCell::Int(v) => *v as f64,

                    PengCell::Uint(v) => *v as f64,

                    _ => {
                        return Err(PengError::CannotCallValue(
                            "random:float64() expected number as first argument".into(),
                        ));
                    }
                },

                None => {
                    return Err(PengError::CannotCallValue(
                        "random:float64() expected min".into(),
                    ));
                }
            };

            let max = match ctx.get_arg_cell(1) {
                Some(arg) => match arg.value() {
                    PengCell::Float64(v) => *v,

                    PengCell::Int(v) => *v as f64,

                    PengCell::Uint(v) => *v as f64,

                    _ => {
                        return Err(PengError::CannotCallValue(
                            "random:float64() expected number as second argument".into(),
                        ));
                    }
                },

                None => {
                    return Err(PengError::CannotCallValue(
                        "random:float64() expected max".into(),
                    ));
                }
            };

            let value = rand::rng().random_range(min..=max);

            Ok(PengBinded::Mutable(PengCell::Float64(value)))
        })
        .unwrap();

    module
        .register_native_function(peng, "boolean", |_| {
            let mut rng = rand::rng();
            let value: bool = rng.random();

            Ok(PengBinded::Mutable(PengCell::Bool(value)))
        })
        .unwrap();

    module
}
