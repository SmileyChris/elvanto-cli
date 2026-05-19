use serde::Deserialize;

#[allow(dead_code)]
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

#[derive(Debug, Deserialize)]
pub struct CategoriesResponse {
    #[serde(default)]
    pub categories: CategoryList,
}

#[derive(Debug, Deserialize, Default)]
pub struct CategoryList {
    #[serde(default)]
    pub category: Vec<RawCategory>,
}

#[derive(Debug, Deserialize)]
pub struct RawCategory {
    pub id: String,
    pub name: String,
}
