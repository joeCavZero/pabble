use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

use libffi::middle::{arg, Arg, Cif, CodePtr, Type};
use libloading::Library;
use penguin::prelude::*;

use super::utils;

#[derive(Debug)]
struct FfiLibraryEntry {
    path: String,
    library: Library,
}

#[derive(Debug)]
struct FfiBufferEntry {
    bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
enum FfiTypeKind {
    Void,
    Bool,

    Char,
    UChar,

    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,

    Int,
    UInt,
    Long,
    ULong,
    LongLong,
    ULongLong,

    F32,
    F64,

    Size,
    SSize,
    USize,
    ISize,

    Ptr,
    CString,
}

#[derive(Debug, Clone)]
struct FfiCallableData {
    library_id: usize,
    library_path: String,
    symbol_name: String,
    symbol_addr: usize,
    params: Vec<FfiTypeKind>,
    result: FfiTypeKind,
}

enum FfiArgValue {
    Bool(u8),

    Char(i8),
    UChar(u8),

    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),

    Int(i32),
    UInt(u32),
    Long(isize),
    ULong(usize),
    LongLong(i64),
    ULongLong(u64),

    F32(f32),
    F64(f64),

    Size(usize),
    SSize(isize),
    USize(usize),
    ISize(isize),

    Ptr(*mut c_void),
}

static NEXT_LIBRARY_ID: AtomicUsize = AtomicUsize::new(1);
static NEXT_BUFFER_ID: AtomicUsize = AtomicUsize::new(1);

static LIBRARIES: OnceLock<Mutex<HashMap<usize, FfiLibraryEntry>>> = OnceLock::new();
static BUFFERS: OnceLock<Mutex<HashMap<usize, FfiBufferEntry>>> = OnceLock::new();

pub fn setup(peng: &mut PengEnv) -> PengUnit {
    let mut module = PengUnit::library();

    register_type_global(&mut module, peng, "c_void");
    register_type_global(&mut module, peng, "c_bool");

    register_type_global(&mut module, peng, "c_char");
    register_type_global(&mut module, peng, "c_uchar");

    register_type_global(&mut module, peng, "c_i8");
    register_type_global(&mut module, peng, "c_u8");
    register_type_global(&mut module, peng, "c_i16");
    register_type_global(&mut module, peng, "c_u16");
    register_type_global(&mut module, peng, "c_i32");
    register_type_global(&mut module, peng, "c_u32");
    register_type_global(&mut module, peng, "c_i64");
    register_type_global(&mut module, peng, "c_u64");

    register_type_global(&mut module, peng, "c_int");
    register_type_global(&mut module, peng, "c_uint");
    register_type_global(&mut module, peng, "c_long");
    register_type_global(&mut module, peng, "c_ulong");
    register_type_global(&mut module, peng, "c_longlong");
    register_type_global(&mut module, peng, "c_ulonglong");

    register_type_global(&mut module, peng, "c_f32");
    register_type_global(&mut module, peng, "c_f64");

    register_type_global(&mut module, peng, "c_size");
    register_type_global(&mut module, peng, "c_ssize");
    register_type_global(&mut module, peng, "c_usize");
    register_type_global(&mut module, peng, "c_isize");

    register_type_global(&mut module, peng, "c_ptr");
    register_type_global(&mut module, peng, "c_string");

    module
        .register_immutable_native_function(peng, "open", open)
        .unwrap();
    module
        .register_immutable_native_function(peng, "open_platform", open_platform)
        .unwrap();
    module
        .register_immutable_native_function(peng, "close", close)
        .unwrap();
    module
        .register_immutable_native_function(peng, "is_open", is_open)
        .unwrap();

    module
        .register_immutable_native_function(peng, "symbol", symbol)
        .unwrap();
    module
        .register_immutable_native_function(peng, "has_symbol", has_symbol)
        .unwrap();
    module
        .register_immutable_native_function(peng, "bind", bind)
        .unwrap();

    module
        .register_immutable_native_function(peng, "null", null)
        .unwrap();
    module
        .register_immutable_native_function(peng, "ptr", ptr)
        .unwrap();
    module
        .register_immutable_native_function(peng, "addr", addr)
        .unwrap();
    module
        .register_immutable_native_function(peng, "is_null", is_null)
        .unwrap();

    module
        .register_immutable_native_function(peng, "malloc", malloc)
        .unwrap();
    module
        .register_immutable_native_function(peng, "free", free)
        .unwrap();
    module
        .register_immutable_native_function(peng, "cstring", cstring)
        .unwrap();
    module
        .register_immutable_native_function(peng, "free_cstring", free_cstring)
        .unwrap();

    module
        .register_immutable_native_function(peng, "buffer", buffer)
        .unwrap();

    module
        .register_immutable_native_function(peng, "ptr_size", ptr_size)
        .unwrap();

    module
}

fn register_type_global(module: &mut PengUnit, peng: &mut PengEnv, name: &str) {
    let value = PengValue::Box(PengBox::String(name.to_string()));
    module.register_immutable_global(peng, name, value).unwrap();
}

fn libraries() -> &'static Mutex<HashMap<usize, FfiLibraryEntry>> {
    LIBRARIES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn buffers() -> &'static Mutex<HashMap<usize, FfiBufferEntry>> {
    BUFFERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn open(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let path = match get_string_arg(ctx, 0, "open") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    open_library_path(ctx, path)
}

fn open_platform(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let arg = match ctx.get_arg_cell(0) {
        Some(arg) => arg.clone(),
        None => {
            return Err(PengError::CannotCallValue(
                "ffi:open_platform() expected object".to_string(),
            ));
        }
    };

    let fields = match utils::get_object_fields_from_cell(ctx, &arg) {
        Ok(fields) => fields,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "ffi:open_platform() expected object".to_string(),
            ));
        }
    };

    let os_name = std::env::consts::OS.to_string();

    match get_optional_string_field(ctx, &fields, &os_name, "open_platform") {
        Ok(Some(path)) => return open_library_path(ctx, path),
        Ok(None) => {}
        Err(e) => return Err(e),
    }

    if os_name == "macos" {
        match get_optional_string_field(ctx, &fields, "darwin", "open_platform") {
            Ok(Some(path)) => return open_library_path(ctx, path),
            Ok(None) => {}
            Err(e) => return Err(e),
        }
    }

    if os_name == "linux"
        || os_name == "macos"
        || os_name == "freebsd"
        || os_name == "openbsd"
        || os_name == "netbsd"
    {
        match get_optional_string_field(ctx, &fields, "unix", "open_platform") {
            Ok(Some(path)) => return open_library_path(ctx, path),
            Ok(None) => {}
            Err(e) => return Err(e),
        }
    }

    match get_optional_string_field(ctx, &fields, "default", "open_platform") {
        Ok(Some(path)) => open_library_path(ctx, path),
        Ok(None) => Err(PengError::CannotCallValue(format!(
            "ffi:open_platform() no library path for '{}'",
            os_name
        ))),
        Err(e) => Err(e),
    }
}

