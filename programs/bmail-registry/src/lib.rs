//! # B-Mail Solana On-Chain Program (Anchor Reference)
//!
//! This module provides the Solana on-chain smart contract reference implementation
//! for the B-Mail Identity & Key Registry and Mail Dispatch notifications.
//!
//! ## PDAs (Program Derived Addresses):
//! - `UserRegistry`: `[b"user_registry", user_authority.key().as_ref()]`

use bmail_types::{EncryptionPublicKey, Signature64, WalletAddress};

/// Account state storing a user's on-chain public encryption key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserRegistryAccount {
    /// The user's Solana wallet public key (authority).
    pub authority: WalletAddress,
    /// The user's X25519 public encryption key used by senders to wrap CEKs.
    pub encryption_pubkey: EncryptionPublicKey,
    /// Unix timestamp when the user registered on-chain.
    pub registered_at: i64,
    /// Key version (incremented on rotation).
    pub key_version: u32,
    /// Active status flag (false if revoked).
    pub is_active: bool,
}

/// On-chain event emitted when an encrypted mail is dispatched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailDispatchEvent {
    /// 32-byte unique message identifier.
    pub message_id: [u8; 32],
    /// Sender's Solana wallet address.
    pub sender: WalletAddress,
    /// Recipient's Solana wallet address.
    pub recipient: WalletAddress,
    /// Ephemeral X25519 public key used for Diffie-Hellman / HPKE.
    pub ephemeral_pubkey: [u8; 32],
    /// HPKE-wrapped Content Encryption Key (CEK).
    pub wrapped_cek: Vec<u8>,
    /// SHA-256 hash of the encrypted payload blob stored in decentralized storage.
    pub payload_hash: [u8; 32],
    /// Content-addressed storage URI (e.g., `ipfs://<cid>`).
    pub storage_uri: String,
    /// On-chain timestamp.
    pub timestamp: i64,
    /// Sender's Ed25519 signature over the dispatch commitment.
    pub sender_signature: Signature64,
}

/// Anchor program instruction definitions (Reference).
pub mod instructions {
    use super::*;

    /// Instruction 1: Register or rotate a user's public encryption key on-chain.
    pub fn register_key(
        authority: WalletAddress,
        encryption_pubkey: EncryptionPublicKey,
        proof_of_possession_sig: [u8; 64],
    ) -> Result<UserRegistryAccount, &'static str> {
        let mut msg = Vec::new();
        msg.extend_from_slice(b"BMAIL-REGISTER-KEY-V1");
        msg.extend_from_slice(authority.as_bytes());
        msg.extend_from_slice(encryption_pubkey.as_bytes());

        bmail_crypto::verify_signature(authority.as_bytes(), &msg, &proof_of_possession_sig)
            .map_err(|_| "SignatureVerificationFailed")?;

        Ok(UserRegistryAccount {
            authority,
            encryption_pubkey,
            registered_at: chrono::Utc::now().timestamp(),
            key_version: 1,
            is_active: true,
        })
    }

    /// Instruction 2: Dispatch an encrypted email to a recipient.
    pub fn dispatch_mail(event: MailDispatchEvent) -> Result<(), &'static str> {
        let commitment = event.compute_signature_commitment();
        bmail_crypto::verify_signature(
            event.sender.as_bytes(),
            &commitment,
            event.sender_signature.as_bytes(),
        )
        .map_err(|_| "InvalidSenderSignature")?;

        // In Solana / Anchor, this triggers `emit!(event)` to notify the recipient via RPC websockets.
        Ok(())
    }
}

impl MailDispatchEvent {
    pub fn compute_signature_commitment(&self) -> [u8; 32] {
        let mut data = Vec::new();
        data.extend_from_slice(b"BMAIL-DISPATCH-V1");
        data.extend_from_slice(&self.message_id);
        data.extend_from_slice(self.sender.as_bytes());
        data.extend_from_slice(self.recipient.as_bytes());
        data.extend_from_slice(&self.payload_hash);
        data.extend_from_slice(self.storage_uri.as_bytes());
        bmail_crypto::sha256_digest(&data)
    }
}
