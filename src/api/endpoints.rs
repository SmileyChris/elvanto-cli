use crate::api::raw::{CategoriesResponse, RawCategory};
use crate::api::Client;
use crate::error::CliError;

impl Client {
    pub async fn list_categories(&self) -> Result<Vec<RawCategory>, CliError> {
        let resp: CategoriesResponse = self
            .post("songs/categories/getAll", &serde_json::json!({}))
            .await?;
        Ok(resp.categories.category)
    }
}
