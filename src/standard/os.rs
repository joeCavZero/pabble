use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use penguin::prelude::*;

use super::utils;

#[derive(Debug, Clone)]
struct OsProcessData {
    program: String,
    args: Vec<String>,
    cwd: Option<String>,
    env: Vec<(String, String)>,
    stdin: Option<String>,
}

#[derive(Debug, Clone)]
struct OsProcessResultData {
    stdout: String,
    stderr: String,
    exit_code: isize,
    success: bool,
}

#[derive(Debug, Clone)]
enum OsTaskState {
    Running,
    Finished(Result<OsProcessResultData, String>),
}

pub fn setup(peng: &mut PengEnv) -> PengUnit {
    let mut module = PengUnit::library();

    let process_type = process_type_value(peng);

    module.register_immutable_global(peng, "Process", process_type).unwrap();

    module.register_immutable_native_function(peng, "name", name).unwrap();

    module.register_immutable_native_function(peng, "family", family).unwrap();

    module.register_immutable_native_function(peng, "arch", arch).unwrap();

    module.register_immutable_native_function(peng, "current_dir", current_dir).unwrap();

    module.register_immutable_native_function(peng, "home_dir", home_dir).unwrap();

    module.register_immutable_native_function(peng, "temp_dir", temp_dir).unwrap();

    module.register_immutable_native_function(peng, "pid", pid).unwrap();

    module.register_immutable_native_function(peng, "args", args).unwrap();

    module.register_immutable_native_function(peng, "cpu_count", cpu_count).unwrap();

    module.register_immutable_native_function(peng, "get_env", get_env).unwrap();

    module.register_immutable_native_function(peng, "set_env", set_env).unwrap();

    module.register_immutable_native_function(peng, "remove_env", remove_env).unwrap();

    module.register_immutable_native_function(peng, "envs", envs).unwrap();

    module.register_immutable_native_function(peng, "run", run).unwrap();

    module.register_immutable_native_function(peng, "spawn", spawn).unwrap();

    module
}

