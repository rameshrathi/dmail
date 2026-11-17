use dmail_types::{EncryptionPublicKey, MailDispatchRecord, UserKeyRecord, WalletAddress};
use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ChainError {
    #[error("User not registered on-chain: {0}")]
    UserNotRegistered(String),

    #[error("Invalid signature: failed on-chain signature verification")]
    InvalidSignature,

    #[error("User already registered with active key")]
    UserAlreadyRegistered,

    #[error("Key is revoked")]
    KeyRevoked,

    #[error("RPC or network error: {0}")]
    RpcError(String),
}

/// Blockchain Client interface representing Solana smart contract interactions.
pub trait BlockchainClient: Send + Sync {
    /// Register an encryption public key on-chain for a wallet.
    fn register_key(
        &self,
        wallet: &WalletAddress,
        encryption_pubkey: &EncryptionPublicKey,
        proof_of_possession_sig: &[u8; 64],
    ) -> Result<(), ChainError>;

    /// Query the active public encryption key for a recipient wallet.
    fn get_encryption_key(
        &self,
        wallet: &WalletAddress,
    ) -> Result<Option<EncryptionPublicKey>, ChainError>;

    /// Submit a mail dispatch transaction.
    fn dispatch_mail(&self, record: MailDispatchRecord) -> Result<(), ChainError>;

    /// Query incoming mail dispatch records for a recipient wallet.
    fn fetch_inbox(&self, recipient: &WalletAddress)
        -> Result<Vec<MailDispatchRecord>, ChainError>;
}

/// In-memory blockchain simulator modeling Solana PDA accounts and events.
pub struct LocalChainSimulator {
    /// Simulates Solana PDAs: `[b"user_registry", wallet_address]` => `UserKeyRecord`
    user_registry: Mutex<HashMap<WalletAddress, UserKeyRecord>>,
    /// Simulates on-chain mail dispatch account events indexed by recipient
    mailboxes: Mutex<HashMap<WalletAddress, Vec<MailDispatchRecord>>>,
}

impl Default for LocalChainSimulator {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalChainSimulator {
    pub fn new() -> Self {
        Self {
            user_registry: Mutex::new(HashMap::new()),
            mailboxes: Mutex::new(HashMap::new()),
        }
    }
}

impl BlockchainClient for LocalChainSimulator {
    fn register_key(
        &self,
        wallet: &WalletAddress,
        encryption_pubkey: &EncryptionPublicKey,
        proof_of_possession_sig: &[u8; 64],
    ) -> Result<(), ChainError> {
        // Verify on-chain signature: Wallet must sign its own encryption pubkey
        let mut msg = Vec::new();
        msg.extend_from_slice(b"DMAIL-REGISTER-KEY-V1");
        msg.extend_from_slice(wallet.as_bytes());
        msg.extend_from_slice(encryption_pubkey.as_bytes());

        dmail_crypto::verify_signature(wallet.as_bytes(), &msg, proof_of_possession_sig)
            .map_err(|_| ChainError::InvalidSignature)?;

        let mut registry = self.user_registry.lock().unwrap();
        let record = UserKeyRecord {
            wallet_address: *wallet,
            encryption_pubkey: *encryption_pubkey,
            registered_at: chrono::Utc::now().timestamp(),
            key_version: 1,
            is_active: true,
        };
        registry.insert(*wallet, record);
        Ok(())
    }

    fn get_encryption_key(
        &self,
        wallet: &WalletAddress,
    ) -> Result<Option<EncryptionPublicKey>, ChainError> {
        let registry = self.user_registry.lock().unwrap();
        match registry.get(wallet) {
            Some(record) => {
                if record.is_active {
                    Ok(Some(record.encryption_pubkey))
                } else {
                    Err(ChainError::KeyRevoked)
                }
            }
            None => Ok(None),
        }
    }

    fn dispatch_mail(&self, record: MailDispatchRecord) -> Result<(), ChainError> {
        // Verify sender signature on-chain
        let commitment = record.compute_signature_commitment();
        dmail_crypto::verify_signature(
            record.sender.as_bytes(),
            &commitment,
            record.sender_signature.as_bytes(),
        )
        .map_err(|_| ChainError::InvalidSignature)?;

        let mut mailboxes = self.mailboxes.lock().unwrap();
        let inbox = mailboxes.entry(record.recipient).or_default();
        inbox.push(record);
        Ok(())
    }

    fn fetch_inbox(
        &self,
        recipient: &WalletAddress,
    ) -> Result<Vec<MailDispatchRecord>, ChainError> {
        let mailboxes = self.mailboxes.lock().unwrap();
        Ok(mailboxes.get(recipient).cloned().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dmail_crypto::WalletKeyPair;

    #[test]
    fn test_register_and_query_key() {
        let chain = LocalChainSimulator::new();
        let wallet = WalletKeyPair::generate();
        let wallet_address = WalletAddress::from_bytes(wallet.public_key_bytes());
        let enc_key = EncryptionPublicKey::from_bytes([42u8; 32]);

        // Construct valid proof of possession signature
        let mut msg = Vec::new();
        msg.extend_from_slice(b"DMAIL-REGISTER-KEY-V1");
        msg.extend_from_slice(wallet_address.as_bytes());
        msg.extend_from_slice(enc_key.as_bytes());
        let sig = wallet.sign(&msg);

        // Registration should succeed
        chain.register_key(&wallet_address, &enc_key, &sig).unwrap();

        // Query key should return the registered encryption key
        let queried = chain.get_encryption_key(&wallet_address).unwrap();
        assert_eq!(queried, Some(enc_key));

        // Unknown wallet should return None
        let unknown = WalletAddress::from_bytes([99u8; 32]);
        assert_eq!(chain.get_encryption_key(&unknown).unwrap(), None);
    }
}