fn open_library_path(
    ctx: &mut PengNativeFunctionCallContext,
    path: String,
) -> Result<PengBindedCell, PengError> {
    let library = unsafe { Library::new(path.clone()) };

    let library = match library {
        Ok(library) => library,
        Err(e) => {
            return Err(PengError::CannotCallValue(format!(
                "ffi:open() failed to open '{}': {}",
                path, e
            )));
        }
    };

    let id = NEXT_LIBRARY_ID.fetch_add(1, Ordering::Relaxed);

    let mut locked = match libraries().lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "ffi:open() library registry lock failed".to_string(),
            ));
        }
    };

    locked.insert(
        id,
        FfiLibraryEntry {
            path: path.clone(),
            library,
        },
    );

    library_object(ctx, id, path)
}

fn close(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let id = match get_library_id_arg(ctx, 0, "close") {
        Ok(id) => id,
        Err(e) => return Err(e),
    };

    let mut locked = match libraries().lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "ffi:close() library registry lock failed".to_string(),
            ));
        }
    };

    locked.remove(&id);

    utils::nil()
}

fn is_open(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let id = match get_library_id_arg(ctx, 0, "is_open") {
        Ok(id) => id,
        Err(e) => return Err(e),
    };

    let locked = match libraries().lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "ffi:is_open() library registry lock failed".to_string(),
            ));
        }
    };

    Ok(PengBindedCell::Mutable(PengCell::Bool(
        locked.contains_key(&id),
    )))
}

fn symbol(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let id = match get_library_id_arg(ctx, 0, "symbol") {
        Ok(id) => id,
        Err(e) => return Err(e),
    };

    let name = match get_string_arg(ctx, 1, "symbol") {
        Ok(name) => name,
        Err(e) => return Err(e),
    };

    let data = match get_symbol_data(id, &name, "symbol") {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    symbol_object(ctx, id, data.0, name, data.1)
}

fn has_symbol(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let id = match get_library_id_arg(ctx, 0, "has_symbol") {
        Ok(id) => id,
        Err(e) => return Err(e),
    };

    let name = match get_string_arg(ctx, 1, "has_symbol") {
        Ok(name) => name,
        Err(e) => return Err(e),
    };

    match get_symbol_data(id, &name, "has_symbol") {
        Ok(_) => Ok(PengBindedCell::Mutable(PengCell::Bool(true))),
        Err(_) => Ok(PengBindedCell::Mutable(PengCell::Bool(false))),
    }
}

fn bind(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let id = match get_library_id_arg(ctx, 0, "bind") {
        Ok(id) => id,
        Err(e) => return Err(e),
    };

    let spec_arg = match ctx.get_arg_cell(1) {
        Some(arg) => arg.clone(),
        None => {
            return Err(PengError::CannotCallValue(
                "ffi:bind() expected spec object".to_string(),
            ));
        }
    };

    let spec = match get_bind_spec(ctx, &spec_arg, "bind") {
        Ok(spec) => spec,
        Err(e) => return Err(e),
    };

    let data = match get_symbol_data(id, &spec.symbol_name, "bind") {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    let callable = FfiCallableData {
        library_id: id,
        library_path: data.0,
        symbol_name: spec.symbol_name,
        symbol_addr: data.1,
        params: spec.params,
        result: spec.result,
    };

    let function_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
        call_bound(ctx, callable.clone())
    })));

    Ok(PengBindedCell::Mutable(PengCell::Reference(function_ptr)))
}

fn null(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    pointer_object(ctx, 0)
}

fn ptr(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let addr = match get_uint_arg(ctx, 0, "ptr") {
        Ok(addr) => addr,
        Err(e) => return Err(e),
    };

    pointer_object(ctx, addr)
}

fn addr(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let addr = match get_pointer_addr_arg(ctx, 0, "addr") {
        Ok(addr) => addr,
        Err(e) => return Err(e),
    };

    Ok(PengBindedCell::Mutable(PengCell::Uint(addr)))
}

fn is_null(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let addr = match get_pointer_addr_arg(ctx, 0, "is_null") {
        Ok(addr) => addr,
        Err(e) => return Err(e),
    };

    Ok(PengBindedCell::Mutable(PengCell::Bool(addr == 0)))
}

fn malloc(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let size = match get_uint_arg(ctx, 0, "malloc") {
        Ok(size) => size,
        Err(e) => return Err(e),
    };

    let ptr = unsafe { libc::malloc(size) };

    pointer_object(ctx, ptr as usize)
}

fn free(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let addr = match get_pointer_addr_arg(ctx, 0, "free") {
        Ok(addr) => addr,
        Err(e) => return Err(e),
    };

    if addr != 0 {
        unsafe {
            libc::free(addr as *mut c_void);
        }
    }

    utils::nil()
}

fn cstring(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let value = match get_string_arg(ctx, 0, "cstring") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let cstr = match CString::new(value) {
        Ok(cstr) => cstr,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "ffi:cstring() string cannot contain NUL byte".to_string(),
            ));
        }
    };

    let ptr = cstr.into_raw();

    pointer_object(ctx, ptr as usize)
}

fn free_cstring(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let addr = match get_pointer_addr_arg(ctx, 0, "free_cstring") {
        Ok(addr) => addr,
        Err(e) => return Err(e),
    };

    if addr != 0 {
        unsafe {
            let _ = CString::from_raw(addr as *mut c_char);
        }
    }

    utils::nil()
}

fn buffer(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let size = match get_uint_arg(ctx, 0, "buffer") {
        Ok(size) => size,
        Err(e) => return Err(e),
    };

    let id = NEXT_BUFFER_ID.fetch_add(1, Ordering::Relaxed);

    let mut locked = match buffers().lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "ffi:buffer() buffer registry lock failed".to_string(),
            ));
        }
    };

    locked.insert(
        id,
        FfiBufferEntry {
            bytes: vec![0u8; size],
        },
    );

    buffer_object(ctx, id, size)
}

fn library_object(
    ctx: &mut PengNativeFunctionCallContext,
    id: usize,
    path: String,
) -> Result<PengBindedCell, PengError> {
    let close_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(library_close)));

    let is_open_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(library_is_open)));

    let symbol_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(library_symbol)));

    let has_symbol_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        library_has_symbol,
    )));

    let bind_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(library_bind)));

    let kind_cell = utils::string_cell(ctx, "library".to_string());

    let path_cell = utils::string_cell(ctx, path);

    utils::new_object(
        ctx,
        vec![
            ("kind", kind_cell),
            ("id", PengBindedCell::Mutable(PengCell::Uint(id))),
            ("path", path_cell),
            (
                "close",
                PengBindedCell::Immutable(PengCell::Reference(close_ptr)),
            ),
            (
                "is_open",
                PengBindedCell::Immutable(PengCell::Reference(is_open_ptr)),
            ),
            (
                "symbol",
                PengBindedCell::Immutable(PengCell::Reference(symbol_ptr)),
            ),
            (
                "has_symbol",
                PengBindedCell::Immutable(PengCell::Reference(has_symbol_ptr)),
            ),
            (
                "bind",
                PengBindedCell::Immutable(PengCell::Reference(bind_ptr)),
            ),
        ],
    )
}

