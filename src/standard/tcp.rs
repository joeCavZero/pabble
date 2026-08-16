use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use penguin::prelude::*;

use super::utils;

#[derive(Debug, Clone)]
struct TcpClientData {
    host: String,
    port: usize,
    timeout: Option<usize>,
    read_timeout: Option<usize>,
    write_timeout: Option<usize>,
    nodelay: bool,
    non_blocking: bool,
}

#[derive(Debug, Clone)]
struct TcpServerData {
    host: String,
    port: usize,
    read_timeout: Option<usize>,
    write_timeout: Option<usize>,
    nodelay: bool,
    non_blocking: bool,
    non_blocking_connections: bool,
}

#[derive(Debug, Clone)]
struct TcpConnectionHandle {
    stream: Arc<Mutex<Option<TcpStream>>>,
}

#[derive(Debug, Clone)]
struct TcpServerHandle {
    listener: Arc<Mutex<Option<TcpListener>>>,
    read_timeout: Option<usize>,
    write_timeout: Option<usize>,
    nodelay: bool,
    non_blocking_connections: bool,
}

#[derive(Debug, Clone)]
enum TcpTaskValue {
    Connection(TcpConnectionHandle),
    Server(TcpServerHandle),
}

#[derive(Debug, Clone)]
enum TcpTaskState {
    Running,
    Finished(Result<TcpTaskValue, String>),
}

pub fn setup(peng: &mut PengEnv) -> PengUnit {
    let mut module = PengUnit::library();

    let client_type = client_type_value(peng);
    module
        .register_immutable_global(peng, "Client", client_type)
        .unwrap();

    let server_type = server_type_value(peng);
    module
        .register_immutable_global(peng, "Server", server_type)
        .unwrap();

    module
        .register_immutable_native_function(peng, "connect", connect)
        .unwrap();

    module
        .register_immutable_native_function(peng, "connect_async", connect_async)
        .unwrap();

    module
        .register_immutable_native_function(peng, "listen", listen)
        .unwrap();

    module
        .register_immutable_native_function(peng, "listen_async", listen_async)
        .unwrap();

    module
}

