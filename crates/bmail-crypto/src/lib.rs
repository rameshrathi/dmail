pub mod encryption;
pub mod error;
pub mod hash;
pub mod wallet;

pub use encryption::{
    decrypt_payload, encrypt_payload, unwrap_cek, wrap_cek, ContentKey, EncryptionKeyPair,
    WrappedCek,
};
pub use error::CryptoError;
pub use hash::sha256_digest;
pub use wallet::{address_from_base58, address_to_base58, verify_signature, WalletKeyPair};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_signing_and_verification() {
        let wallet = WalletKeyPair::generate();
        let message = b"Hello, Solana B-Mail Authentication!";
        let signature = wallet.sign(message);
        assert!(verify_signature(&wallet.public_key_bytes(), message, &signature).is_ok());

        // Tampered message must fail verification
        let tampered_message = b"Tampered message content";
        assert!(
            verify_signature(&wallet.public_key_bytes(), tampered_message, &signature).is_err()
        );

        // Wrong public key must fail verification
        let another_wallet = WalletKeyPair::generate();
        assert!(verify_signature(&another_wallet.public_key_bytes(), message, &signature).is_err());
    }

    #[test]
    fn test_base58_address_roundtrip() {
        let wallet = WalletKeyPair::generate();
        let addr = wallet.address();
        let pubkey_bytes = address_from_base58(&addr).expect("Base58 decoding failed");
        assert_eq!(pubkey_bytes, wallet.public_key_bytes());
        assert_eq!(address_to_base58(&pubkey_bytes), addr);
    }

    #[test]
    fn test_deterministic_key_derivation() {
        let wallet = WalletKeyPair::generate();
        let signature = wallet.sign(b"B-MAIL-DERIVATION-MESSAGE");

        let keypair1 = EncryptionKeyPair::derive_from_signature(&signature).unwrap();
        let keypair2 = EncryptionKeyPair::derive_from_signature(&signature).unwrap();

        assert_eq!(keypair1.public_key_bytes(), keypair2.public_key_bytes());
    }

    #[test]
    fn test_cek_wrapping_and_unwrapping() {
        let bob_enc_keypair = EncryptionKeyPair::generate();
        let cek = ContentKey::generate();

        let wrapped = wrap_cek(&bob_enc_keypair.public_key_bytes(), &cek).unwrap();

        // Bob unwraps with his private key
        let unwrapped_cek = unwrap_cek(
            &bob_enc_keypair,
            &wrapped.ephemeral_public_key,
            &wrapped.wrapped_key,
        )
        .unwrap();

        assert_eq!(cek.as_bytes(), unwrapped_cek.as_bytes());

        // An attacker (Charlie) attempts to unwrap using his own keypair
        let charlie_keypair = EncryptionKeyPair::generate();
        let attack_result = unwrap_cek(
            &charlie_keypair,
            &wrapped.ephemeral_public_key,
            &wrapped.wrapped_key,
        );
        assert_eq!(attack_result.unwrap_err(), CryptoError::DecryptionFailed);
    }

    #[test]
    fn test_payload_encryption_and_decryption() {
        let cek = ContentKey::generate();
        let plaintext = b"Top Secret Blockchain Mail Body";
        let aad = b"BMAIL/1.0:test_message_id";

        let (ciphertext, nonce) = encrypt_payload(&cek, plaintext, aad).unwrap();

        // Valid decrypt
        let decrypted = decrypt_payload(&cek, &ciphertext, &nonce, aad).unwrap();
        assert_eq!(plaintext, decrypted.as_slice());

        // Tampered ciphertext must fail
        let mut tampered = ciphertext.clone();
        tampered[0] ^= 0xFF;
        assert_eq!(
            decrypt_payload(&cek, &tampered, &nonce, aad).unwrap_err(),
            CryptoError::DecryptionFailed
        );

        // Modified AAD must fail
        let tampered_aad = b"BMAIL/1.0:wrong_message_id";
        assert_eq!(
            decrypt_payload(&cek, &ciphertext, &nonce, tampered_aad).unwrap_err(),
            CryptoError::DecryptionFailed
        );
    }
}
