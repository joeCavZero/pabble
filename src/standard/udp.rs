use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;

use penguin::prelude::*;

use super::utils;

#[derive(Debug, Clone)]
struct UdpSocketData {
    host: String,
    port: usize,
    read_timeout: Option<usize>,
    write_timeout: Option<usize>,
    timeout: Option<usize>,
    broadcast: bool,
    non_blocking: bool,
}

#[derive(Debug, Clone)]
struct UdpPacketData {
    data: String,
    host: String,
    port: usize,
    addr: String,
    size: usize,
}

#[derive(Debug, Clone)]
struct UdpSocketHandle {
    socket: Arc<Mutex<Option<UdpSocket>>>,
}

#[derive(Debug, Clone)]
enum UdpTaskValue {
    Socket(UdpSocketHandle),
    Packet(UdpPacketData),
    Data(String),
    Size(usize),
    Nil,
}

#[derive(Debug, Clone)]
enum UdpTaskState {
    Running,
    Finished(Result<UdpTaskValue, String>),
}

pub fn setup(peng: &mut PengEnv) -> PengUnit {
    let mut module = PengUnit::library();

    let socket_type = socket_type_value(peng);
    module.register_immutable_global(peng, "Socket", socket_type).unwrap();

    module.register_immutable_native_function(peng, "bind", bind).unwrap();

    module.register_immutable_native_function(peng, "bind_async", bind_async).unwrap();

    module
}

