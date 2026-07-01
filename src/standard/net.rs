use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread;

use penguin::prelude::*;

use super::utils;

#[derive(Debug, Clone)]
struct NetAddressData {
    host: String,
    port: usize,
}

#[derive(Debug, Clone)]
struct NetAddressObjectData {
    host: String,
    port: usize,
    addr: String,
    ip: Option<IpAddr>,
}

#[derive(Debug, Clone)]
enum NetTaskValue {
    Addresses(Vec<NetAddressObjectData>),
}

#[derive(Debug, Clone)]
enum NetTaskState {
    Running,
    Finished(Result<NetTaskValue, String>),
}

pub fn setup(peng: &mut PengEnv) -> PengUnit {
    let mut module = PengUnit::library();

    let address_type = address_type_value(peng);
    module.register_immutable_global(peng, "Address", address_type).unwrap();

    module.register_immutable_global(
        peng,
        "LOCALHOST",
        PengValue::Box(PengBox::String("localhost".to_string())),
    ).unwrap();

    module.register_immutable_global(
        peng,
        "IPV4_LOOPBACK",
        PengValue::Box(PengBox::String("127.0.0.1".to_string())),
    ).unwrap();

    module.register_immutable_global(
        peng,
        "IPV6_LOOPBACK",
        PengValue::Box(PengBox::String("::1".to_string())),
    ).unwrap();

    module.register_immutable_global(
        peng,
        "IPV4_UNSPECIFIED",
        PengValue::Box(PengBox::String("0.0.0.0".to_string())),
    ).unwrap();

    module.register_immutable_global(
        peng,
        "IPV6_UNSPECIFIED",
        PengValue::Box(PengBox::String("::".to_string())),
    ).unwrap();

    module
        .register_immutable_native_function(peng, "local_ip", local_ip)
        .unwrap();

    module
        .register_immutable_native_function(peng, "local_addr", local_addr)
        .unwrap();

    module.register_immutable_native_function(peng, "hostname", hostname).unwrap();

    module.register_immutable_native_function(peng, "join", join).unwrap();

    module.register_immutable_native_function(peng, "split", split).unwrap();

    module.register_immutable_native_function(peng, "parse", parse).unwrap();

    module.register_immutable_native_function(peng, "normalize", normalize).unwrap();

    module.register_immutable_native_function(peng, "resolve", resolve).unwrap();

    module.register_immutable_native_function(peng, "resolve_one", resolve_one).unwrap();

    module.register_immutable_native_function(peng, "resolve_async", resolve_async).unwrap();

    module.register_immutable_native_function(peng, "ip_info", ip_info).unwrap();

    module.register_immutable_native_function(peng, "is_ip", is_ip).unwrap();

    module.register_immutable_native_function(peng, "is_ipv4", is_ipv4).unwrap();

    module.register_immutable_native_function(peng, "is_ipv6", is_ipv6).unwrap();

    module.register_immutable_native_function(peng, "is_loopback", is_loopback).unwrap();

    module.register_immutable_native_function(peng, "is_private", is_private).unwrap();

    module.register_immutable_native_function(peng, "is_unspecified", is_unspecified).unwrap();

    module.register_immutable_native_function(peng, "is_multicast", is_multicast).unwrap();

    module.register_immutable_native_function(peng, "valid_port", valid_port).unwrap();

    module
}

