#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Elvanto returned code {code}: {message}")]
    Api { code: i64, message: String },

    #[error("network: {0}")]
    Network(String),

    #[error("io: {0}")]
    Io(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("{0}")]
    Usage(String),
}

impl CliError {
    pub fn exit_code(&self) -> u8 {
        match self {
            CliError::Api { .. }
            | CliError::Network(_)
            | CliError::Io(_)
            | CliError::NotFound(_) => 1,
            CliError::Usage(_) => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_error_exits_1() {
        let err = CliError::Api {
            code: 250,
            message: "No search parameters provided.".into(),
        };
        assert_eq!(err.exit_code(), 1u8);
        assert_eq!(
            err.to_string(),
            "Elvanto returned code 250: No search parameters provided."
        );
    }

    #[test]
    fn network_error_exits_1() {
        let err = CliError::Network("connection timed out".into());
        assert_eq!(err.exit_code(), 1u8);
        assert_eq!(err.to_string(), "network: connection timed out");
    }

    #[test]
    fn usage_error_exits_2() {
        let err = CliError::Usage("ELVANTO_API_KEY is not set".into());
        assert_eq!(err.exit_code(), 2u8);
    }

    #[test]
    fn io_error_exits_1() {
        let err = CliError::Io("broken pipe".into());
        assert_eq!(err.exit_code(), 1u8);
        assert_eq!(err.to_string(), "io: broken pipe");
    }

    #[test]
    fn not_found_exits_1() {
        let err = CliError::NotFound("song abc".into());
        assert_eq!(err.exit_code(), 1u8);
        assert_eq!(err.to_string(), "not found: song abc");
    }
}
