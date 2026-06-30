use std::collections::HashMap;
use std::env;
use std::fs as std_fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use penguin::prelude::*;

type NativeResult = Result<PengBindedCell, PengError>;

pub fn setup(peng: &mut PengEnv) -> PengUnit {
    let mut module = PengUnit::library();

    module.register_immutable_native_function(peng, "read", read).unwrap();
    module.register_immutable_native_function(peng, "write", write).unwrap();
    module.register_immutable_native_function(peng, "append", append).unwrap();

    module.register_immutable_native_function(peng, "exists", exists).unwrap();
    module.register_immutable_native_function(peng, "is_file", is_file).unwrap();
    module.register_immutable_native_function(peng, "is_dir", is_dir).unwrap();

    module.register_immutable_native_function(peng, "create_dir", create_dir).unwrap();
    module
        .register_immutable_native_function(peng, "create_dir_all", create_dir_all)
        .unwrap();
    module
        .register_immutable_native_function(peng, "remove_file", remove_file)
        .unwrap();
    module.register_immutable_native_function(peng, "remove_dir", remove_dir).unwrap();
    module
        .register_immutable_native_function(peng, "remove_dir_all", remove_dir_all)
        .unwrap();

    module.register_immutable_native_function(peng, "list_dir", list_dir).unwrap();

    module.register_immutable_native_function(peng, "copy", copy).unwrap();
    module.register_immutable_native_function(peng, "rename", rename).unwrap();

    module.register_immutable_native_function(peng, "metadata", metadata).unwrap();
    module.register_immutable_native_function(peng, "file_size", file_size).unwrap();

    module
        .register_immutable_native_function(peng, "current_dir", current_dir)
        .unwrap();
    module
        .register_immutable_native_function(peng, "set_current_dir", set_current_dir)
        .unwrap();

    module.register_immutable_native_function(peng, "absolute", absolute).unwrap();
    module
        .register_immutable_native_function(peng, "canonicalize", canonicalize)
        .unwrap();

    module.register_immutable_native_function(peng, "join", join).unwrap();
    module.register_immutable_native_function(peng, "file_name", file_name).unwrap();
    module.register_immutable_native_function(peng, "extension", extension).unwrap();
    module.register_immutable_native_function(peng, "parent", parent).unwrap();

    module
}

fn read(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match get_string_arg(ctx, 0, "read") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    let content = match std_fs::read_to_string(&path) {
        Ok(content) => content,
        Err(_) => {
            return Err(PengError::CannotCallValue(format!(
                "fs:read() failed to read '{}'",
                path
            )));
        }
    };

    string(ctx, content)
}

fn write(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match get_string_arg(ctx, 0, "write") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    let content = match get_string_arg(ctx, 1, "write") {
        Ok(content) => content,
        Err(e) => return Err(e),
    };

    match std_fs::write(&path, content) {
        Ok(_) => nil(),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "fs:write() failed to write '{}'",
            path
        ))),
    }
}

fn append(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match get_string_arg(ctx, 0, "append") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    let content = match get_string_arg(ctx, 1, "append") {
        Ok(content) => content,
        Err(e) => return Err(e),
    };

    let mut file = match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(file) => file,
        Err(_) => {
            return Err(PengError::CannotCallValue(format!(
                "fs:append() failed to open '{}'",
                path
            )));
        }
    };

    match file.write_all(content.as_bytes()) {
        Ok(_) => nil(),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "fs:append() failed to write '{}'",
            path
        ))),
    }
}

fn exists(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match get_string_arg(ctx, 0, "exists") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    bool_cell(Path::new(&path).exists())
}

fn is_file(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match get_string_arg(ctx, 0, "is_file") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    bool_cell(Path::new(&path).is_file())
}

fn is_dir(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match get_string_arg(ctx, 0, "is_dir") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    bool_cell(Path::new(&path).is_dir())
}

fn create_dir(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match get_string_arg(ctx, 0, "create_dir") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    match std_fs::create_dir(&path) {
        Ok(_) => nil(),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "fs:create_dir() failed to create '{}'",
            path
        ))),
    }
}

fn create_dir_all(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match get_string_arg(ctx, 0, "create_dir_all") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    match std_fs::create_dir_all(&path) {
        Ok(_) => nil(),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "fs:create_dir_all() failed to create '{}'",
            path
        ))),
    }
}

