use std::collections::HashMap;
use std::sync::OnceLock;
use std::thread;
use std::time::{
    Duration as StdDuration,
    Instant as StdInstant,
    SystemTime,
    UNIX_EPOCH,
};

use penguin::prelude::*;

use super::utils;

static PROGRAM_START: OnceLock<StdInstant> = OnceLock::new();

#[derive(Debug, Clone)]
struct TimeDateTimeData {
    timestamp: usize,
    millis: usize,
    nanos: usize,
}

pub fn setup(peng: &mut PengEnv) -> PengUnit {
    let mut module = PengUnit::library();

    let duration_type = duration_type_value(peng);
    module.register_immutable_global(peng, "Duration", duration_type).unwrap();

    let instant_type = instant_type_value(peng);
    module.register_immutable_global(peng, "Instant", instant_type).unwrap();

    let datetime_type = datetime_type_value(peng);
    module.register_immutable_global(peng, "DateTime", datetime_type).unwrap();

    module.register_immutable_native_function(peng, "now", now).unwrap();

    module.register_immutable_native_function(peng, "unix", unix).unwrap();

    module.register_immutable_native_function(peng, "millis", millis).unwrap();

    module.register_immutable_native_function(peng, "nanos", nanos).unwrap();

    module.register_immutable_native_function(peng, "monotonic_nanos", monotonic_nanos_native).unwrap();

    module.register_immutable_native_function(peng, "instant", instant).unwrap();

    module.register_immutable_native_function(peng, "duration", duration).unwrap();

    module.register_immutable_native_function(peng, "elapsed", elapsed).unwrap();

    module.register_immutable_native_function(peng, "elapsed_ms", elapsed_ms).unwrap();

    module.register_immutable_native_function(peng, "elapsed_nanos", elapsed_nanos).unwrap();

    module
}