fn socket_type_value(peng: &mut PengEnv) -> PengValue {
    let mut fields = HashMap::new();

    let bind_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(socket_bind),
    )));

    let bind_async_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(socket_bind_async),
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
        peng.ensure_pooled_name_ptr("timeout".to_string()),
        PengBindedCell::Mutable(PengCell::Nil),
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
        peng.ensure_pooled_name_ptr("broadcast".to_string()),
        PengBindedCell::Mutable(PengCell::Bool(false)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("non_blocking".to_string()),
        PengBindedCell::Mutable(PengCell::Bool(false)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("bind".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(bind_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("bind_async".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(bind_async_ptr)),
    );

    PengValue::Box(PengBox::Type(PengType::Custom(PengCustomType { fields })))
}

fn bind(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let data = match get_socket_data_from_args(ctx, "bind") {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    let handle = match execute_bind_data(data, "bind") {
        Ok(handle) => handle,
        Err(e) => return Err(PengError::CannotCallValue(e)),
    };

    socket_handle_object(ctx, handle)
}

fn bind_async(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let data = match get_socket_data_from_args(ctx, "bind_async") {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    let state = Arc::new(Mutex::new(UdpTaskState::Running));
    let thread_state = state.clone();

    thread::spawn(move || {
        let result = match execute_bind_data(data, "bind_async") {
            Ok(handle) => Ok(UdpTaskValue::Socket(handle)),
            Err(e) => Err(e),
        };

        match thread_state.lock() {
            Ok(mut locked) => {
                *locked = UdpTaskState::Finished(result);
            }

            Err(_) => {}
        }
    });

    task_object(ctx, state)
}

fn socket_bind(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    bind(ctx)
}

fn socket_bind_async(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    bind_async(ctx)
}

fn execute_bind_data(data: UdpSocketData, function_name: &str) -> Result<UdpSocketHandle, String> {
    if data.host.is_empty() {
        return Err(format!("udp:{}() socket.host cannot be empty", function_name));
    }

    if data.port > u16::MAX as usize {
        return Err(format!(
            "udp:{}() socket.port is out of range",
            function_name
        ));
    }

    let address = format!("{}:{}", data.host, data.port);

    let socket = match UdpSocket::bind(&address) {
        Ok(socket) => socket,
        Err(e) => {
            return Err(format!(
                "udp:{}() failed to bind '{}': {}",
                function_name, address, e
            ));
        }
    };

    match apply_socket_config(&socket, &data, function_name) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    Ok(UdpSocketHandle {
        socket: Arc::new(Mutex::new(Some(socket))),
    })
}

fn socket_handle_object(
    ctx: &mut PengNativeFunctionCallContext,
    handle: UdpSocketHandle,
) -> Result<PengBindedCell, PengError> {
    let connect_handle = handle.clone();
    let connect_async_handle = handle.clone();
    let send_to_handle = handle.clone();
    let send_to_async_handle = handle.clone();
    let recv_from_handle = handle.clone();
    let recv_from_async_handle = handle.clone();
    let send_handle = handle.clone();
    let send_async_handle = handle.clone();
    let recv_handle = handle.clone();
    let recv_async_handle = handle.clone();
    let close_handle = handle.clone();
    let is_closed_handle = handle.clone();
    let local_addr_handle = handle.clone();
    let peer_addr_handle = handle.clone();
    let set_broadcast_handle = handle.clone();
    let set_read_timeout_handle = handle.clone();
    let set_write_timeout_handle = handle.clone();
    let set_non_blocking_handle = handle.clone();

    let connect_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| socket_connect(ctx, connect_handle.clone()),
    )));

    let connect_async_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| socket_connect_async(ctx, connect_async_handle.clone()),
    )));

    let send_to_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| socket_send_to(ctx, send_to_handle.clone()),
    )));

    let send_to_async_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| socket_send_to_async(ctx, send_to_async_handle.clone()),
    )));

    let recv_from_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| socket_recv_from(ctx, recv_from_handle.clone()),
    )));

    let recv_from_async_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| socket_recv_from_async(ctx, recv_from_async_handle.clone()),
    )));

    let send_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| socket_send(ctx, send_handle.clone()),
    )));

    let send_async_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| socket_send_async(ctx, send_async_handle.clone()),
    )));

    let recv_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| socket_recv(ctx, recv_handle.clone()),
    )));

    let recv_async_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| socket_recv_async(ctx, recv_async_handle.clone()),
    )));

    let close_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| socket_close(ctx, close_handle.clone()),
    )));

    let is_closed_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| socket_is_closed(ctx, is_closed_handle.clone()),
    )));

    let local_addr_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| socket_local_addr(ctx, local_addr_handle.clone()),
    )));

    let peer_addr_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| socket_peer_addr(ctx, peer_addr_handle.clone()),
    )));

    let set_broadcast_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| socket_set_broadcast(ctx, set_broadcast_handle.clone()),
    )));

    let set_read_timeout_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| socket_set_read_timeout(ctx, set_read_timeout_handle.clone()),
    )));

    let set_write_timeout_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| socket_set_write_timeout(ctx, set_write_timeout_handle.clone()),
    )));

    let set_non_blocking_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(
        move |ctx| socket_set_non_blocking(ctx, set_non_blocking_handle.clone()),
    )));

    utils::new_object(
        ctx,
        vec![
            (
                "connect",
                PengBindedCell::Immutable(PengCell::Reference(connect_ptr)),
            ),
            (
                "connect_async",
                PengBindedCell::Immutable(PengCell::Reference(connect_async_ptr)),
            ),
            (
                "send_to",
                PengBindedCell::Immutable(PengCell::Reference(send_to_ptr)),
            ),
            (
                "send_to_async",
                PengBindedCell::Immutable(PengCell::Reference(send_to_async_ptr)),
            ),
            (
                "recv_from",
                PengBindedCell::Immutable(PengCell::Reference(recv_from_ptr)),
            ),
            (
                "recv_from_async",
                PengBindedCell::Immutable(PengCell::Reference(recv_from_async_ptr)),
            ),
            (
                "send",
                PengBindedCell::Immutable(PengCell::Reference(send_ptr)),
            ),
            (
                "send_async",
                PengBindedCell::Immutable(PengCell::Reference(send_async_ptr)),
            ),
            (
                "recv",
                PengBindedCell::Immutable(PengCell::Reference(recv_ptr)),
            ),
            (
                "recv_async",
                PengBindedCell::Immutable(PengCell::Reference(recv_async_ptr)),
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
            (
                "peer_addr",
                PengBindedCell::Immutable(PengCell::Reference(peer_addr_ptr)),
            ),
            (
                "set_broadcast",
                PengBindedCell::Immutable(PengCell::Reference(set_broadcast_ptr)),
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

fn task_object(ctx: &mut PengNativeFunctionCallContext, state: Arc<Mutex<UdpTaskState>>) -> Result<PengBindedCell, PengError> {
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
            ("get", PengBindedCell::Immutable(PengCell::Reference(get_ptr))),
            (
                "error",
                PengBindedCell::Immutable(PengCell::Reference(error_ptr)),
            ),
        ],
    )
}

fn socket_connect(ctx: &mut PengNativeFunctionCallContext, handle: UdpSocketHandle) -> Result<PengBindedCell, PengError> {
    let host = match utils::get_string_arg(ctx, 1) {
        Ok(host) => host,
        Err(e) => return Err(e),
    };

    let port = match utils::get_uint_arg(ctx, 2) {
        Ok(port) => port,
        Err(e) => return Err(e),
    };

    match execute_socket_connect(handle, host, port, "Socket.connect") {
        Ok(_) => utils::nil(),
        Err(e) => Err(PengError::CannotCallValue(e)),
    }
}

fn socket_connect_async(
    ctx: &mut PengNativeFunctionCallContext,
    handle: UdpSocketHandle,
) -> Result<PengBindedCell, PengError> {
    let host = match utils::get_string_arg(ctx, 1) {
        Ok(host) => host,
        Err(e) => return Err(e),
    };

    let port = match utils::get_uint_arg(ctx, 2) {
        Ok(port) => port,
        Err(e) => return Err(e),
    };

    let state = Arc::new(Mutex::new(UdpTaskState::Running));
    let thread_state = state.clone();

    thread::spawn(move || {
        let result = match execute_socket_connect(handle, host, port, "Socket.connect_async") {
            Ok(_) => Ok(UdpTaskValue::Nil),
            Err(e) => Err(e),
        };

        match thread_state.lock() {
            Ok(mut locked) => {
                *locked = UdpTaskState::Finished(result);
            }

            Err(_) => {}
        }
    });

    task_object(ctx, state)
}

fn socket_send_to(ctx: &mut PengNativeFunctionCallContext, handle: UdpSocketHandle) -> Result<PengBindedCell, PengError> {
    let data = match utils::get_string_arg(ctx, 1) {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    let host = match utils::get_string_arg(ctx, 2) {
        Ok(host) => host,
        Err(e) => return Err(e),
    };

    let port = match utils::get_uint_arg(ctx, 3) {
        Ok(port) => port,
        Err(e) => return Err(e),
    };

    match execute_socket_send_to(handle, data, host, port, "Socket.send_to") {
        Ok(size) => Ok(PengBindedCell::Mutable(PengCell::Uint(size))),
        Err(e) => Err(PengError::CannotCallValue(e)),
    }
}

fn socket_send_to_async(
    ctx: &mut PengNativeFunctionCallContext,
    handle: UdpSocketHandle,
) -> Result<PengBindedCell, PengError> {
    let data = match utils::get_string_arg(ctx, 1) {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    let host = match utils::get_string_arg(ctx, 2) {
        Ok(host) => host,
        Err(e) => return Err(e),
    };

    let port = match utils::get_uint_arg(ctx, 3) {
        Ok(port) => port,
        Err(e) => return Err(e),
    };

    let state = Arc::new(Mutex::new(UdpTaskState::Running));
    let thread_state = state.clone();

    thread::spawn(move || {
        let result = match execute_socket_send_to(handle, data, host, port, "Socket.send_to_async") {
            Ok(size) => Ok(UdpTaskValue::Size(size)),
            Err(e) => Err(e),
        };

        match thread_state.lock() {
            Ok(mut locked) => {
                *locked = UdpTaskState::Finished(result);
            }

            Err(_) => {}
        }
    });

    task_object(ctx, state)
}

fn socket_recv_from(
    ctx: &mut PengNativeFunctionCallContext,
    handle: UdpSocketHandle,
) -> Result<PengBindedCell, PengError> {
    let size = match utils::get_uint_arg(ctx, 1) {
        Ok(size) => size,
        Err(e) => return Err(e),
    };

    let socket = match clone_socket_for_action(&handle, "Socket.recv_from") {
        Ok(Some(socket)) => socket,
        Ok(None) => return utils::nil(),
        Err(e) => return Err(PengError::CannotCallValue(e)),
    };

    match execute_recv_from_socket(&socket, size, "Socket.recv_from") {
        Ok(Some(packet)) => packet_object(ctx, packet),
        Ok(None) => utils::nil(),
        Err(e) => Err(PengError::CannotCallValue(e)),
    }
}

fn socket_recv_from_async(
    ctx: &mut PengNativeFunctionCallContext,
    handle: UdpSocketHandle,
) -> Result<PengBindedCell, PengError> {
    let size = match utils::get_uint_arg(ctx, 1) {
        Ok(size) => size,
        Err(e) => return Err(e),
    };

    let socket = match clone_socket_for_action(&handle, "Socket.recv_from_async") {
        Ok(Some(socket)) => socket,
        Ok(None) => return utils::nil(),
        Err(e) => return Err(PengError::CannotCallValue(e)),
    };

    let state = Arc::new(Mutex::new(UdpTaskState::Running));
    let thread_state = state.clone();

    thread::spawn(move || {
        let result = match execute_recv_from_socket(&socket, size, "Socket.recv_from_async") {
            Ok(Some(packet)) => Ok(UdpTaskValue::Packet(packet)),
            Ok(None) => Ok(UdpTaskValue::Nil),
            Err(e) => Err(e),
        };

        match thread_state.lock() {
            Ok(mut locked) => {
                *locked = UdpTaskState::Finished(result);
            }

            Err(_) => {}
        }
    });

    task_object(ctx, state)
}

fn socket_send(ctx: &mut PengNativeFunctionCallContext, handle: UdpSocketHandle) -> Result<PengBindedCell, PengError> {
    let data = match utils::get_string_arg(ctx, 1) {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    match execute_socket_send(handle, data, "Socket.send") {
        Ok(size) => Ok(PengBindedCell::Mutable(PengCell::Uint(size))),
        Err(e) => Err(PengError::CannotCallValue(e)),
    }
}

fn socket_send_async(
    ctx: &mut PengNativeFunctionCallContext,
    handle: UdpSocketHandle,
) -> Result<PengBindedCell, PengError> {
    let data = match utils::get_string_arg(ctx, 1) {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    let state = Arc::new(Mutex::new(UdpTaskState::Running));
    let thread_state = state.clone();

    thread::spawn(move || {
        let result = match execute_socket_send(handle, data, "Socket.send_async") {
            Ok(size) => Ok(UdpTaskValue::Size(size)),
            Err(e) => Err(e),
        };

        match thread_state.lock() {
            Ok(mut locked) => {
                *locked = UdpTaskState::Finished(result);
            }

            Err(_) => {}
        }
    });

    task_object(ctx, state)
}

fn socket_recv(ctx: &mut PengNativeFunctionCallContext, handle: UdpSocketHandle) -> Result<PengBindedCell, PengError> {
    let size = match utils::get_uint_arg(ctx, 1) {
        Ok(size) => size,
        Err(e) => return Err(e),
    };

    let socket = match clone_socket_for_action(&handle, "Socket.recv") {
        Ok(Some(socket)) => socket,
        Ok(None) => return utils::nil(),
        Err(e) => return Err(PengError::CannotCallValue(e)),
    };

    match execute_recv_socket(&socket, size, "Socket.recv") {
        Ok(Some(data)) => utils::string(ctx, data),
        Ok(None) => utils::nil(),
        Err(e) => Err(PengError::CannotCallValue(e)),
    }
}

fn socket_recv_async(
    ctx: &mut PengNativeFunctionCallContext,
    handle: UdpSocketHandle,
) -> Result<PengBindedCell, PengError> {
    let size = match utils::get_uint_arg(ctx, 1) {
        Ok(size) => size,
        Err(e) => return Err(e),
    };

    let socket = match clone_socket_for_action(&handle, "Socket.recv_async") {
        Ok(Some(socket)) => socket,
        Ok(None) => return utils::nil(),
        Err(e) => return Err(PengError::CannotCallValue(e)),
    };

    let state = Arc::new(Mutex::new(UdpTaskState::Running));
    let thread_state = state.clone();

    thread::spawn(move || {
        let result = match execute_recv_socket(&socket, size, "Socket.recv_async") {
            Ok(Some(data)) => Ok(UdpTaskValue::Data(data)),
            Ok(None) => Ok(UdpTaskValue::Nil),
            Err(e) => Err(e),
        };

        match thread_state.lock() {
            Ok(mut locked) => {
                *locked = UdpTaskState::Finished(result);
            }

            Err(_) => {}
        }
    });

    task_object(ctx, state)
}

fn socket_close(_ctx: &mut PengNativeFunctionCallContext, handle: UdpSocketHandle) -> Result<PengBindedCell, PengError> {
    let mut locked = match handle.socket.lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "udp:Socket.close() socket lock failed".to_string(),
            ));
        }
    };

    *locked = None;

    utils::nil()
}

fn socket_is_closed(_ctx: &mut PengNativeFunctionCallContext, handle: UdpSocketHandle) -> Result<PengBindedCell, PengError> {
    let locked = match handle.socket.lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(PengError::CannotCallValue(
                "udp:Socket.is_closed() socket lock failed".to_string(),
            ));
        }
    };

    Ok(PengBindedCell::Mutable(PengCell::Bool(locked.is_none())))
}