fn client_type_value(peng: &mut PengEnv) -> PengValue {
    let mut fields = HashMap::new();

    let connect_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(client_connect),
    )));

    let connect_async_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(client_connect_async),
    )));

    fields.insert(
        peng.ensure_pooled_name_ptr("host".to_string()),
        utils::string_binded_cell_from_env(peng, "127.0.0.1".to_string()),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("port".to_string()),
        PengBindedCell::Mutable(PengCell::Uint(0)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("timeout".to_string()),
        PengBindedCell::Mutable(PengCell::Uint(30000)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("read_timeout".to_string()),
        PengBindedCell::Mutable(PengCell::Nil),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("write_timeout".to_string()),
        PengBindedCell::Mutable(PengCell::Nil),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("nodelay".to_string()),
        PengBindedCell::Mutable(PengCell::Bool(false)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("non_blocking".to_string()),
        PengBindedCell::Mutable(PengCell::Bool(false)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("connect".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(connect_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("connect_async".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(connect_async_ptr)),
    );

    PengValue::Box(PengBox::Type(PengType::Custom(PengCustomType { fields })))
}

fn server_type_value(peng: &mut PengEnv) -> PengValue {
    let mut fields = HashMap::new();

    let listen_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(server_listen),
    )));

    let listen_async_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(server_listen_async),
    )));

    fields.insert(
        peng.ensure_pooled_name_ptr("host".to_string()),
        utils::string_binded_cell_from_env(peng, "0.0.0.0".to_string()),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("port".to_string()),
        PengBindedCell::Mutable(PengCell::Uint(0)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("read_timeout".to_string()),
        PengBindedCell::Mutable(PengCell::Nil),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("write_timeout".to_string()),
        PengBindedCell::Mutable(PengCell::Nil),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("nodelay".to_string()),
        PengBindedCell::Mutable(PengCell::Bool(false)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("non_blocking".to_string()),
        PengBindedCell::Mutable(PengCell::Bool(false)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("non_blocking_connections".to_string()),
        PengBindedCell::Mutable(PengCell::Bool(false)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("listen".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(listen_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("listen_async".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(listen_async_ptr)),
    );

    PengValue::Box(PengBox::Type(PengType::Custom(PengCustomType { fields })))
}

fn connect(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let client = match get_client_data_from_args(ctx, "connect") {
        Ok(client) => client,
        Err(e) => return Err(e),
    };

    let handle = match execute_connect_data(client, "connect") {
        Ok(handle) => handle,
        Err(e) => return Err(PengError::CannotCallValue(e)),
    };

    connection_object(ctx, handle)
}

fn connect_async(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let client = match get_client_data_from_args(ctx, "connect_async") {
        Ok(client) => client,
        Err(e) => return Err(e),
    };

    let state = Arc::new(Mutex::new(TcpTaskState::Running));
    let thread_state = state.clone();

    thread::spawn(move || {
        let result = match execute_connect_data(client, "connect_async") {
            Ok(handle) => Ok(TcpTaskValue::Connection(handle)),
            Err(e) => Err(e),
        };

        match thread_state.lock() {
            Ok(mut locked) => {
                *locked = TcpTaskState::Finished(result);
            }

            Err(_) => {}
        }
    });

    task_object(ctx, state)
}

fn listen(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let server = match get_server_data_from_args(ctx, "listen") {
        Ok(server) => server,
        Err(e) => return Err(e),
    };

    let handle = match execute_listen_data(server, "listen") {
        Ok(handle) => handle,
        Err(e) => return Err(PengError::CannotCallValue(e)),
    };

    server_handle_object(ctx, handle)
}

fn listen_async(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let server = match get_server_data_from_args(ctx, "listen_async") {
        Ok(server) => server,
        Err(e) => return Err(e),
    };

    let state = Arc::new(Mutex::new(TcpTaskState::Running));
    let thread_state = state.clone();

    thread::spawn(move || {
        let result = match execute_listen_data(server, "listen_async") {
            Ok(handle) => Ok(TcpTaskValue::Server(handle)),
            Err(e) => Err(e),
        };

        match thread_state.lock() {
            Ok(mut locked) => {
                *locked = TcpTaskState::Finished(result);
            }

            Err(_) => {}
        }
    });

    task_object(ctx, state)
}

fn client_connect(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    connect(ctx)
}

fn client_connect_async(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    connect_async(ctx)
}

fn server_listen(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    listen(ctx)
}

fn server_listen_async(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    listen_async(ctx)
}

fn execute_connect_data(
    client: TcpClientData,
    function_name: &str,
) -> Result<TcpConnectionHandle, String> {
    if client.host.is_empty() {
        return Err(format!(
            "tcp:{}() client.host cannot be empty",
            function_name
        ));
    }

    if client.port > u16::MAX as usize {
        return Err(format!(
            "tcp:{}() client.port is out of range",
            function_name
        ));
    }

    let address = format!("{}:{}", client.host, client.port);

    let stream = match client.timeout {
        Some(timeout) => {
            let socket_addr = match resolve_first_addr(&address, function_name) {
                Ok(addr) => addr,
                Err(e) => return Err(e),
            };

            TcpStream::connect_timeout(
                &socket_addr,
                Duration::from_millis(utils::usize_to_u64_saturating(timeout)),
            )
        }

        None => TcpStream::connect(&address),
    };

    let stream = match stream {
        Ok(stream) => stream,
        Err(e) => {
            return Err(format!(
                "tcp:{}() failed to connect to '{}': {}",
                function_name, address, e
            ));
        }
    };

    match apply_stream_config(
        &stream,
        client.read_timeout.or(client.timeout),
        client.write_timeout.or(client.timeout),
        client.nodelay,
        client.non_blocking,
        function_name,
    ) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    Ok(TcpConnectionHandle {
        stream: Arc::new(Mutex::new(Some(stream))),
    })
}

fn execute_listen_data(
    server: TcpServerData,
    function_name: &str,
) -> Result<TcpServerHandle, String> {
    if server.host.is_empty() {
        return Err(format!(
            "tcp:{}() server.host cannot be empty",
            function_name
        ));
    }

    if server.port > u16::MAX as usize {
        return Err(format!(
            "tcp:{}() server.port is out of range",
            function_name
        ));
    }

    let address = format!("{}:{}", server.host, server.port);

    let listener = match TcpListener::bind(&address) {
        Ok(listener) => listener,
        Err(e) => {
            return Err(format!(
                "tcp:{}() failed to listen on '{}': {}",
                function_name, address, e
            ));
        }
    };

    match listener.set_nonblocking(server.non_blocking) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!(
                "tcp:{}() failed to set non_blocking: {}",
                function_name, e
            ));
        }
    }

    Ok(TcpServerHandle {
        listener: Arc::new(Mutex::new(Some(listener))),
        read_timeout: server.read_timeout,
        write_timeout: server.write_timeout,
        nodelay: server.nodelay,
        non_blocking_connections: server.non_blocking_connections,
    })
}

fn connection_object(
    ctx: &mut PengNativeFunctionCallContext,
    handle: TcpConnectionHandle,
) -> Result<PengBindedCell, PengError> {
    let read_handle = handle.clone();
    let read_line_handle = handle.clone();
    let read_all_handle = handle.clone();
    let read_byte_handle = handle.clone();
    let write_handle = handle.clone();
    let write_line_handle = handle.clone();
    let flush_handle = handle.clone();
    let close_handle = handle.clone();
    let is_closed_handle = handle.clone();
    let peer_addr_handle = handle.clone();
    let local_addr_handle = handle.clone();
    let set_read_timeout_handle = handle.clone();
    let set_write_timeout_handle = handle.clone();
    let set_non_blocking_handle = handle.clone();

    let read_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
        connection_read(ctx, read_handle.clone())
    })));

    let read_line_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
        connection_read_line(ctx, read_line_handle.clone())
    })));

    let read_all_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
        connection_read_all(ctx, read_all_handle.clone())
    })));

    let read_byte_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
        connection_read_byte(ctx, read_byte_handle.clone())
    })));

    let write_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
        connection_write(ctx, write_handle.clone())
    })));

    let write_line_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
        connection_write_line(ctx, write_line_handle.clone())
    })));

    let flush_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
        connection_flush(ctx, flush_handle.clone())
    })));

    let close_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
        connection_close(ctx, close_handle.clone())
    })));

    let is_closed_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
        connection_is_closed(ctx, is_closed_handle.clone())
    })));

    let peer_addr_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
        connection_peer_addr(ctx, peer_addr_handle.clone())
    })));

    let local_addr_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
        connection_local_addr(ctx, local_addr_handle.clone())
    })));

    let set_read_timeout_ptr =
        ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
            connection_set_read_timeout(ctx, set_read_timeout_handle.clone())
        })));

    let set_write_timeout_ptr =
        ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
            connection_set_write_timeout(ctx, set_write_timeout_handle.clone())
        })));

    let set_non_blocking_ptr =
        ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
            connection_set_non_blocking(ctx, set_non_blocking_handle.clone())
        })));

    utils::new_object(
        ctx,
        vec![
            (
                "read",
                PengBindedCell::Immutable(PengCell::Reference(read_ptr)),
            ),
            (
                "read_line",
                PengBindedCell::Immutable(PengCell::Reference(read_line_ptr)),
            ),
            (
                "read_all",
                PengBindedCell::Immutable(PengCell::Reference(read_all_ptr)),
            ),
            (
                "read_byte",
                PengBindedCell::Immutable(PengCell::Reference(read_byte_ptr)),
            ),
            (
                "write",
                PengBindedCell::Immutable(PengCell::Reference(write_ptr)),
            ),
            (
                "write_line",
                PengBindedCell::Immutable(PengCell::Reference(write_line_ptr)),
            ),
            (
                "flush",
                PengBindedCell::Immutable(PengCell::Reference(flush_ptr)),
            ),
            (
                "close",
                PengBindedCell::Immutable(PengCell::Reference(close_ptr)),
            ),
            (
                "is_closed",
                PengBindedCell::Immutable(PengCell::Reference(is_closed_ptr)),
            ),
            (
                "peer_addr",
                PengBindedCell::Immutable(PengCell::Reference(peer_addr_ptr)),
            ),
            (
                "local_addr",
                PengBindedCell::Immutable(PengCell::Reference(local_addr_ptr)),
            ),
            (
                "set_read_timeout",
                PengBindedCell::Immutable(PengCell::Reference(set_read_timeout_ptr)),
            ),
            (
                "set_write_timeout",
                PengBindedCell::Immutable(PengCell::Reference(set_write_timeout_ptr)),
            ),
            (
                "set_non_blocking",
                PengBindedCell::Immutable(PengCell::Reference(set_non_blocking_ptr)),
            ),
        ],
    )
}

