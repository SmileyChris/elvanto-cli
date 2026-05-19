use crate::api::{redact_key, Client};
use crate::error::CliError;
use crate::keyring_store;

enum Source {
    Env,
    Keyring,
    None,
}

impl Source {
    fn label(&self) -> &'static str {
        match self {
            Source::Env => "env (ELVANTO_API_KEY)",
            Source::Keyring => "keyring",
            Source::None => "none",
        }
    }
}

fn discover_key() -> Result<(Source, Option<String>), CliError> {
    if let Ok(k) = std::env::var("ELVANTO_API_KEY") {
        if !k.is_empty() {
            return Ok((Source::Env, Some(k)));
        }
    }
    match keyring_store::get()? {
        Some(k) if !k.is_empty() => Ok((Source::Keyring, Some(k))),
        _ => Ok((Source::None, None)),
    }
}

pub async fn run() -> Result<(), CliError> {
    let (source, key) = discover_key()?;
    println!("source: {}", source.label());

    let Some(key) = key else {
        println!("status: no API key — run `elvanto auth login` or set ELVANTO_API_KEY");
        return Err(CliError::Usage("no API key".into()));
    };

    println!("key:    {}", redact_key(&key));

    let client = match std::env::var("ELVANTO_BASE_URL") {
        Ok(url) => Client::with_base_url(key, url)?,
        Err(_) => Client::new(key)?,
    };

    match client
        .post::<_, serde_json::Value>("songs/categories/getAll", &serde_json::json!({}))
        .await
    {
        Ok(_) => {
            println!("status: ok");
            Ok(())
        }
        Err(e @ CliError::Api { .. }) => {
            println!("status: invalid — {e}");
            Err(e)
        }
        Err(e) => {
            println!("status: error — {e}");
            Err(e)
        }
    }
}