fn socket_local_addr(ctx: &mut PengNativeFunctionCallContext, handle: UdpSocketHandle) -> Result<PengBindedCell, PengError> {
    let socket = match clone_socket_for_action(&handle, "Socket.local_addr") {
        Ok(Some(socket)) => socket,
        Ok(None) => return utils::nil(),
        Err(e) => return Err(PengError::CannotCallValue(e)),
    };

    match socket.local_addr() {
        Ok(addr) => utils::string(ctx, addr.to_string()),
        Err(e) => Err(PengError::CannotCallValue(format!(
            "udp:Socket.local_addr() failed: {}",
            e
        ))),
    }
}

fn socket_peer_addr(ctx: &mut PengNativeFunctionCallContext, handle: UdpSocketHandle) -> Result<PengBindedCell, PengError> {
    let socket = match clone_socket_for_action(&handle, "Socket.peer_addr") {
        Ok(Some(socket)) => socket,
        Ok(None) => return utils::nil(),
        Err(e) => return Err(PengError::CannotCallValue(e)),
    };

    match socket.peer_addr() {
        Ok(addr) => utils::string(ctx, addr.to_string()),
        Err(_) => utils::nil(),
    }
}

fn socket_set_broadcast(
    ctx: &mut PengNativeFunctionCallContext,
    handle: UdpSocketHandle,
) -> Result<PengBindedCell, PengError> {
    let value = match utils::get_bool_arg(ctx, 1) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let socket = match clone_socket_for_action(&handle, "Socket.set_broadcast") {
        Ok(Some(socket)) => socket,
        Ok(None) => return utils::nil(),
        Err(e) => return Err(PengError::CannotCallValue(e)),
    };

    match socket.set_broadcast(value) {
        Ok(_) => utils::nil(),
        Err(e) => Err(PengError::CannotCallValue(format!(
            "udp:Socket.set_broadcast() failed: {}",
            e
        ))),
    }
}

