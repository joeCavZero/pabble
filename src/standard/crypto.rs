use penguin::prelude::*;

use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine as _,
};
use hmac::{Hmac, KeyInit, Mac};
use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};
use rand::RngExt;

use super::utils;

pub fn setup(peng: &mut PengEnv) -> PengUnit {
    let mut module = PengUnit::library();

    module.register_immutable_native_function(peng, "sha1", sha1).unwrap();
    module.register_immutable_native_function(peng, "sha256", sha256).unwrap();
    module.register_immutable_native_function(peng, "sha384", sha384).unwrap();
    module.register_immutable_native_function(peng, "sha512", sha512).unwrap();
    module.register_immutable_native_function(peng, "md5", md5).unwrap();

    module.register_immutable_native_function(peng, "hmac_sha256", hmac_sha256).unwrap();
    module.register_immutable_native_function(peng, "hmac_sha512", hmac_sha512).unwrap();
    module.register_immutable_native_function(peng, "verify_hmac_sha256", verify_hmac_sha256).unwrap();
    module.register_immutable_native_function(peng, "verify_hmac_sha512", verify_hmac_sha512).unwrap();

    module.register_immutable_native_function(peng, "base64_encode", base64_encode).unwrap();
    module.register_immutable_native_function(peng, "base64_decode", base64_decode).unwrap();
    module.register_immutable_native_function(peng, "base64_decode_string", base64_decode_string).unwrap();

    module.register_immutable_native_function(peng, "base64_url_encode", base64_url_encode).unwrap();
    module.register_immutable_native_function(peng, "base64_url_decode", base64_url_decode).unwrap();
    module.register_immutable_native_function(peng, "base64_url_decode_string", base64_url_decode_string).unwrap();

    module.register_immutable_native_function(peng, "hex_encode", hex_encode).unwrap();
    module.register_immutable_native_function(peng, "hex_decode", hex_decode).unwrap();
    module.register_immutable_native_function(peng, "hex_decode_string", hex_decode_string).unwrap();

    module.register_immutable_native_function(peng, "random_bytes", random_bytes).unwrap();
    module.register_immutable_native_function(peng, "random_hex", random_hex).unwrap();
    module.register_immutable_native_function(peng, "random_base64", random_base64).unwrap();
    module.register_immutable_native_function(peng, "uuid_v4", uuid_v4).unwrap();

    module.register_immutable_native_function(peng, "equals", equals).unwrap();

    module
}

fn sha1(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    digest_function::<Sha1>(ctx, "sha1")
}

fn sha256(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    digest_function::<Sha256>(ctx, "sha256")
}

fn sha384(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    digest_function::<Sha384>(ctx, "sha384")
}

fn sha512(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    digest_function::<Sha512>(ctx, "sha512")
}

fn md5(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    digest_function::<Md5>(ctx, "md5")
}

fn hmac_sha256(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let key = match get_bytes_arg(ctx, 0, "hmac_sha256") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let message = match get_bytes_arg(ctx, 1, "hmac_sha256") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let bytes = match hmac_sha256_bytes(&key, &message) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    utils::string(ctx, bytes_to_hex(&bytes))
}

fn hmac_sha512(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let key = match get_bytes_arg(ctx, 0, "hmac_sha512") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let message = match get_bytes_arg(ctx, 1, "hmac_sha512") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let bytes = match hmac_sha512_bytes(&key, &message) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    utils::string(ctx, bytes_to_hex(&bytes))
}

fn verify_hmac_sha256(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let key = match get_bytes_arg(ctx, 0, "verify_hmac_sha256") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let message = match get_bytes_arg(ctx, 1, "verify_hmac_sha256") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let expected = match utils::get_string_arg(ctx, 2) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let expected_bytes = match hex_to_bytes(&expected, "verify_hmac_sha256") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let actual_bytes = match hmac_sha256_bytes(&key, &message) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    utils::bool_cell(constant_time_eq(&actual_bytes, &expected_bytes))
}

