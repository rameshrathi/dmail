use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: authentication tag mismatch or corrupted data")]
    DecryptionFailed,

    #[error("Invalid signature: verification failed")]
    InvalidSignature,

    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    #[error("Invalid key or signature length: expected {expected}, got {actual}")]
    InvalidLength { expected: usize, actual: usize },

    #[error("Base58 decoding error: {0}")]
    Base58Error(String),
}