fn symbol_object(
    ctx: &mut PengNativeFunctionCallContext,
    library_id: usize,
    library_path: String,
    name: String,
    addr: usize,
) -> Result<PengBindedCell, PengError> {
    let kind_cell = utils::string_cell(ctx, "symbol".to_string());
    let library_path_cell = utils::string_cell(ctx, library_path);
    let name_cell = utils::string_cell(ctx, name);

    utils::new_object(
        ctx,
        vec![
            ("kind", kind_cell),
            (
                "library_id",
                PengBindedCell::Mutable(PengCell::Uint(library_id)),
            ),
            ("library_path", library_path_cell),
            ("name", name_cell),
            ("addr", PengBindedCell::Mutable(PengCell::Uint(addr))),
        ],
    )
}

fn pointer_object(
    ctx: &mut PengNativeFunctionCallContext,
    addr: usize,
) -> Result<PengBindedCell, PengError> {
    let is_null_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(pointer_is_null)));
    let addr_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(pointer_addr)));
    let offset_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(pointer_offset)));

    let kind_cell = utils::string_cell(ctx, "ptr".to_string());

    utils::new_object(
        ctx,
        vec![
            ("kind", kind_cell),
            ("addr", PengBindedCell::Mutable(PengCell::Uint(addr))),
            (
                "is_null",
                PengBindedCell::Immutable(PengCell::Reference(is_null_ptr)),
            ),
            (
                "addr_of",
                PengBindedCell::Immutable(PengCell::Reference(addr_ptr)),
            ),
            (
                "offset",
                PengBindedCell::Immutable(PengCell::Reference(offset_ptr)),
            ),
        ],
    )
}

fn buffer_object(
    ctx: &mut PengNativeFunctionCallContext,
    id: usize,
    size: usize,
) -> Result<PengBindedCell, PengError> {
    let len_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(buffer_len)));

    let ptr_fn = ctx.create_box(PengBox::Function(PengFunction::new_native(buffer_ptr)));

    let read_byte_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        buffer_read_byte,
    )));

    let write_byte_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        buffer_write_byte,
    )));

    let read_ptr_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(buffer_read_ptr)));

    let write_ptr_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        buffer_write_ptr,
    )));

    let free_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(buffer_free)));

    let kind_cell = utils::string_cell(ctx, "buffer".to_string());

    utils::new_object(
        ctx,
        vec![
            ("kind", kind_cell),
            ("id", PengBindedCell::Mutable(PengCell::Uint(id))),
            ("size", PengBindedCell::Mutable(PengCell::Uint(size))),
            (
                "len",
                PengBindedCell::Immutable(PengCell::Reference(len_ptr)),
            ),
            (
                "ptr",
                PengBindedCell::Immutable(PengCell::Reference(ptr_fn)),
            ),
            (
                "read_byte",
                PengBindedCell::Immutable(PengCell::Reference(read_byte_ptr)),
            ),
            (
                "write_byte",
                PengBindedCell::Immutable(PengCell::Reference(write_byte_ptr)),
            ),
            (
                "read_ptr",
                PengBindedCell::Immutable(PengCell::Reference(read_ptr_ptr)),
            ),
            (
                "write_ptr",
                PengBindedCell::Immutable(PengCell::Reference(write_ptr_ptr)),
            ),
            (
                "free",
                PengBindedCell::Immutable(PengCell::Reference(free_ptr)),
            ),
        ],
    )
}

fn library_close(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    close(ctx)
}

fn library_is_open(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    is_open(ctx)
}

fn library_symbol(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    symbol(ctx)
}

fn library_has_symbol(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    has_symbol(ctx)
}

fn library_bind(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    bind(ctx)
}

fn pointer_is_null(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    is_null(ctx)
}

fn pointer_addr(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    addr(ctx)
}

fn pointer_offset(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let base = match get_pointer_addr_arg(ctx, 0, "Pointer.offset") {
        Ok(base) => base,
        Err(e) => return Err(e),
    };

    let offset = match get_int_arg(ctx, 1, "Pointer.offset") {
        Ok(offset) => offset,
        Err(e) => return Err(e),
    };

    let result = if offset < 0 {
        base.wrapping_sub((-offset) as usize)
    } else {
        base.wrapping_add(offset as usize)
    };

    pointer_object(ctx, result)
}

fn buffer_len(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let id = match get_buffer_id_arg(ctx, 0, "Buffer.len") {
        Ok(id) => id,
        Err(e) => return Err(e),
    };

    let locked = match buffers().lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "ffi:Buffer.len() buffer registry lock failed".to_string(),
            ));
        }
    };

    match locked.get(&id) {
        Some(buffer) => Ok(PengBindedCell::Mutable(PengCell::Uint(buffer.bytes.len()))),
        None => Err(PengError::CannotCallValue(
            "ffi:Buffer.len() buffer is closed".to_string(),
        )),
    }
}

fn buffer_ptr(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let id = match get_buffer_id_arg(ctx, 0, "Buffer.ptr") {
        Ok(id) => id,
        Err(e) => return Err(e),
    };

    let mut locked = match buffers().lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "ffi:Buffer.ptr() buffer registry lock failed".to_string(),
            ));
        }
    };

    match locked.get_mut(&id) {
        Some(buffer) => pointer_object(ctx, buffer.bytes.as_mut_ptr() as usize),
        None => Err(PengError::CannotCallValue(
            "ffi:Buffer.ptr() buffer is closed".to_string(),
        )),
    }
}

fn buffer_read_byte(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let id = match get_buffer_id_arg(ctx, 0, "Buffer.read_byte") {
        Ok(id) => id,
        Err(e) => return Err(e),
    };

    let index = match get_uint_arg(ctx, 1, "Buffer.read_byte") {
        Ok(index) => index,
        Err(e) => return Err(e),
    };

    let locked = match buffers().lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "ffi:Buffer.read_byte() buffer registry lock failed".to_string(),
            ));
        }
    };

    match locked.get(&id) {
        Some(buffer) => match buffer.bytes.get(index) {
            Some(value) => Ok(PengBindedCell::Mutable(PengCell::Byte(*value))),
            None => Err(PengError::CannotCallValue(
                "ffi:Buffer.read_byte() index out of bounds".to_string(),
            )),
        },
        None => Err(PengError::CannotCallValue(
            "ffi:Buffer.read_byte() buffer is closed".to_string(),
        )),
    }
}

fn buffer_write_byte(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let id = match get_buffer_id_arg(ctx, 0, "Buffer.write_byte") {
        Ok(id) => id,
        Err(e) => return Err(e),
    };

    let index = match get_uint_arg(ctx, 1, "Buffer.write_byte") {
        Ok(index) => index,
        Err(e) => return Err(e),
    };

    let value = match get_byte_arg(ctx, 2, "Buffer.write_byte") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let mut locked = match buffers().lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "ffi:Buffer.write_byte() buffer registry lock failed".to_string(),
            ));
        }
    };

    match locked.get_mut(&id) {
        Some(buffer) => match buffer.bytes.get_mut(index) {
            Some(slot) => {
                *slot = value;
                utils::nil()
            }
            None => Err(PengError::CannotCallValue(
                "ffi:Buffer.write_byte() index out of bounds".to_string(),
            )),
        },
        None => Err(PengError::CannotCallValue(
            "ffi:Buffer.write_byte() buffer is closed".to_string(),
        )),
    }
}

fn buffer_free(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let id = match get_buffer_id_arg(ctx, 0, "Buffer.free") {
        Ok(id) => id,
        Err(e) => return Err(e),
    };

    let mut locked = match buffers().lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "ffi:Buffer.free() buffer registry lock failed".to_string(),
            ));
        }
    };

    locked.remove(&id);

    utils::nil()
}