fn server_handle_object(
    ctx: &mut PengNativeFunctionCallContext,
    handle: TcpServerHandle,
) -> Result<PengBindedCell, PengError> {
    let accept_handle = handle.clone();
    let accept_async_handle = handle.clone();
    let close_handle = handle.clone();
    let is_closed_handle = handle.clone();
    let local_addr_handle = handle.clone();

    let accept_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
        server_accept(ctx, accept_handle.clone())
    })));

    let accept_async_ptr =
        ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
            server_accept_async(ctx, accept_async_handle.clone())
        })));

    let close_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
        server_close(ctx, close_handle.clone())
    })));

    let is_closed_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
        server_is_closed(ctx, is_closed_handle.clone())
    })));

    let local_addr_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
        server_local_addr(ctx, local_addr_handle.clone())
    })));

    utils::new_object(
        ctx,
        vec![
            (
                "accept",
                PengBindedCell::Immutable(PengCell::Reference(accept_ptr)),
            ),
            (
                "accept_async",
                PengBindedCell::Immutable(PengCell::Reference(accept_async_ptr)),
            ),
            (
                "close",
                PengBindedCell::Immutable(PengCell::Reference(close_ptr)),
            ),
            (
                "is_closed",
                PengBindedCell::Immutable(PengCell::Reference(is_closed_ptr)),
            ),
            (
                "local_addr",
                PengBindedCell::Immutable(PengCell::Reference(local_addr_ptr)),
            ),
        ],
    )
}

fn task_object(
    ctx: &mut PengNativeFunctionCallContext,
    state: Arc<Mutex<TcpTaskState>>,
) -> Result<PengBindedCell, PengError> {
    let is_finished_state = state.clone();
    let get_state = state.clone();
    let error_state = state.clone();

    let is_finished_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
        task_is_finished(ctx, is_finished_state.clone())
    })));

    let get_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
        task_get(ctx, get_state.clone())
    })));

    let error_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(move |ctx| {
        task_error(ctx, error_state.clone())
    })));

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

