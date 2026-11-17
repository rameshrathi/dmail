//! Core protocol types for B-Mail.
pub mod error;
pub mod identity;
pub mod message;

pub use error::TypeError;
pub use identity::{EncryptionPublicKey, Signature64, UserKeyRecord, WalletAddress};
pub use message::{
    EncryptedPayload, MailAttachment, MailDispatchRecord, MailPayload, WrappedCekEnvelope,
};