fn address_type_value(peng: &mut PengEnv) -> PengValue {
    let mut fields = HashMap::new();

    let parse_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(address_parse),
    )));

    let resolve_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(address_resolve),
    )));

    let resolve_one_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(address_resolve_one),
    )));

    let resolve_async_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(address_resolve_async),
    )));

    let to_string_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(address_to_string),
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
        peng.ensure_pooled_name_ptr("addr".to_string()),
        PengBindedCell::Mutable(PengCell::Nil),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("parse".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(parse_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("resolve".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(resolve_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("resolve_one".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(resolve_one_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("resolve_async".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(resolve_async_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("to_string".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(to_string_ptr)),
    );

    PengValue::Box(PengBox::Type(PengType::Custom(PengCustomType { fields })))
}

fn hostname(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    match env::var("HOSTNAME") {
        Ok(value) => {
            if !value.trim().is_empty() {
                return utils::string(ctx, value);
            }
        }
        Err(_) => {}
    }

    match env::var("COMPUTERNAME") {
        Ok(value) => {
            if !value.trim().is_empty() {
                return utils::string(ctx, value);
            }
        }
        Err(_) => {}
    }

    match fs::read_to_string("/etc/hostname") {
        Ok(value) => {
            let value = value.trim().to_string();

            if value.is_empty() {
                utils::nil()
            } else {
                utils::string(ctx, value)
            }
        }

        Err(_) => utils::nil(),
    }
}

fn join(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let data = match get_address_data_from_args(ctx, "join") {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    utils::string(ctx, join_host_port(&data.host, data.port))
}

fn split(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let address = match utils::get_string_arg(ctx, 0) {
        Ok(address) => address,
        Err(e) => return Err(e),
    };

    let data = match split_address_string(&address, "split") {
        Ok(data) => data,
        Err(e) => return Err(PengError::CannotCallValue(e)),
    };

    address_data_to_object(ctx, data)
}

fn parse(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let data = match get_address_data_from_args(ctx, "parse") {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    address_data_to_object(ctx, data)
}

fn normalize(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    join(ctx)
}

fn resolve(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let data = match get_address_data_from_args(ctx, "resolve") {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    let addresses = match execute_resolve_data(data, "resolve") {
        Ok(addresses) => addresses,
        Err(e) => return Err(PengError::CannotCallValue(e)),
    };

    address_vector(ctx, addresses)
}

fn resolve_one(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let data = match get_address_data_from_args(ctx, "resolve_one") {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    let addresses = match execute_resolve_data(data, "resolve_one") {
        Ok(addresses) => addresses,
        Err(e) => return Err(PengError::CannotCallValue(e)),
    };

    match addresses.into_iter().next() {
        Some(address) => resolved_address_to_object(ctx, address),
        None => utils::nil(),
    }
}

fn resolve_async(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let data = match get_address_data_from_args(ctx, "resolve_async") {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    let state = Arc::new(Mutex::new(NetTaskState::Running));
    let thread_state = state.clone();

    thread::spawn(move || {
        let result = match execute_resolve_data(data, "resolve_async") {
            Ok(addresses) => Ok(NetTaskValue::Addresses(addresses)),
            Err(e) => Err(e),
        };

        match thread_state.lock() {
            Ok(mut locked) => {
                *locked = NetTaskState::Finished(result);
            }

            Err(_) => {}
        }
    });

    task_object(ctx, state)
}

fn ip_info(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let ip = match get_ip_arg(ctx, 0, "ip_info") {
        Ok(ip) => ip,
        Err(e) => return Err(e),
    };

    ip_info_to_object(ctx, ip)
}

fn is_ip(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let value = match utils::get_string_arg(ctx, 0) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(PengBindedCell::Mutable(PengCell::Bool(
        value.parse::<IpAddr>().is_ok(),
    )))
}

fn is_ipv4(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let value = match utils::get_string_arg(ctx, 0) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    match value.parse::<IpAddr>() {
        Ok(IpAddr::V4(_)) => Ok(PengBindedCell::Mutable(PengCell::Bool(true))),
        Ok(IpAddr::V6(_)) => Ok(PengBindedCell::Mutable(PengCell::Bool(false))),
        Err(_) => Ok(PengBindedCell::Mutable(PengCell::Bool(false))),
    }
}

fn is_ipv6(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let value = match utils::get_string_arg(ctx, 0) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    match value.parse::<IpAddr>() {
        Ok(IpAddr::V4(_)) => Ok(PengBindedCell::Mutable(PengCell::Bool(false))),
        Ok(IpAddr::V6(_)) => Ok(PengBindedCell::Mutable(PengCell::Bool(true))),
        Err(_) => Ok(PengBindedCell::Mutable(PengCell::Bool(false))),
    }
}

fn is_loopback(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    ip_predicate(ctx, "is_loopback", |ip| ip.is_loopback())
}

fn is_private(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    ip_predicate(ctx, "is_private", is_private_ip)
}

fn is_unspecified(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    ip_predicate(ctx, "is_unspecified", |ip| ip.is_unspecified())
}

fn is_multicast(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    ip_predicate(ctx, "is_multicast", |ip| ip.is_multicast())
}

fn valid_port(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let port = match utils::get_uint_arg(ctx, 0) {
        Ok(port) => port,
        Err(e) => return Err(e),
    };

    Ok(PengBindedCell::Mutable(PengCell::Bool(
        port <= u16::MAX as usize,
    )))
}

fn address_parse(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    parse(ctx)
}

fn address_resolve(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    resolve(ctx)
}

fn address_resolve_one(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    resolve_one(ctx)
}

fn address_resolve_async(
    ctx: &mut PengNativeFunctionCallContext,
) -> Result<PengBindedCell, PengError> {
    resolve_async(ctx)
}

fn address_to_string(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    join(ctx)
}

fn execute_resolve_data(
    data: NetAddressData,
    function_name: &str,
) -> Result<Vec<NetAddressObjectData>, String> {
    if data.host.is_empty() {
        return Err(format!("net:{}() host cannot be empty", function_name));
    }

    if data.port > u16::MAX as usize {
        return Err(format!("net:{}() port is out of range", function_name));
    }

    let address = join_host_port(&data.host, data.port);

    let addrs = match address.to_socket_addrs() {
        Ok(addrs) => addrs,
        Err(e) => {
            return Err(format!(
                "net:{}() failed to resolve '{}': {}",
                function_name, address, e
            ));
        }
    };

    let mut output = Vec::new();

    for socket_addr in addrs {
        output.push(socket_addr_to_data(socket_addr, data.host.clone()));
    }

    Ok(output)
}

fn task_object(
    ctx: &mut PengNativeFunctionCallContext,
    state: Arc<Mutex<NetTaskState>>,
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
            ("get", PengBindedCell::Immutable(PengCell::Reference(get_ptr))),
            (
                "error",
                PengBindedCell::Immutable(PengCell::Reference(error_ptr)),
            ),
        ],
    )
}

fn task_is_finished(
    _ctx: &mut PengNativeFunctionCallContext,
    state: Arc<Mutex<NetTaskState>>,
) -> Result<PengBindedCell, PengError> {
    match state.lock() {
        Ok(locked) => match &*locked {
            NetTaskState::Running => Ok(PengBindedCell::Mutable(PengCell::Bool(false))),
            NetTaskState::Finished(_) => Ok(PengBindedCell::Mutable(PengCell::Bool(true))),
        },

        Err(_) => Err(PengError::CannotCallValue(
            "net task lock failed".to_string(),
        )),
    }
}

fn task_get(
    ctx: &mut PengNativeFunctionCallContext,
    state: Arc<Mutex<NetTaskState>>,
) -> Result<PengBindedCell, PengError> {
    let result = match state.lock() {
        Ok(locked) => match &*locked {
            NetTaskState::Running => return utils::nil(),
            NetTaskState::Finished(result) => result.clone(),
        },

        Err(_) => {
            return Err(PengError::CannotCallValue(
                "net task lock failed".to_string(),
            ));
        }
    };

    match result {
        Ok(NetTaskValue::Addresses(addresses)) => address_vector(ctx, addresses),
        Err(e) => Err(PengError::CannotCallValue(e)),
    }
}

fn task_error(
    ctx: &mut PengNativeFunctionCallContext,
    state: Arc<Mutex<NetTaskState>>,
) -> Result<PengBindedCell, PengError> {
    match state.lock() {
        Ok(locked) => match &*locked {
            NetTaskState::Running => utils::nil(),

            NetTaskState::Finished(result) => match result {
                Ok(_) => utils::nil(),
                Err(e) => utils::string(ctx, e.clone()),
            },
        },

        Err(_) => Err(PengError::CannotCallValue(
            "net task lock failed".to_string(),
        )),
    }
}

fn address_vector(
    ctx: &mut PengNativeFunctionCallContext,
    addresses: Vec<NetAddressObjectData>,
) -> Result<PengBindedCell, PengError> {
    let mut values = Vec::new();

    for address in addresses {
        let cell = match resolved_address_to_object(ctx, address) {
            Ok(cell) => cell,
            Err(e) => return Err(e),
        };

        values.push(cell);
    }

    let ptr = ctx.create_box(PengBox::Vector(PengVector { values }));

    Ok(PengBindedCell::Mutable(PengCell::Reference(ptr)))
}

fn address_data_to_object(
    ctx: &mut PengNativeFunctionCallContext,
    data: NetAddressData,
) -> Result<PengBindedCell, PengError> {
    let ip = match data.host.parse::<IpAddr>() {
        Ok(ip) => Some(ip),
        Err(_) => None,
    };

    let address = NetAddressObjectData {
        host: data.host.clone(),
        port: data.port,
        addr: join_host_port(&data.host, data.port),
        ip,
    };

    resolved_address_to_object(ctx, address)
}

fn resolved_address_to_object(
    ctx: &mut PengNativeFunctionCallContext,
    data: NetAddressObjectData,
) -> Result<PengBindedCell, PengError> {
    let ip = data.ip;
    let family = ip_family(ip.as_ref());

    let host_cell = utils::string_cell(ctx, data.host);
    let addr_cell = utils::string_cell(ctx, data.addr);
    let family_cell = utils::string_cell(ctx, family);

    let ip_cell = match ip {
        Some(ip) => utils::string_cell(ctx, ip.to_string()),
        None => PengBindedCell::Mutable(PengCell::Nil),
    };

    let is_ipv4 = matches!(ip, Some(IpAddr::V4(_)));
    let is_ipv6 = matches!(ip, Some(IpAddr::V6(_)));
    let is_loopback = match ip {
        Some(ip) => ip.is_loopback(),
        None => false,
    };
    let is_private = match ip {
        Some(ip) => is_private_ip(ip),
        None => false,
    };
    let is_unspecified = match ip {
        Some(ip) => ip.is_unspecified(),
        None => false,
    };
    let is_multicast = match ip {
        Some(ip) => ip.is_multicast(),
        None => false,
    };

    utils::new_object(
        ctx,
        vec![
            ("host", host_cell),
            ("port", PengBindedCell::Mutable(PengCell::Uint(data.port))),
            ("addr", addr_cell),
            ("ip", ip_cell),
            ("family", family_cell),
            ("is_ip", PengBindedCell::Mutable(PengCell::Bool(ip.is_some()))),
            ("is_ipv4", PengBindedCell::Mutable(PengCell::Bool(is_ipv4))),
            ("is_ipv6", PengBindedCell::Mutable(PengCell::Bool(is_ipv6))),
            (
                "is_loopback",
                PengBindedCell::Mutable(PengCell::Bool(is_loopback)),
            ),
            (
                "is_private",
                PengBindedCell::Mutable(PengCell::Bool(is_private)),
            ),
            (
                "is_unspecified",
                PengBindedCell::Mutable(PengCell::Bool(is_unspecified)),
            ),
            (
                "is_multicast",
                PengBindedCell::Mutable(PengCell::Bool(is_multicast)),
            ),
        ],
    )
}

fn ip_info_to_object(
    ctx: &mut PengNativeFunctionCallContext,
    ip: IpAddr,
) -> Result<PengBindedCell, PengError> {
    let family = ip_family(Some(&ip));
    let ip_cell = utils::string_cell(ctx, ip.to_string());
    let family_cell = utils::string_cell(ctx, family);

    utils::new_object(
        ctx,
        vec![
            ("ip", ip_cell),
            ("family", family_cell),
            (
                "is_ipv4",
                PengBindedCell::Mutable(PengCell::Bool(matches!(ip, IpAddr::V4(_)))),
            ),
            (
                "is_ipv6",
                PengBindedCell::Mutable(PengCell::Bool(matches!(ip, IpAddr::V6(_)))),
            ),
            (
                "is_loopback",
                PengBindedCell::Mutable(PengCell::Bool(ip.is_loopback())),
            ),
            (
                "is_private",
                PengBindedCell::Mutable(PengCell::Bool(is_private_ip(ip))),
            ),
            (
                "is_unspecified",
                PengBindedCell::Mutable(PengCell::Bool(ip.is_unspecified())),
            ),
            (
                "is_multicast",
                PengBindedCell::Mutable(PengCell::Bool(ip.is_multicast())),
            ),
        ],
    )
}

fn socket_addr_to_data(socket_addr: SocketAddr, host: String) -> NetAddressObjectData {
    NetAddressObjectData {
        host,
        port: socket_addr.port() as usize,
        addr: socket_addr.to_string(),
        ip: Some(socket_addr.ip()),
    }
}

fn get_address_data_from_args(
    ctx: &mut PengNativeFunctionCallContext,
    function_name: &str,
) -> Result<NetAddressData, PengError> {
    let first_arg = match ctx.get_arg_cell(0) {
        Some(arg) => arg.clone(),
        None => {
            return Err(PengError::CannotCallValue(format!(
                "net:{}() expected address, host or object",
                function_name
            )));
        }
    };

    match utils::cell_to_string(ctx, &first_arg) {
        Ok(value) => match ctx.get_arg_cell(1) {
            Some(_) => {
                let port = match utils::get_uint_arg(ctx, 1) {
                    Ok(port) => port,
                    Err(e) => return Err(e),
                };

                validate_host_port(value, port, function_name)
            }

            None => match split_address_string(&value, function_name) {
                Ok(data) => Ok(data),
                Err(_) => validate_host_port(value, 0, function_name),
            },
        },

        Err(_) => get_address_data_from_object_cell(ctx, &first_arg, function_name),
    }
}

fn get_address_data_from_object_cell(
    ctx: &mut PengNativeFunctionCallContext,
    cell: &PengBindedCell,
    function_name: &str,
) -> Result<NetAddressData, PengError> {
    let fields = match utils::get_object_fields_from_cell(ctx, cell) {
        Ok(fields) => fields,
        Err(_) => {
            return Err(PengError::CannotCallValue(format!(
                "net:{}() expected address object",
                function_name
            )));
        }
    };

    let addr = match get_optional_string_field(ctx, &fields, "addr", function_name) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    match addr {
        Some(addr) => {
            if !addr.is_empty() {
                match split_address_string(&addr, function_name) {
                    Ok(data) => return Ok(data),
                    Err(e) => return Err(PengError::CannotCallValue(e)),
                }
            }
        }

        None => {}
    }

    let host = match get_optional_string_field(ctx, &fields, "host", function_name) {
        Ok(Some(host)) => host,
        Ok(None) => "127.0.0.1".to_string(),
        Err(e) => return Err(e),
    };

    let port = match get_optional_uint_field(ctx, &fields, "port", function_name) {
        Ok(Some(port)) => port,
        Ok(None) => 0,
        Err(e) => return Err(e),
    };

    validate_host_port(host, port, function_name)
}

fn validate_host_port(
    host: String,
    port: usize,
    function_name: &str,
) -> Result<NetAddressData, PengError> {
    if host.is_empty() {
        return Err(PengError::CannotCallValue(format!(
            "net:{}() host cannot be empty",
            function_name
        )));
    }

    if port > u16::MAX as usize {
        return Err(PengError::CannotCallValue(format!(
            "net:{}() port is out of range",
            function_name
        )));
    }

    Ok(NetAddressData { host, port })
}

fn split_address_string(address: &str, function_name: &str) -> Result<NetAddressData, String> {
    if address.is_empty() {
        return Err(format!("net:{}() address cannot be empty", function_name));
    }

    if address.starts_with('[') {
        let end = match address.find(']') {
            Some(end) => end,
            None => {
                return Err(format!(
                    "net:{}() invalid bracketed IPv6 address",
                    function_name
                ));
            }
        };

        let host = address[1..end].to_string();
        let rest = &address[end + 1..];

        if !rest.starts_with(':') {
            return Err(format!(
                "net:{}() address is missing port separator",
                function_name
            ));
        }

        let port = match parse_port_string(&rest[1..], function_name) {
            Ok(port) => port,
            Err(e) => return Err(e),
        };

        return Ok(NetAddressData { host, port });
    }

    let colon_count = address.chars().filter(|c| *c == ':').count();

    if colon_count == 0 {
        return Err(format!("net:{}() address is missing port", function_name));
    }

    if colon_count > 1 {
        return Err(format!(
            "net:{}() IPv6 address with port must use brackets",
            function_name
        ));
    }

    let mut parts = address.rsplitn(2, ':');

    let port_part = match parts.next() {
        Some(port) => port,
        None => return Err(format!("net:{}() address is missing port", function_name)),
    };

    let host_part = match parts.next() {
        Some(host) => host,
        None => return Err(format!("net:{}() address is missing host", function_name)),
    };

    if host_part.is_empty() {
        return Err(format!("net:{}() address host cannot be empty", function_name));
    }

    let port = match parse_port_string(port_part, function_name) {
        Ok(port) => port,
        Err(e) => return Err(e),
    };

    Ok(NetAddressData {
        host: host_part.to_string(),
        port,
    })
}

fn parse_port_string(value: &str, function_name: &str) -> Result<usize, String> {
    if value.is_empty() {
        return Err(format!("net:{}() port cannot be empty", function_name));
    }

    let port = match value.parse::<usize>() {
        Ok(port) => port,
        Err(_) => {
            return Err(format!(
                "net:{}() invalid port '{}'",
                function_name, value
            ));
        }
    };

    if port > u16::MAX as usize {
        return Err(format!("net:{}() port is out of range", function_name));
    }

    Ok(port)
}

fn join_host_port(host: &str, port: usize) -> String {
    if host.contains(':') && !host.starts_with('[') && !host.ends_with(']') {
        format!("[{}]:{}", host, port)
    } else {
        format!("{}:{}", host, port)
    }
}

fn ip_family(ip: Option<&IpAddr>) -> String {
    match ip {
        Some(IpAddr::V4(_)) => "ipv4".to_string(),
        Some(IpAddr::V6(_)) => "ipv6".to_string(),
        None => "domain".to_string(),
    }
}

fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => ip.is_private(),
        IpAddr::V6(ip) => ip.is_unique_local(),
    }
}

fn ip_predicate(
    ctx: &mut PengNativeFunctionCallContext,
    function_name: &str,
    predicate: fn(IpAddr) -> bool,
) -> Result<PengBindedCell, PengError> {
    let ip = match get_ip_arg(ctx, 0, function_name) {
        Ok(ip) => ip,
        Err(e) => return Err(e),
    };

    Ok(PengBindedCell::Mutable(PengCell::Bool(predicate(ip))))
}

fn get_ip_arg(
    ctx: &PengNativeFunctionCallContext,
    index: usize,
    function_name: &str,
) -> Result<IpAddr, PengError> {
    let value = match utils::get_string_arg(ctx, index) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    match value.parse::<IpAddr>() {
        Ok(ip) => Ok(ip),
        Err(_) => Err(PengError::CannotCallValue(format!(
            "net:{}() expected IP address at index {}",
            function_name, index
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
        Some(value) => match value.value() {
            PengCell::Nil => Ok(None),
            _ => match utils::cell_to_string(ctx, &value) {
                Ok(value) => Ok(Some(value)),
                Err(_) => Err(PengError::CannotCallValue(format!(
                    "net:{}() field '{}' must be string or nil",
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
                    "net:{}() field '{}' must be uint, int, byte or nil",
                    function_name, name
                ))),
            },
        },

        None => Ok(None),
    }
}

fn local_ip(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| PengError::CannotCallValue(format!("net:local_ip(): {}", e)))?;

    socket
        .connect("8.8.8.8:80")
        .map_err(|e| PengError::CannotCallValue(format!("net:local_ip(): {}", e)))?;

    let addr = socket
        .local_addr()
        .map_err(|e| PengError::CannotCallValue(format!("net:local_ip(): {}", e)))?;

    utils::string(ctx, addr.ip().to_string())
}

fn local_addr(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let socket = match std::net::UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => socket,
        Err(e) => {
            return Err(PengError::CannotCallValue(format!(
                "net:local_addr() failed to bind udp socket: {}",
                e
            )));
        }
    };

    match socket.connect("8.8.8.8:80") {
        Ok(_) => {}
        Err(e) => {
            return Err(PengError::CannotCallValue(format!(
                "net:local_addr() failed to detect local address: {}",
                e
            )));
        }
    }

    let addr = match socket.local_addr() {
        Ok(addr) => addr,
        Err(e) => {
            return Err(PengError::CannotCallValue(format!(
                "net:local_addr() failed to read local address: {}",
                e
            )));
        }
    };

    resolved_address_to_object(
        ctx,
        socket_addr_to_data(addr, addr.ip().to_string()),
    )
}