fn socket_set_read_timeout(
    ctx: &mut PengNativeFunctionCallContext,
    handle: UdpSocketHandle,
) -> Result<PengBindedCell, PengError> {
    let timeout = match get_optional_uint_arg(ctx, 1, "Socket.set_read_timeout") {
        Ok(timeout) => timeout,
        Err(e) => return Err(e),
    };

    let socket = match clone_socket_for_action(&handle, "Socket.set_read_timeout") {
        Ok(Some(socket)) => socket,
        Ok(None) => return utils::nil(),
        Err(e) => return Err(PengError::CannotCallValue(e)),
    };

    match socket.set_read_timeout(utils::timeout_to_duration(timeout)) {
        Ok(_) => utils::nil(),
        Err(e) => Err(PengError::CannotCallValue(format!(
            "udp:Socket.set_read_timeout() failed: {}",
            e
        ))),
    }
}

fn socket_set_write_timeout(
    ctx: &mut PengNativeFunctionCallContext,
    handle: UdpSocketHandle,
) -> Result<PengBindedCell, PengError> {
    let timeout = match get_optional_uint_arg(ctx, 1, "Socket.set_write_timeout") {
        Ok(timeout) => timeout,
        Err(e) => return Err(e),
    };

    let socket = match clone_socket_for_action(&handle, "Socket.set_write_timeout") {
        Ok(Some(socket)) => socket,
        Ok(None) => return utils::nil(),
        Err(e) => return Err(PengError::CannotCallValue(e)),
    };

    match socket.set_write_timeout(utils::timeout_to_duration(timeout)) {
        Ok(_) => utils::nil(),
        Err(e) => Err(PengError::CannotCallValue(format!(
            "udp:Socket.set_write_timeout() failed: {}",
            e
        ))),
    }
}