#[derive(Debug, Clone)]
struct BindSpec {
    symbol_name: String,
    params: Vec<FfiTypeKind>,
    result: FfiTypeKind,
}

fn get_bind_spec(
    ctx: &mut PengNativeFunctionCallContext,
    cell: &PengBindedCell,
    function_name: &str,
) -> Result<BindSpec, PengError> {
    let fields = match utils::get_object_fields_from_cell(ctx, cell) {
        Ok(fields) => fields,
        Err(_) => {
            return Err(PengError::CannotCallValue(format!(
                "ffi:{}() expected spec object",
                function_name
            )));
        }
    };

    let symbol_name = match get_required_string_field(ctx, &fields, "name", function_name) {
        Ok(name) => name,
        Err(e) => return Err(e),
    };

    let params = match get_optional_type_vector_field(ctx, &fields, "params", function_name) {
        Ok(params) => params,
        Err(e) => return Err(e),
    };

    let result = match get_optional_type_field(ctx, &fields, "result", function_name) {
        Ok(Some(result)) => result,
        Ok(None) => FfiTypeKind::Void,
        Err(e) => return Err(e),
    };

    Ok(BindSpec {
        symbol_name,
        params,
        result,
    })
}

fn call_bound(
    ctx: &mut PengNativeFunctionCallContext,
    data: FfiCallableData,
) -> Result<PengBindedCell, PengError> {
    if !library_is_alive(data.library_id) {
        return Err(PengError::CannotCallValue(format!(
            "ffi:{}() library '{}' is closed",
            data.symbol_name, data.library_path
        )));
    }

    let mut keepalive_strings = Vec::new();
    let mut values = Vec::new();

    let mut index = 0usize;

    for kind in data.params.iter() {
        let arg_cell = match ctx.get_arg_cell(index) {
            Some(arg) => arg.clone(),
            None => {
                return Err(PengError::CannotCallValue(format!(
                    "ffi:{}() missing argument at index {}",
                    data.symbol_name, index
                )));
            }
        };

        let value = match prepare_arg(
            ctx,
            &arg_cell,
            kind,
            &mut keepalive_strings,
            &data.symbol_name,
            index,
        ) {
            Ok(value) => value,
            Err(e) => return Err(e),
        };

        values.push(value);
        index += 1;
    }

    if ctx.get_arg_cell(index).is_some() {
        return Err(PengError::CannotCallValue(format!(
            "ffi:{}() expected {} arguments",
            data.symbol_name,
            data.params.len()
        )));
    }

    let mut ffi_arg_values = Vec::new();

    for value in values.iter() {
        ffi_arg_values.push(arg_from_value(value));
    }

    let mut ffi_types = Vec::new();

    for kind in data.params.iter() {
        match kind_to_ffi_type(kind) {
            Ok(kind) => ffi_types.push(kind),
            Err(e) => return Err(PengError::CannotCallValue(e)),
        }
    }

    let result_type = match kind_to_ffi_type(&data.result) {
        Ok(kind) => kind,
        Err(e) => return Err(PengError::CannotCallValue(e)),
    };

    let cif = Cif::new(ffi_types.into_iter(), result_type);
    let code = CodePtr(data.symbol_addr as *mut c_void);

    call_cif(
        ctx,
        &cif,
        code,
        &ffi_arg_values,
        &data.result,
        &data.symbol_name,
    )
}

fn call_cif(
    ctx: &mut PengNativeFunctionCallContext,
    cif: &Cif,
    code: CodePtr,
    args: &[Arg],
    result: &FfiTypeKind,
    symbol_name: &str,
) -> Result<PengBindedCell, PengError> {
    match result {
        FfiTypeKind::Void => {
            let _: () = unsafe { cif.call(code, args) };
            utils::nil()
        }

        FfiTypeKind::Bool => {
            let value: u8 = unsafe { cif.call(code, args) };
            Ok(PengBindedCell::Mutable(PengCell::Bool(value != 0)))
        }

        FfiTypeKind::Char | FfiTypeKind::I8 => {
            let value: i8 = unsafe { cif.call(code, args) };
            Ok(PengBindedCell::Mutable(PengCell::Int(value as isize)))
        }

        FfiTypeKind::UChar | FfiTypeKind::U8 => {
            let value: u8 = unsafe { cif.call(code, args) };
            Ok(PengBindedCell::Mutable(PengCell::Byte(value)))
        }

        FfiTypeKind::I16 => {
            let value: i16 = unsafe { cif.call(code, args) };
            Ok(PengBindedCell::Mutable(PengCell::Int(value as isize)))
        }

        FfiTypeKind::U16 => {
            let value: u16 = unsafe { cif.call(code, args) };
            Ok(PengBindedCell::Mutable(PengCell::Uint(value as usize)))
        }

        FfiTypeKind::I32 | FfiTypeKind::Int => {
            let value: i32 = unsafe { cif.call(code, args) };
            Ok(PengBindedCell::Mutable(PengCell::Int(value as isize)))
        }

        FfiTypeKind::U32 | FfiTypeKind::UInt => {
            let value: u32 = unsafe { cif.call(code, args) };
            Ok(PengBindedCell::Mutable(PengCell::Uint(value as usize)))
        }

        FfiTypeKind::I64 | FfiTypeKind::LongLong => {
            let value: i64 = unsafe { cif.call(code, args) };
            Ok(PengBindedCell::Mutable(PengCell::Int(value as isize)))
        }

        FfiTypeKind::U64 | FfiTypeKind::ULongLong => {
            let value: u64 = unsafe { cif.call(code, args) };
            Ok(PengBindedCell::Mutable(PengCell::Uint(value as usize)))
        }

        FfiTypeKind::Long | FfiTypeKind::SSize | FfiTypeKind::ISize => {
            let value: isize = unsafe { cif.call(code, args) };
            Ok(PengBindedCell::Mutable(PengCell::Int(value)))
        }

        FfiTypeKind::ULong | FfiTypeKind::Size | FfiTypeKind::USize => {
            let value: usize = unsafe { cif.call(code, args) };
            Ok(PengBindedCell::Mutable(PengCell::Uint(value)))
        }

        FfiTypeKind::F32 => {
            let value: f32 = unsafe { cif.call(code, args) };
            Ok(PengBindedCell::Mutable(PengCell::Float32(value)))
        }

        FfiTypeKind::F64 => {
            let value: f64 = unsafe { cif.call(code, args) };
            Ok(PengBindedCell::Mutable(PengCell::Float64(value)))
        }

        FfiTypeKind::Ptr => {
            let value: *mut c_void = unsafe { cif.call(code, args) };
            pointer_object(ctx, value as usize)
        }

        FfiTypeKind::CString => {
            let value: *const c_char = unsafe { cif.call(code, args) };

            if value.is_null() {
                return utils::nil();
            }

            let cstr = unsafe { CStr::from_ptr(value) };
            match cstr.to_str() {
                Ok(value) => utils::string(ctx, value.to_string()),
                Err(_) => Err(PengError::CannotCallValue(format!(
                    "ffi:{}() returned invalid UTF-8 c_string",
                    symbol_name
                ))),
            }
        }
    }
}