fn process_type_value(peng: &mut PengEnv) -> PengValue {
    let mut fields = HashMap::new();

    let run_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(process_run),
    )));

    let spawn_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(process_spawn),
    )));

    let program = string_binded_cell_from_env(peng, "".to_string());

    let args_ptr = peng.create_heap_value(PengValue::Box(PengBox::Vector(PengVector {
        values: Vec::new(),
    })));

    let env_ptr = peng.create_heap_value(PengValue::Box(PengBox::Object(PengObject {
        fields: HashMap::new(),
    })));

    fields.insert(
        peng.ensure_pooled_name_ptr("program".to_string()),
        program,
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("args".to_string()),
        PengBindedCell::Mutable(PengCell::Reference(args_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("cwd".to_string()),
        PengBindedCell::Mutable(PengCell::Nil),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("env".to_string()),
        PengBindedCell::Mutable(PengCell::Reference(env_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("stdin".to_string()),
        PengBindedCell::Mutable(PengCell::Nil),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("run".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(run_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("spawn".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(spawn_ptr)),
    );

    PengValue::Box(PengBox::Type(PengType::Custom(PengCustomType {
        fields,
    })))
}

fn name(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::string(ctx, env::consts::OS.to_string())
}

fn family(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::string(ctx, env::consts::FAMILY.to_string())
}

fn arch(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    utils::string(ctx, env::consts::ARCH.to_string())
}

fn current_dir(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    match env::current_dir() {
        Ok(path) => utils::string(ctx, path.to_string_lossy().to_string()),
        Err(e) => Err(PengError::CannotCallValue(format!(
            "os:current_dir() failed: {}",
            e
        ))),
    }
}

fn home_dir(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    match env::var("HOME") {
        Ok(value) => return utils::string(ctx, value),
        Err(_) => {}
    }

    match env::var("USERPROFILE") {
        Ok(value) => utils::string(ctx, value),
        Err(_) => utils::nil(),
    }
}

fn temp_dir(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = env::temp_dir();
    utils::string(ctx, path.to_string_lossy().to_string())
}

fn pid(_ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    Ok(PengBindedCell::Mutable(PengCell::Uint(
        std::process::id() as usize,
    )))
}

fn args(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let mut values = Vec::new();

    for arg in env::args() {
        let string_ptr = ctx.create_box(PengBox::String(arg));
        values.push(PengBindedCell::Mutable(PengCell::Reference(string_ptr)));
    }

    let vector_ptr = ctx.create_box(PengBox::Vector(PengVector {
        values,
    }));

    Ok(PengBindedCell::Mutable(PengCell::Reference(vector_ptr)))
}

fn cpu_count(_ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    match thread::available_parallelism() {
        Ok(count) => Ok(PengBindedCell::Mutable(PengCell::Uint(count.get()))),
        Err(e) => Err(PengError::CannotCallValue(format!(
            "os:cpu_count() failed: {}",
            e
        ))),
    }
}

fn get_env(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let name = match get_string_arg(ctx, 0, "get_env") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    match env::var(name) {
        Ok(value) => utils::string(ctx, value),
        Err(_) => utils::nil(),
    }
}

fn set_env(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let name = match get_string_arg(ctx, 0, "set_env") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let value = match get_string_arg(ctx, 1, "set_env") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    unsafe {
        env::set_var(name, value);
    }

    utils::nil()
}

fn remove_env(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let name = match get_string_arg(ctx, 0, "remove_env") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    unsafe {
        env::remove_var(name);
    }

    utils::nil()
}

fn envs(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let mut pairs = Vec::new();

    for (name, value) in env::vars() {
        pairs.push((name, value));
    }

    Ok(utils::object_from_string_pairs(ctx, pairs))
}

fn run(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let process = match get_process_data(ctx, "run") {
        Ok(process) => process,
        Err(e) => return Err(e),
    };

    let result = match execute_process_data(process, "run") {
        Ok(result) => result,
        Err(e) => return Err(PengError::CannotCallValue(e)),
    };

    process_result_to_object(ctx, result)
}

fn spawn(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let process = match get_process_data(ctx, "spawn") {
        Ok(process) => process,
        Err(e) => return Err(e),
    };

    let state = Arc::new(Mutex::new(OsTaskState::Running));
    let thread_state = state.clone();

    thread::spawn(move || {
        let result = execute_process_data(process, "spawn");

        match thread_state.lock() {
            Ok(mut locked) => {
                *locked = OsTaskState::Finished(result);
            }

            Err(_) => {}
        }
    });

    task_object(ctx, state)
}

fn process_run(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    run(ctx)
}

fn process_spawn(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    spawn(ctx)
}

fn get_process_data(
    ctx: &mut PengNativeFunctionCallContext,
    function_name: &str,
) -> Result<OsProcessData, PengError> {
    let process_arg = match ctx.get_arg_cell(0) {
        Some(arg) => arg.clone(),
        None => {
            return Err(PengError::CannotCallValue(format!(
                "os:{}() missing process argument",
                function_name
            )));
        }
    };

    let fields = match utils::get_object_fields_from_cell(ctx, &process_arg) {
        Ok(fields) => fields,
        Err(e) => return Err(e),
    };

    let program = match get_optional_string_field(ctx, &fields, "program", function_name) {
        Ok(Some(value)) => value,
        Ok(None) => String::new(),
        Err(e) => return Err(e),
    };

    let args = match get_optional_string_vector_field(ctx, &fields, "args", function_name) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let cwd = match get_optional_string_field(ctx, &fields, "cwd", function_name) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let env = match get_optional_string_object_field(ctx, &fields, "env", function_name) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let stdin = match get_optional_string_field(ctx, &fields, "stdin", function_name) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(OsProcessData {
        program,
        args,
        cwd,
        env,
        stdin,
    })
}

fn execute_process_data(
    process: OsProcessData,
    function_name: &str,
) -> Result<OsProcessResultData, String> {
    if process.program.is_empty() {
        return Err(format!(
            "os:{}() process.program cannot be empty",
            function_name
        ));
    }

    let mut command = Command::new(process.program.clone());

    command.args(process.args);

    match process.cwd {
        Some(cwd) => {
            command.current_dir(cwd);
        }

        None => {}
    }

    for (name, value) in process.env {
        command.env(name, value);
    }

    match process.stdin {
        Some(stdin) => {
            command.stdin(Stdio::piped());
            command.stdout(Stdio::piped());
            command.stderr(Stdio::piped());

            let mut child = match command.spawn() {
                Ok(child) => child,
                Err(e) => {
                    return Err(format!(
                        "os:{}() failed to spawn process '{}': {}",
                        function_name, process.program, e
                    ));
                }
            };

            match child.stdin.take() {
                Some(mut child_stdin) => {
                    match child_stdin.write_all(stdin.as_bytes()) {
                        Ok(_) => {}
                        Err(e) => {
                            return Err(format!(
                                "os:{}() failed to write process stdin: {}",
                                function_name, e
                            ));
                        }
                    }
                }

                None => {
                    return Err(format!(
                        "os:{}() failed to open process stdin",
                        function_name
                    ));
                }
            }

            let output = match child.wait_with_output() {
                Ok(output) => output,
                Err(e) => {
                    return Err(format!(
                        "os:{}() failed to wait process '{}': {}",
                        function_name, process.program, e
                    ));
                }
            };

            Ok(output_to_result_data(output))
        }

        None => {
            let output = match command.output() {
                Ok(output) => output,
                Err(e) => {
                    return Err(format!(
                        "os:{}() failed to run process '{}': {}",
                        function_name, process.program, e
                    ));
                }
            };

            Ok(output_to_result_data(output))
        }
    }
}

fn output_to_result_data(output: std::process::Output) -> OsProcessResultData {
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let exit_code = match output.status.code() {
        Some(code) => code as isize,
        None => -1,
    };

    OsProcessResultData {
        stdout,
        stderr,
        exit_code,
        success: output.status.success(),
    }
}

fn process_result_to_object(
    ctx: &mut PengNativeFunctionCallContext,
    result: OsProcessResultData,
) -> Result<PengBindedCell, PengError> {
    let stdout = utils::string_cell(ctx, result.stdout);
    let stderr = utils::string_cell(ctx, result.stderr);

    utils::new_object(
        ctx,
        vec![
            ("stdout", stdout),
            ("stderr", stderr),
            (
                "exit_code",
                PengBindedCell::Mutable(PengCell::Int(result.exit_code)),
            ),
            (
                "success",
                PengBindedCell::Mutable(PengCell::Bool(result.success)),
            ),
        ],
    )
}

fn task_object(
    ctx: &mut PengNativeFunctionCallContext,
    state: Arc<Mutex<OsTaskState>>,
) -> Result<PengBindedCell, PengError> {
    let is_finished_state = state.clone();
    let get_state = state.clone();
    let error_state = state.clone();

    let is_finished_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| task_is_finished(ctx, is_finished_state.clone()),
    )));

    let get_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| task_get(ctx, get_state.clone()),
    )));

    let error_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| task_error(ctx, error_state.clone()),
    )));

    utils::new_object(
        ctx,
        vec![
            (
                "is_finished",
                PengBindedCell::Immutable(PengCell::Reference(is_finished_ptr)),
            ),
            (
                "get",
                PengBindedCell::Immutable(PengCell::Reference(get_ptr)),
            ),
            (
                "error",
                PengBindedCell::Immutable(PengCell::Reference(error_ptr)),
            ),
        ],
    )
}

