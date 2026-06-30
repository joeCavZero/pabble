use std::collections::HashMap;
use std::time::Duration;

use penguin::prelude::*;

use super::utils;

use ureq::ResponseExt;
use ureq::http::Method;

#[derive(Debug, Clone)]
struct HttpClientConfig {
    timeout: Option<usize>,
    max_redirects: usize,
    max_redirects_will_error: bool,
    https_only: bool,
    status_as_error: bool,
    allow_non_standard_methods: bool,
}

impl HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: None,
            max_redirects: 10,
            max_redirects_will_error: false,
            https_only: false,
            status_as_error: false,
            allow_non_standard_methods: true,
        }
    }
}

pub fn setup(peng: &mut PengEnv) -> PengUnit {
    let mut module = PengUnit::library();

    module.register_native_function(peng, "new_client", client).unwrap();
    module
        .register_native_function(peng, "new_request", new_request)
        .unwrap();
    module.register_native_function(peng, "send", send).unwrap();

    module
        .register_native_function(peng, "set_header", set_header)
        .unwrap();
    module
        .register_native_function(peng, "get_header", get_header)
        .unwrap();
    module
        .register_native_function(peng, "remove_header", remove_header)
        .unwrap();

    module
        .register_native_function(peng, "status_text", status_text)
        .unwrap();

    module
}

fn client(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let config = match ctx.get_arg_cell(0) {
        Some(arg) => {
            let fields = match get_object_fields_from_cell(ctx, arg, "client") {
                Ok(fields) => fields,
                Err(e) => return Err(e),
            };

            match client_config_from_fields(ctx, &fields, "client") {
                Ok(config) => config,
                Err(e) => return Err(e),
            }
        }

        None => HttpClientConfig::default(),
    };

    new_client_object(ctx, config)
}

fn new_request(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let method = match get_string_arg(ctx, 0, "new_request") {
        Ok(method) => method,
        Err(e) => return Err(e),
    };

    let url = match get_string_arg(ctx, 1, "new_request") {
        Ok(url) => url,
        Err(e) => return Err(e),
    };

    let body_arg = match ctx.get_arg_cell(2) {
        Some(arg) => Some(arg.clone()),
        None => None,
    };

    let body_cell = match body_arg {
        Some(arg) => match arg.value() {
            PengCell::Nil => PengBindedCell::Mutable(PengCell::Nil),
            _ => {
                let body = match cell_to_string(ctx, &arg, "new_request") {
                    Ok(body) => body,
                    Err(e) => return Err(e),
                };

                string_cell(ctx, body)
            }
        },

        None => PengBindedCell::Mutable(PengCell::Nil),
    };

    let headers_fields = match ctx.get_arg_cell(3) {
        Some(arg) => match get_object_fields_from_cell(ctx, arg, "new_request") {
            Ok(fields) => fields,
            Err(e) => return Err(e),
        },

        None => HashMap::new(),
    };

    request_object(ctx, method, url, body_cell, headers_fields)
}

fn send(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let first_arg = match ctx.get_arg_cell(0) {
        Some(arg) => arg.clone(),
        None => {
            return Err(PengError::CannotCallValue(
                "http:send() missing request argument".into(),
            ));
        }
    };

    let second_arg = match ctx.get_arg_cell(1) {
        Some(arg) => Some(arg.clone()),
        None => None,
    };

    let (client_config, request_fields) = match second_arg {
        Some(request_arg) => {
            let client_fields = match get_object_fields_from_cell(ctx, &first_arg, "send") {
                Ok(fields) => fields,
                Err(e) => return Err(e),
            };

            let client_config = match client_config_from_fields(ctx, &client_fields, "send") {
                Ok(config) => config,
                Err(e) => return Err(e),
            };

            let request_fields = match get_object_fields_from_cell(ctx, &request_arg, "send") {
                Ok(fields) => fields,
                Err(e) => return Err(e),
            };

            (client_config, request_fields)
        }

        None => {
            let request_fields = match get_object_fields_from_cell(ctx, &first_arg, "send") {
                Ok(fields) => fields,
                Err(e) => return Err(e),
            };

            (HttpClientConfig::default(), request_fields)
        }
    };

    let method = match get_required_string_field(ctx, &request_fields, "method", "send") {
        Ok(method) => method,
        Err(e) => return Err(e),
    };

    let url = match get_required_string_field(ctx, &request_fields, "url", "send") {
        Ok(url) => url,
        Err(e) => return Err(e),
    };

    let headers_fields = match get_headers_from_request(ctx, &request_fields, "send") {
        Ok(fields) => fields,
        Err(e) => return Err(e),
    };

    let headers = match fields_to_string_pairs(ctx, &headers_fields, "send") {
        Ok(headers) => headers,
        Err(e) => return Err(e),
    };

    let body = match get_optional_body_from_request(ctx, &request_fields, "send") {
        Ok(body) => body,
        Err(e) => return Err(e),
    };

    let method = match Method::from_bytes(method.as_bytes()) {
        Ok(method) => method,
        Err(_) => {
            return Err(PengError::CannotCallValue(format!(
                "http:send() invalid HTTP method '{}'",
                method
            )));
        }
    };

    let agent = build_agent(client_config);
    let mut builder = ureq::http::Request::builder().method(method).uri(url.clone());

    for (name, value) in headers {
        builder = builder.header(name, value);
    }

    match body {
        Some(body) => {
            let request = match builder.body(body) {
                Ok(request) => request,
                Err(e) => {
                    return Err(PengError::CannotCallValue(format!(
                        "http:send() failed to build request: {}",
                        e
                    )));
                }
            };

            execute_request(ctx, &agent, request)
        }

        None => {
            let request = match builder.body(()) {
                Ok(request) => request,
                Err(e) => {
                    return Err(PengError::CannotCallValue(format!(
                        "http:send() failed to build request: {}",
                        e
                    )));
                }
            };

            execute_request(ctx, &agent, request)
        }
    }
}

