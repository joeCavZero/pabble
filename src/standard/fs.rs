use std::env;
use std::fs as std_fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use penguin::prelude::*;

use super::utils;

pub fn setup(peng: &mut PengEnv) -> PengUnit {
    let mut module = PengUnit::library();

    module
        .register_immutable_native_function(peng, "read", read)
        .unwrap();
    module
        .register_immutable_native_function(peng, "write", write)
        .unwrap();
    module
        .register_immutable_native_function(peng, "append", append)
        .unwrap();

    module
        .register_immutable_native_function(peng, "exists", exists)
        .unwrap();
    module
        .register_immutable_native_function(peng, "is_file", is_file)
        .unwrap();
    module
        .register_immutable_native_function(peng, "is_dir", is_dir)
        .unwrap();

    module
        .register_immutable_native_function(peng, "create_dir", create_dir)
        .unwrap();
    module
        .register_immutable_native_function(peng, "create_dir_all", create_dir_all)
        .unwrap();
    module
        .register_immutable_native_function(peng, "remove_file", remove_file)
        .unwrap();
    module
        .register_immutable_native_function(peng, "remove_dir", remove_dir)
        .unwrap();
    module
        .register_immutable_native_function(peng, "remove_dir_all", remove_dir_all)
        .unwrap();

    module
        .register_immutable_native_function(peng, "list_dir", list_dir)
        .unwrap();

    module
        .register_immutable_native_function(peng, "copy", copy)
        .unwrap();
    module
        .register_immutable_native_function(peng, "rename", rename)
        .unwrap();

    module
        .register_immutable_native_function(peng, "metadata", metadata)
        .unwrap();
    module
        .register_immutable_native_function(peng, "file_size", file_size)
        .unwrap();

    module
        .register_immutable_native_function(peng, "current_dir", current_dir)
        .unwrap();
    module
        .register_immutable_native_function(peng, "set_current_dir", set_current_dir)
        .unwrap();

    module
        .register_immutable_native_function(peng, "absolute", absolute)
        .unwrap();
    module
        .register_immutable_native_function(peng, "canonicalize", canonicalize)
        .unwrap();

    module
        .register_immutable_native_function(peng, "join", join)
        .unwrap();
    module
        .register_immutable_native_function(peng, "file_name", file_name)
        .unwrap();
    module
        .register_immutable_native_function(peng, "extension", extension)
        .unwrap();
    module
        .register_immutable_native_function(peng, "parent", parent)
        .unwrap();

    module
}

fn read(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match utils::get_string_arg(ctx, 0) {
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

    utils::string(ctx, content)
}

fn write(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match utils::get_string_arg(ctx, 0) {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    let content = match utils::get_string_arg(ctx, 1) {
        Ok(content) => content,
        Err(e) => return Err(e),
    };

    match std_fs::write(&path, content) {
        Ok(_) => utils::nil(),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "fs:write() failed to write '{}'",
            path
        ))),
    }
}

fn append(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match utils::get_string_arg(ctx, 0) {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    let content = match utils::get_string_arg(ctx, 1) {
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
        Ok(_) => utils::nil(),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "fs:append() failed to write '{}'",
            path
        ))),
    }
}

fn exists(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match utils::get_string_arg(ctx, 0) {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    utils::bool_cell(Path::new(&path).exists())
}

fn is_file(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match utils::get_string_arg(ctx, 0) {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    utils::bool_cell(Path::new(&path).is_file())
}

fn is_dir(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match utils::get_string_arg(ctx, 0) {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    utils::bool_cell(Path::new(&path).is_dir())
}

fn create_dir(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match utils::get_string_arg(ctx, 0) {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    match std_fs::create_dir(&path) {
        Ok(_) => utils::nil(),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "fs:create_dir() failed to create '{}'",
            path
        ))),
    }
}

fn create_dir_all(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match utils::get_string_arg(ctx, 0) {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    match std_fs::create_dir_all(&path) {
        Ok(_) => utils::nil(),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "fs:create_dir_all() failed to create '{}'",
            path
        ))),
    }
}