fn task_is_finished(
    _ctx: &mut PengNativeFunctionCallContext,
    state: Arc<Mutex<OsTaskState>>,
) -> Result<PengBindedCell, PengError> {
    match state.lock() {
        Ok(locked) => match &*locked {
            OsTaskState::Running => Ok(PengBindedCell::Mutable(PengCell::Bool(false))),
            OsTaskState::Finished(_) => Ok(PengBindedCell::Mutable(PengCell::Bool(true))),
        },

        Err(_) => Err(PengError::CannotCallValue(
            "os task lock failed".to_string(),
        )),
    }
}

fn task_get(
    ctx: &mut PengNativeFunctionCallContext,
    state: Arc<Mutex<OsTaskState>>,
) -> Result<PengBindedCell, PengError> {
    let result = match state.lock() {
        Ok(locked) => match &*locked {
            OsTaskState::Running => return utils::nil(),
            OsTaskState::Finished(result) => result.clone(),
        },

        Err(_) => {
            return Err(PengError::CannotCallValue(
                "os task lock failed".to_string(),
            ));
        }
    };

    match result {
        Ok(result) => process_result_to_object(ctx, result),
        Err(e) => Err(PengError::CannotCallValue(e)),
    }
}

fn task_error(
    ctx: &mut PengNativeFunctionCallContext,
    state: Arc<Mutex<OsTaskState>>,
) -> Result<PengBindedCell, PengError> {
    match state.lock() {
        Ok(locked) => match &*locked {
            OsTaskState::Running => utils::nil(),

            OsTaskState::Finished(result) => match result {
                Ok(_) => utils::nil(),
                Err(e) => utils::string(ctx, e.clone()),
            },
        },

        Err(_) => Err(PengError::CannotCallValue(
            "os task lock failed".to_string(),
        )),
    }
}

