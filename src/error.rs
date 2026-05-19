// Variants added ahead of their first callers; will be removed once all are constructed.
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Elvanto returned code {code}: {message}")]
    Api { code: i64, message: String },

    #[error("network: {0}")]
    Network(String),

    #[error("{0}")]
    Usage(String),
}

impl CliError {
    pub fn exit_code(&self) -> u8 {
        match self {
            CliError::Api { .. } | CliError::Network(_) => 1,
            CliError::Usage(_) => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_error_exits_1() {
        let err = CliError::Api { code: 250, message: "No search parameters provided.".into() };
        assert_eq!(err.exit_code(), 1u8);
        assert_eq!(err.to_string(), "Elvanto returned code 250: No search parameters provided.");
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
}
