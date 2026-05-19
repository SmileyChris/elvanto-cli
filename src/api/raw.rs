use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RawEnvelope {
    pub status: String,
    #[serde(default)]
    pub error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub code: i64,
    pub message: String,
}
