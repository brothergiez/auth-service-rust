use axum::http::header::HeaderMap;
use axum::http::HeaderName;
use serde_json::{Map, Value};

const SENSITIVE_HEADERS: &[&str] = &[
    "authorization",
    "cookie",
    "set-cookie",
    "x-api-key",
    "x-api-key-secret",
    "proxy-authorization",
    "x-auth-token",
    "x-refresh-token",
    "x-forwarded-authorization",
];

const SENSITIVE_JSON_KEYS: &[&str] = &[
    "password",
    "passwd",
    "current_password",
    "new_password",
    "old_password",
    "secret",
    "token",
    "access_token",
    "refresh_token",
    "id_token",
    "jwt",
    "jwt_secret",
    "api_key",
    "apikey",
    "client_secret",
    "authorization",
    "credit_card",
    "card_number",
    "cvv",
    "ssn",
];

pub(crate) const MAX_LOG_BODY_CHARS: usize = 16_384;

fn header_name_lc(name: &HeaderName) -> String {
    name.as_str().to_ascii_lowercase()
}

fn is_sensitive_header(name: &str) -> bool {
    SENSITIVE_HEADERS.iter().any(|&h| h == name)
}

/// Request/response headers as a JSON object (lowercase keys), sensitive values `"<redacted>"`.
pub(crate) fn headers_to_redacted_json(headers: &HeaderMap) -> Value {
    let mut map = Map::new();
    for (name, value) in headers.iter() {
        let key = header_name_lc(name);
        let val = if is_sensitive_header(&key) {
            "<redacted>".to_string()
        } else {
            value.to_str().unwrap_or("<binary>").to_string()
        };
        map.insert(key, Value::String(val));
    }
    Value::Object(map)
}

fn is_sensitive_json_key(key: &str) -> bool {
    let k = key.to_ascii_lowercase();
    SENSITIVE_JSON_KEYS.iter().any(|&sk| sk == k)
}

fn redact_json_value(v: Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, val) in map {
                if is_sensitive_json_key(&k) {
                    out.insert(k, Value::String("<redacted>".into()));
                } else {
                    out.insert(k, redact_json_value(val));
                }
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(redact_json_value).collect()),
        other => other,
    }
}

fn truncate_chars(s: String, max: usize) -> String {
    if s.chars().count() <= max {
        return s;
    }
    let truncated: String = s.chars().take(max).collect();
    format!("{truncated}… <truncated>")
}

pub(crate) fn body_to_log_value(content_type: Option<&str>, bytes: &[u8]) -> Value {
    if bytes.is_empty() {
        return Value::Null;
    }

    let ct = content_type.unwrap_or("").to_ascii_lowercase();
    if ct.contains("application/json") {
        match serde_json::from_slice::<Value>(bytes) {
            Ok(v) => redact_json_value(v),
            Err(_) => Value::String(truncate_chars(
                String::from_utf8_lossy(bytes).into_owned(),
                MAX_LOG_BODY_CHARS,
            )),
        }
    } else if ct.starts_with("text/")
        || ct.contains("javascript")
        || ct.contains("xml")
    {
        Value::String(truncate_chars(
            String::from_utf8_lossy(bytes).into_owned(),
            MAX_LOG_BODY_CHARS,
        ))
    } else {
        Value::String(format!(
            "<non-text body: {} bytes, content-type={}>",
            bytes.len(),
            content_type.unwrap_or("?")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::header;

    #[test]
    fn redacts_authorization_header_in_json_object() {
        let mut map = HeaderMap::new();
        map.insert(
            header::AUTHORIZATION,
            axum::http::HeaderValue::from_static("Bearer secret-token"),
        );
        map.insert(
            header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("application/json"),
        );
        let v = headers_to_redacted_json(&map);
        let obj = v.as_object().unwrap();
        assert_eq!(
            obj.get("authorization").unwrap().as_str().unwrap(),
            "<redacted>"
        );
        assert_eq!(
            obj.get("content-type").unwrap().as_str().unwrap(),
            "application/json"
        );
    }

    #[test]
    fn redacts_password_in_request_body_object() {
        let raw = br#"{"email":"a@b.com","password":"hunter2"}"#;
        let v = body_to_log_value(Some("application/json"), raw);
        let obj = v.as_object().unwrap();
        assert_eq!(obj.get("password").unwrap().as_str().unwrap(), "<redacted>");
        assert_eq!(obj.get("email").unwrap().as_str().unwrap(), "a@b.com");
    }
}
