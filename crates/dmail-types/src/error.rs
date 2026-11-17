use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum TypeError {
    #[error("Invalid address length: expected {expected}, got {actual}")]
    InvalidLength { expected: usize, actual: usize },

    #[error("Base58 decoding error: {0}")]
    Base58Error(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}
