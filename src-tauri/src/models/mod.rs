// models/mod.rs — ClipVault data models

use serde::{Deserialize, Serialize};

/// Content type of a clipboard record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Text,
    Image,
    FilePath,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Text => "text",
            ContentType::Image => "image",
            ContentType::FilePath => "file_path",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "text" => Some(ContentType::Text),
            "image" => Some(ContentType::Image),
            "file_path" => Some(ContentType::FilePath),
            _ => None,
        }
    }
}

/// Full clipboard record — includes binary content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: String,
    pub content_type: ContentType,
    /// Raw bytes: UTF-8 text, PNG/JPEG binary, or UTF-8 file path.
    #[serde(with = "base64_bytes")]
    pub content: Vec<u8>,
    /// First 200 chars of text or "Image (NxN)" / path preview.
    pub preview: String,
    /// SHA-256 hex digest for dedup.
    pub content_hash: String,
    pub is_favorite: bool,
    pub is_sensitive: bool,
    /// Unix timestamp in milliseconds.
    pub created_at: i64,
}

/// Lightweight summary for list/search — no binary content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipSummary {
    pub id: String,
    pub content_type: ContentType,
    pub preview: String,
    pub is_favorite: bool,
    pub is_sensitive: bool,
    pub created_at: i64,
}

/// Monitor service status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorStatus {
    pub is_running: bool,
    pub clips_captured: u64,
}

/// Search request parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SearchParams {
    pub query: String,
    pub content_type: Option<ContentType>,
    pub limit: u32,
}

/// Create clip request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct CreateClipReq {
    pub content_type: ContentType,
    #[serde(with = "base64_bytes")]
    pub content: Vec<u8>,
}

/// Sensitivity classification from the detection pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Sensitivity {
    Clean,
    Sensitive(SensitiveKind),
    Transient,
}

/// What kind of sensitive data was detected.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SensitiveKind {
    ApiKey(String),
    Jwt,
    BearerToken,
    PrivateKey,
    ConnectionString,
}

impl SensitiveKind {
    pub fn masked_preview(&self) -> String {
        match self {
            SensitiveKind::ApiKey(provider) => {
                format!("API Key ({}) \u{2022}\u{2022}\u{2022}\u{2022}XXXX", provider)
            }
            SensitiveKind::Jwt => "JWT Token \u{2022}\u{2022}\u{2022}\u{2022}".into(),
            SensitiveKind::BearerToken => "Bearer Token \u{2022}\u{2022}\u{2022}\u{2022}".into(),
            SensitiveKind::PrivateKey => "Private Key (PEM) \u{2022}\u{2022}\u{2022}\u{2022}".into(),
            SensitiveKind::ConnectionString => {
                "Connection String \u{2022}\u{2022}\u{2022}\u{2022}".into()
            }
        }
    }
}

/// Base64 serialization helper for Vec<u8> over IPC.
mod base64_bytes {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(data: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&BASE64.encode(data))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(d)?;
        BASE64.decode(&s).map_err(serde::de::Error::custom)
    }
}
