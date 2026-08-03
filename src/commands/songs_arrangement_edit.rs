use crate::api::Client;
use crate::cli::ArrangementEditArgs;
use crate::error::CliError;

pub async fn run(client: &Client, args: ArrangementEditArgs) -> Result<(), CliError> {
    if args.key_male.is_none() && args.key_female.is_none() {
        return Err(CliError::Usage(
            "at least one of --key-male or --key-female is required".into(),
        ));
    }

    // Create a key entry for male key
    if let Some(km) = &args.key_male {
        client
            .create_arrangement_key(&args.id, "Male", km)
            .await?;
        eprintln!("Set key_male={km} for arrangement {}", args.id);
    }

    // Create a key entry for female key
    if let Some(kf) = &args.key_female {
        client
            .create_arrangement_key(&args.id, "Female", kf)
            .await?;
        eprintln!("Set key_female={kf} for arrangement {}", args.id);
    }

    Ok(())
}
