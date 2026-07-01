use std::collections::HashMap;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::thread;

use penguin::prelude::*;

use super::utils;

use ureq::ResponseExt;
use ureq::http::Method;

#[derive(Debug, Clone)]
struct HttpRequestData {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

#[derive(Debug, Clone)]
struct HttpResponseData {
    status: usize,
    ok: bool,
    reason: String,
    headers: Vec<(String, String)>,
    body: String,
    url: String,
}

#[derive(Debug, Clone)]
enum HttpTaskState {
    Running,
    Finished(Result<HttpResponseData, String>),
}

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

    let client_type = client_type_value(peng);
    module.register_immutable_global(peng, "Client", client_type).unwrap();

    let request_type = request_type_value(peng);
    module.register_immutable_global(peng, "Request", request_type).unwrap();

    module.register_immutable_native_function(peng, "send", send).unwrap();

    module
        .register_immutable_native_function(peng, "set_header", set_header)
        .unwrap();
    module
        .register_immutable_native_function(peng, "get_header", get_header)
        .unwrap();
    module
        .register_immutable_native_function(peng, "remove_header", remove_header)
        .unwrap();

    module
        .register_immutable_native_function(peng, "status_text", status_text)
        .unwrap();

    module
}

fn client_type_value(peng: &mut PengEnv) -> PengValue {
    let mut fields = HashMap::new();

    let send_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(client_send),
    )));

    let send_async_ptr = peng.create_heap_value(PengValue::Box(PengBox::Function(
        PengFunction::new_native(client_send_async),
    )));

    fields.insert(
        peng.ensure_pooled_name_ptr("timeout".to_string()),
        PengBindedCell::Mutable(PengCell::Uint(30000)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("max_redirects".to_string()),
        PengBindedCell::Mutable(PengCell::Uint(10)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("https_only".to_string()),
        PengBindedCell::Mutable(PengCell::Bool(false)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("send".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(send_ptr)),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("send_async".to_string()),
        PengBindedCell::Immutable(PengCell::Reference(send_async_ptr)),
    );

    PengValue::Box(PengBox::Type(PengType::Custom(PengCustomType {
        fields,
    })))
}

fn request_type_value(peng: &mut PengEnv) -> PengValue {
    let mut fields = HashMap::new();

    let method = utils::string_binded_cell_from_env(peng, "GET".to_string());
    let url = utils::string_binded_cell_from_env(peng, "".to_string());

    let headers_ptr = peng.create_heap_value(PengValue::Box(PengBox::Object(
        PengObject {
            fields: HashMap::new(),
        },
    )));

    fields.insert(
        peng.ensure_pooled_name_ptr("method".to_string()),
        method,
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("url".to_string()),
        url,
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("body".to_string()),
        PengBindedCell::Mutable(PengCell::Nil),
    );

    fields.insert(
        peng.ensure_pooled_name_ptr("headers".to_string()),
        PengBindedCell::Mutable(PengCell::Reference(headers_ptr)),
    );

    PengValue::Box(PengBox::Type(PengType::Custom(PengCustomType {
        fields,
    })))
}

fn send(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let (client_config, request_data) = match get_send_data(ctx, "send") {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    let response = match execute_request_data(client_config, request_data, "send") {
        Ok(response) => response,
        Err(e) => return Err(PengError::CannotCallValue(e)),
    };

    response_data_to_object(ctx, response)
}

fn send_async(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let (client_config, request_data) = match get_send_data(ctx, "send_async") {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    let state = Arc::new(Mutex::new(HttpTaskState::Running));
    let thread_state = state.clone();

    thread::spawn(move || {
        let result = execute_request_data(client_config, request_data, "send_async");

        match thread_state.lock() {
            Ok(mut locked) => {
                *locked = HttpTaskState::Finished(result);
            }

            Err(_) => {}
        }
    });

    task_object(ctx, state)
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

    let name = match utils::get_string_arg(ctx, 1) {
        Ok(name) => name,
        Err(e) => return Err(e),
    };

    let value = match utils::get_string_arg(ctx, 2) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let request_fields = match utils::get_object_fields_from_cell(ctx, &request_arg) {
        Ok(fields) => fields,
        Err(e) => return Err(e),
    };

    let mut headers_fields = match get_headers_from_request(ctx, &request_fields) {
        Ok(fields) => fields,
        Err(e) => return Err(e),
    };

    let name_ptr = ctx.env_mut().ensure_pooled_name_ptr(name);
    headers_fields.insert(name_ptr, utils::string_cell(ctx, value));

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

    let name = match utils::get_string_arg(ctx, 1) {
        Ok(name) => name,
        Err(e) => return Err(e),
    };

    let request_fields = match utils::get_object_fields_from_cell(ctx, &request_arg) {
        Ok(fields) => fields,
        Err(e) => return Err(e),
    };

    let headers_fields = match get_headers_from_request(ctx, &request_fields) {
        Ok(fields) => fields,
        Err(e) => return Err(e),
    };

    let name_ptr = ctx.env_mut().ensure_pooled_name_ptr(name);

    match headers_fields.get(&name_ptr) {
        Some(value) => match utils::cell_to_string(ctx, value) {
            Ok(value) => utils::string(ctx, value),
            Err(e) => Err(e),
        },

        None => utils::nil(),
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

    let name = match utils::get_string_arg(ctx, 1) {
        Ok(name) => name,
        Err(e) => return Err(e),
    };

    let request_fields = match utils::get_object_fields_from_cell(ctx, &request_arg) {
        Ok(fields) => fields,
        Err(e) => return Err(e),
    };

    let mut headers_fields = match get_headers_from_request(ctx, &request_fields) {
        Ok(fields) => fields,
        Err(e) => return Err(e),
    };

    let name_ptr = ctx.env_mut().ensure_pooled_name_ptr(name);
    headers_fields.remove(&name_ptr);

    rebuild_request_with_headers(ctx, request_fields, headers_fields, "remove_header")
}

fn status_text(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let code = match utils::get_uint_arg(ctx, 0) {
        Ok(code) => code,
        Err(e) => return Err(e),
    };

    if code > u16::MAX as usize {
        return utils::nil();
    }

    let status = match ureq::http::StatusCode::from_u16(code as u16) {
        Ok(status) => status,
        Err(_) => return utils::nil(),
    };

    match status.canonical_reason() {
        Some(reason) => utils::string(ctx, reason.to_string()),
        None => utils::nil(),
    }
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
) -> Result<HttpClientConfig, PengError> {
    let mut config = HttpClientConfig::default();

    match get_optional_uint_field(ctx, fields, "timeout") {
        Ok(value) => config.timeout = value,
        Err(e) => return Err(e),
    }

    match get_optional_uint_field(ctx, fields, "max_redirects") {
        Ok(Some(value)) => config.max_redirects = value,
        Ok(None) => {}
        Err(e) => return Err(e),
    }

    match get_optional_bool_field(ctx, fields, "max_redirects_will_error") {
        Ok(Some(value)) => config.max_redirects_will_error = value,
        Ok(None) => {}
        Err(e) => return Err(e),
    }

    match get_optional_bool_field(ctx, fields, "https_only") {
        Ok(Some(value)) => config.https_only = value,
        Ok(None) => {}
        Err(e) => return Err(e),
    }

    match get_optional_bool_field(ctx, fields, "status_as_error") {
        Ok(Some(value)) => config.status_as_error = value,
        Ok(None) => {}
        Err(e) => return Err(e),
    }

    match get_optional_bool_field(ctx, fields, "allow_non_standard_methods") {
        Ok(Some(value)) => config.allow_non_standard_methods = value,
        Ok(None) => {}
        Err(e) => return Err(e),
    }

    Ok(config)
}

fn request_object(
    ctx: &mut PengNativeFunctionCallContext,
    method: String,
    url: String,
    body: PengBindedCell,
    headers: HashMap<PengNamePoolPtr, PengBindedCell>,
) -> Result<PengBindedCell, PengError> {
    let method_cell = utils::string_cell(ctx, method);
    let url_cell = utils::string_cell(ctx, url);
    let headers_cell = utils::object_from_fields(ctx, headers);

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

    let body = match utils::get_map_field(ctx, &request_fields, "body") {
        Some(body) => body,
        None => PengBindedCell::Mutable(PengCell::Nil),
    };

    request_object(ctx, method, url, body, headers_fields)
}

fn get_headers_from_request(
    ctx: &mut PengNativeFunctionCallContext,
    request_fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
) -> Result<HashMap<PengNamePoolPtr, PengBindedCell>, PengError> {
    match utils::get_map_field(ctx, request_fields, "headers") {
        Some(headers) => utils::get_object_fields_from_cell(ctx, &headers),
        None => Ok(HashMap::new()),
    }
}

fn get_optional_body_from_request(
    ctx: &mut PengNativeFunctionCallContext,
    request_fields: &HashMap<PengNamePoolPtr, PengBindedCell>,
) -> Result<Option<String>, PengError> {
    match utils::get_map_field(ctx, request_fields, "body") {
        Some(body) => match body.value() {
            PengCell::Nil => Ok(None),
            _ => match utils::cell_to_string(ctx, &body) {
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

        let value = match utils::cell_to_string(ctx, value) {
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
    match utils::get_map_field(ctx, fields, name) {
        Some(value) => utils::cell_to_string(ctx, &value),
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
) -> Result<Option<usize>, PengError> {
    match utils::get_map_field(ctx, fields, name) {
        Some(value) => match value.value() {
            PengCell::Nil => Ok(None),
            _ => match utils::cell_to_uint(&value) {
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
) -> Result<Option<bool>, PengError> {
    match utils::get_map_field(ctx, fields, name) {
        Some(value) => match value.value() {
            PengCell::Nil => Ok(None),
            _ => match utils::cell_to_bool(&value) {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(e),
            },
        },

        None => Ok(None),
    }
}

fn client_send(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    send(ctx)
}

fn client_send_async(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    send_async(ctx)
}

fn get_send_data(
    ctx: &mut PengNativeFunctionCallContext,
    function_name: &str,
) -> Result<(HttpClientConfig, HttpRequestData), PengError> {
    let first_arg = match ctx.get_arg_cell(0) {
        Some(arg) => arg.clone(),
        None => {
            return Err(PengError::CannotCallValue(format!(
                "http:{}() missing request argument",
                function_name
            )));
        }
    };

    let second_arg = match ctx.get_arg_cell(1) {
        Some(arg) => Some(arg.clone()),
        None => None,
    };

    let (client_config, request_fields) = match second_arg {
        Some(request_arg) => {
            let client_fields = match utils::get_object_fields_from_cell(ctx, &first_arg) {
                Ok(fields) => fields,
                Err(e) => return Err(e),
            };

            let client_config = match client_config_from_fields(ctx, &client_fields) {
                Ok(config) => config,
                Err(e) => return Err(e),
            };

            let request_fields = match utils::get_object_fields_from_cell(ctx, &request_arg) {
                Ok(fields) => fields,
                Err(e) => return Err(e),
            };

            (client_config, request_fields)
        }

        None => {
            let request_fields = match utils::get_object_fields_from_cell(ctx, &first_arg) {
                Ok(fields) => fields,
                Err(e) => return Err(e),
            };

            (HttpClientConfig::default(), request_fields)
        }
    };

    let method = match get_required_string_field(ctx, &request_fields, "method", function_name) {
        Ok(method) => method,
        Err(e) => return Err(e),
    };

    let url = match get_required_string_field(ctx, &request_fields, "url", function_name) {
        Ok(url) => url,
        Err(e) => return Err(e),
    };

    let headers_fields = match get_headers_from_request(ctx, &request_fields) {
        Ok(fields) => fields,
        Err(e) => return Err(e),
    };

    let headers = match fields_to_string_pairs(ctx, &headers_fields, function_name) {
        Ok(headers) => headers,
        Err(e) => return Err(e),
    };

    let body = match get_optional_body_from_request(ctx, &request_fields) {
        Ok(body) => body,
        Err(e) => return Err(e),
    };

    Ok((
        client_config,
        HttpRequestData {
            method,
            url,
            headers,
            body,
        },
    ))
}

fn execute_request_data(
    client_config: HttpClientConfig,
    request_data: HttpRequestData,
    function_name: &str,
) -> Result<HttpResponseData, String> {
    let method = match Method::from_bytes(request_data.method.as_bytes()) {
        Ok(method) => method,
        Err(_) => {
            return Err(format!(
                "http:{}() invalid HTTP method '{}'",
                function_name, request_data.method
            ));
        }
    };

    let agent = build_agent(client_config);

    let mut builder = ureq::http::Request::builder()
        .method(method)
        .uri(request_data.url.clone());

    for (name, value) in request_data.headers {
        builder = builder.header(name, value);
    }

    match request_data.body {
        Some(body) => {
            let request = match builder.body(body) {
                Ok(request) => request,
                Err(e) => {
                    return Err(format!(
                        "http:{}() failed to build request: {}",
                        function_name, e
                    ));
                }
            };

            execute_built_request(&agent, request, function_name)
        }

        None => {
            let request = match builder.body(()) {
                Ok(request) => request,
                Err(e) => {
                    return Err(format!(
                        "http:{}() failed to build request: {}",
                        function_name, e
                    ));
                }
            };

            execute_built_request(&agent, request, function_name)
        }
    }
}

fn execute_built_request<S>(
    agent: &ureq::Agent,
    request: ureq::http::Request<S>,
    function_name: &str,
) -> Result<HttpResponseData, String>
where
    S: ureq::AsSendBody,
{
    let mut response = match agent.run(request) {
        Ok(response) => response,
        Err(e) => {
            return Err(format!(
                "http:{}() request failed: {}",
                function_name, e
            ));
        }
    };

    let status = response.status().as_u16() as usize;
    let ok = status >= 200 && status <= 299;

    let reason = match response.status().canonical_reason() {
        Some(reason) => reason.to_string(),
        None => String::new(),
    };

    let url = response.get_uri().to_string();

    let mut headers = Vec::new();

    for (name, value) in response.headers().iter() {
        let value = match value.to_str() {
            Ok(value) => value.to_string(),
            Err(_) => String::new(),
        };

        headers.push((name.to_string(), value));
    }

    let body = match response.body_mut().read_to_string() {
        Ok(body) => body,
        Err(e) => {
            return Err(format!(
                "http:{}() failed to read response body: {}",
                function_name, e
            ));
        }
    };

    Ok(HttpResponseData {
        status,
        ok,
        reason,
        headers,
        body,
        url,
    })
}

fn response_data_to_object(
    ctx: &mut PengNativeFunctionCallContext,
    response: HttpResponseData,
) -> Result<PengBindedCell, PengError> {
    let reason = utils::string_cell(ctx, response.reason);
    let headers = utils::object_from_string_pairs(ctx, response.headers);
    let body = utils::string_cell(ctx, response.body);
    let url = utils::string_cell(ctx, response.url);

    utils::new_object(
        ctx,
        vec![
            ("status", PengBindedCell::Mutable(PengCell::Uint(response.status))),
            ("ok", PengBindedCell::Mutable(PengCell::Bool(response.ok))),
            ("reason", reason),
            ("headers", headers),
            ("body", body),
            ("url", url),
        ],
    )
}

fn task_object(
    ctx: &mut PengNativeFunctionCallContext,
    state: Arc<Mutex<HttpTaskState>>,
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
    state: Arc<Mutex<HttpTaskState>>,
) -> Result<PengBindedCell, PengError> {
    match state.lock() {
        Ok(locked) => match &*locked {
            HttpTaskState::Running => Ok(PengBindedCell::Mutable(PengCell::Bool(false))),
            HttpTaskState::Finished(_) => Ok(PengBindedCell::Mutable(PengCell::Bool(true))),
        },

        Err(_) => Err(PengError::CannotCallValue(
            "http task lock failed".to_string(),
        )),
    }
}

fn task_get(
    ctx: &mut PengNativeFunctionCallContext,
    state: Arc<Mutex<HttpTaskState>>,
) -> Result<PengBindedCell, PengError> {
    let result = match state.lock() {
        Ok(locked) => match &*locked {
            HttpTaskState::Running => return utils::nil(),
            HttpTaskState::Finished(result) => result.clone(),
        },

        Err(_) => {
            return Err(PengError::CannotCallValue(
                "http task lock failed".to_string(),
            ));
        }
    };

    match result {
        Ok(response) => response_data_to_object(ctx, response),
        Err(e) => Err(PengError::CannotCallValue(e)),
    }
}

fn task_error(
    ctx: &mut PengNativeFunctionCallContext,
    state: Arc<Mutex<HttpTaskState>>,
) -> Result<PengBindedCell, PengError> {
    match state.lock() {
        Ok(locked) => match &*locked {
            HttpTaskState::Running => utils::nil(),

            HttpTaskState::Finished(result) => match result {
                Ok(_) => utils::nil(),
                Err(e) => utils::string(ctx, e.clone()),
            },
        },

        Err(_) => Err(PengError::CannotCallValue(
            "http task lock failed".to_string(),
        )),
    }
}