fn connection_read(
    ctx: &mut PengNativeFunctionCallContext,
    handle: TcpConnectionHandle,
) -> Result<PengBindedCell, PengError> {
    let size = match utils::get_uint_arg(ctx, 1) {
        Ok(size) => size,
        Err(e) => return Err(e),
    };

    if size == 0 {
        return utils::string(ctx, String::new());
    }

    let mut buffer = vec![0u8; size];

    let mut locked = match handle.stream.lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "tcp:Connection.read() stream lock failed".to_string(),
            ));
        }
    };

    let read_result = match locked.as_mut() {
        Some(stream) => stream.read(&mut buffer),
        None => {
            return utils::nil();
        }
    };

    match read_result {
        Ok(0) => {
            *locked = None;
            utils::nil()
        }

        Ok(n) => {
            buffer.truncate(n);
            utils::string(ctx, String::from_utf8_lossy(&buffer).to_string())
        }

        Err(e) => {
            if utils::is_temporary_read_error(&e) {
                return utils::nil();
            }

            Err(PengError::CannotCallValue(format!(
                "tcp:Connection.read() failed: {}",
                e
            )))
        }
    }
}

fn connection_read_line(
    ctx: &mut PengNativeFunctionCallContext,
    handle: TcpConnectionHandle,
) -> Result<PengBindedCell, PengError> {
    let mut bytes = Vec::new();
    let mut close_after = false;

    {
        let mut locked = match handle.stream.lock() {
            Ok(locked) => locked,
            Err(_) => {
                return Err(PengError::CannotCallValue(
                    "tcp:Connection.read_line() stream lock failed".to_string(),
                ));
            }
        };

        let stream = match locked.as_mut() {
            Some(stream) => stream,
            None => return utils::nil(),
        };

        loop {
            let mut one = [0u8; 1];

            match stream.read(&mut one) {
                Ok(0) => {
                    close_after = true;
                    break;
                }

                Ok(_) => {
                    if one[0] == b'\n' {
                        break;
                    }

                    bytes.push(one[0]);
                }

                Err(e) => {
                    if utils::is_temporary_read_error(&e) {
                        break;
                    }

                    return Err(PengError::CannotCallValue(format!(
                        "tcp:Connection.read_line() failed: {}",
                        e
                    )));
                }
            }
        }

        if close_after {
            *locked = None;
        }
    }

    if bytes.len() == 0 && close_after {
        return utils::nil();
    }

    if bytes.last() == Some(&b'\r') {
        bytes.pop();
    }

    utils::string(ctx, String::from_utf8_lossy(&bytes).to_string())
}

fn connection_read_all(
    ctx: &mut PengNativeFunctionCallContext,
    handle: TcpConnectionHandle,
) -> Result<PengBindedCell, PengError> {
    let mut output = String::new();

    let mut locked = match handle.stream.lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "tcp:Connection.read_all() stream lock failed".to_string(),
            ));
        }
    };

    let read_result = match locked.as_mut() {
        Some(stream) => stream.read_to_string(&mut output),
        None => return utils::nil(),
    };

    match read_result {
        Ok(_) => {
            *locked = None;
            utils::string(ctx, output)
        }

        Err(e) => {
            if utils::is_temporary_read_error(&e) {
                return utils::string(ctx, output);
            }

            Err(PengError::CannotCallValue(format!(
                "tcp:Connection.read_all() failed: {}",
                e
            )))
        }
    }
}

fn connection_read_byte(
    _ctx: &mut PengNativeFunctionCallContext,
    handle: TcpConnectionHandle,
) -> Result<PengBindedCell, PengError> {
    let mut buffer = [0u8; 1];

    let mut locked = match handle.stream.lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "tcp:Connection.read_byte() stream lock failed".to_string(),
            ));
        }
    };

    let read_result = match locked.as_mut() {
        Some(stream) => stream.read(&mut buffer),
        None => return utils::nil(),
    };

    match read_result {
        Ok(0) => {
            *locked = None;
            utils::nil()
        }

        Ok(_) => Ok(PengBindedCell::Mutable(PengCell::Byte(buffer[0]))),

        Err(e) => {
            if utils::is_temporary_read_error(&e) {
                return utils::nil();
            }

            Err(PengError::CannotCallValue(format!(
                "tcp:Connection.read_byte() failed: {}",
                e
            )))
        }
    }
}

fn connection_write(
    ctx: &mut PengNativeFunctionCallContext,
    handle: TcpConnectionHandle,
) -> Result<PengBindedCell, PengError> {
    let data = match utils::get_string_arg(ctx, 1) {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    write_to_connection(handle, data.as_bytes(), "Connection.write")
}

fn connection_write_line(
    ctx: &mut PengNativeFunctionCallContext,
    handle: TcpConnectionHandle,
) -> Result<PengBindedCell, PengError> {
    let mut data = match utils::get_string_arg(ctx, 1) {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    data.push('\n');

    write_to_connection(handle, data.as_bytes(), "Connection.write_line")
}

fn connection_flush(
    _ctx: &mut PengNativeFunctionCallContext,
    handle: TcpConnectionHandle,
) -> Result<PengBindedCell, PengError> {
    let mut locked = match handle.stream.lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "tcp:Connection.flush() stream lock failed".to_string(),
            ));
        }
    };

    let stream = match locked.as_mut() {
        Some(stream) => stream,
        None => return utils::nil(),
    };

    match stream.flush() {
        Ok(_) => utils::nil(),
        Err(e) => Err(PengError::CannotCallValue(format!(
            "tcp:Connection.flush() failed: {}",
            e
        ))),
    }
}

