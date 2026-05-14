use std::time::Instant;

use axum::body::{to_bytes, Body};
use axum::extract::Request;
use axum::http::header;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::{json, Value};

use super::redact::{body_to_log_value, headers_to_redacted_json};

/// Max body size buffered for logging (and re-injected into the request/response).
const MAX_LOG_BODY_BYTES: usize = 256 * 1024;

fn omit_response_body_capture(path: &str) -> bool {
    path.starts_with("/swagger-ui") || path.starts_with("/api-docs/")
}

fn emit_http_request_log(
    method: &axum::http::Method,
    path: &str,
    status: StatusCode,
    latency_ms: f64,
    request_headers: Value,
    request_body: Value,
    response_headers: Value,
    response_body: Value,
) {
    let payload = json!({
        "http_request": {
            "method": method.as_str(),
            "path": path,
            "status": status.as_u16(),
            "latency_ms": latency_ms,
            "request_headers": request_headers,
            "request_body": request_body,
            "response_headers": response_headers,
            "response_body": response_body,
        }
    });
    match serde_json::to_string(&payload) {
        Ok(line) => tracing::info!(target: "http_request", "{}", line),
        Err(e) => tracing::warn!(target: "http_request", error = %e, "failed to serialize http_request log"),
    }
}

/// Logs one JSON line: `{"http_request":{...}}` (redacted headers / sensitive JSON keys).
pub async fn log_request(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());

    let content_type_req = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);

    let req_headers_json = headers_to_redacted_json(request.headers());

    let (parts, body) = request.into_parts();

    let req_bytes = match to_bytes(body, MAX_LOG_BODY_BYTES).await {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!(error = %e, path = %path, "request body read failed or exceeds limit");
            return (StatusCode::PAYLOAD_TOO_LARGE, "request body too large for this server").into_response();
        }
    };

    let req_body_json = body_to_log_value(content_type_req.as_deref(), &req_bytes);

    let request = Request::from_parts(parts, Body::from(req_bytes));

    let start = Instant::now();
    let response = next.run(request).await;
    let status = response.status();
    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

    let resp_headers_json = headers_to_redacted_json(response.headers());

    if omit_response_body_capture(&path) {
        emit_http_request_log(
            &method,
            &path,
            status,
            latency_ms,
            req_headers_json,
            req_body_json,
            resp_headers_json,
            Value::String("<omitted>".into()),
        );
        return response;
    }

    let (parts, body) = response.into_parts();
    let content_type_resp = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);

    let resp_bytes = match to_bytes(body, MAX_LOG_BODY_BYTES).await {
        Ok(b) => b,
        Err(e) => {
            tracing::error!(
                error = %e,
                path = %path,
                "response body read failed — returning 500"
            );
            return (StatusCode::INTERNAL_SERVER_ERROR, "response logging buffer failed").into_response();
        }
    };

    let resp_body_json = body_to_log_value(content_type_resp.as_deref(), &resp_bytes);

    emit_http_request_log(
        &method,
        &path,
        status,
        latency_ms,
        req_headers_json,
        req_body_json,
        resp_headers_json,
        resp_body_json,
    );

    Response::from_parts(parts, Body::from(resp_bytes))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn log_payload_wraps_http_request() {
        use serde_json::Map;

        let line = serde_json::to_string(&json!({
            "http_request": {
                "method": "POST",
                "path": "/api/v1/auth/login",
                "status": 200u16,
                "latency_ms": 1.5f64,
                "request_headers": Map::new(),
                "request_body": serde_json::Value::Null,
                "response_headers": Map::new(),
                "response_body": serde_json::Value::Null,
            }
        }))
        .unwrap();
        assert!(line.starts_with("{\"http_request\":"));
    }
}