fn prepare_arg(
    ctx: &mut PengNativeFunctionCallContext,
    cell: &PengBindedCell,
    kind: &FfiTypeKind,
    keepalive_strings: &mut Vec<CString>,
    symbol_name: &str,
    index: usize,
) -> Result<FfiArgValue, PengError> {
    match kind {
        FfiTypeKind::Void => Err(PengError::CannotCallValue(format!(
            "ffi:{}() c_void cannot be used as argument at index {}",
            symbol_name, index
        ))),

        FfiTypeKind::Bool => match cell.value() {
            PengCell::Bool(value) => {
                if *value {
                    Ok(FfiArgValue::Bool(1))
                } else {
                    Ok(FfiArgValue::Bool(0))
                }
            }
            _ => Err(expected_arg_error(symbol_name, index, "bool")),
        },

        FfiTypeKind::Char | FfiTypeKind::I8 => {
            let value = match cell_to_i64(cell) {
                Ok(value) => value,
                Err(_) => return Err(expected_arg_error(symbol_name, index, "number")),
            };

            if value < i8::MIN as i64 || value > i8::MAX as i64 {
                return Err(range_arg_error(symbol_name, index));
            }

            if *kind == FfiTypeKind::Char {
                Ok(FfiArgValue::Char(value as i8))
            } else {
                Ok(FfiArgValue::I8(value as i8))
            }
        }

        FfiTypeKind::UChar | FfiTypeKind::U8 => {
            let value = match cell_to_u64(cell) {
                Ok(value) => value,
                Err(_) => return Err(expected_arg_error(symbol_name, index, "number")),
            };

            if value > u8::MAX as u64 {
                return Err(range_arg_error(symbol_name, index));
            }

            if *kind == FfiTypeKind::UChar {
                Ok(FfiArgValue::UChar(value as u8))
            } else {
                Ok(FfiArgValue::U8(value as u8))
            }
        }

        FfiTypeKind::I16 => {
            let value = match cell_to_i64(cell) {
                Ok(value) => value,
                Err(_) => return Err(expected_arg_error(symbol_name, index, "number")),
            };

            if value < i16::MIN as i64 || value > i16::MAX as i64 {
                return Err(range_arg_error(symbol_name, index));
            }

            Ok(FfiArgValue::I16(value as i16))
        }

        FfiTypeKind::U16 => {
            let value = match cell_to_u64(cell) {
                Ok(value) => value,
                Err(_) => return Err(expected_arg_error(symbol_name, index, "number")),
            };

            if value > u16::MAX as u64 {
                return Err(range_arg_error(symbol_name, index));
            }

            Ok(FfiArgValue::U16(value as u16))
        }

        FfiTypeKind::I32 | FfiTypeKind::Int => {
            let value = match cell_to_i64(cell) {
                Ok(value) => value,
                Err(_) => return Err(expected_arg_error(symbol_name, index, "number")),
            };

            if value < i32::MIN as i64 || value > i32::MAX as i64 {
                return Err(range_arg_error(symbol_name, index));
            }

            if *kind == FfiTypeKind::Int {
                Ok(FfiArgValue::Int(value as i32))
            } else {
                Ok(FfiArgValue::I32(value as i32))
            }
        }

        FfiTypeKind::U32 | FfiTypeKind::UInt => {
            let value = match cell_to_u64(cell) {
                Ok(value) => value,
                Err(_) => return Err(expected_arg_error(symbol_name, index, "number")),
            };

            if value > u32::MAX as u64 {
                return Err(range_arg_error(symbol_name, index));
            }

            if *kind == FfiTypeKind::UInt {
                Ok(FfiArgValue::UInt(value as u32))
            } else {
                Ok(FfiArgValue::U32(value as u32))
            }
        }

        FfiTypeKind::I64 => {
            let value = match cell_to_i64(cell) {
                Ok(value) => value,
                Err(_) => return Err(expected_arg_error(symbol_name, index, "number")),
            };

            Ok(FfiArgValue::I64(value))
        }

        FfiTypeKind::U64 => {
            let value = match cell_to_u64(cell) {
                Ok(value) => value,
                Err(_) => return Err(expected_arg_error(symbol_name, index, "number")),
            };

            Ok(FfiArgValue::U64(value))
        }

        FfiTypeKind::Long => {
            let value = match cell_to_i64(cell) {
                Ok(value) => value,
                Err(_) => return Err(expected_arg_error(symbol_name, index, "number")),
            };

            Ok(FfiArgValue::Long(value as isize))
        }

        FfiTypeKind::ULong => {
            let value = match cell_to_u64(cell) {
                Ok(value) => value,
                Err(_) => return Err(expected_arg_error(symbol_name, index, "number")),
            };

            Ok(FfiArgValue::ULong(value as usize))
        }

        FfiTypeKind::LongLong => {
            let value = match cell_to_i64(cell) {
                Ok(value) => value,
                Err(_) => return Err(expected_arg_error(symbol_name, index, "number")),
            };

            Ok(FfiArgValue::LongLong(value))
        }

        FfiTypeKind::ULongLong => {
            let value = match cell_to_u64(cell) {
                Ok(value) => value,
                Err(_) => return Err(expected_arg_error(symbol_name, index, "number")),
            };

            Ok(FfiArgValue::ULongLong(value))
        }

        FfiTypeKind::F32 => match cell.value() {
            PengCell::Float32(value) => Ok(FfiArgValue::F32(*value)),
            PengCell::Float64(value) => Ok(FfiArgValue::F32(*value as f32)),
            PengCell::Int(value) => Ok(FfiArgValue::F32(*value as f32)),
            PengCell::Uint(value) => Ok(FfiArgValue::F32(*value as f32)),
            PengCell::Byte(value) => Ok(FfiArgValue::F32(*value as f32)),
            _ => Err(expected_arg_error(symbol_name, index, "number")),
        },

        FfiTypeKind::F64 => match cell.value() {
            PengCell::Float32(value) => Ok(FfiArgValue::F64(*value as f64)),
            PengCell::Float64(value) => Ok(FfiArgValue::F64(*value)),
            PengCell::Int(value) => Ok(FfiArgValue::F64(*value as f64)),
            PengCell::Uint(value) => Ok(FfiArgValue::F64(*value as f64)),
            PengCell::Byte(value) => Ok(FfiArgValue::F64(*value as f64)),
            _ => Err(expected_arg_error(symbol_name, index, "number")),
        },

        FfiTypeKind::Size | FfiTypeKind::USize => {
            let value = match cell_to_u64(cell) {
                Ok(value) => value,
                Err(_) => return Err(expected_arg_error(symbol_name, index, "number")),
            };

            if *kind == FfiTypeKind::Size {
                Ok(FfiArgValue::Size(value as usize))
            } else {
                Ok(FfiArgValue::USize(value as usize))
            }
        }

        FfiTypeKind::SSize | FfiTypeKind::ISize => {
            let value = match cell_to_i64(cell) {
                Ok(value) => value,
                Err(_) => return Err(expected_arg_error(symbol_name, index, "number")),
            };

            if *kind == FfiTypeKind::SSize {
                Ok(FfiArgValue::SSize(value as isize))
            } else {
                Ok(FfiArgValue::ISize(value as isize))
            }
        }

        FfiTypeKind::Ptr => {
            let addr = match cell_to_pointer_addr(ctx, cell) {
                Ok(addr) => addr,
                Err(_) => return Err(expected_arg_error(symbol_name, index, "pointer")),
            };

            Ok(FfiArgValue::Ptr(addr as *mut c_void))
        }

        FfiTypeKind::CString => match cell.value() {
            PengCell::Reference(_) => {
                let value = match utils::cell_to_string(ctx, cell) {
                    Ok(value) => value,
                    Err(_) => return Err(expected_arg_error(symbol_name, index, "string")),
                };

                let cstr = match CString::new(value) {
                    Ok(cstr) => cstr,
                    Err(_) => {
                        return Err(PengError::CannotCallValue(format!(
                            "ffi:{}() string argument at index {} contains NUL byte",
                            symbol_name, index
                        )));
                    }
                };

                let ptr = cstr.as_ptr() as *mut c_void;
                keepalive_strings.push(cstr);
                Ok(FfiArgValue::Ptr(ptr))
            }

            _ => {
                let addr = match cell_to_pointer_addr(ctx, cell) {
                    Ok(addr) => addr,
                    Err(_) => {
                        return Err(expected_arg_error(symbol_name, index, "string or pointer"));
                    }
                };

                Ok(FfiArgValue::Ptr(addr as *mut c_void))
            }
        },
    }
}