fn duration_type_value(peng: &mut PengEnv) -> PengValue {
    let mut fields = HashMap::new();

    let sleep_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(duration_sleep),
    )));

    let seconds_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(duration_seconds),
    )));

    let nanos_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(duration_nanos),
    )));

    let add_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(duration_add),
    )));

    fields.insert(
        peng.ensure_pooled_name_ptr("millis".to_string()),
        PengBindedCell::Mutable(PengCell::Uint(0)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("sleep".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(sleep_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("seconds".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(seconds_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("nanos".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(nanos_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("add".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(add_ptr)),
    );

    PengValue::Box(PengBox::Type(PengType::Custom(PengCustomType {
        fields,
    })))
}

fn instant_type_value(peng: &mut PengEnv) -> PengValue {
    let mut fields = HashMap::new();

    let elapsed_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(instant_elapsed),
    )));

    let elapsed_ms_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(instant_elapsed_ms),
    )));

    let elapsed_nanos_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(instant_elapsed_nanos),
    )));

    fields.insert(
        peng.ensure_pooled_name_ptr("nanos".to_string()),
        PengBindedCell::Mutable(PengCell::Uint(0)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("elapsed".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(elapsed_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("elapsed_ms".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(elapsed_ms_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("elapsed_nanos".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(elapsed_nanos_ptr)),
    );

    PengValue::Box(PengBox::Type(PengType::Custom(PengCustomType {
        fields,
    })))
}

fn datetime_type_value(peng: &mut PengEnv) -> PengValue {
    let mut fields = HashMap::new();

    let format_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(datetime_format),
    )));

    let elapsed_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(datetime_elapsed),
    )));

    let elapsed_ms_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(datetime_elapsed_ms),
    )));

    let elapsed_nanos_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(datetime_elapsed_nanos),
    )));

    fields.insert(
        peng.ensure_pooled_name_ptr("timestamp".to_string()),
        PengBindedCell::Mutable(PengCell::Uint(0)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("millis".to_string()),
        PengBindedCell::Mutable(PengCell::Uint(0)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("nanos".to_string()),
        PengBindedCell::Mutable(PengCell::Uint(0)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("format".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(format_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("elapsed".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(elapsed_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("elapsed_ms".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(elapsed_ms_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("elapsed_nanos".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(elapsed_nanos_ptr)),
    );

    PengValue::Box(PengBox::Type(PengType::Custom(PengCustomType {
        fields,
    })))
}

fn now(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let data = match now_data() {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    datetime_object(ctx, data)
}

fn unix(_ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let data = match now_data() {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    Ok(PengBindedCell::Mutable(PengCell::Uint(data.timestamp)))
}

fn millis(_ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let data = match now_data() {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    Ok(PengBindedCell::Mutable(PengCell::Uint(data.millis)))
}

fn nanos(_ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let data = match now_data() {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    Ok(PengBindedCell::Mutable(PengCell::Uint(data.nanos)))
}

fn monotonic_nanos_native(
    _ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    Ok(PengBindedCell::Mutable(PengCell::Uint(monotonic_nanos_value())))
}

fn instant(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let nanos = monotonic_nanos_value();

    instant_object(ctx, nanos)
}

fn duration(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let millis = match utils::get_uint_arg(ctx, 0) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    duration_object(ctx, millis)
}

fn elapsed(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let start = match get_instant_nanos_arg(ctx, 0, "elapsed") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let end = match get_instant_nanos_arg(ctx, 1, "elapsed") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let nanos = match checked_elapsed_nanos(start, end, "elapsed") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    duration_object(ctx, nanos / 1_000_000)
}

fn elapsed_ms(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let start = match get_instant_nanos_arg(ctx, 0, "elapsed_ms") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let end = match get_instant_nanos_arg(ctx, 1, "elapsed_ms") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let nanos = match checked_elapsed_nanos(start, end, "elapsed_ms") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(PengBindedCell::Mutable(PengCell::Uint(nanos / 1_000_000)))
}

fn elapsed_nanos(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let start = match get_instant_nanos_arg(ctx, 0, "elapsed_nanos") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let end = match get_instant_nanos_arg(ctx, 1, "elapsed_nanos") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let nanos = match checked_elapsed_nanos(start, end, "elapsed_nanos") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(PengBindedCell::Mutable(PengCell::Uint(nanos)))
}

fn duration_sleep(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let millis = match get_duration_millis_arg(ctx, 0, "Duration.sleep") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    thread::sleep(StdDuration::from_millis(millis as u64));

    utils::nil()
}

fn duration_seconds(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let millis = match get_duration_millis_arg(ctx, 0, "Duration.seconds") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(PengBindedCell::Mutable(PengCell::Uint(millis / 1000)))
}

fn duration_nanos(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let millis = match get_duration_millis_arg(ctx, 0, "Duration.nanos") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let nanos = saturating_mul_usize(millis, 1_000_000);

    Ok(PengBindedCell::Mutable(PengCell::Uint(nanos)))
}

fn duration_add(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let left = match get_duration_millis_arg(ctx, 0, "Duration.add") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let right = match get_duration_millis_arg(ctx, 1, "Duration.add") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let result = saturating_add_usize(left, right);

    duration_object(ctx, result)
}

fn instant_elapsed(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let start = match get_instant_nanos_arg(ctx, 0, "Instant.elapsed") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let end = match get_instant_nanos_arg(ctx, 1, "Instant.elapsed") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let nanos = match checked_elapsed_nanos(start, end, "Instant.elapsed") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    duration_object(ctx, nanos / 1_000_000)
}

fn instant_elapsed_ms(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    let start = match get_instant_nanos_arg(ctx, 0, "Instant.elapsed_ms") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let end = match get_instant_nanos_arg(ctx, 1, "Instant.elapsed_ms") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let nanos = match checked_elapsed_nanos(start, end, "Instant.elapsed_ms") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(PengBindedCell::Mutable(PengCell::Uint(nanos / 1_000_000)))
}

fn instant_elapsed_nanos(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    let start = match get_instant_nanos_arg(ctx, 0, "Instant.elapsed_nanos") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let end = match get_instant_nanos_arg(ctx, 1, "Instant.elapsed_nanos") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let nanos = match checked_elapsed_nanos(start, end, "Instant.elapsed_nanos") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(PengBindedCell::Mutable(PengCell::Uint(nanos)))
}

fn datetime_format(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let data = match get_datetime_data_arg(ctx, 0, "DateTime.format") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let millis_part = data.millis % 1000;
    let value = format!("{}.{:03}", data.timestamp, millis_part);

    utils::string(ctx, value)
}

fn datetime_elapsed(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let start = match get_datetime_data_arg(ctx, 0, "DateTime.elapsed") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let end = match get_datetime_data_arg(ctx, 1, "DateTime.elapsed") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let nanos = match checked_elapsed_nanos(start.nanos, end.nanos, "DateTime.elapsed") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    duration_object(ctx, nanos / 1_000_000)
}

fn datetime_elapsed_ms(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    let start = match get_datetime_data_arg(ctx, 0, "DateTime.elapsed_ms") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let end = match get_datetime_data_arg(ctx, 1, "DateTime.elapsed_ms") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let nanos = match checked_elapsed_nanos(start.nanos, end.nanos, "DateTime.elapsed_ms") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(PengBindedCell::Mutable(PengCell::Uint(nanos / 1_000_000)))
}

fn datetime_elapsed_nanos(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    let start = match get_datetime_data_arg(ctx, 0, "DateTime.elapsed_nanos") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let end = match get_datetime_data_arg(ctx, 1, "DateTime.elapsed_nanos") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let nanos = match checked_elapsed_nanos(start.nanos, end.nanos, "DateTime.elapsed_nanos") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(PengBindedCell::Mutable(PengCell::Uint(nanos)))
}

fn duration_object(
    ctx: &mut PengNativeFunctionCallContext,
    millis: usize,
) -> Result<PengBindedCell, PengError> {
    let sleep_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        duration_sleep,
    )));

    let seconds_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        duration_seconds,
    )));

    let nanos_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        duration_nanos,
    )));

    let add_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        duration_add,
    )));

    let millis_cell = PengBindedCell::Mutable(PengCell::Uint(millis));
    let sleep_cell = PengBindedCell::Immutable(PengCell::Reference(sleep_ptr));
    let seconds_cell = PengBindedCell::Immutable(PengCell::Reference(seconds_ptr));
    let nanos_cell = PengBindedCell::Immutable(PengCell::Reference(nanos_ptr));
    let add_cell = PengBindedCell::Immutable(PengCell::Reference(add_ptr));

    utils::new_object(
        ctx,
        vec![
            ("millis", millis_cell),
            ("sleep", sleep_cell),
            ("seconds", seconds_cell),
            ("nanos", nanos_cell),
            ("add", add_cell),
        ],
    )
}