fn verify_hmac_sha512(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let key = match get_bytes_arg(ctx, 0, "verify_hmac_sha512") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let message = match get_bytes_arg(ctx, 1, "verify_hmac_sha512") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let expected = match utils::get_string_arg(ctx, 2) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let expected_bytes = match hex_to_bytes(&expected, "verify_hmac_sha512") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let actual_bytes = match hmac_sha512_bytes(&key, &message) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    utils::bool_cell(constant_time_eq(&actual_bytes, &expected_bytes))
}

fn base64_encode(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let bytes = match get_bytes_arg(ctx, 0, "base64_encode") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    utils::string(ctx, STANDARD.encode(bytes))
}

fn base64_decode(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let text = match utils::get_string_arg(ctx, 0) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let bytes = match STANDARD.decode(text.as_bytes()) {
        Ok(value) => value,
        Err(e) => {
            return Err(crypto_error(
                "base64_decode",
                format!("invalid base64 input: {}", e),
            ));
        }
    };

    bytes_to_vector(ctx, bytes)
}

fn base64_decode_string(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let text = match utils::get_string_arg(ctx, 0) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let bytes = match STANDARD.decode(text.as_bytes()) {
        Ok(value) => value,
        Err(e) => {
            return Err(crypto_error(
                "base64_decode_string",
                format!("invalid base64 input: {}", e),
            ));
        }
    };

    let decoded = match String::from_utf8(bytes) {
        Ok(value) => value,
        Err(e) => {
            return Err(crypto_error(
                "base64_decode_string",
                format!("decoded data is not valid UTF-8: {}", e),
            ));
        }
    };

    utils::string(ctx, decoded)
}

fn base64_url_encode(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let bytes = match get_bytes_arg(ctx, 0, "base64_url_encode") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    utils::string(ctx, URL_SAFE_NO_PAD.encode(bytes))
}

fn base64_url_decode(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let text = match utils::get_string_arg(ctx, 0) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let bytes = match URL_SAFE_NO_PAD.decode(text.as_bytes()) {
        Ok(value) => value,
        Err(e) => {
            return Err(crypto_error(
                "base64_url_decode",
                format!("invalid base64url input: {}", e),
            ));
        }
    };

    bytes_to_vector(ctx, bytes)
}

fn base64_url_decode_string(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let text = match utils::get_string_arg(ctx, 0) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let bytes = match URL_SAFE_NO_PAD.decode(text.as_bytes()) {
        Ok(value) => value,
        Err(e) => {
            return Err(crypto_error(
                "base64_url_decode_string",
                format!("invalid base64url input: {}", e),
            ));
        }
    };

    let decoded = match String::from_utf8(bytes) {
        Ok(value) => value,
        Err(e) => {
            return Err(crypto_error(
                "base64_url_decode_string",
                format!("decoded data is not valid UTF-8: {}", e),
            ));
        }
    };

    utils::string(ctx, decoded)
}

fn hex_encode(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let bytes = match get_bytes_arg(ctx, 0, "hex_encode") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    utils::string(ctx, bytes_to_hex(&bytes))
}

fn hex_decode(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let text = match utils::get_string_arg(ctx, 0) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let bytes = match hex_to_bytes(&text, "hex_decode") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    bytes_to_vector(ctx, bytes)
}

fn hex_decode_string(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let text = match utils::get_string_arg(ctx, 0) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let bytes = match hex_to_bytes(&text, "hex_decode_string") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let decoded = match String::from_utf8(bytes) {
        Ok(value) => value,
        Err(e) => {
            return Err(crypto_error(
                "hex_decode_string",
                format!("decoded data is not valid UTF-8: {}", e),
            ));
        }
    };

    utils::string(ctx, decoded)
}

fn random_bytes(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let len = match utils::get_uint_arg(ctx, 0) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let bytes = match secure_random_bytes(len, "random_bytes") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    bytes_to_vector(ctx, bytes)
}