fn remove_file(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match utils::get_string_arg(ctx, 0) {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    match std_fs::remove_file(&path) {
        Ok(_) => utils::nil(),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "fs:remove_file() failed to remove '{}'",
            path
        ))),
    }
}

fn remove_dir(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match utils::get_string_arg(ctx, 0) {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    match std_fs::remove_dir(&path) {
        Ok(_) => utils::nil(),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "fs:remove_dir() failed to remove '{}'",
            path
        ))),
    }
}

fn remove_dir_all(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match utils::get_string_arg(ctx, 0) {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    match std_fs::remove_dir_all(&path) {
        Ok(_) => utils::nil(),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "fs:remove_dir_all() failed to remove '{}'",
            path
        ))),
    }
}

fn list_dir(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match utils::get_string_arg(ctx, 0) {
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
        values.push(utils::string_cell(ctx, name));
    }

    utils::vector(ctx, values)
}

fn copy(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let from = match utils::get_string_arg(ctx, 0) {
        Ok(from) => from,
        Err(e) => return Err(e),
    };

    let to = match utils::get_string_arg(ctx, 1) {
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

    utils::uint_cell(copied as usize)
}

fn rename(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let from = match utils::get_string_arg(ctx, 0) {
        Ok(from) => from,
        Err(e) => return Err(e),
    };

    let to = match utils::get_string_arg(ctx, 1) {
        Ok(to) => to,
        Err(e) => return Err(e),
    };

    match std_fs::rename(&from, &to) {
        Ok(_) => utils::nil(),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "fs:rename() failed to rename '{}' to '{}'",
            from, to
        ))),
    }
}

fn metadata(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match utils::get_string_arg(ctx, 0) {
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

    utils::object(
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

fn file_size(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match utils::get_string_arg(ctx, 0) {
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

    utils::uint_cell(metadata.len() as usize)
}

fn current_dir(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match env::current_dir() {
        Ok(path) => path,
        Err(_) => return Err(PengError::CannotCallValue("fs:current_dir() failed".into())),
    };

    utils::string(ctx, path_to_string(path))
}

fn set_current_dir(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match utils::get_string_arg(ctx, 0) {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    match env::set_current_dir(&path) {
        Ok(_) => utils::nil(),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "fs:set_current_dir() failed to set '{}'",
            path
        ))),
    }
}

fn absolute(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match utils::get_string_arg(ctx, 0) {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    let path_buf = PathBuf::from(&path);

    if path_buf.is_absolute() {
        return utils::string(ctx, path_to_string(path_buf));
    }

    let current_dir = match env::current_dir() {
        Ok(current_dir) => current_dir,
        Err(_) => return Err(PengError::CannotCallValue("fs:absolute() failed".into())),
    };

    utils::string(ctx, path_to_string(current_dir.join(path_buf)))
}

fn canonicalize(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match utils::get_string_arg(ctx, 0) {
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

    utils::string(ctx, path_to_string(canonical))
}

fn join(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let left = match utils::get_string_arg(ctx, 0) {
        Ok(left) => left,
        Err(e) => return Err(e),
    };

    let right = match utils::get_string_arg(ctx, 1) {
        Ok(right) => right,
        Err(e) => return Err(e),
    };

    let joined = PathBuf::from(left).join(right);

    utils::string(ctx, path_to_string(joined))
}

fn file_name(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match utils::get_string_arg(ctx, 0) {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    match Path::new(&path).file_name() {
        Some(name) => utils::string(ctx, name.to_string_lossy().into_owned()),
        None => utils::nil(),
    }
}

fn extension(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match utils::get_string_arg(ctx, 0) {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    match Path::new(&path).extension() {
        Some(extension) => utils::string(ctx, extension.to_string_lossy().into_owned()),
        None => utils::nil(),
    }
}

fn parent(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match utils::get_string_arg(ctx, 0) {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    match Path::new(&path).parent() {
        Some(parent) => utils::string(ctx, parent.to_string_lossy().into_owned()),
        None => utils::nil(),
    }
}

fn path_to_string(path: PathBuf) -> String {
    path.to_string_lossy().into_owned()
}