fn connection_close(
    _ctx: &mut PengNativeFunctionCallContext,
    handle: TcpConnectionHandle,
) -> Result<PengBindedCell, PengError> {
    let mut locked = match handle.stream.lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "tcp:Connection.close() stream lock failed".to_string(),
            ));
        }
    };

    *locked = None;

    utils::nil()
}

fn connection_is_closed(
    _ctx: &mut PengNativeFunctionCallContext,
    handle: TcpConnectionHandle,
) -> Result<PengBindedCell, PengError> {
    let locked = match handle.stream.lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "tcp:Connection.is_closed() stream lock failed".to_string(),
            ));
        }
    };

    Ok(PengBindedCell::Mutable(PengCell::Bool(locked.is_none())))
}

fn connection_peer_addr(
    ctx: &mut PengNativeFunctionCallContext,
    handle: TcpConnectionHandle,
) -> Result<PengBindedCell, PengError> {
    let locked = match handle.stream.lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "tcp:Connection.peer_addr() stream lock failed".to_string(),
            ));
        }
    };

    let stream = match locked.as_ref() {
        Some(stream) => stream,
        None => return utils::nil(),
    };

    match stream.peer_addr() {
        Ok(addr) => utils::string(ctx, addr.to_string()),
        Err(e) => Err(PengError::CannotCallValue(format!(
            "tcp:Connection.peer_addr() failed: {}",
            e
        ))),
    }
}

fn connection_local_addr(
    ctx: &mut PengNativeFunctionCallContext,
    handle: TcpConnectionHandle,
) -> Result<PengBindedCell, PengError> {
    let locked = match handle.stream.lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "tcp:Connection.local_addr() stream lock failed".to_string(),
            ));
        }
    };

    let stream = match locked.as_ref() {
        Some(stream) => stream,
        None => return utils::nil(),
    };

    match stream.local_addr() {
        Ok(addr) => utils::string(ctx, addr.to_string()),
        Err(e) => Err(PengError::CannotCallValue(format!(
            "tcp:Connection.local_addr() failed: {}",
            e
        ))),
    }
}

fn connection_set_read_timeout(
    ctx: &mut PengNativeFunctionCallContext,
    handle: TcpConnectionHandle,
) -> Result<PengBindedCell, PengError> {
    let timeout = match get_optional_uint_arg(ctx, 1, "Connection.set_read_timeout") {
        Ok(timeout) => timeout,
        Err(e) => return Err(e),
    };

    let locked = match handle.stream.lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "tcp:Connection.set_read_timeout() stream lock failed".to_string(),
            ));
        }
    };

    let stream = match locked.as_ref() {
        Some(stream) => stream,
        None => return utils::nil(),
    };

    match stream.set_read_timeout(utils::timeout_to_duration(timeout)) {
        Ok(_) => utils::nil(),
        Err(e) => Err(PengError::CannotCallValue(format!(
            "tcp:Connection.set_read_timeout() failed: {}",
            e
        ))),
    }
}

fn connection_set_write_timeout(
    ctx: &mut PengNativeFunctionCallContext,
    handle: TcpConnectionHandle,
) -> Result<PengBindedCell, PengError> {
    let timeout = match get_optional_uint_arg(ctx, 1, "Connection.set_write_timeout") {
        Ok(timeout) => timeout,
        Err(e) => return Err(e),
    };

    let locked = match handle.stream.lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "tcp:Connection.set_write_timeout() stream lock failed".to_string(),
            ));
        }
    };

    let stream = match locked.as_ref() {
        Some(stream) => stream,
        None => return utils::nil(),
    };

    match stream.set_write_timeout(utils::timeout_to_duration(timeout)) {
        Ok(_) => utils::nil(),
        Err(e) => Err(PengError::CannotCallValue(format!(
            "tcp:Connection.set_write_timeout() failed: {}",
            e
        ))),
    }
}

fn connection_set_non_blocking(
    ctx: &mut PengNativeFunctionCallContext,
    handle: TcpConnectionHandle,
) -> Result<PengBindedCell, PengError> {
    let value = match utils::get_bool_arg(ctx, 1) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let locked = match handle.stream.lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "tcp:Connection.set_non_blocking() stream lock failed".to_string(),
            ));
        }
    };

    let stream = match locked.as_ref() {
        Some(stream) => stream,
        None => return utils::nil(),
    };

    match stream.set_nonblocking(value) {
        Ok(_) => utils::nil(),
        Err(e) => Err(PengError::CannotCallValue(format!(
            "tcp:Connection.set_non_blocking() failed: {}",
            e
        ))),
    }
}

fn server_accept(
    ctx: &mut PengNativeFunctionCallContext,
    handle: TcpServerHandle,
) -> Result<PengBindedCell, PengError> {
    let mut locked = match handle.listener.lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "tcp:Server.accept() listener lock failed".to_string(),
            ));
        }
    };

    let listener = match locked.as_mut() {
        Some(listener) => listener,
        None => return utils::nil(),
    };

    match accept_from_listener(listener, &handle, "Server.accept") {
        Ok(Some(connection)) => connection_object(ctx, connection),
        Ok(None) => utils::nil(),
        Err(e) => Err(PengError::CannotCallValue(e)),
    }
}