fn random_hex(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let len = match utils::get_uint_arg(ctx, 0) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let bytes = match secure_random_bytes(len, "random_hex") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    utils::string(ctx, bytes_to_hex(&bytes))
}

fn random_base64(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let len = match utils::get_uint_arg(ctx, 0) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let bytes = match secure_random_bytes(len, "random_base64") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    utils::string(ctx, STANDARD.encode(bytes))
}

fn uuid_v4(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let mut bytes = [0u8; 16];

    match fill_random_bytes(&mut bytes) {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    }

    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    utils::string(ctx, uuid_v4_string(&bytes))
}

fn equals(ctx: &mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> {
    let left = match get_bytes_arg(ctx, 0, "equals") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let right = match get_bytes_arg(ctx, 1, "equals") {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    utils::bool_cell(constant_time_eq(&left, &right))
}

fn digest_function<D>(
    ctx: &mut PengNativeFunctionCallContext,
    function_name: &str,
) -> Result<PengBindedCell, PengError>
where
    D: Digest,
{
    let bytes = match get_bytes_arg(ctx, 0, function_name) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    let mut hasher = D::new();
    hasher.update(&bytes);
    let result = hasher.finalize();

    utils::string(ctx, bytes_to_hex(result.as_ref()))
}

fn hmac_sha256_bytes(key: &[u8], message: &[u8]) -> Result<Vec<u8>, PengError> {
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = match HmacSha256::new_from_slice(key) {
        Ok(value) => value,
        Err(e) => {
            return Err(crypto_error(
                "hmac_sha256",
                format!("invalid key: {}", e),
            ));
        }
    };

    mac.update(message);

    Ok(mac.finalize().into_bytes().to_vec())
}

fn hmac_sha512_bytes(key: &[u8], message: &[u8]) -> Result<Vec<u8>, PengError> {
    type HmacSha512 = Hmac<Sha512>;

    let mut mac = match HmacSha512::new_from_slice(key) {
        Ok(value) => value,
        Err(e) => {
            return Err(crypto_error(
                "hmac_sha512",
                format!("invalid key: {}", e),
            ));
        }
    };

    mac.update(message);

    Ok(mac.finalize().into_bytes().to_vec())
}

fn secure_random_bytes(len: usize, function_name: &str) -> Result<Vec<u8>, PengError> {
    let mut bytes = vec![0u8; len];

    match fill_random_bytes(&mut bytes) {
        Ok(_) => Ok(bytes),
        Err(e) => Err(e.push(PengError::InternalError(function_name.into()))),
    }
}

fn get_bytes_arg(
    ctx: &PengNativeFunctionCallContext,
    index: usize,
    function_name: &str,
) -> Result<Vec<u8>, PengError> {
    let arg = match ctx.get_arg_cell(index) {
        Some(arg) => arg.clone(),
        None => {
            return Err(crypto_error(
                function_name,
                format!("missing argument at index {}", index),
            ));
        }
    };

    cell_to_bytes(ctx, &arg, function_name)
}

fn cell_to_bytes(
    ctx: &PengNativeFunctionCallContext,
    cell: &PengBindedCell,
    function_name: &str,
) -> Result<Vec<u8>, PengError> {
    match cell.value() {
        PengCell::Reference(ptr) => match ctx.get_value(*ptr) {
            Some(PengValue::Box(PengBox::String(value))) => Ok(value.as_bytes().to_vec()),

            Some(PengValue::Box(PengBox::Vector(vector))) => {
                vector_to_bytes(ctx, &vector.values, function_name)
            }

            Some(PengValue::Cell(value)) => cell_value_to_bytes(ctx, value, function_name),

            Some(_) => Err(crypto_error(
                function_name,
                "expected string or byte vector argument".to_string(),
            )),

            None => Err(PengError::HeapValueNotFound(*ptr)),
        },

        cell => cell_value_to_bytes(ctx, cell, function_name),
    }
}

fn cell_value_to_bytes(
    _ctx: &PengNativeFunctionCallContext,
    cell: &PengCell,
    function_name: &str,
) -> Result<Vec<u8>, PengError> {
    match cell {
        PengCell::Byte(value) => Ok(vec![*value]),

        _ => Err(crypto_error(
            function_name,
            "expected string or byte vector argument".to_string(),
        )),
    }
}

fn vector_to_bytes(
    ctx: &PengNativeFunctionCallContext,
    values: &[PengBindedCell],
    function_name: &str,
) -> Result<Vec<u8>, PengError> {
    let mut bytes = Vec::with_capacity(values.len());

    for value in values {
        let byte = match cell_to_byte(ctx, value, function_name) {
            Ok(value) => value,
            Err(e) => return Err(e),
        };

        bytes.push(byte);
    }

    Ok(bytes)
}

fn cell_to_byte(
    ctx: &PengNativeFunctionCallContext,
    cell: &PengBindedCell,
    function_name: &str,
) -> Result<u8, PengError> {
    match cell.value() {
        PengCell::Byte(value) => Ok(*value),

        PengCell::Uint(value) => {
            if *value > u8::MAX as usize {
                return Err(crypto_error(
                    function_name,
                    "byte vector value must be between 0 and 255".to_string(),
                ));
            }

            Ok(*value as u8)
        }

        PengCell::Int(value) => {
            if *value < 0 || *value > u8::MAX as isize {
                return Err(crypto_error(
                    function_name,
                    "byte vector value must be between 0 and 255".to_string(),
                ));
            }

            Ok(*value as u8)
        }

        PengCell::Reference(ptr) => match ctx.get_value(*ptr) {
            Some(PengValue::Cell(value)) => {
                let binded = PengBindedCell::Mutable(value.clone());
                cell_to_byte(ctx, &binded, function_name)
            }

            Some(_) => Err(crypto_error(
                function_name,
                "byte vector only accepts byte, int or uint values".to_string(),
            )),

            None => Err(PengError::HeapValueNotFound(*ptr)),
        },

        _ => Err(crypto_error(
            function_name,
            "byte vector only accepts byte, int or uint values".to_string(),
        )),
    }
}

fn bytes_to_vector(
    ctx: &mut PengNativeFunctionCallContext,
    bytes: Vec<u8>,
) -> Result<PengBindedCell, PengError> {
    let mut values = Vec::with_capacity(bytes.len());

    for byte in bytes {
        values.push(PengBindedCell::Mutable(PengCell::Byte(byte)));
    }

    utils::vector(ctx, values)
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let mut output = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }

    output
}

fn hex_to_bytes(text: &str, function_name: &str) -> Result<Vec<u8>, PengError> {
    let clean = text.trim();

    if clean.len() % 2 != 0 {
        return Err(crypto_error(
            function_name,
            "hex input length must be even".to_string(),
        ));
    }

    let mut bytes = Vec::with_capacity(clean.len() / 2);
    let raw = clean.as_bytes();
    let mut i = 0usize;

    while i < raw.len() {
        let high = match hex_value(raw[i]) {
            Some(value) => value,
            None => {
                return Err(crypto_error(
                    function_name,
                    format!("invalid hex character at index {}", i),
                ));
            }
        };

        let low = match hex_value(raw[i + 1]) {
            Some(value) => value,
            None => {
                return Err(crypto_error(
                    function_name,
                    format!("invalid hex character at index {}", i + 1),
                ));
            }
        };

        bytes.push((high << 4) | low);
        i += 2;
    }

    Ok(bytes)
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    let mut diff = 0u8;

    for i in 0..left.len() {
        diff |= left[i] ^ right[i];
    }

    diff == 0
}

fn uuid_v4_string(bytes: &[u8; 16]) -> String {
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    )
}

fn crypto_error(function_name: &str, message: String) -> PengError {
    PengError::CannotCallValue(format!("crypto:{}() {}", function_name, message))
}

fn fill_random_bytes(bytes: &mut [u8]) -> Result<(), PengError> {
    let mut rng = rand::rng();

    rng.fill(bytes);

    Ok(())
}