fn arg_from_value<'a>(value: &'a FfiArgValue) -> Arg<'a> {
    match value {
        FfiArgValue::Bool(value) => arg(value),

        FfiArgValue::Char(value) => arg(value),
        FfiArgValue::UChar(value) => arg(value),

        FfiArgValue::I8(value) => arg(value),
        FfiArgValue::U8(value) => arg(value),
        FfiArgValue::I16(value) => arg(value),
        FfiArgValue::U16(value) => arg(value),
        FfiArgValue::I32(value) => arg(value),
        FfiArgValue::U32(value) => arg(value),
        FfiArgValue::I64(value) => arg(value),
        FfiArgValue::U64(value) => arg(value),

        FfiArgValue::Int(value) => arg(value),
        FfiArgValue::UInt(value) => arg(value),
        FfiArgValue::Long(value) => arg(value),
        FfiArgValue::ULong(value) => arg(value),
        FfiArgValue::LongLong(value) => arg(value),
        FfiArgValue::ULongLong(value) => arg(value),

        FfiArgValue::F32(value) => arg(value),
        FfiArgValue::F64(value) => arg(value),

        FfiArgValue::Size(value) => arg(value),
        FfiArgValue::SSize(value) => arg(value),
        FfiArgValue::USize(value) => arg(value),
        FfiArgValue::ISize(value) => arg(value),

        FfiArgValue::Ptr(value) => arg(value),
    }
}

fn kind_to_ffi_type(kind: &FfiTypeKind) -> Result<Type, String> {
    match kind {
        FfiTypeKind::Void => Ok(Type::void()),
        FfiTypeKind::Bool => Ok(Type::u8()),

        FfiTypeKind::Char => Ok(Type::i8()),
        FfiTypeKind::UChar => Ok(Type::u8()),

        FfiTypeKind::I8 => Ok(Type::i8()),
        FfiTypeKind::U8 => Ok(Type::u8()),
        FfiTypeKind::I16 => Ok(Type::i16()),
        FfiTypeKind::U16 => Ok(Type::u16()),
        FfiTypeKind::I32 => Ok(Type::i32()),
        FfiTypeKind::U32 => Ok(Type::u32()),
        FfiTypeKind::I64 => Ok(Type::i64()),
        FfiTypeKind::U64 => Ok(Type::u64()),

        FfiTypeKind::Int => Ok(Type::c_int()),
        FfiTypeKind::UInt => Ok(Type::c_uint()),
        FfiTypeKind::Long => Ok(Type::c_long()),
        FfiTypeKind::ULong => Ok(Type::c_ulong()),
        FfiTypeKind::LongLong => Ok(Type::c_longlong()),
        FfiTypeKind::ULongLong => Ok(Type::c_ulonglong()),

        FfiTypeKind::F32 => Ok(Type::f32()),
        FfiTypeKind::F64 => Ok(Type::f64()),

        FfiTypeKind::Size => Ok(Type::usize()),
        FfiTypeKind::SSize => Ok(Type::isize()),
        FfiTypeKind::USize => Ok(Type::usize()),
        FfiTypeKind::ISize => Ok(Type::isize()),

        FfiTypeKind::Ptr => Ok(Type::pointer()),
        FfiTypeKind::CString => Ok(Type::pointer()),
    }
}

fn type_kind_from_name(name: &str) -> Result<FfiTypeKind, String> {
    match name {
        "c_void" => Ok(FfiTypeKind::Void),
        "c_bool" => Ok(FfiTypeKind::Bool),

        "c_char" => Ok(FfiTypeKind::Char),
        "c_uchar" => Ok(FfiTypeKind::UChar),

        "c_i8" => Ok(FfiTypeKind::I8),
        "c_u8" => Ok(FfiTypeKind::U8),
        "c_i16" => Ok(FfiTypeKind::I16),
        "c_u16" => Ok(FfiTypeKind::U16),
        "c_i32" => Ok(FfiTypeKind::I32),
        "c_u32" => Ok(FfiTypeKind::U32),
        "c_i64" => Ok(FfiTypeKind::I64),
        "c_u64" => Ok(FfiTypeKind::U64),

        "c_int" => Ok(FfiTypeKind::Int),
        "c_uint" => Ok(FfiTypeKind::UInt),
        "c_long" => Ok(FfiTypeKind::Long),
        "c_ulong" => Ok(FfiTypeKind::ULong),
        "c_longlong" => Ok(FfiTypeKind::LongLong),
        "c_ulonglong" => Ok(FfiTypeKind::ULongLong),

        "c_f32" => Ok(FfiTypeKind::F32),
        "c_f64" => Ok(FfiTypeKind::F64),

        "c_size" => Ok(FfiTypeKind::Size),
        "c_ssize" => Ok(FfiTypeKind::SSize),
        "c_usize" => Ok(FfiTypeKind::USize),
        "c_isize" => Ok(FfiTypeKind::ISize),

        "c_ptr" => Ok(FfiTypeKind::Ptr),
        "c_string" => Ok(FfiTypeKind::CString),

        _ => Err(format!("unknown ffi type '{}'", name)),
    }
}

fn type_kind_from_cell(
    ctx: &mut PengNativeFunctionCallContext,
    cell: &PengBindedCell,
    function_name: &str,
) -> Result<FfiTypeKind, PengError> {
    let name = match utils::cell_to_string(ctx, cell) {
        Ok(name) => name,
        Err(_) => {
            return Err(PengError::CannotCallValue(format!(
                "ffi:{}() expected ffi type",
                function_name
            )));
        }
    };

    match type_kind_from_name(&name) {
        Ok(kind) => Ok(kind),
        Err(e) => Err(PengError::CannotCallValue(format!(
            "ffi:{}() {}",
            function_name, e
        ))),
    }
}