fn set_header(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let request_arg = match ctx.get_arg_cell(0) {
        Some(arg) => arg.clone(),
        None => {
            return Err(PengError::CannotCallValue(
                "http:set_header() missing request argument".into(),
            ));
        }
    };

    let name = match get_string_arg(ctx, 1, "set_header") {
        Ok(name) => name,
        Err(e) => return Err(e),
    };

    let value = match get_string_arg(ctx, 2, "set_header") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let request_fields = match get_object_fields_from_cell(ctx, &request_arg, "set_header") {
        Ok(fields) => fields,
        Err(e) => return Err(e),
    };

    let mut headers_fields = match get_headers_from_request(ctx, &request_fields, "set_header") {
        Ok(fields) => fields,
        Err(e) => return Err(e),
    };

    let name_ptr = ctx.env_mut().ensure_pooled_name_ptr(name);
    headers_fields.insert(name_ptr, string_cell(ctx, value));

    rebuild_request_with_headers(ctx, request_fields, headers_fields, "set_header")
}

fn get_header(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let request_arg = match ctx.get_arg_cell(0) {
        Some(arg) => arg.clone(),
        None => {
            return Err(PengError::CannotCallValue(
                "http:get_header() missing request argument".into(),
            ));
        }
    };

    let name = match get_string_arg(ctx, 1, "get_header") {
        Ok(name) => name,
        Err(e) => return Err(e),
    };

    let request_fields = match get_object_fields_from_cell(ctx, &request_arg, "get_header") {
        Ok(fields) => fields,
        Err(e) => return Err(e),
    };

    let headers_fields = match get_headers_from_request(ctx, &request_fields, "get_header") {
        Ok(fields) => fields,
        Err(e) => return Err(e),
    };

    let name_ptr = ctx.env_mut().ensure_pooled_name_ptr(name);

    match headers_fields.get(&name_ptr) {
        Some(value) => match cell_to_string(ctx, value, "get_header") {
            Ok(value) => string(ctx, value),
            Err(e) => Err(e),
        },

        None => nil(),
    }
}

fn remove_header(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let request_arg = match ctx.get_arg_cell(0) {
        Some(arg) => arg.clone(),
        None => {
            return Err(PengError::CannotCallValue(
                "http:remove_header() missing request argument".into(),
            ));
        }
    };

    let name = match get_string_arg(ctx, 1, "remove_header") {
        Ok(name) => name,
        Err(e) => return Err(e),
    };

    let request_fields = match get_object_fields_from_cell(ctx, &request_arg, "remove_header") {
        Ok(fields) => fields,
        Err(e) => return Err(e),
    };

    let mut headers_fields = match get_headers_from_request(ctx, &request_fields, "remove_header") {
        Ok(fields) => fields,
        Err(e) => return Err(e),
    };

    let name_ptr = ctx.env_mut().ensure_pooled_name_ptr(name);
    headers_fields.remove(&name_ptr);

    rebuild_request_with_headers(ctx, request_fields, headers_fields, "remove_header")
}

fn status_text(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let code = match get_uint_arg(ctx, 0, "status_text") {
        Ok(code) => code,
        Err(e) => return Err(e),
    };

    if code > u16::MAX as usize {
        return nil();
    }

    let status = match ureq::http::StatusCode::from_u16(code as u16) {
        Ok(status) => status,
        Err(_) => return nil(),
    };

    match status.canonical_reason() {
        Some(reason) => string(ctx, reason.to_string()),
        None => nil(),
    }
}