fn server_accept_async(
    ctx: &mut PengNativeFunctionCallContext,
    handle: TcpServerHandle,
) -> Result<PengBindedCell, PengError> {
    let listener = {
        let locked = match handle.listener.lock() {
            Ok(locked) => locked,
            Err(_) => {
                return Err(PengError::CannotCallValue(
                    "tcp:Server.accept_async() listener lock failed".to_string(),
                ));
            }
        };

        match locked.as_ref() {
            Some(listener) => match listener.try_clone() {
                Ok(listener) => listener,
                Err(e) => {
                    return Err(PengError::CannotCallValue(format!(
                        "tcp:Server.accept_async() failed to clone listener: {}",
                        e
                    )));
                }
            },

            None => return utils::nil(),
        }
    };

    let state = Arc::new(Mutex::new(TcpTaskState::Running));
    let thread_state = state.clone();

    thread::spawn(move || {
        let result = match accept_from_listener(&listener, &handle, "Server.accept_async") {
            Ok(Some(connection)) => Ok(TcpTaskValue::Connection(connection)),
            Ok(None) => Err("tcp:Server.accept_async() no pending connection".to_string()),
            Err(e) => Err(e),
        };

        match thread_state.lock() {
            Ok(mut locked) => {
                *locked = TcpTaskState::Finished(result);
            }

            Err(_) => {}
        }
    });

    task_object(ctx, state)
}

fn server_close(
    _ctx: &mut PengNativeFunctionCallContext,
    handle: TcpServerHandle,
) -> Result<PengBindedCell, PengError> {
    let mut locked = match handle.listener.lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "tcp:Server.close() listener lock failed".to_string(),
            ));
        }
    };

    *locked = None;

    utils::nil()
}

fn server_is_closed(
    _ctx: &mut PengNativeFunctionCallContext,
    handle: TcpServerHandle,
) -> Result<PengBindedCell, PengError> {
    let locked = match handle.listener.lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "tcp:Server.is_closed() listener lock failed".to_string(),
            ));
        }
    };

    Ok(PengBindedCell::Mutable(PengCell::Bool(locked.is_none())))
}

fn server_local_addr(
    ctx: &mut PengNativeFunctionCallContext,
    handle: TcpServerHandle,
) -> Result<PengBindedCell, PengError> {
    let locked = match handle.listener.lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "tcp:Server.local_addr() listener lock failed".to_string(),
            ));
        }
    };

    let listener = match locked.as_ref() {
        Some(listener) => listener,
        None => return utils::nil(),
    };

    match listener.local_addr() {
        Ok(addr) => utils::string(ctx, addr.to_string()),
        Err(e) => Err(PengError::CannotCallValue(format!(
            "tcp:Server.local_addr() failed: {}",
            e
        ))),
    }
}

fn task_is_finished(
    _ctx: &mut PengNativeFunctionCallContext,
    state: Arc<Mutex<TcpTaskState>>,
) -> Result<PengBindedCell, PengError> {
    match state.lock() {
        Ok(locked) => match &*locked {
            TcpTaskState::Running => Ok(PengBindedCell::Mutable(PengCell::Bool(false))),
            TcpTaskState::Finished(_) => Ok(PengBindedCell::Mutable(PengCell::Bool(true))),
        },

        Err(_) => Err(PengError::CannotCallValue(
            "tcp task lock failed".to_string(),
        )),
    }
}

fn task_get(
    ctx: &mut PengNativeFunctionCallContext,
    state: Arc<Mutex<TcpTaskState>>,
) -> Result<PengBindedCell, PengError> {
    let result = match state.lock() {
        Ok(locked) => match &*locked {
            TcpTaskState::Running => return utils::nil(),
            TcpTaskState::Finished(result) => result.clone(),
        },

        Err(_) => {
            return Err(PengError::CannotCallValue(
                "tcp task lock failed".to_string(),
            ));
        }
    };

    match result {
        Ok(TcpTaskValue::Connection(connection)) => connection_object(ctx, connection),
        Ok(TcpTaskValue::Server(server)) => server_handle_object(ctx, server),
        Err(e) => Err(PengError::CannotCallValue(e)),
    }
}

fn task_error(
    ctx: &mut PengNativeFunctionCallContext,
    state: Arc<Mutex<TcpTaskState>>,
) -> Result<PengBindedCell, PengError> {
    match state.lock() {
        Ok(locked) => match &*locked {
            TcpTaskState::Running => utils::nil(),

            TcpTaskState::Finished(result) => match result {
                Ok(_) => utils::nil(),
                Err(e) => utils::string(ctx, e.clone()),
            },
        },

        Err(_) => Err(PengError::CannotCallValue(
            "tcp task lock failed".to_string(),
        )),
    }
}