fn remove_file(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match get_string_arg(ctx, 0, "remove_file") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    match std_fs::remove_file(&path) {
        Ok(_) => nil(),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "fs:remove_file() failed to remove '{}'",
            path
        ))),
    }
}

fn remove_dir(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match get_string_arg(ctx, 0, "remove_dir") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    match std_fs::remove_dir(&path) {
        Ok(_) => nil(),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "fs:remove_dir() failed to remove '{}'",
            path
        ))),
    }
}

fn remove_dir_all(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match get_string_arg(ctx, 0, "remove_dir_all") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    match std_fs::remove_dir_all(&path) {
        Ok(_) => nil(),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "fs:remove_dir_all() failed to remove '{}'",
            path
        ))),
    }
}

fn list_dir(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match get_string_arg(ctx, 0, "list_dir") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    let entries = match std_fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(_) => {
            return Err(PengError::CannotCallValue(format!(
                "fs:list_dir() failed to read '{}'",
                path
            )));
        }
    };

    let mut values = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                return Err(PengError::CannotCallValue(format!(
                    "fs:list_dir() failed to read entry in '{}'",
                    path
                )));
            }
        };

        let name = entry.file_name().to_string_lossy().into_owned();
        values.push(string_cell(ctx, name));
    }

    vector(ctx, values)
}

fn copy(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let from = match get_string_arg(ctx, 0, "copy") {
        Ok(from) => from,
        Err(e) => return Err(e),
    };

    let to = match get_string_arg(ctx, 1, "copy") {
        Ok(to) => to,
        Err(e) => return Err(e),
    };

    let copied = match std_fs::copy(&from, &to) {
        Ok(copied) => copied,
        Err(_) => {
            return Err(PengError::CannotCallValue(format!(
                "fs:copy() failed to copy '{}' to '{}'",
                from, to
            )));
        }
    };

    uint_cell(copied as usize)
}

fn rename(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let from = match get_string_arg(ctx, 0, "rename") {
        Ok(from) => from,
        Err(e) => return Err(e),
    };

    let to = match get_string_arg(ctx, 1, "rename") {
        Ok(to) => to,
        Err(e) => return Err(e),
    };

    match std_fs::rename(&from, &to) {
        Ok(_) => nil(),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "fs:rename() failed to rename '{}' to '{}'",
            from, to
        ))),
    }
}

fn metadata(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match get_string_arg(ctx, 0, "metadata") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    let metadata = match std_fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(_) => {
            return Err(PengError::CannotCallValue(format!(
                "fs:metadata() failed to read '{}'",
                path
            )));
        }
    };

    object(
        ctx,
        vec![
            (
                "is_file",
                PengBindedCell::Mutable(PengCell::Bool(metadata.is_file())),
            ),
            (
                "is_dir",
                PengBindedCell::Mutable(PengCell::Bool(metadata.is_dir())),
            ),
            (
                "is_symlink",
                PengBindedCell::Mutable(PengCell::Bool(metadata.file_type().is_symlink())),
            ),
            (
                "readonly",
                PengBindedCell::Mutable(PengCell::Bool(metadata.permissions().readonly())),
            ),
            (
                "size",
                PengBindedCell::Mutable(PengCell::Uint(metadata.len() as usize)),
            ),
        ],
    )
}

fn file_size(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match get_string_arg(ctx, 0, "file_size") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    let metadata = match std_fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(_) => {
            return Err(PengError::CannotCallValue(format!(
                "fs:file_size() failed to read '{}'",
                path
            )));
        }
    };

    uint_cell(metadata.len() as usize)
}

fn current_dir(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match env::current_dir() {
        Ok(path) => path,
        Err(_) => return Err(PengError::CannotCallValue("fs:current_dir() failed".into())),
    };

    string(ctx, path_to_string(path))
}

fn set_current_dir(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match get_string_arg(ctx, 0, "set_current_dir") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    match env::set_current_dir(&path) {
        Ok(_) => nil(),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "fs:set_current_dir() failed to set '{}'",
            path
        ))),
    }
}

