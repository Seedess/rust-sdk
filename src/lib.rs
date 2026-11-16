//! Official Seedess SDK for Rust.
//!
//! Server-side only: uses a secret API key (`key_…`). Playback uses the separate
//! `@seedess/*-p2p` packages and a per-stream public token.

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub const DEFAULT_BASE_URL: &str = "https://seedess.com";

#[derive(Debug, Error)]
pub enum Error {
    #[error("Seedess: api_key must be a server API key (key_…)")]
    InvalidKey,
    #[error("Seedess API error {status} {code}: {message}")]
    Api {
        status: u16,
        code: String,
        message: String,
    },
    #[error("Seedess transport error: {0}")]
    Transport(String),
    #[error("Seedess IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Seedess JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Playback {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub master: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playlist: Option<String>,
    #[serde(default)]
    pub signaling: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stream {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub live: Option<bool>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default, rename = "killSwitch")]
    pub kill_switch: Option<bool>,
    #[serde(default, rename = "targetDuration")]
    pub target_duration: Option<f64>,
    #[serde(default, rename = "mediaSequence")]
    pub media_sequence: Option<i64>,
    #[serde(default)]
    pub variants: Option<Vec<Value>>,
    #[serde(default)]
    pub segments: Option<i64>,
    #[serde(default, rename = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(default, rename = "publicToken")]
    pub public_token: Option<String>,
    #[serde(default)]
    pub playback: Option<Playback>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metrics {
    #[serde(default, rename = "cdnBytes")]
    pub cdn_bytes: i64,
    #[serde(default, rename = "p2pBytes")]
    pub p2p_bytes: i64,
    #[serde(default, rename = "offloadPercent")]
    pub offload_percent: f64,
    #[serde(default, rename = "hashFailures")]
    pub hash_failures: i64,
    #[serde(default)]
    pub fallbacks: i64,
    #[serde(default)]
    pub rebuffers: i64,
    #[serde(default, rename = "peakPeers")]
    pub peak_peers: i64,
    #[serde(default)]
    pub sessions: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateStreamInput {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live: Option<bool>,
}

impl CreateStreamInput {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            id: None,
            title: None,
            live: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryInput {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "streamId")]
    pub stream_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "cdnBytes")]
    pub cdn_bytes: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "p2pBytes")]
    pub p2p_bytes: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "hashFailures")]
    pub hash_failures: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallbacks: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rebuffers: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "peakPeers")]
    pub peak_peers: Option<i64>,
}

impl TelemetryInput {
    pub fn new(session_id: impl Into<String>, stream_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            stream_id: stream_id.into(),
            cdn_bytes: None,
            p2p_bytes: None,
            hash_failures: None,
            fallbacks: None,
            rebuffers: None,
            peak_peers: None,
        }
    }
}

pub struct Client {
    api_key: String,
    base_url: String,
    agent: ureq::Agent,
}

impl Client {
    pub fn new(api_key: impl Into<String>) -> Result<Self, Error> {
        Self::with_base_url(api_key, DEFAULT_BASE_URL)
    }

    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Result<Self, Error> {
        let api_key = api_key.into();
        if !api_key.starts_with("key_") {
            return Err(Error::InvalidKey);
        }
        Ok(Self {
            api_key,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            agent: ureq::AgentBuilder::new()
                .timeout(std::time::Duration::from_secs(30))
                .build(),
        })
    }

    fn escape(value: &str) -> String {
        utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
    }

    pub fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<&Value>,
        auth: bool,
    ) -> Result<Value, Error> {
        let full_path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        };
        let url = format!("{}{}", self.base_url, full_path);
        let mut request = self.agent.request(method, &url).set("accept", "application/json");
        if auth {
            request = request.set("authorization", &format!("Bearer {}", self.api_key));
        }
        let result = match body {
            Some(value) => request.send_json(value),
            None => request.call(),
        };

        let response = match result {
            Ok(response) => response,
            Err(ureq::Error::Status(status, response)) => {
                let raw = response.into_string().unwrap_or_default();
                let parsed: Value = serde_json::from_str(&raw).unwrap_or(Value::Null);
                let code = parsed
                    .pointer("/error/code")
                    .and_then(Value::as_str)
                    .unwrap_or("http_error")
                    .to_string();
                let message = parsed
                    .pointer("/error/message")
                    .and_then(Value::as_str)
                    .unwrap_or("HTTP error")
                    .to_string();
                return Err(Error::Api { status, code, message });
            }
            Err(error) => return Err(Error::Transport(error.to_string())),
        };
        let value: Value = {
            let text = response.into_string()?;
            if text.trim().is_empty() {
                Value::Null
            } else {
                serde_json::from_str(&text)?
            }
        };
        Ok(value)
    }

    pub fn create_stream(&self, input: CreateStreamInput) -> Result<Stream, Error> {
        let body = serde_json::to_value(input)?;
        let value = self.request("POST", "/api/v1/streams", Some(&body), true)?;
        Ok(serde_json::from_value(value)?)
    }

    pub fn get_stream(&self, id: &str) -> Result<Stream, Error> {
        let path = format!("/api/v1/streams/{}", Self::escape(id));
        let value = self.request("GET", &path, None, true)?;
        Ok(serde_json::from_value(value)?)
    }

    pub fn list_streams(&self) -> Result<Vec<Stream>, Error> {
        let value = self.request("GET", "/api/v1/streams", None, true)?;
        let streams = value.get("streams").cloned().unwrap_or(Value::Array(vec![]));
        Ok(serde_json::from_value(streams)?)
    }

    pub fn refresh_stream(&self, id: &str) -> Result<Value, Error> {
        let path = format!("/api/v1/streams/{}/refresh", Self::escape(id));
        self.request("POST", &path, None, true)
    }

    pub fn disable_peers(&self, id: &str) -> Result<Value, Error> {
        let path = format!("/api/v1/streams/{}/kill", Self::escape(id));
        self.request("POST", &path, None, true)
    }

    pub fn enable_peers(&self, id: &str) -> Result<Value, Error> {
        let path = format!("/api/v1/streams/{}/revive", Self::escape(id));
        self.request("POST", &path, None, true)
    }

    pub fn get_metrics(&self, id: &str) -> Result<Metrics, Error> {
        let path = format!("/api/v1/streams/{}/metrics", Self::escape(id));
        let value = self.request("GET", &path, None, true)?;
        let metrics = value.get("metrics").cloned().unwrap_or(Value::Null);
        Ok(serde_json::from_value(metrics)?)
    }

    pub fn send_telemetry(&self, input: TelemetryInput) -> Result<(), Error> {
        let body = serde_json::to_value(input)?;
        self.request("POST", "/api/v1/telemetry", Some(&body), false)?;
        Ok(())
    }
}