fn get_symbol_data(
    id: usize,
    name: &str,
    function_name: &str,
) -> Result<(String, usize), PengError> {
    let locked = match libraries().lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(format!(
                "ffi:{}() library registry lock failed",
                function_name
            )));
        }
    };

    let entry = match locked.get(&id) {
        Some(entry) => entry,
        None => {
            return Err(PengError::CannotCallValue(format!(
                "ffi:{}() library is closed",
                function_name
            )));
        }
    };

    let symbol = unsafe { entry.library.get::<unsafe extern "C" fn()>(name.as_bytes()) };

    match symbol {
        Ok(symbol) => Ok((entry.path.clone(), *symbol as usize)),
        Err(e) => Err(PengError::CannotCallValue(format!(
            "ffi:{}() symbol '{}' not found: {}",
            function_name, name, e
        ))),
    }
}

fn library_is_alive(id: usize) -> bool {
    let locked = match libraries().lock() {
        Ok(locked) => locked,
        Err(_) => return false,
    };

    locked.contains_key(&id)
}

fn get_library_id_arg(
    ctx: &mut PengNativeFunctionCallContext,
    index: usize,
    function_name: &str,
) -> Result<usize, PengError> {
    let arg = match ctx.get_arg_cell(index) {
        Some(arg) => arg.clone(),
        None => {
            return Err(PengError::CannotCallValue(format!(
                "ffi:{}() expected library at index {}",
                function_name, index
            )));
        }
    };

    let fields = match utils::get_object_fields_from_cell(ctx, &arg) {
        Ok(fields) => fields,
        Err(_) => {
            return Err(PengError::CannotCallValue(format!(
                "ffi:{}() expected library object at index {}",
                function_name, index
            )));
        }
    };

    match get_required_uint_field(ctx, &fields, "id", function_name) {
        Ok(id) => Ok(id),
        Err(e) => Err(e),
    }
}

fn get_buffer_id_arg(
    ctx: &mut PengNativeFunctionCallContext,
    index: usize,
    function_name: &str,
) -> Result<usize, PengError> {
    let arg = match ctx.get_arg_cell(index) {
        Some(arg) => arg.clone(),
        None => {
            return Err(PengError::CannotCallValue(format!(
                "ffi:{}() expected buffer at index {}",
                function_name, index
            )));
        }
    };

    let fields = match utils::get_object_fields_from_cell(ctx, &arg) {
        Ok(fields) => fields,
        Err(_) => {
            return Err(PengError::CannotCallValue(format!(
                "ffi:{}() expected buffer object at index {}",
                function_name, index
            )));
        }
    };

    match get_required_uint_field(ctx, &fields, "id", function_name) {
        Ok(id) => Ok(id),
        Err(e) => Err(e),
    }
}

fn get_pointer_addr_arg(
    ctx: &mut PengNativeFunctionCallContext,
    index: usize,
    function_name: &str,
) -> Result<usize, PengError> {
    let arg = match ctx.get_arg_cell(index) {
        Some(arg) => arg.clone(),
        None => {
            return Err(PengError::CannotCallValue(format!(
                "ffi:{}() expected pointer at index {}",
                function_name, index
            )));
        }
    };

    match cell_to_pointer_addr(ctx, &arg) {
        Ok(addr) => Ok(addr),
        Err(e) => Err(e),
    }
}

fn cell_to_pointer_addr(
    ctx: &mut PengNativeFunctionCallContext,
    cell: &PengBindedCell,
) -> Result<usize, PengError> {
    match cell.value() {
        PengCell::Uint(value) => Ok(*value),

        PengCell::Int(value) => {
            if *value < 0 {
                return Err(PengError::CannotCallValue(
                    "expected non-negative pointer address".to_string(),
                ));
            }

            Ok(*value as usize)
        }

        PengCell::Nil => Ok(0),

        PengCell::Reference(_) => {
            let fields = match utils::get_object_fields_from_cell(ctx, cell) {
                Ok(fields) => fields,
                Err(_) => {
                    return Err(PengError::CannotCallValue(
                        "expected pointer object".to_string(),
                    ));
                }
            };

            let addr_cell = match utils::get_map_field(ctx, &fields, "addr") {
                Some(value) => value,
                None => {
                    return Err(PengError::CannotCallValue(
                        "expected pointer object".to_string(),
                    ));
                }
            };

            match utils::cell_to_uint(&addr_cell) {
                Ok(value) => Ok(value),
                Err(e) => Err(e),
            }
        }

        _ => Err(PengError::CannotCallValue(
            "expected pointer object".to_string(),
        )),
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
                "ffi:{}() missing string argument at index {}",
                function_name, index
            )));
        }
    };

    match utils::cell_to_string(ctx, arg) {
        Ok(value) => Ok(value),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "ffi:{}() expected string at index {}",
            function_name, index
        ))),
    }
}

fn get_uint_arg(
    ctx: &PengNativeFunctionCallContext,
    index: usize,
    function_name: &str,
) -> Result<usize, PengError> {
    let arg = match ctx.get_arg_cell(index) {
        Some(arg) => arg,
        None => {
            return Err(PengError::CannotCallValue(format!(
                "ffi:{}() missing uint argument at index {}",
                function_name, index
            )));
        }
    };

    match utils::cell_to_uint(arg) {
        Ok(value) => Ok(value),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "ffi:{}() expected uint at index {}",
            function_name, index
        ))),
    }
}

fn get_int_arg(
    ctx: &PengNativeFunctionCallContext,
    index: usize,
    function_name: &str,
) -> Result<isize, PengError> {
    let arg = match ctx.get_arg_cell(index) {
        Some(arg) => arg,
        None => {
            return Err(PengError::CannotCallValue(format!(
                "ffi:{}() missing int argument at index {}",
                function_name, index
            )));
        }
    };

    match arg.value() {
        PengCell::Int(value) => Ok(*value),
        PengCell::Uint(value) => Ok(*value as isize),
        PengCell::Byte(value) => Ok(*value as isize),
        _ => Err(PengError::CannotCallValue(format!(
            "ffi:{}() expected int at index {}",
            function_name, index
        ))),
    }
}

fn get_byte_arg(
    ctx: &PengNativeFunctionCallContext,
    index: usize,
    function_name: &str,
) -> Result<u8, PengError> {
    let arg = match ctx.get_arg_cell(index) {
        Some(arg) => arg,
        None => {
            return Err(PengError::CannotCallValue(format!(
                "ffi:{}() missing byte argument at index {}",
                function_name, index
            )));
        }
    };

    match arg.value() {
        PengCell::Byte(value) => Ok(*value),
        PengCell::Uint(value) => {
            if *value > u8::MAX as usize {
                return Err(PengError::CannotCallValue(format!(
                    "ffi:{}() byte argument out of range at index {}",
                    function_name, index
                )));
            }
            Ok(*value as u8)
        }
        PengCell::Int(value) => {
            if *value < 0 || *value > u8::MAX as isize {
                return Err(PengError::CannotCallValue(format!(
                    "ffi:{}() byte argument out of range at index {}",
                    function_name, index
                )));
            }
            Ok(*value as u8)
        }
        _ => Err(PengError::CannotCallValue(format!(
            "ffi:{}() expected byte at index {}",
            function_name, index
        ))),
    }
}