fn accept_from_listener(
    listener: &TcpListener,
    handle: &TcpServerHandle,
    function_name: &str,
) -> Result<Option<TcpConnectionHandle>, String> {
    let (stream, _) = match listener.accept() {
        Ok(value) => value,
        Err(e) => {
            if e.kind() == ErrorKind::WouldBlock {
                return Ok(None);
            }

            return Err(format!("tcp:{}() failed: {}", function_name, e));
        }
    };

    match apply_stream_config(
        &stream,
        handle.read_timeout,
        handle.write_timeout,
        handle.nodelay,
        handle.non_blocking_connections,
        function_name,
    ) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    Ok(Some(TcpConnectionHandle {
        stream: Arc::new(Mutex::new(Some(stream))),
    }))
}

fn write_to_connection(
    handle: TcpConnectionHandle,
    data: &[u8],
    function_name: &str,
) -> Result<PengBindedCell, PengError> {
    let mut locked = match handle.stream.lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(format!(
                "tcp:{}() stream lock failed",
                function_name
            )));
        }
    };

    let stream = match locked.as_mut() {
        Some(stream) => stream,
        None => return utils::nil(),
    };

    match stream.write_all(data) {
        Ok(_) => Ok(PengBindedCell::Mutable(PengCell::Uint(data.len()))),
        Err(e) => Err(PengError::CannotCallValue(format!(
            "tcp:{}() failed: {}",
            function_name, e
        ))),
    }
}

fn get_client_data_from_args(
    ctx: &mut PengNativeFunctionCallContext,
    function_name: &str,
) -> Result<TcpClientData, PengError> {
    let first_arg = match ctx.get_arg_cell(0) {
        Some(arg) => arg.clone(),
        None => {
            return Err(PengError::CannotCallValue(format!(
                "tcp:{}() expected Client or host",
                function_name
            )));
        }
    };

    match utils::cell_to_string(ctx, &first_arg) {
        Ok(host) => {
            let port = match utils::get_uint_arg(ctx, 1) {
                Ok(port) => port,
                Err(e) => return Err(e),
            };

            Ok(TcpClientData {
                host,
                port,
                timeout: Some(30000),
                read_timeout: None,
                write_timeout: None,
                nodelay: false,
                non_blocking: false,
            })
        }

        Err(_) => {
            let fields = match utils::get_object_fields_from_cell(ctx, &first_arg) {
                Ok(fields) => fields,
                Err(e) => return Err(e),
            };

            client_data_from_fields(ctx, &fields, function_name)
        }
    }
}

fn client_data_from_fields(
    ctx: &mut PengNativeFunctionCallContext,
    fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    function_name: &str,
) -> Result<TcpClientData, PengError> {
    let host = match get_optional_string_field(ctx, fields, "host", function_name) {
        Ok(Some(host)) => host,
        Ok(None) => "127.0.0.1".to_string(),
        Err(e) => return Err(e),
    };

    let port = match get_optional_uint_field(ctx, fields, "port", function_name) {
        Ok(Some(port)) => port,
        Ok(None) => 0,
        Err(e) => return Err(e),
    };

    let timeout = match get_optional_uint_field(ctx, fields, "timeout", function_name) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let read_timeout = match get_optional_uint_field(ctx, fields, "read_timeout", function_name) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let write_timeout = match get_optional_uint_field(ctx, fields, "write_timeout", function_name) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let nodelay = match get_optional_bool_field(ctx, fields, "nodelay", function_name) {
        Ok(Some(value)) => value,
        Ok(None) => false,
        Err(e) => return Err(e),
    };

    let non_blocking = match get_optional_bool_field(ctx, fields, "non_blocking", function_name) {
        Ok(Some(value)) => value,
        Ok(None) => false,
        Err(e) => return Err(e),
    };

    Ok(TcpClientData {
        host,
        port,
        timeout,
        read_timeout,
        write_timeout,
        nodelay,
        non_blocking,
    })
}

fn get_server_data_from_args(
    ctx: &mut PengNativeFunctionCallContext,
    function_name: &str,
) -> Result<TcpServerData, PengError> {
    let first_arg = match ctx.get_arg_cell(0) {
        Some(arg) => arg.clone(),
        None => {
            return Err(PengError::CannotCallValue(format!(
                "tcp:{}() expected Server or host",
                function_name
            )));
        }
    };

    match utils::cell_to_string(ctx, &first_arg) {
        Ok(host) => {
            let port = match utils::get_uint_arg(ctx, 1) {
                Ok(port) => port,
                Err(e) => return Err(e),
            };

            Ok(TcpServerData {
                host,
                port,
                read_timeout: None,
                write_timeout: None,
                nodelay: false,
                non_blocking: false,
                non_blocking_connections: false,
            })
        }

        Err(_) => {
            let fields = match utils::get_object_fields_from_cell(ctx, &first_arg) {
                Ok(fields) => fields,
                Err(e) => return Err(e),
            };

            server_data_from_fields(ctx, &fields, function_name)
        }
    }
}

