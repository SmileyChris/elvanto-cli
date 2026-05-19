pub mod raw;

use crate::error::CliError;
use reqwest::Client as Http;
use serde::de::DeserializeOwned;
use serde::Serialize;

const DEFAULT_BASE_URL: &str = "https://api.elvanto.com/v1";

pub struct Client {
    http: Http,
    base_url: String,
    api_key: String,
}

impl Client {
    pub fn new(api_key: String) -> Result<Self, CliError> {
        Self::with_base_url(api_key, DEFAULT_BASE_URL.to_string())
    }

    pub fn with_base_url(api_key: String, base_url: String) -> Result<Self, CliError> {
        if api_key.is_empty() {
            return Err(CliError::Usage("ELVANTO_API_KEY is empty".into()));
        }
        let http = Http::builder()
            .user_agent(concat!("elvanto-cli/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| CliError::Network(e.to_string()))?;
        Ok(Self { http, base_url, api_key })
    }

    pub fn redacted_key(&self) -> String {
        let k = &self.api_key;
        let char_count = k.chars().count();
        if char_count <= 8 {
            return "*".repeat(char_count);
        }
        let head: String = k.chars().take(4).collect();
        let tail: String = k.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
        format!("{head}…{tail}")
    }

    pub async fn post<B, T>(&self, endpoint: &str, body: &B) -> Result<T, CliError>
    where
        B: Serialize,
        T: DeserializeOwned,
    {
        let url = format!("{}/{}.json", self.base_url, endpoint);
        let resp = self
            .http
            .post(&url)
            .basic_auth(&self.api_key, Some("x"))
            .json(body)
            .send()
            .await
            .map_err(|e| CliError::Network(e.to_string()))?;

        let value: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| CliError::Network(format!("invalid response body: {e}")))?;

        match value.get("status").and_then(|v| v.as_str()) {
            Some("ok") => serde_json::from_value(value)
                .map_err(|e| CliError::Network(format!("decode error: {e}"))),
            Some("error") => {
                let err: raw::ApiError = serde_json::from_value(
                    value.get("error").cloned().unwrap_or(serde_json::Value::Null),
                )
                .map_err(|e| CliError::Network(format!("decode error: {e}")))?;
                Err(CliError::Api { code: err.code, message: err.message })
            }
            other => Err(CliError::Network(format!(
                "unexpected status: {:?}",
                other
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[derive(Debug, Deserialize)]
    struct Pong {
        #[allow(dead_code)]
        status: String,
        pong: String,
    }

    #[tokio::test]
    async fn ok_envelope_decodes_payload() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/ping.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "ok",
                "pong": "hi"
            })))
            .mount(&server)
            .await;

        let client = Client::with_base_url("key123abc".into(), server.uri()).unwrap();
        let out: Pong = client.post("ping", &serde_json::json!({})).await.unwrap();
        assert_eq!(out.pong, "hi");
    }

    #[tokio::test]
    async fn error_envelope_becomes_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/songs/getAll.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "error",
                "error": { "code": 250, "message": "No search parameters provided." }
            })))
            .mount(&server)
            .await;

        let client = Client::with_base_url("k".repeat(10), server.uri()).unwrap();
        let res: Result<serde_json::Value, _> =
            client.post("songs/getAll", &serde_json::json!({})).await;
        match res {
            Err(CliError::Api { code, message }) => {
                assert_eq!(code, 250);
                assert_eq!(message, "No search parameters provided.");
            }
            other => panic!("expected Api error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn sends_basic_auth_header() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/auth/probe.json"))
            // "abcd1234:x" base64 == "YWJjZDEyMzQ6eA=="
            .and(header("authorization", "Basic YWJjZDEyMzQ6eA=="))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "ok"
            })))
            .mount(&server)
            .await;

        let client = Client::with_base_url("abcd1234".into(), server.uri()).unwrap();
        let _: serde_json::Value = client
            .post("auth/probe", &serde_json::json!({}))
            .await
            .unwrap();
    }

    #[test]
    fn redact_short_key() {
        let c = Client::with_base_url("abcdefgh".into(), "http://x".into()).unwrap();
        assert_eq!(c.redacted_key(), "********");
    }

    #[test]
    fn redact_long_key() {
        let c = Client::with_base_url("abcdefghijkl".into(), "http://x".into()).unwrap();
        assert_eq!(c.redacted_key(), "abcd…ijkl");
    }

    #[test]
    fn redact_boundary_9_chars() {
        let c = Client::with_base_url("abcdefghi".into(), "http://x".into()).unwrap();
        // 9 chars > 8 → take head/tail
        assert_eq!(c.redacted_key(), "abcd…fghi");
    }
}