fn get_required_string_field(
    ctx: &mut PengNativeFunctionCallContext,
    fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    name: &str,
    function_name: &str,
) -> Result<String, PengError> {
    let cell = match utils::get_map_field(ctx, fields, name) {
        Some(cell) => cell,
        None => {
            return Err(PengError::CannotCallValue(format!(
                "ffi:{}() missing '{}' field",
                function_name, name
            )));
        }
    };

    match utils::cell_to_string(ctx, &cell) {
        Ok(value) => Ok(value),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "ffi:{}() field '{}' must be string",
            function_name, name
        ))),
    }
}

fn get_optional_string_field(
    ctx: &mut PengNativeFunctionCallContext,
    fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    name: &str,
    function_name: &str,
) -> Result<Option<String>, PengError> {
    match utils::get_map_field(ctx, fields, name) {
        Some(cell) => match cell.value() {
            PengCell::Nil => Ok(None),
            _ => match utils::cell_to_string(ctx, &cell) {
                Ok(value) => Ok(Some(value)),
                Err(_) => Err(PengError::CannotCallValue(format!(
                    "ffi:{}() field '{}' must be string or nil",
                    function_name, name
                ))),
            },
        },
        None => Ok(None),
    }
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
                "ffi:{}() missing '{}' field",
                function_name, name
            )));
        }
    };

    match utils::cell_to_uint(&cell) {
        Ok(value) => Ok(value),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "ffi:{}() field '{}' must be uint",
            function_name, name
        ))),
    }
}

fn get_optional_type_field(
    ctx: &mut PengNativeFunctionCallContext,
    fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    name: &str,
    function_name: &str,
) -> Result<Option<FfiTypeKind>, PengError> {
    match utils::get_map_field(ctx, fields, name) {
        Some(cell) => match cell.value() {
            PengCell::Nil => Ok(None),
            _ => match type_kind_from_cell(ctx, &cell, function_name) {
                Ok(kind) => Ok(Some(kind)),
                Err(e) => Err(e),
            },
        },
        None => Ok(None),
    }
}

fn get_optional_type_vector_field(
    ctx: &mut PengNativeFunctionCallContext,
    fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    name: &str,
    function_name: &str,
) -> Result<Vec<FfiTypeKind>, PengError> {
    let cell = match utils::get_map_field(ctx, fields, name) {
        Some(cell) => cell,
        None => return Ok(Vec::new()),
    };

    match cell.value() {
        PengCell::Nil => Ok(Vec::new()),

        PengCell::Reference(ptr) => {
            let values = match ctx.get_value(*ptr) {
                Some(PengValue::Box(PengBox::Vector(vector))) => vector.values.clone(),

                Some(_) => {
                    return Err(PengError::CannotCallValue(format!(
                        "ffi:{}() field '{}' must be vector of ffi types",
                        function_name, name
                    )));
                }

                None => return Err(PengError::HeapValueNotFound(*ptr)),
            };

            let mut output = Vec::new();

            for item in values.iter() {
                let kind = match type_kind_from_cell(ctx, item, function_name) {
                    Ok(kind) => kind,
                    Err(e) => return Err(e),
                };

                output.push(kind);
            }

            Ok(output)
        }

        _ => Err(PengError::CannotCallValue(format!(
            "ffi:{}() field '{}' must be vector of ffi types",
            function_name, name
        ))),
    }
}

fn cell_to_i64(cell: &PengBindedCell) -> Result<i64, ()> {
    match cell.value() {
        PengCell::Int(value) => Ok(*value as i64),
        PengCell::Uint(value) => Ok(*value as i64),
        PengCell::Byte(value) => Ok(*value as i64),
        _ => Err(()),
    }
}

fn cell_to_u64(cell: &PengBindedCell) -> Result<u64, ()> {
    match cell.value() {
        PengCell::Uint(value) => Ok(*value as u64),
        PengCell::Int(value) => {
            if *value < 0 {
                return Err(());
            }
            Ok(*value as u64)
        }
        PengCell::Byte(value) => Ok(*value as u64),
        _ => Err(()),
    }
}

fn expected_arg_error(symbol_name: &str, index: usize, expected: &str) -> PengError {
    PengError::CannotCallValue(format!(
        "ffi:{}() expected {} at index {}",
        symbol_name, expected, index
    ))
}

fn range_arg_error(symbol_name: &str, index: usize) -> PengError {
    PengError::CannotCallValue(format!(
        "ffi:{}() argument at index {} is out of range",
        symbol_name, index
    ))
}

fn ptr_size(_ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    Ok(PengBindedCell::Mutable(PengCell::Uint(
        std::mem::size_of::<usize>(),
    )))
}

fn buffer_read_ptr(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let id = match get_buffer_id_arg(ctx, 0, "Buffer.read_ptr") {
        Ok(id) => id,
        Err(e) => return Err(e),
    };

    let index = match get_uint_arg(ctx, 1, "Buffer.read_ptr") {
        Ok(index) => index,
        Err(e) => return Err(e),
    };

    let locked = match buffers().lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "ffi:Buffer.read_ptr() buffer registry lock failed".to_string(),
            ));
        }
    };

    let buffer = match locked.get(&id) {
        Some(buffer) => buffer,
        None => {
            return Err(PengError::CannotCallValue(
                "ffi:Buffer.read_ptr() buffer is closed".to_string(),
            ));
        }
    };

    let ptr_size = std::mem::size_of::<usize>();

    if index + ptr_size > buffer.bytes.len() {
        return Err(PengError::CannotCallValue(
            "ffi:Buffer.read_ptr() index out of bounds".to_string(),
        ));
    }

    let mut bytes = [0u8; std::mem::size_of::<usize>()];

    for i in 0..ptr_size {
        bytes[i] = buffer.bytes[index + i];
    }

    let addr = usize::from_ne_bytes(bytes);

    pointer_object(ctx, addr)
}

fn buffer_write_ptr(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let id = match get_buffer_id_arg(ctx, 0, "Buffer.write_ptr") {
        Ok(id) => id,
        Err(e) => return Err(e),
    };

    let index = match get_uint_arg(ctx, 1, "Buffer.write_ptr") {
        Ok(index) => index,
        Err(e) => return Err(e),
    };

    let ptr_arg = match ctx.get_arg_cell(2) {
        Some(arg) => arg.clone(),
        None => {
            return Err(PengError::CannotCallValue(
                "ffi:Buffer.write_ptr() missing pointer argument".to_string(),
            ));
        }
    };

    let addr = match cell_to_pointer_addr(ctx, &ptr_arg) {
        Ok(addr) => addr,
        Err(e) => return Err(e),
    };

    let mut locked = match buffers().lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "ffi:Buffer.write_ptr() buffer registry lock failed".to_string(),
            ));
        }
    };

    let buffer = match locked.get_mut(&id) {
        Some(buffer) => buffer,
        None => {
            return Err(PengError::CannotCallValue(
                "ffi:Buffer.write_ptr() buffer is closed".to_string(),
            ));
        }
    };

    let ptr_size = std::mem::size_of::<usize>();

    if index + ptr_size > buffer.bytes.len() {
        return Err(PengError::CannotCallValue(
            "ffi:Buffer.write_ptr() index out of bounds".to_string(),
        ));
    }

    let bytes = addr.to_ne_bytes();

    for i in 0..ptr_size {
        buffer.bytes[index + i] = bytes[i];
    }

    utils::nil()
}