fn execute_request<S>(
    ctx: &mut PengNativeFunctionCallContext,
    agent: &ureq::Agent,
    request: ureq::http::Request<S>,
) -> Result<PengBindedCell, PengError>
where
    S: ureq::AsSendBody,
{
    let mut response = match agent.run(request) {
        Ok(response) => response,
        Err(e) => {
            return Err(PengError::CannotCallValue(format!(
                "http:send() request failed: {}",
                e
            )));
        }
    };

    let status = response.status().as_u16() as usize;
    let ok = status >= 200 && status <= 299;
    let reason = match response.status().canonical_reason() {
        Some(reason) => reason.to_string(),
        None => String::new(),
    };

    let url = response.get_uri().to_string();

    let mut response_headers = Vec::new();

    for (name, value) in response.headers().iter() {
        let value = match value.to_str() {
            Ok(value) => value.to_string(),
            Err(_) => String::new(),
        };

        response_headers.push((name.to_string(), value));
    }

    let body = match response.body_mut().read_to_string() {
        Ok(body) => body,
        Err(e) => {
            return Err(PengError::CannotCallValue(format!(
                "http:send() failed to read response body: {}",
                e
            )));
        }
    };

    let status_cell = PengBindedCell::Mutable(PengCell::Uint(status));
    let ok_cell = PengBindedCell::Mutable(PengCell::Bool(ok));
    let reason_cell = string_cell(ctx, reason);
    let headers_cell = object_from_string_pairs(ctx, response_headers);
    let body_cell = string_cell(ctx, body);
    let url_cell = string_cell(ctx, url);

    utils::new_object(
        ctx,
        vec![
            ("status", status_cell),
            ("ok", ok_cell),
            ("reason", reason_cell),
            ("headers", headers_cell),
            ("body", body_cell),
            ("url", url_cell),
        ],
    )
}

fn build_agent(config: HttpClientConfig) -> ureq::Agent {
    let max_redirects = if config.max_redirects > u32::MAX as usize {
        u32::MAX
    } else {
        config.max_redirects as u32
    };

    let mut builder = ureq::Agent::config_builder()
        .http_status_as_error(config.status_as_error)
        .https_only(config.https_only)
        .max_redirects(max_redirects)
        .max_redirects_will_error(config.max_redirects_will_error)
        .allow_non_standard_methods(config.allow_non_standard_methods);

    match config.timeout {
        Some(timeout) => {
            builder = builder.timeout_global(Some(Duration::from_millis(timeout as u64)));
        }

        None => {}
    }

    let config = builder.build();

    ureq::Agent::new_with_config(config)
}

fn client_config_from_fields(
    ctx: &mut PengNativeFunctionCallContext,
    fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    function_name: &str,
) -> Result<HttpClientConfig, PengError> {
    let mut config = HttpClientConfig::default();

    match get_optional_uint_field(ctx, fields, "timeout", function_name) {
        Ok(value) => config.timeout = value,
        Err(e) => return Err(e),
    }

    match get_optional_uint_field(ctx, fields, "max_redirects", function_name) {
        Ok(Some(value)) => config.max_redirects = value,
        Ok(None) => {}
        Err(e) => return Err(e),
    }

    match get_optional_bool_field(ctx, fields, "max_redirects_will_error", function_name) {
        Ok(Some(value)) => config.max_redirects_will_error = value,
        Ok(None) => {}
        Err(e) => return Err(e),
    }

    match get_optional_bool_field(ctx, fields, "https_only", function_name) {
        Ok(Some(value)) => config.https_only = value,
        Ok(None) => {}
        Err(e) => return Err(e),
    }

    match get_optional_bool_field(ctx, fields, "status_as_error", function_name) {
        Ok(Some(value)) => config.status_as_error = value,
        Ok(None) => {}
        Err(e) => return Err(e),
    }

    match get_optional_bool_field(ctx, fields, "allow_non_standard_methods", function_name) {
        Ok(Some(value)) => config.allow_non_standard_methods = value,
        Ok(None) => {}
        Err(e) => return Err(e),
    }

    Ok(config)
}

