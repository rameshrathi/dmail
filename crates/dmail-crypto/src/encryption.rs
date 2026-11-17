use crate::error::CryptoError;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::Sha256;
use std::fmt;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519Secret};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// 32-byte Content Encryption Key (CEK) wrapper that zeroes memory on drop.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct ContentKey([u8; 32]);

impl fmt::Debug for ContentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContentKey([REDACTED])")
    }
}

impl ContentKey {
    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        Self(key)
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// X25519 Device Encryption Keypair for recipient asymmetric wrapping.
pub struct EncryptionKeyPair {
    secret: X25519Secret,
    public: X25519PublicKey,
}

impl EncryptionKeyPair {
    /// Generate a fresh random X25519 keypair.
    pub fn generate() -> Self {
        let secret = X25519Secret::random_from_rng(OsRng);
        let public = X25519PublicKey::from(&secret);
        Self { secret, public }
    }

    /// Deterministically derive an X25519 keypair from a wallet signature.
    pub fn derive_from_signature(signature_bytes: &[u8; 64]) -> Result<Self, CryptoError> {
        let hk = Hkdf::<Sha256>::new(Some(b"dmail-encryption-key-salt"), signature_bytes);
        let mut okm = [0u8; 32];
        hk.expand(b"dmail-x25519-seed", &mut okm)
            .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
        let secret = X25519Secret::from(okm);
        okm.zeroize();
        let public = X25519PublicKey::from(&secret);
        Ok(Self { secret, public })
    }

    pub fn public_key_bytes(&self) -> [u8; 32] {
        *self.public.as_bytes()
    }

    pub fn public_key(&self) -> X25519PublicKey {
        self.public
    }

    pub fn secret(&self) -> &X25519Secret {
        &self.secret
    }
}

/// Result of wrapping a Content Encryption Key (CEK) for a recipient.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrappedCek {
    pub ephemeral_public_key: [u8; 32],
    pub wrapped_key: Vec<u8>,
}

/// Asymmetrically wrap a CEK to a recipient's X25519 public key.
pub fn wrap_cek(
    recipient_public_key: &[u8; 32],
    cek: &ContentKey,
) -> Result<WrappedCek, CryptoError> {
    let recipient_pk = X25519PublicKey::from(*recipient_public_key);
    let ephemeral_sk = X25519Secret::random_from_rng(OsRng);
    let ephemeral_pk = X25519PublicKey::from(&ephemeral_sk);

    // Compute Diffie-Hellman shared secret
    let shared_secret = ephemeral_sk.diffie_hellman(&recipient_pk);

    // Derive symmetric wrapping key via HKDF-SHA256
    let hk = Hkdf::<Sha256>::new(Some(b"dmail-hpke-cek-salt"), shared_secret.as_bytes());
    let mut wrap_key_bytes = [0u8; 32];
    hk.expand(b"dmail-cek-wrapping-key", &mut wrap_key_bytes)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    let cipher = ChaCha20Poly1305::new(Key::from_slice(&wrap_key_bytes));
    wrap_key_bytes.zeroize();

    // Deterministic 12-byte nonce for CEK wrapping (key is single-use derived from ephemeral secret)
    let nonce = Nonce::from_slice(b"DMAIL-WRAP-N");
    let wrapped_key = cipher
        .encrypt(nonce, cek.as_bytes().as_ref())
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    Ok(WrappedCek {
        ephemeral_public_key: *ephemeral_pk.as_bytes(),
        wrapped_key,
    })
}

/// Decapsulate a CEK using the recipient's X25519 private key.
pub fn unwrap_cek(
    recipient_keypair: &EncryptionKeyPair,
    ephemeral_public_key: &[u8; 32],
    wrapped_key: &[u8],
) -> Result<ContentKey, CryptoError> {
    let ephemeral_pk = X25519PublicKey::from(*ephemeral_public_key);
    let shared_secret = recipient_keypair.secret.diffie_hellman(&ephemeral_pk);

    let hk = Hkdf::<Sha256>::new(Some(b"dmail-hpke-cek-salt"), shared_secret.as_bytes());
    let mut wrap_key_bytes = [0u8; 32];
    hk.expand(b"dmail-cek-wrapping-key", &mut wrap_key_bytes)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    let cipher = ChaCha20Poly1305::new(Key::from_slice(&wrap_key_bytes));
    wrap_key_bytes.zeroize();

    let nonce = Nonce::from_slice(b"DMAIL-WRAP-N");
    let cek_bytes = cipher
        .decrypt(nonce, wrapped_key)
        .map_err(|_| CryptoError::DecryptionFailed)?;

    if cek_bytes.len() != 32 {
        return Err(CryptoError::InvalidLength {
            expected: 32,
            actual: cek_bytes.len(),
        });
    }

    let mut key_arr = [0u8; 32];
    key_arr.copy_from_slice(&cek_bytes);
    Ok(ContentKey::from_bytes(key_arr))
}

/// Symmetrically encrypt payload bytes with ChaCha20-Poly1305 and Associated Data.
pub fn encrypt_payload(
    cek: &ContentKey,
    plaintext: &[u8],
    aad: &[u8],
) -> Result<(Vec<u8>, [u8; 12]), CryptoError> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(cek.as_bytes()));
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let payload = chacha20poly1305::aead::Payload {
        msg: plaintext,
        aad,
    };

    let ciphertext = cipher
        .encrypt(nonce, payload)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    Ok((ciphertext, nonce_bytes))
}

/// Symmetrically decrypt payload bytes with ChaCha20-Poly1305 and verify Associated Data.
pub fn decrypt_payload(
    cek: &ContentKey,
    ciphertext: &[u8],
    nonce_bytes: &[u8; 12],
    aad: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(cek.as_bytes()));
    let nonce = Nonce::from_slice(nonce_bytes);

    let payload = chacha20poly1305::aead::Payload {
        msg: ciphertext,
        aad,
    };

    cipher
        .decrypt(nonce, payload)
        .map_err(|_| CryptoError::DecryptionFailed)
}
