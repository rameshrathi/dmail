use dmail_chain::BlockchainClient;
use dmail_crypto::{
    decrypt_payload, encrypt_payload, sha256_digest, unwrap_cek, verify_signature, wrap_cek,
    ContentKey, EncryptionKeyPair, WalletKeyPair,
};
use dmail_storage::StorageAdapter;
use dmail_types::{
    EncryptionPublicKey, MailAttachment, MailDispatchRecord, MailPayload, WalletAddress,
};
use rand::rngs::OsRng;
use rand::RngCore;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Recipient wallet {0} is not registered on-chain")]
    RecipientNotRegistered(String),

    #[error("Crypto error: {0}")]
    CryptoError(#[from] dmail_crypto::CryptoError),

    #[error("Chain error: {0}")]
    ChainError(#[from] dmail_chain::ChainError),

    #[error("Storage error: {0}")]
    StorageError(#[from] dmail_storage::StorageError),

    #[error("Integrity check failed: payload hash mismatch")]
    PayloadHashMismatch,

    #[error("Invalid sender signature: mail was forged or tampered")]
    InvalidSenderSignature,

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Decrypted mail message returned to the recipient client.
#[derive(Debug, Clone)]
pub struct DecryptedMail {
    pub dispatch_record: MailDispatchRecord,
    pub payload: MailPayload,
}

/// Unified D-Mail Client SDK managing wallet keys, encryption, storage, and on-chain dispatch.
pub struct DmailClient {
    wallet: WalletKeyPair,
    encryption_keypair: EncryptionKeyPair,
    chain: Arc<dyn BlockchainClient>,
    storage: Arc<dyn StorageAdapter>,
}

impl DmailClient {
    /// Initialize a new client with a wallet, deriving its encryption key deterministically.
    pub fn new(
        wallet: WalletKeyPair,
        chain: Arc<dyn BlockchainClient>,
        storage: Arc<dyn StorageAdapter>,
    ) -> Result<Self, ClientError> {
        // Derive encryption key deterministically from wallet signature
        let derivation_sig = wallet.sign(b"DMAIL-ENCRYPTION-KEY-V1");
        let encryption_keypair = EncryptionKeyPair::derive_from_signature(&derivation_sig)?;

        Ok(Self {
            wallet,
            encryption_keypair,
            chain,
            storage,
        })
    }

    /// Wallet public address (Solana Base58 format).
    pub fn wallet_address(&self) -> WalletAddress {
        WalletAddress::from_bytes(self.wallet.public_key_bytes())
    }

    /// Device public encryption key (X25519).
    pub fn encryption_public_key(&self) -> EncryptionPublicKey {
        EncryptionPublicKey::from_bytes(self.encryption_keypair.public_key_bytes())
    }

    /// Register the client's public encryption key on the blockchain.
    pub fn register_on_chain(&self) -> Result<(), ClientError> {
        let wallet_addr = self.wallet_address();
        let enc_pubkey = self.encryption_public_key();

        // Sign registration message with wallet key
        let mut msg = Vec::new();
        msg.extend_from_slice(b"DMAIL-REGISTER-KEY-V1");
        msg.extend_from_slice(wallet_addr.as_bytes());
        msg.extend_from_slice(enc_pubkey.as_bytes());
        let sig = self.wallet.sign(&msg);

        self.chain.register_key(&wallet_addr, &enc_pubkey, &sig)?;
        Ok(())
    }

    /// Compose, encrypt, store, and dispatch an end-to-end encrypted email.
    pub fn send_mail(
        &self,
        recipient: &WalletAddress,
        subject: &str,
        body: &str,
        attachments: Vec<MailAttachment>,
    ) -> Result<[u8; 32], ClientError> {
        // 1. Resolve recipient's public encryption key from the blockchain
        let recipient_enc_pubkey = match self.chain.get_encryption_key(recipient)? {
            Some(pk) => pk,
            None => return Err(ClientError::RecipientNotRegistered(recipient.to_base58())),
        };

        // 2. Generate unique 32-byte message_id
        let mut message_id = [0u8; 32];
        OsRng.fill_bytes(&mut message_id);
        let message_id_hex = hex_string(&message_id);

        let now = chrono::Utc::now().timestamp();
        let payload = MailPayload {
            version: "1.0".to_string(),
            message_id: message_id_hex.clone(),
            sender: self.wallet_address(),
            recipient: *recipient,
            timestamp: now,
            subject: subject.to_string(),
            body: body.to_string(),
            attachments,
        };

        let plaintext_bytes = serde_json::to_vec(&payload)?;

        // 3. Generate fresh random 256-bit CEK (Content Encryption Key)
        let cek = ContentKey::generate();

        // 4. Construct Authenticated Associated Data (AAD)
        let mut aad = Vec::new();
        aad.extend_from_slice(b"DMAIL/1.0:");
        aad.extend_from_slice(&message_id);
        aad.extend_from_slice(self.wallet_address().as_bytes());
        aad.extend_from_slice(recipient.as_bytes());

        // 5. Symmetric encryption with ChaCha20-Poly1305
        let (ciphertext, nonce) = encrypt_payload(&cek, &plaintext_bytes, &aad)?;

        // Package encrypted blob: [12-byte nonce] + [ciphertext with tag]
        let mut encrypted_blob = Vec::with_capacity(12 + ciphertext.len());
        encrypted_blob.extend_from_slice(&nonce);
        encrypted_blob.extend_from_slice(&ciphertext);

        // 6. Compute payload hash and store in content-addressed storage
        let payload_hash = sha256_digest(&encrypted_blob);
        let storage_uri = self.storage.put(&encrypted_blob)?;

        // 7. Wrap CEK using HPKE-X25519 for recipient's public key
        let wrapped = wrap_cek(recipient_enc_pubkey.as_bytes(), &cek)?;

        // 8. Construct on-chain dispatch record and sign commitment
        let mut dispatch_record = MailDispatchRecord {
            message_id,
            sender: self.wallet_address(),
            recipient: *recipient,
            ephemeral_pubkey: wrapped.ephemeral_public_key,
            wrapped_cek: wrapped.wrapped_key,
            payload_hash,
            storage_uri,
            timestamp: now,
            sender_signature: dmail_types::Signature64::from_bytes([0u8; 64]),
        };

        let commitment = dispatch_record.compute_signature_commitment();
        dispatch_record.sender_signature =
            dmail_types::Signature64::from_bytes(self.wallet.sign(&commitment));

        // 9. Dispatch transaction to the blockchain
        self.chain.dispatch_mail(dispatch_record)?;

        Ok(message_id)
    }

    /// Retrieve and decrypt all incoming mail for the client's wallet address.
    pub fn fetch_inbox(&self) -> Result<Vec<DecryptedMail>, ClientError> {
        let records = self.chain.fetch_inbox(&self.wallet_address())?;
        let mut decrypted_list = Vec::new();

        for record in records {
            // 1. Verify sender signature using on-chain sender address
            let commitment = record.compute_signature_commitment();
            verify_signature(
                record.sender.as_bytes(),
                &commitment,
                record.sender_signature.as_bytes(),
            )
            .map_err(|_| ClientError::InvalidSenderSignature)?;

            // 2. Fetch ciphertext from storage URI
            let encrypted_blob = self.storage.get(&record.storage_uri)?;

            // 3. Verify payload integrity against on-chain hash
            if sha256_digest(&encrypted_blob) != record.payload_hash {
                return Err(ClientError::PayloadHashMismatch);
            }

            if encrypted_blob.len() < 12 {
                return Err(ClientError::PayloadHashMismatch);
            }

            let mut nonce = [0u8; 12];
            nonce.copy_from_slice(&encrypted_blob[..12]);
            let ciphertext = &encrypted_blob[12..];

            // 4. Decapsulate CEK using our private encryption key
            let cek = unwrap_cek(
                &self.encryption_keypair,
                &record.ephemeral_pubkey,
                &record.wrapped_cek,
            )?;

            // 5. Reconstruct Associated Data
            let mut aad = Vec::new();
            aad.extend_from_slice(b"DMAIL/1.0:");
            aad.extend_from_slice(&record.message_id);
            aad.extend_from_slice(record.sender.as_bytes());
            aad.extend_from_slice(self.wallet_address().as_bytes());

            // 6. Symmetrically decrypt payload
            let plaintext_bytes = decrypt_payload(&cek, ciphertext, &nonce, &aad)?;
            let payload: MailPayload = serde_json::from_slice(&plaintext_bytes)?;

            decrypted_list.push(DecryptedMail {
                dispatch_record: record,
                payload,
            });
        }

        Ok(decrypted_list)
    }
}

fn hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