fn get_optional_string_field(
    ctx: &mut PengNativeFunctionCallContext,
    fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    name: &str,
    function_name: &str,
) -> Result<Option<String>, PengError> {
    match get_field(ctx, fields, name) {
        Some(value) => match value.value() {
            PengCell::Nil => Ok(None),
            _ => match strict_cell_to_string(ctx, &value, function_name) {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(e),
            },
        },

        None => Ok(None),
    }
}

fn get_optional_string_vector_field(
    ctx: &mut PengNativeFunctionCallContext,
    fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    name: &str,
    function_name: &str,
) -> Result<Vec<String>, PengError> {
    let value = match get_field(ctx, fields, name) {
        Some(value) => value,
        None => return Ok(Vec::new()),
    };

    match value.value() {
        PengCell::Nil => Ok(Vec::new()),

        PengCell::Reference(ptr) => match ctx.env().get_heap(*ptr) {
            Some(PengValue::Box(PengBox::Vector(vector))) => {
                let mut output = Vec::new();

                for item in vector.values.iter() {
                    let item = match strict_cell_to_string(ctx, item, function_name) {
                        Ok(item) => item,
                        Err(e) => return Err(e),
                    };

                    output.push(item);
                }

                Ok(output)
            }

            Some(_) => Err(PengError::CannotCallValue(format!(
                "os:{}() field '{}' must be a vector of strings",
                function_name, name
            ))),

            None => Err(PengError::CannotCallValue(format!(
                "os:{}() field '{}' points to missing heap value",
                function_name, name
            ))),
        },

        _ => Err(PengError::CannotCallValue(format!(
            "os:{}() field '{}' must be a vector of strings",
            function_name, name
        ))),
    }
}

fn get_optional_string_object_field(
    ctx: &mut PengNativeFunctionCallContext,
    fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    name: &str,
    function_name: &str,
) -> Result<Vec<(String, String)>, PengError> {
    let value = match get_field(ctx, fields, name) {
        Some(value) => value,
        None => return Ok(Vec::new()),
    };

    match value.value() {
        PengCell::Nil => Ok(Vec::new()),

        _ => {
            let object_fields = match utils::get_object_fields_from_cell(ctx, &value) {
                Ok(fields) => fields,
                Err(e) => return Err(e),
            };

            let mut output = Vec::new();

            for (name_ptr, value) in object_fields.iter() {
                let key = match ctx.env().get_pooled_name(*name_ptr) {
                    Some(key) => key.clone(),
                    None => {
                        return Err(PengError::CannotCallValue(format!(
                            "os:{}() field '{}' has invalid env key",
                            function_name, name
                        )));
                    }
                };

                let value = match strict_cell_to_string(ctx, value, function_name) {
                    Ok(value) => value,
                    Err(e) => return Err(e),
                };

                output.push((key, value));
            }

            Ok(output)
        }
    }
}

fn strict_cell_to_string(
    ctx: &PengNativeFunctionCallContext,
    cell: &PengBindedCell,
    function_name: &str,
) -> Result<String, PengError> {
    match cell.value() {
        PengCell::Reference(ptr) => match ctx.env().get_heap(*ptr) {
            Some(PengValue::Box(PengBox::String(value))) => Ok(value.clone()),

            Some(_) => Err(PengError::CannotCallValue(format!(
                "os:{}() expected string",
                function_name
            ))),

            None => Err(PengError::CannotCallValue(format!(
                "os:{}() expected string, but heap value is missing",
                function_name
            ))),
        },

        _ => Err(PengError::CannotCallValue(format!(
            "os:{}() expected string",
            function_name
        ))),
    }
}

fn get_field(
    ctx: &mut PengNativeFunctionCallContext,
    fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    name: &str,
) -> Option<PengBindedCell> {
    let name_ptr = ctx.env_mut().ensure_pooled_name_ptr(name.to_string());

    match fields.get(&name_ptr) {
        Some(value) => Some(value.clone()),
        None => None,
    }
}

fn get_string_arg(
    ctx: &PengNativeFunctionCallContext,
    index: usize,
    function_name: &str,
) -> Result<String, PengError> {
    let arg = match ctx.get_arg_cell(index) {
        Some(arg) => arg,
        None => {
            return Err(PengError::CannotCallValue(format!(
                "os:{}() missing argument at index {}",
                function_name, index
            )));
        }
    };

    strict_cell_to_string(ctx, arg, function_name)
}

fn string_binded_cell_from_env(env: &mut PengEnv, value: String) -> PengBindedCell {
    let ptr = env.create_heap_value(PengValue::Box(PengBox::String(value)));

    PengBindedCell::Mutable(PengCell::Reference(ptr))
}