fn socket_set_non_blocking(
    ctx: &mut PengNativeFunctionCallContext,
    handle: UdpSocketHandle,
) -> Result<PengBindedCell, PengError> {
    let value = match utils::get_bool_arg(ctx, 1) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let socket = match clone_socket_for_action(&handle, "Socket.set_non_blocking") {
        Ok(Some(socket)) => socket,
        Ok(None) => return utils::nil(),
        Err(e) => return Err(PengError::CannotCallValue(e)),
    };

    match socket.set_nonblocking(value) {
        Ok(_) => utils::nil(),
        Err(e) => Err(PengError::CannotCallValue(format!(
            "udp:Socket.set_non_blocking() failed: {}",
            e
        ))),
    }
}

fn task_is_finished(_ctx: &mut PengNativeFunctionCallContext, state: Arc<Mutex<UdpTaskState>>) -> Result<PengBindedCell, PengError> {
    match state.lock() {
        Ok(locked) => match &*locked {
            UdpTaskState::Running => Ok(PengBindedCell::Mutable(PengCell::Bool(false))),
            UdpTaskState::Finished(_) => Ok(PengBindedCell::Mutable(PengCell::Bool(true))),
        },

        Err(_) => Err(PengError::CannotCallValue(
            "udp task lock failed".to_string(),
        )),
    }
}