fn instant_object(
    ctx: &mut PengNativeFunctionCallContext,
    nanos: usize,
) -> Result<PengBindedCell, PengError> {
    let elapsed_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        instant_elapsed,
    )));

    let elapsed_ms_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        instant_elapsed_ms,
    )));

    let elapsed_nanos_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        instant_elapsed_nanos,
    )));

    let nanos_cell = PengBindedCell::Mutable(PengCell::Uint(nanos));
    let elapsed_cell = PengBindedCell::Immutable(PengCell::Reference(elapsed_ptr));
    let elapsed_ms_cell = PengBindedCell::Immutable(PengCell::Reference(elapsed_ms_ptr));
    let elapsed_nanos_cell = PengBindedCell::Immutable(PengCell::Reference(elapsed_nanos_ptr));

    utils::new_object(
        ctx,
        vec![
            ("nanos", nanos_cell),
            ("elapsed", elapsed_cell),
            ("elapsed_ms", elapsed_ms_cell),
            ("elapsed_nanos", elapsed_nanos_cell),
        ],
    )
}

fn datetime_object(
    ctx: &mut PengNativeFunctionCallContext,
    data: TimeDateTimeData,
) -> Result<PengBindedCell, PengError> {
    let format_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        datetime_format,
    )));

    let elapsed_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        datetime_elapsed,
    )));

    let elapsed_ms_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        datetime_elapsed_ms,
    )));

    let elapsed_nanos_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        datetime_elapsed_nanos,
    )));

    let timestamp_cell = PengBindedCell::Mutable(PengCell::Uint(data.timestamp));
    let millis_cell = PengBindedCell::Mutable(PengCell::Uint(data.millis));
    let nanos_cell = PengBindedCell::Mutable(PengCell::Uint(data.nanos));

    let format_cell = PengBindedCell::Immutable(PengCell::Reference(format_ptr));
    let elapsed_cell = PengBindedCell::Immutable(PengCell::Reference(elapsed_ptr));
    let elapsed_ms_cell = PengBindedCell::Immutable(PengCell::Reference(elapsed_ms_ptr));
    let elapsed_nanos_cell = PengBindedCell::Immutable(PengCell::Reference(elapsed_nanos_ptr));

    utils::new_object(
        ctx,
        vec![
            ("timestamp", timestamp_cell),
            ("millis", millis_cell),
            ("nanos", nanos_cell),
            ("format", format_cell),
            ("elapsed", elapsed_cell),
            ("elapsed_ms", elapsed_ms_cell),
            ("elapsed_nanos", elapsed_nanos_cell),
        ],
    )
}

fn now_data() -> Result<TimeDateTimeData, PengError> {
    let duration = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration,
        Err(e) => {
            return Err(PengError::CannotCallValue(format!(
                "time:now() system time is before UNIX_EPOCH: {}",
                e
            )));
        }
    };

    Ok(TimeDateTimeData {
        timestamp: u64_to_usize_saturating(duration.as_secs()),
        millis: u128_to_usize_saturating(duration.as_millis()),
        nanos: u128_to_usize_saturating(duration.as_nanos()),
    })
}

fn monotonic_nanos_value() -> usize {
    let start = PROGRAM_START.get_or_init(StdInstant::now);
    let elapsed = start.elapsed();

    u128_to_usize_saturating(elapsed.as_nanos())
}