fn server_data_from_fields(
    ctx: &mut PengNativeFunctionCallContext,
    fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    function_name: &str,
) -> Result<TcpServerData, PengError> {
    let host = match get_optional_string_field(ctx, fields, "host", function_name) {
        Ok(Some(host)) => host,
        Ok(None) => "0.0.0.0".to_string(),
        Err(e) => return Err(e),
    };

    let port = match get_optional_uint_field(ctx, fields, "port", function_name) {
        Ok(Some(port)) => port,
        Ok(None) => 0,
        Err(e) => return Err(e),
    };

    let read_timeout = match get_optional_uint_field(ctx, fields, "read_timeout", function_name) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let write_timeout = match get_optional_uint_field(ctx, fields, "write_timeout", function_name) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let nodelay = match get_optional_bool_field(ctx, fields, "nodelay", function_name) {
        Ok(Some(value)) => value,
        Ok(None) => false,
        Err(e) => return Err(e),
    };

    let non_blocking = match get_optional_bool_field(ctx, fields, "non_blocking", function_name) {
        Ok(Some(value)) => value,
        Ok(None) => false,
        Err(e) => return Err(e),
    };

    let non_blocking_connections =
        match get_optional_bool_field(ctx, fields, "non_blocking_connections", function_name) {
            Ok(Some(value)) => value,
            Ok(None) => false,
            Err(e) => return Err(e),
        };

    Ok(TcpServerData {
        host,
        port,
        read_timeout,
        write_timeout,
        nodelay,
        non_blocking,
        non_blocking_connections,
    })
}

fn get_optional_string_field(
    ctx: &mut PengNativeFunctionCallContext,
    fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    name: &str,
    function_name: &str,
) -> Result<Option<String>, PengError> {
    match utils::get_map_field(ctx, fields, name) {
        Some(value) => match value.value() {
            PengCell::Nil => Ok(None),
            _ => match utils::cell_to_string(ctx, &value) {
                Ok(value) => Ok(Some(value)),
                Err(_) => Err(PengError::CannotCallValue(format!(
                    "tcp:{}() field '{}' must be string or nil",
                    function_name, name
                ))),
            },
        },

        None => Ok(None),
    }
}

fn get_optional_uint_field(
    ctx: &mut PengNativeFunctionCallContext,
    fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    name: &str,
    function_name: &str,
) -> Result<Option<usize>, PengError> {
    match utils::get_map_field(ctx, fields, name) {
        Some(value) => match value.value() {
            PengCell::Nil => Ok(None),
            _ => match utils::cell_to_uint(&value) {
                Ok(value) => Ok(Some(value)),
                Err(_) => Err(PengError::CannotCallValue(format!(
                    "tcp:{}() field '{}' must be uint, int, byte or nil",
                    function_name, name
                ))),
            },
        },

        None => Ok(None),
    }
}

fn get_optional_bool_field(
    ctx: &mut PengNativeFunctionCallContext,
    fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    name: &str,
    function_name: &str,
) -> Result<Option<bool>, PengError> {
    match utils::get_map_field(ctx, fields, name) {
        Some(value) => match value.value() {
            PengCell::Nil => Ok(None),
            _ => match utils::cell_to_bool(&value) {
                Ok(value) => Ok(Some(value)),
                Err(_) => Err(PengError::CannotCallValue(format!(
                    "tcp:{}() field '{}' must be bool or nil",
                    function_name, name
                ))),
            },
        },

        None => Ok(None),
    }
}

fn get_optional_uint_arg(
    ctx: &PengNativeFunctionCallContext,
    index: usize,
    function_name: &str,
) -> Result<Option<usize>, PengError> {
    let arg = match ctx.get_arg_cell(index) {
        Some(arg) => arg,
        None => return Ok(None),
    };

    match arg.value() {
        PengCell::Nil => Ok(None),
        _ => match utils::cell_to_uint(arg) {
            Ok(value) => Ok(Some(value)),
            Err(_) => Err(PengError::CannotCallValue(format!(
                "tcp:{}() expected uint or nil at index {}",
                function_name, index
            ))),
        },
    }
}

fn resolve_first_addr(address: &str, function_name: &str) -> Result<std::net::SocketAddr, String> {
    let mut addrs = match address.to_socket_addrs() {
        Ok(addrs) => addrs,
        Err(e) => {
            return Err(format!(
                "tcp:{}() failed to resolve '{}': {}",
                function_name, address, e
            ));
        }
    };

    match addrs.next() {
        Some(addr) => Ok(addr),
        None => Err(format!(
            "tcp:{}() failed to resolve '{}'",
            function_name, address
        )),
    }
}

fn apply_stream_config(
    stream: &TcpStream,
    read_timeout: Option<usize>,
    write_timeout: Option<usize>,
    nodelay: bool,
    non_blocking: bool,
    function_name: &str,
) -> Result<(), String> {
    match stream.set_read_timeout(utils::timeout_to_duration(read_timeout)) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!(
                "tcp:{}() failed to set read_timeout: {}",
                function_name, e
            ));
        }
    }

    match stream.set_write_timeout(utils::timeout_to_duration(write_timeout)) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!(
                "tcp:{}() failed to set write_timeout: {}",
                function_name, e
            ));
        }
    }

    match stream.set_nodelay(nodelay) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!(
                "tcp:{}() failed to set nodelay: {}",
                function_name, e
            ));
        }
    }

    match stream.set_nonblocking(non_blocking) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!(
                "tcp:{}() failed to set non_blocking: {}",
                function_name, e
            ));
        }
    }

    Ok(())
}