fn absolute(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match get_string_arg(ctx, 0, "absolute") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    let path_buf = PathBuf::from(&path);

    if path_buf.is_absolute() {
        return string(ctx, path_to_string(path_buf));
    }

    let current_dir = match env::current_dir() {
        Ok(current_dir) => current_dir,
        Err(_) => return Err(PengError::CannotCallValue("fs:absolute() failed".into())),
    };

    string(ctx, path_to_string(current_dir.join(path_buf)))
}

fn canonicalize(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match get_string_arg(ctx, 0, "canonicalize") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    let canonical = match std_fs::canonicalize(&path) {
        Ok(canonical) => canonical,
        Err(_) => {
            return Err(PengError::CannotCallValue(format!(
                "fs:canonicalize() failed to canonicalize '{}'",
                path
            )));
        }
    };

    string(ctx, path_to_string(canonical))
}

fn join(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let left = match get_string_arg(ctx, 0, "join") {
        Ok(left) => left,
        Err(e) => return Err(e),
    };

    let right = match get_string_arg(ctx, 1, "join") {
        Ok(right) => right,
        Err(e) => return Err(e),
    };

    let joined = PathBuf::from(left).join(right);

    string(ctx, path_to_string(joined))
}

fn file_name(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match get_string_arg(ctx, 0, "file_name") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    match Path::new(&path).file_name() {
        Some(name) => string(ctx, name.to_string_lossy().into_owned()),
        None => nil(),
    }
}

fn extension(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match get_string_arg(ctx, 0, "extension") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    match Path::new(&path).extension() {
        Some(extension) => string(ctx, extension.to_string_lossy().into_owned()),
        None => nil(),
    }
}

fn parent(ctx: &mut PengNativeFunctionCallContext) -> NativeResult {
    let path = match get_string_arg(ctx, 0, "parent") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    match Path::new(&path).parent() {
        Some(parent) => string(ctx, parent.to_string_lossy().into_owned()),
        None => nil(),
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
                "fs:{}() missing argument at index {}",
                function_name, index
            )));
        }
    };

    match arg.value() {
        PengCell::Reference(ptr) => match ctx.get_value(*ptr) {
            Some(PengValue::Box(PengBox::String(value))) => Ok(value.clone()),

            Some(_) => Err(PengError::CannotCallValue(format!(
                "fs:{}() expected string argument at index {}",
                function_name, index
            ))),

            None => Err(PengError::CannotCallValue(format!(
                "fs:{}() got missing heap value at index {}",
                function_name, index
            ))),
        },

        _ => Err(PengError::CannotCallValue(format!(
            "fs:{}() expected string argument at index {}",
            function_name, index
        ))),
    }
}

fn nil() -> NativeResult {
    Ok(PengBindedCell::Mutable(PengCell::Nil))
}

fn bool_cell(value: bool) -> NativeResult {
    Ok(PengBindedCell::Mutable(PengCell::Bool(value)))
}

fn uint_cell(value: usize) -> NativeResult {
    Ok(PengBindedCell::Mutable(PengCell::Uint(value)))
}

fn string(ctx: &mut PengNativeFunctionCallContext, value: String) -> NativeResult {
    Ok(string_cell(ctx, value))
}

fn string_cell(ctx: &mut PengNativeFunctionCallContext, value: String) -> PengBindedCell {
    let ptr = ctx.create_box(PengBox::String(value));

    PengBindedCell::Mutable(PengCell::Reference(ptr))
}

fn vector(ctx: &mut PengNativeFunctionCallContext, values: Vec<PengBindedCell>) -> NativeResult {
    let ptr = ctx.create_box(PengBox::Vector(PengVector { values }));

    Ok(PengBindedCell::Mutable(PengCell::Reference(ptr)))
}

fn object(
    ctx: &mut PengNativeFunctionCallContext,
    values: Vec<(&str, PengBindedCell)>,
) -> NativeResult {
    let mut fields = HashMap::new();

    for (name, value) in values {
        let name_ptr = ctx.env_mut().ensure_pooled_name_ptr(name.to_string());
        fields.insert(name_ptr, value);
    }

    let ptr = ctx.create_box(PengBox::Object(PengObject { fields }));

    Ok(PengBindedCell::Mutable(PengCell::Reference(ptr)))
}

fn path_to_string(path: PathBuf) -> String {
    path.to_string_lossy().into_owned()
}