fn task_get(ctx: &mut PengNativeFunctionCallContext, state: Arc<Mutex<UdpTaskState>>) -> Result<PengBindedCell, PengError> {
    let result = match state.lock() {
        Ok(locked) => match &*locked {
            UdpTaskState::Running => return utils::nil(),
            UdpTaskState::Finished(result) => result.clone(),
        },

        Err(_) => {
            return Err(PengError::CannotCallValue(
                "udp task lock failed".to_string(),
            ));
        }
    };

    match result {
        Ok(UdpTaskValue::Socket(socket)) => socket_handle_object(ctx, socket),
        Ok(UdpTaskValue::Packet(packet)) => packet_object(ctx, packet),
        Ok(UdpTaskValue::Data(data)) => utils::string(ctx, data),
        Ok(UdpTaskValue::Size(size)) => Ok(PengBindedCell::Mutable(PengCell::Uint(size))),
        Ok(UdpTaskValue::Nil) => utils::nil(),
        Err(e) => Err(PengError::CannotCallValue(e)),
    }
}

fn task_error(ctx: &mut PengNativeFunctionCallContext, state: Arc<Mutex<UdpTaskState>>) -> Result<PengBindedCell, PengError> {
    match state.lock() {
        Ok(locked) => match &*locked {
            UdpTaskState::Running => utils::nil(),

            UdpTaskState::Finished(result) => match result {
                Ok(_) => utils::nil(),
                Err(e) => utils::string(ctx, e.clone()),
            },
        },

        Err(_) => Err(PengError::CannotCallValue(
            "udp task lock failed".to_string(),
        )),
    }
}

