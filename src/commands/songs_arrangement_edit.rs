use crate::api::Client;
use crate::cli::ArrangementEditArgs;
use crate::error::CliError;

pub async fn run(client: &Client, args: ArrangementEditArgs) -> Result<(), CliError> {
    if args.key_male.is_none() && args.key_female.is_none() {
        return Err(CliError::Usage(
            "at least one of --key-male or --key-female is required".into(),
        ));
    }

    // Update (or create) the Male key record — keys/create appends duplicates,
    // so edit existing records in place when present.
    if let Some(km) = &args.key_male {
        client.set_arrangement_key(&args.id, "Male", km).await?;
        eprintln!("Set key_male={km} for arrangement {}", args.id);
    }

    // Update (or create) the Female key record.
    if let Some(kf) = &args.key_female {
        client.set_arrangement_key(&args.id, "Female", kf).await?;
        eprintln!("Set key_female={kf} for arrangement {}", args.id);
    }

    Ok(())
}
