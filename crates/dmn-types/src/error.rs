use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum TypeError {
    #[error("Invalid length: expected {expected}, got {actual}")]
    InvalidLength { expected: usize, actual: usize },

    #[error("Invalid base64url encoding: {0}")]
    InvalidBase64(String),

    #[error("Invalid domain name: {0}")]
    InvalidDomain(String),

    #[error("Invalid email address: {0}")]
    InvalidEmail(String),

    #[error("Invalid protocol version string: {0}")]
    InvalidProtocolVersion(String),

    #[error("Invalid identifier: {0}")]
    InvalidIdentifier(String),

    #[error("Unknown algorithm suite: {0}")]
    UnknownAlgorithm(String),

    #[error("Unknown key purpose: {0}")]
    UnknownKeyPurpose(String),

    #[error("Unknown status: {0}")]
    UnknownStatus(String),
}