fn packet_object(ctx: &mut PengNativeFunctionCallContext, packet: UdpPacketData) -> Result<PengBindedCell, PengError> {
    let data = utils::string_cell(ctx, packet.data);
    let host = utils::string_cell(ctx, packet.host);
    let addr = utils::string_cell(ctx, packet.addr);

    utils::new_object(
        ctx,
        vec![
            ("data", data),
            ("host", host),
            ("port", PengBindedCell::Mutable(PengCell::Uint(packet.port))),
            ("addr", addr),
            ("size", PengBindedCell::Mutable(PengCell::Uint(packet.size))),
        ],
    )
}

fn execute_socket_connect(
    handle: UdpSocketHandle,
    host: String,
    port: usize,
    function_name: &str,
) -> Result<(), String> {
    if host.is_empty() {
        return Err(format!("udp:{}() host cannot be empty", function_name));
    }

    if port > u16::MAX as usize {
        return Err(format!("udp:{}() port is out of range", function_name));
    }

    let address = format!("{}:{}", host, port);

    let socket = match clone_socket_for_action(&handle, function_name) {
        Ok(Some(socket)) => socket,
        Ok(None) => return Err(format!("udp:{}() socket is closed", function_name)),
        Err(e) => return Err(e),
    };

    match socket.connect(&address) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!(
            "udp:{}() failed to connect to '{}': {}",
            function_name, address, e
        )),
    }
}

fn execute_socket_send_to(
    handle: UdpSocketHandle,
    data: String,
    host: String,
    port: usize,
    function_name: &str,
) -> Result<usize, String> {
    if host.is_empty() {
        return Err(format!("udp:{}() host cannot be empty", function_name));
    }

    if port > u16::MAX as usize {
        return Err(format!("udp:{}() port is out of range", function_name));
    }

    let address = format!("{}:{}", host, port);

    let socket = match clone_socket_for_action(&handle, function_name) {
        Ok(Some(socket)) => socket,
        Ok(None) => return Err(format!("udp:{}() socket is closed", function_name)),
        Err(e) => return Err(e),
    };

    match socket.send_to(data.as_bytes(), &address) {
        Ok(size) => Ok(size),
        Err(e) => Err(format!(
            "udp:{}() failed to send to '{}': {}",
            function_name, address, e
        )),
    }
}

fn execute_socket_send(
    handle: UdpSocketHandle,
    data: String,
    function_name: &str,
) -> Result<usize, String> {
    let socket = match clone_socket_for_action(&handle, function_name) {
        Ok(Some(socket)) => socket,
        Ok(None) => return Err(format!("udp:{}() socket is closed", function_name)),
        Err(e) => return Err(e),
    };

    match socket.send(data.as_bytes()) {
        Ok(size) => Ok(size),
        Err(e) => Err(format!(
            "udp:{}() failed. Use connect(host, port) before send(): {}",
            function_name, e
        )),
    }
}

fn execute_recv_from_socket(
    socket: &UdpSocket,
    size: usize,
    function_name: &str,
) -> Result<Option<UdpPacketData>, String> {
    if size == 0 {
        return Err(format!("udp:{}() size must be greater than zero", function_name));
    }

    let mut buffer = vec![0u8; size];

    match socket.recv_from(&mut buffer) {
        Ok((received, addr)) => {
            buffer.truncate(received);

            Ok(Some(UdpPacketData {
                data: String::from_utf8_lossy(&buffer).to_string(),
                host: addr.ip().to_string(),
                port: addr.port() as usize,
                addr: addr.to_string(),
                size: received,
            }))
        }

        Err(e) => {
            if utils::is_temporary_read_error(&e) {
                return Ok(None);
            }

            Err(format!("udp:{}() failed: {}", function_name, e))
        }
    }
}

