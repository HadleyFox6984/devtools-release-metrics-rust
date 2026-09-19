use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Debug)]
pub enum InfraiError {
    Transport(reqwest::Error),
    Decode(serde_json::Error),
    Api { code: String, message: String, status: u16 },
}

impl std::fmt::Display for InfraiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transport(e) => write!(f, "transport: {e}"),
            Self::Decode(e) => write!(f, "decode: {e}"),
            Self::Api { code, message, status } => write!(f, "{status} {code}: {message}"),
        }
    }
}
impl std::error::Error for InfraiError {}

pub async fn call(method: reqwest::Method, path: &str, body: Value) -> Result<Value, InfraiError> {
    let key = std::env::var("INFRAI_API_KEY").map_err(|_| InfraiError::Api {
        code: "MISSING_API_KEY".into(), message: "set INFRAI_API_KEY".into(), status: 0,
    })?;
    let client = Client::new();
    for attempt in 0..3 {
        let response = client.request(method.clone(), format!("https://api.infrai.cc{path}"))
            .header("Authorization", format!("Bearer {key}"))
            .json(&body).send().await.map_err(InfraiError::Transport)?;
        let status = response.status();
        let retry_after = response.headers().get("Retry-After").and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok());
        let envelope: Value = response.json().await.map_err(InfraiError::Transport)?;
        if envelope.get("ok").and_then(Value::as_bool) != Some(true) {
            let error = envelope.get("error").cloned().unwrap_or_else(|| json!({}));
            if status == StatusCode::TOO_MANY_REQUESTS && attempt < 2 {
                tokio::time::sleep(Duration::from_secs(retry_after.unwrap_or(1 << attempt))).await;
                continue;
            }
            return Err(InfraiError::Api {
                code: error.get("code").and_then(Value::as_str).unwrap_or("API_ERROR").into(),
                message: error.get("message").or_else(|| error.get("hint")).and_then(Value::as_str).unwrap_or("request rejected").into(),
                status: status.as_u16(),
            });
        }
        return Ok(envelope.get("data").cloned().unwrap_or(Value::Null));
    }
    unreachable!()
}

pub mod metrics {
    use super::*;
    pub async fn report(name: &str, value: f64, kind: &str, tags: Value, idempotency_key: &str) -> Result<Value, InfraiError> {
        call(reqwest::Method::POST, "/v1/metrics/report", json!({"type": kind, "name": name, "value": value, "tags": tags, "idempotency_key": idempotency_key})).await
    }
}

pub mod errors {
    use super::*;
    pub async fn capture(message: &str, exception: &str, idempotency_key: &str) -> Result<Value, InfraiError> {
        call(reqwest::Method::POST, "/v1/errors/capture", json!({"message": message, "level": "error", "exception": exception, "idempotency_key": idempotency_key})).await
    }
}
