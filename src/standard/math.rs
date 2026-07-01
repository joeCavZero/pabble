use penguin::prelude::*;

use super::utils;

pub fn setup(peng: &mut PengEnv) -> PengUnit {
    let mut module = PengUnit::library();

    module.register_immutable_global(peng, "PI", PengValue::Cell(PengCell::Float64(std::f64::consts::PI))).unwrap();
    module.register_immutable_global(peng, "TAU", PengValue::Cell(PengCell::Float64(std::f64::consts::TAU))).unwrap();
    module.register_immutable_global(peng, "E", PengValue::Cell(PengCell::Float64(std::f64::consts::E))).unwrap();

    module.register_immutable_native_function(peng, "sin", sin).unwrap();
    module.register_immutable_native_function(peng, "cos", cos).unwrap();
    module.register_immutable_native_function(peng, "tan", tan).unwrap();

    module.register_immutable_native_function(peng, "asin", asin).unwrap();
    module.register_immutable_native_function(peng, "acos", acos).unwrap();
    module.register_immutable_native_function(peng, "atan", atan).unwrap();
    module.register_immutable_native_function(peng, "atan2", atan2).unwrap();

    module.register_immutable_native_function(peng, "sinh", sinh).unwrap();
    module.register_immutable_native_function(peng, "cosh", cosh).unwrap();
    module.register_immutable_native_function(peng, "tanh", tanh).unwrap();

    module.register_immutable_native_function(peng, "to_radians", to_radians).unwrap();
    module.register_immutable_native_function(peng, "to_degrees", to_degrees).unwrap();

    module.register_immutable_native_function(peng, "sqrt", sqrt).unwrap();
    module.register_immutable_native_function(peng, "cbrt", cbrt).unwrap();
    module.register_immutable_native_function(peng, "pow", pow).unwrap();
    module.register_immutable_native_function(peng, "exp", exp).unwrap();
    module.register_immutable_native_function(peng, "ln", ln).unwrap();
    module.register_immutable_native_function(peng, "log", log).unwrap();
    module.register_immutable_native_function(peng, "log10", log10).unwrap();
    module.register_immutable_native_function(peng, "log2", log2).unwrap();

    module.register_immutable_native_function(peng, "abs", abs).unwrap();
    module.register_immutable_native_function(peng, "floor", floor).unwrap();
    module.register_immutable_native_function(peng, "ceil", ceil).unwrap();
    module.register_immutable_native_function(peng, "round", round).unwrap();
    module.register_immutable_native_function(peng, "trunc", trunc).unwrap();
    module.register_immutable_native_function(peng, "fract", fract).unwrap();

    module.register_immutable_native_function(peng, "min", min).unwrap();
    module.register_immutable_native_function(peng, "max", max).unwrap();
    module.register_immutable_native_function(peng, "clamp", clamp).unwrap();

    module.register_immutable_native_function(peng, "is_nan", is_nan).unwrap();
    module.register_immutable_native_function(peng, "is_finite", is_finite).unwrap();
    module.register_immutable_native_function(peng, "is_infinite", is_infinite).unwrap();

    module
}

fn sin(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx, |v| v.sin())
}

fn cos(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx, |v| v.cos())
}

fn tan(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx, |v| v.tan())
}

fn asin(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx, |v| v.asin())
}

fn acos(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx,  |v| v.acos())
}

fn atan(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx,  |v| v.atan())
}

fn atan2(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::binary_f64(ctx, |y, x| y.atan2(x))
}

fn sinh(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx, |v| v.sinh())
}

fn cosh(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx, |v| v.cosh())
}

fn tanh(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx,  |v| v.tanh())
}

fn to_radians(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx, |v| v.to_radians())
}

fn to_degrees(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx, |v| v.to_degrees())
}

fn sqrt(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx,  |v| v.sqrt())
}

fn cbrt(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx, |v| v.cbrt())
}

fn pow(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::binary_f64(ctx, |left, right| left.powf(right))
}

fn exp(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx, |v| v.exp())
}

fn ln(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx,  |v| v.ln())
}

fn log(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::binary_f64(ctx, |value, base| value.log(base))
}

fn log10(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx,  |v| v.log10())
}

fn log2(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx,  |v| v.log2())
}

fn abs(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let arg = match ctx.get_arg_cell(0) {
        Some(arg) => arg,
        None => {
            return Err(PengError::CannotCallValue(
                "math:abs() expected number".into(),
            ));
        }
    };

    match arg.value() {
        PengCell::Int(v) => Ok(PengBindedCell::Mutable(PengCell::Int(v.abs()))),
        PengCell::Uint(v) => Ok(PengBindedCell::Mutable(PengCell::Uint(*v))),
        PengCell::Byte(v) => Ok(PengBindedCell::Mutable(PengCell::Byte(*v))),
        PengCell::Float32(v) => Ok(PengBindedCell::Mutable(PengCell::Float32(v.abs()))),
        PengCell::Float64(v) => Ok(PengBindedCell::Mutable(PengCell::Float64(v.abs()))),

        _ => Err(PengError::CannotCallValue(
            "math:abs() expected number".into(),
        )),
    }
}

fn floor(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx, |v| v.floor())
}

fn ceil(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx,  |v| v.ceil())
}

fn round(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx, |v| v.round())
}

fn trunc(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx, |v| v.trunc())
}

fn fract(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::unary_f64(ctx,  |v| v.fract())
}

fn min(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::binary_f64(ctx,|left, right| left.min(right))
}

fn max(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::binary_f64(ctx, |left, right| left.max(right))
}

fn clamp(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let value = match utils::get_number_arg(ctx, 0) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let min = match utils::get_number_arg(ctx, 1) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let max = match utils::get_number_arg(ctx, 2) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    if min > max {
        return Err(PengError::CannotCallValue(
            "math:clamp() min cannot be greater than max".into(),
        ));
    }

    Ok(PengBindedCell::Mutable(PengCell::Float64(value.clamp(min, max))))
}

fn is_nan(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let value = match utils::get_number_arg(ctx, 0 ) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(PengBindedCell::Mutable(PengCell::Bool(value.is_nan())))
}

fn is_finite(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let value = match utils::get_number_arg(ctx, 0) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(PengBindedCell::Mutable(PengCell::Bool(value.is_finite())))
}

fn is_infinite(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let value = match utils::get_number_arg(ctx, 0) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(PengBindedCell::Mutable(PengCell::Bool(value.is_infinite())))
}