fn execute_recv_socket(
    socket: &UdpSocket,
    size: usize,
    function_name: &str,
) -> Result<Option<String>, String> {
    if size == 0 {
        return Err(format!("udp:{}() size must be greater than zero", function_name));
    }

    let mut buffer = vec![0u8; size];

    match socket.recv(&mut buffer) {
        Ok(received) => {
            buffer.truncate(received);
            Ok(Some(String::from_utf8_lossy(&buffer).to_string()))
        }

        Err(e) => {
            if utils::is_temporary_read_error(&e) {
                return Ok(None);
            }

            Err(format!(
                "udp:{}() failed. Use connect(host, port) before recv(): {}",
                function_name, e
            ))
        }
    }
}

fn clone_socket_for_action(
    handle: &UdpSocketHandle,
    function_name: &str,
) -> Result<Option<UdpSocket>, String> {
    let locked = match handle.socket.lock() {
        Ok(locked) => locked,
        Err(_) => {
            return Err(format!(
                "udp:{}() socket lock failed",
                function_name
            ));
        }
    };

    match locked.as_ref() {
        Some(socket) => match socket.try_clone() {
            Ok(socket) => Ok(Some(socket)),
            Err(e) => Err(format!(
                "udp:{}() failed to clone socket: {}",
                function_name, e
            )),
        },

        None => Ok(None),
    }
}

fn get_socket_data_from_args(
    ctx: &mut PengNativeFunctionCallContext,
    function_name: &str,
) -> Result<UdpSocketData, PengError> {
    let first_arg = match ctx.get_arg_cell(0) {
        Some(arg) => arg.clone(),
        None => {
            return Err(PengError::CannotCallValue(format!(
                "udp:{}() expected Socket or host",
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

            Ok(UdpSocketData {
                host,
                port,
                read_timeout: None,
                write_timeout: None,
                timeout: None,
                broadcast: false,
                non_blocking: false,
            })
        }

        Err(_) => {
            let fields = match utils::get_object_fields_from_cell(ctx, &first_arg) {
                Ok(fields) => fields,
                Err(e) => return Err(e),
            };

            socket_data_from_fields(ctx, &fields, function_name)
        }
    }
}

fn socket_data_from_fields(
    ctx: &mut PengNativeFunctionCallContext,
    fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    function_name: &str,
) -> Result<UdpSocketData, PengError> {
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

    let broadcast = match get_optional_bool_field(ctx, fields, "broadcast", function_name) {
        Ok(Some(value)) => value,
        Ok(None) => false,
        Err(e) => return Err(e),
    };

    let non_blocking = match get_optional_bool_field(ctx, fields, "non_blocking", function_name) {
        Ok(Some(value)) => value,
        Ok(None) => false,
        Err(e) => return Err(e),
    };

    Ok(UdpSocketData {
        host,
        port,
        read_timeout,
        write_timeout,
        timeout,
        broadcast,
        non_blocking,
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
                    "udp:{}() field '{}' must be string or nil",
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
                    "udp:{}() field '{}' must be uint, int, byte or nil",
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
                    "udp:{}() field '{}' must be bool or nil",
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
                "udp:{}() expected uint or nil at index {}",
                function_name, index
            ))),
        },
    }
}

fn apply_socket_config(
    socket: &UdpSocket,
    data: &UdpSocketData,
    function_name: &str,
) -> Result<(), String> {
    let read_timeout = data.read_timeout.or(data.timeout);
    let write_timeout = data.write_timeout.or(data.timeout);

    match socket.set_read_timeout(utils::timeout_to_duration(read_timeout)) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!(
                "udp:{}() failed to set read_timeout: {}",
                function_name, e
            ));
        }
    }

    match socket.set_write_timeout(utils::timeout_to_duration(write_timeout)) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!(
                "udp:{}() failed to set write_timeout: {}",
                function_name, e
            ));
        }
    }

    match socket.set_broadcast(data.broadcast) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!(
                "udp:{}() failed to set broadcast: {}",
                function_name, e
            ));
        }
    }

    match socket.set_nonblocking(data.non_blocking) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!(
                "udp:{}() failed to set non_blocking: {}",
                function_name, e
            ));
        }
    }

    Ok(())
}