fn new_client_object(
    ctx: &mut PengNativeFunctionCallContext,
    config: HttpClientConfig,
) -> Result<PengBindedCell, PengError> {
    let timeout_cell = match config.timeout {
        Some(timeout) => PengBindedCell::Mutable(PengCell::Uint(timeout)),
        None => PengBindedCell::Mutable(PengCell::Nil),
    };

    let send_ptr = ctx.create_box(PengBox::Function(PengFunction::new_native(client_send)));

    utils::new_object(
        ctx,
        vec![
            ("timeout", timeout_cell),
            (
                "max_redirects",
                PengBindedCell::Mutable(PengCell::Uint(config.max_redirects)),
            ),
            (
                "max_redirects_will_error",
                PengBindedCell::Mutable(PengCell::Bool(config.max_redirects_will_error)),
            ),
            (
                "https_only",
                PengBindedCell::Mutable(PengCell::Bool(config.https_only)),
            ),
            (
                "status_as_error",
                PengBindedCell::Mutable(PengCell::Bool(config.status_as_error)),
            ),
            (
                "allow_non_standard_methods",
                PengBindedCell::Mutable(PengCell::Bool(config.allow_non_standard_methods)),
            ),
            (
                "send",
                PengBindedCell::Immutable(PengCell::Reference(send_ptr)),
            ),
        ],
    )
}

fn request_object(
    ctx: &mut PengNativeFunctionCallContext,
    method: String,
    url: String,
    body: PengBindedCell,
    headers: HashMap<PengNamePoolPtr, PengBindedCell>,
) -> Result<PengBindedCell, PengError> {
    let method_cell = string_cell(ctx, method);
    let url_cell = string_cell(ctx, url);
    let headers_cell = object_from_fields(ctx, headers);

    utils::new_object(
        ctx,
        vec![
            ("method", method_cell),
            ("url", url_cell),
            ("headers", headers_cell),
            ("body", body),
        ],
    )
}

fn rebuild_request_with_headers(
    ctx: &mut PengNativeFunctionCallContext,
    request_fields: HashMap<PengNamePoolPtr, PengBindedCell>,
    headers_fields: HashMap<PengNamePoolPtr, PengBindedCell>,
    function_name: &str,
) -> Result<PengBindedCell, PengError> {
    let method = match get_required_string_field(ctx, &request_fields, "method", function_name) {
        Ok(method) => method,
        Err(e) => return Err(e),
    };

    let url = match get_required_string_field(ctx, &request_fields, "url", function_name) {
        Ok(url) => url,
        Err(e) => return Err(e),
    };

    let body = match get_field(ctx, &request_fields, "body") {
        Some(body) => body,
        None => PengBindedCell::Mutable(PengCell::Nil),
    };

    request_object(ctx, method, url, body, headers_fields)
}

fn get_headers_from_request(
    ctx: &mut PengNativeFunctionCallContext,
    request_fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    function_name: &str,
) -> Result<HashMap<PengNamePoolPtr, PengBindedCell>, PengError> {
    match get_field(ctx, request_fields, "headers") {
        Some(headers) => get_object_fields_from_cell(ctx, &headers, function_name),
        None => Ok(HashMap::new()),
    }
}

fn get_optional_body_from_request(
    ctx: &mut PengNativeFunctionCallContext,
    request_fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    function_name: &str,
) -> Result<Option<String>, PengError> {
    match get_field(ctx, request_fields, "body") {
        Some(body) => match body.value() {
            PengCell::Nil => Ok(None),
            _ => match cell_to_string(ctx, &body, function_name) {
                Ok(body) => Ok(Some(body)),
                Err(e) => Err(e),
            },
        },

        None => Ok(None),
    }
}

fn fields_to_string_pairs(
    ctx: &PengNativeFunctionCallContext,
    fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    function_name: &str,
) -> Result<Vec<(String, String)>, PengError> {
    let mut pairs = Vec::new();

    for (name_ptr, value) in fields.iter() {
        let name = match ctx.env().get_pooled_name(*name_ptr) {
            Some(name) => name.clone(),
            None => {
                return Err(PengError::CannotCallValue(format!(
                    "http:{}() got missing pooled header name",
                    function_name
                )));
            }
        };

        let value = match cell_to_string(ctx, value, function_name) {
            Ok(value) => value,
            Err(e) => return Err(e),
        };

        pairs.push((name, value));
    }

    Ok(pairs)
}

fn get_required_string_field(
    ctx: &mut PengNativeFunctionCallContext,
    fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    name: &str,
    function_name: &str,
) -> Result<String, PengError> {
    match get_field(ctx, fields, name) {
        Some(value) => cell_to_string(ctx, &value, function_name),
        None => Err(PengError::CannotCallValue(format!(
            "http:{}() missing '{}' field",
            function_name, name
        ))),
    }
}