fn get_duration_millis_arg(
    ctx: &mut PengNativeFunctionCallContext,
    index: usize,
    function_name: &str,
) -> Result<usize, PengError> {
    let arg = match ctx.get_arg_cell(index) {
        Some(arg) => arg.clone(),
        None => {
            return Err(PengError::CannotCallValue(format!(
                "time:{}() missing argument at index {}",
                function_name, index
            )));
        }
    };

    get_duration_millis_from_cell(ctx, &arg, function_name)
}

fn get_duration_millis_from_cell(
    ctx: &mut PengNativeFunctionCallContext,
    cell: &PengBindedCell,
    function_name: &str,
) -> Result<usize, PengError> {
    match utils::cell_to_uint(cell) {
        Ok(value) => return Ok(value),
        Err(_) => {}
    }

    let fields = match utils::get_object_fields_from_cell(ctx, cell) {
        Ok(fields) => fields,
        Err(_) => {
            return Err(PengError::CannotCallValue(format!(
                "time:{}() expected Duration or uint",
                function_name
            )));
        }
    };

    get_required_uint_field(ctx, &fields, "millis", function_name)
}

fn get_instant_nanos_arg(
    ctx: &mut PengNativeFunctionCallContext,
    index: usize,
    function_name: &str,
) -> Result<usize, PengError> {
    let arg = match ctx.get_arg_cell(index) {
        Some(arg) => arg.clone(),
        None => {
            return Err(PengError::CannotCallValue(format!(
                "time:{}() missing argument at index {}",
                function_name, index
            )));
        }
    };

    get_instant_nanos_from_cell(ctx, &arg, function_name)
}

fn get_instant_nanos_from_cell(
    ctx: &mut PengNativeFunctionCallContext,
    cell: &PengBindedCell,
    function_name: &str,
) -> Result<usize, PengError> {
    match utils::cell_to_uint(cell) {
        Ok(value) => return Ok(value),
        Err(_) => {}
    }

    let fields = match utils::get_object_fields_from_cell(ctx, cell) {
        Ok(fields) => fields,
        Err(_) => {
            return Err(PengError::CannotCallValue(format!(
                "time:{}() expected Instant or uint nanos",
                function_name
            )));
        }
    };

    get_required_uint_field(ctx, &fields, "nanos", function_name)
}

fn get_datetime_data_arg(
    ctx: &mut PengNativeFunctionCallContext,
    index: usize,
    function_name: &str,
) -> Result<TimeDateTimeData, PengError> {
    let arg = match ctx.get_arg_cell(index) {
        Some(arg) => arg.clone(),
        None => {
            return Err(PengError::CannotCallValue(format!(
                "time:{}() missing argument at index {}",
                function_name, index
            )));
        }
    };

    let fields = match utils::get_object_fields_from_cell(ctx, &arg) {
        Ok(fields) => fields,
        Err(_) => {
            return Err(PengError::CannotCallValue(format!(
                "time:{}() expected DateTime",
                function_name
            )));
        }
    };

    let timestamp = match get_required_uint_field(ctx, &fields, "timestamp", function_name) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let millis = match get_required_uint_field(ctx, &fields, "millis", function_name) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let nanos = match get_required_uint_field(ctx, &fields, "nanos", function_name) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(TimeDateTimeData {
        timestamp,
        millis,
        nanos,
    })
}

fn get_required_uint_field(
    ctx: &mut PengNativeFunctionCallContext,
    fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    name: &str,
    function_name: &str,
) -> Result<usize, PengError> {
    let cell = match utils::get_map_field(ctx, fields, name) {
        Some(cell) => cell,
        None => {
            return Err(PengError::CannotCallValue(format!(
                "time:{}() missing '{}' field",
                function_name, name
            )));
        }
    };

    match utils::cell_to_uint(&cell) {
        Ok(value) => Ok(value),
        Err(e) => Err(e),
    }
}

fn checked_elapsed_nanos(
    start: usize,
    end: usize,
    function_name: &str,
) -> Result<usize, PengError> {
    if end < start {
        return Err(PengError::CannotCallValue(format!(
            "time:{}() end is before start",
            function_name
        )));
    }

    Ok(end - start)
}

fn saturating_add_usize(left: usize, right: usize) -> usize {
    match left.checked_add(right) {
        Some(value) => value,
        None => usize::MAX,
    }
}

fn saturating_mul_usize(left: usize, right: usize) -> usize {
    match left.checked_mul(right) {
        Some(value) => value,
        None => usize::MAX,
    }
}

fn u64_to_usize_saturating(value: u64) -> usize {
    if value > usize::MAX as u64 {
        usize::MAX
    } else {
        value as usize
    }
}

fn u128_to_usize_saturating(value: u128) -> usize {
    if value > usize::MAX as u128 {
        usize::MAX
    } else {
        value as usize
    }
}
