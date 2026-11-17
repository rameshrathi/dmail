use crate::identity::{Signature64, WalletAddress};
use serde::{Deserialize, Serialize};

/// Plaintext mail payload created and decrypted strictly on user devices.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MailPayload {
    pub version: String,
    pub message_id: String,
    pub sender: WalletAddress,
    pub recipient: WalletAddress,
    pub timestamp: i64,
    pub subject: String,
    pub body: String,
    pub attachments: Vec<MailAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MailAttachment {
    pub filename: String,
    pub mime_type: String,
    pub data_base64: String,
}

/// Symmetrically encrypted payload with Poly1305 authentication tag.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncryptedPayload {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
    pub payload_hash: [u8; 32],
}

/// HPKE-encapsulated Content Encryption Key envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WrappedCekEnvelope {
    pub ephemeral_pubkey: [u8; 32],
    pub wrapped_cek: Vec<u8>,
}

/// On-chain dispatch event / mailbox notification record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MailDispatchRecord {
    pub message_id: [u8; 32],
    pub sender: WalletAddress,
    pub recipient: WalletAddress,
    pub ephemeral_pubkey: [u8; 32],
    pub wrapped_cek: Vec<u8>,
    pub payload_hash: [u8; 32],
    pub storage_uri: String,
    pub timestamp: i64,
    pub sender_signature: Signature64,
}

impl MailDispatchRecord {
    /// Compute the commitment hash covered by the sender's Ed25519 signature.
    pub fn compute_signature_commitment(&self) -> [u8; 32] {
        let mut data = Vec::new();
        data.extend_from_slice(b"DMAIL-DISPATCH-V1");
        data.extend_from_slice(&self.message_id);
        data.extend_from_slice(self.sender.as_bytes());
        data.extend_from_slice(self.recipient.as_bytes());
        data.extend_from_slice(&self.payload_hash);
        data.extend_from_slice(self.storage_uri.as_bytes());
        dmail_crypto::sha256_digest(&data)
    }
}