fn get_optional_uint_field(
    ctx: &mut PengNativeFunctionCallContext,
    fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
    name: &str,
    function_name: &str,
) -> Result<Option<usize>, PengError> {
    match get_field(ctx, fields, name) {
        Some(value) => match value.value() {
            PengCell::Nil => Ok(None),
            _ => match cell_to_uint(&value, function_name) {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(e),
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
    match get_field(ctx, fields, name) {
        Some(value) => match value.value() {
            PengCell::Nil => Ok(None),
            _ => match cell_to_bool(&value, function_name) {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(e),
            },
        },

        None => Ok(None),
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
                "http:{}() missing argument at index {}",
                function_name, index
            )));
        }
    };

    cell_to_string(ctx, arg, function_name)
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
                "http:{}() missing argument at index {}",
                function_name, index
            )));
        }
    };

    cell_to_uint(arg, function_name)
}

fn cell_to_string(
    ctx: &PengNativeFunctionCallContext,
    cell: &PengBindedCell,
    function_name: &str,
) -> Result<String, PengError> {
    match cell.value() {
        PengCell::Reference(ptr) => match ctx.get_value(*ptr) {
            Some(PengValue::Box(PengBox::String(value))) => Ok(value.clone()),

            Some(_) => Err(PengError::CannotCallValue(format!(
                "http:{}() expected string value",
                function_name
            ))),

            None => Err(PengError::CannotCallValue(format!(
                "http:{}() got missing heap string value",
                function_name
            ))),
        },

        _ => Err(PengError::CannotCallValue(format!(
            "http:{}() expected string value",
            function_name
        ))),
    }
}

fn cell_to_uint(cell: &PengBindedCell, function_name: &str) -> Result<usize, PengError> {
    match cell.value() {
        PengCell::Uint(value) => Ok(*value),

        PengCell::Int(value) => {
            if *value < 0 {
                return Err(PengError::CannotCallValue(format!(
                    "http:{}() expected non-negative integer value",
                    function_name
                )));
            }

            Ok(*value as usize)
        }

        PengCell::Byte(value) => Ok(*value as usize),

        _ => Err(PengError::CannotCallValue(format!(
            "http:{}() expected unsigned integer value",
            function_name
        ))),
    }
}

fn cell_to_bool(cell: &PengBindedCell, function_name: &str) -> Result<bool, PengError> {
    match cell.value() {
        PengCell::Bool(value) => Ok(*value),

        _ => Err(PengError::CannotCallValue(format!(
            "http:{}() expected bool value",
            function_name
        ))),
    }
}

fn get_object_fields_from_cell(
    ctx: &PengNativeFunctionCallContext,
    cell: &PengBindedCell,
    function_name: &str,
) -> Result<HashMap<PengNamePoolPtr, PengBindedCell>, PengError> {
    match cell.value() {
        PengCell::Reference(ptr) => match ctx.get_value(*ptr) {
            Some(PengValue::Box(PengBox::Object(object))) => Ok(object.fields.clone()),

            Some(_) => Err(PengError::CannotCallValue(format!(
                "http:{}() expected object value",
                function_name
            ))),

            None => Err(PengError::CannotCallValue(format!(
                "http:{}() got missing heap object value",
                function_name
            ))),
        },

        _ => Err(PengError::CannotCallValue(format!(
            "http:{}() expected object value",
            function_name
        ))),
    }
}

fn nil() -> Result<PengBindedCell, PengError> {
    Ok(PengBindedCell::Mutable(PengCell::Nil))
}

fn string(ctx: &mut PengNativeFunctionCallContext, value: String) -> Result<PengBindedCell, PengError> {
    Ok(string_cell(ctx, value))
}

fn string_cell(ctx: &mut PengNativeFunctionCallContext, value: String) -> PengBindedCell {
    let ptr = ctx.create_box(PengBox::String(value));

    PengBindedCell::Mutable(PengCell::Reference(ptr))
}

fn object_from_fields(
    ctx: &mut PengNativeFunctionCallContext,
    fields: HashMap<PengNamePoolPtr, PengBindedCell>,
) -> PengBindedCell {
    let ptr = ctx.create_box(PengBox::Object(PengObject { fields }));

    PengBindedCell::Mutable(PengCell::Reference(ptr))
}

fn object_from_string_pairs(
    ctx: &mut PengNativeFunctionCallContext,
    values: Vec<(String, String)>,
) -> PengBindedCell {
    let mut fields = HashMap::new();

    for (name, value) in values {
        let name_ptr = ctx.env_mut().ensure_pooled_name_ptr(name);
        fields.insert(name_ptr, string_cell(ctx, value));
    }

    object_from_fields(ctx, fields)
}

fn client_send(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    send(ctx)
}