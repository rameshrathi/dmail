use crate::error::CryptoError;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

/// Solana-compatible Ed25519 wallet keypair used for transaction authorization and mail signing.
pub struct WalletKeyPair {
    signing_key: SigningKey,
}

impl WalletKeyPair {
    /// Generate a new random wallet keypair using OS CSPRNG.
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        Self { signing_key }
    }

    /// Construct from raw 32-byte private seed.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        Self { signing_key }
    }

    /// Returns the 32-byte public key of the wallet.
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }

    /// Returns the Solana-style Base58 address of the wallet.
    pub fn address(&self) -> String {
        bs58::encode(self.public_key_bytes()).into_string()
    }

    /// Sign a message payload producing a 64-byte Ed25519 signature.
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        let signature: Signature = self.signing_key.sign(message);
        signature.to_bytes()
    }

    /// Exposes the private seed bytes for local storage/backup.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }
}

/// Verify an Ed25519 signature given the 32-byte public key and message.
pub fn verify_signature(
    public_key_bytes: &[u8; 32],
    message: &[u8],
    signature_bytes: &[u8; 64],
) -> Result<(), CryptoError> {
    let verifying_key =
        VerifyingKey::from_bytes(public_key_bytes).map_err(|_| CryptoError::InvalidSignature)?;
    let signature = Signature::from_bytes(signature_bytes);
    verifying_key
        .verify(message, &signature)
        .map_err(|_| CryptoError::InvalidSignature)
}

/// Parse a Base58 address into 32-byte public key.
pub fn address_from_base58(s: &str) -> Result<[u8; 32], CryptoError> {
    let bytes = bs58::decode(s)
        .into_vec()
        .map_err(|e| CryptoError::Base58Error(e.to_string()))?;
    if bytes.len() != 32 {
        return Err(CryptoError::InvalidLength {
            expected: 32,
            actual: bytes.len(),
        });
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

/// Format a 32-byte public key into a Solana-style Base58 string.
pub fn address_to_base58(bytes: &[u8; 32]) -> String {
    bs58::encode(bytes).into_string()
}
