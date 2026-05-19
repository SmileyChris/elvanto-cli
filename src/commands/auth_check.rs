use crate::api::Client;
use crate::error::CliError;

pub async fn run(client: &Client) -> Result<(), CliError> {
    let _: serde_json::Value = client
        .post("songs/categories/getAll", &serde_json::json!({}))
        .await?;

    println!("auth: ok ({})", client.redacted_key());
    Ok(())
}
