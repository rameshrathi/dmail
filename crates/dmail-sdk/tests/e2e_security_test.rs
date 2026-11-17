use dmail_chain::{BlockchainClient, LocalChainSimulator};
use dmail_crypto::{unwrap_cek, WalletKeyPair};
use dmail_sdk::{ClientError, DmailClient};
use dmail_storage::{MemoryStorageAdapter, StorageAdapter};
use dmail_types::MailAttachment;
use std::sync::Arc;

#[test]
fn test_end_to_end_blockchain_e2ee_mail_flow() {
    let chain = Arc::new(LocalChainSimulator::new());
    let storage = Arc::new(MemoryStorageAdapter::new());

    // 1. Setup Alice (Sender) and Bob (Recipient)
    let alice_wallet = WalletKeyPair::generate();
    let bob_wallet = WalletKeyPair::generate();

    let alice = DmailClient::new(alice_wallet, chain.clone(), storage.clone()).unwrap();
    let bob = DmailClient::new(bob_wallet, chain.clone(), storage.clone()).unwrap();

    // 2. Bob registers on the blockchain
    bob.register_on_chain().unwrap();

    // 3. Alice composes and sends confidential email to Bob
    let subject = "Confidential Smart Contract Audit";
    let body = "The audit report is finalized. Zero vulnerabilities found.";
    let attachment = MailAttachment {
        filename: "audit.pdf".to_string(),
        mime_type: "application/pdf".to_string(),
        data_base64: "JVBERi0xLjQKJcfs...".to_string(),
    };

    let msg_id = alice
        .send_mail(
            &bob.wallet_address(),
            subject,
            body,
            vec![attachment.clone()],
        )
        .unwrap();

    assert_ne!(msg_id, [0u8; 32]);

    // 4. Bob fetches inbox and decrypts
    let bob_inbox = bob.fetch_inbox().unwrap();
    assert_eq!(bob_inbox.len(), 1);

    let decrypted = &bob_inbox[0];
    assert_eq!(decrypted.payload.sender, alice.wallet_address());
    assert_eq!(decrypted.payload.recipient, bob.wallet_address());
    assert_eq!(decrypted.payload.subject, subject);
    assert_eq!(decrypted.payload.body, body);
    assert_eq!(decrypted.payload.attachments.len(), 1);
    assert_eq!(decrypted.payload.attachments[0].filename, "audit.pdf");
}

#[test]
fn test_security_charlie_cannot_decrypt_bobs_mail() {
    let chain = Arc::new(LocalChainSimulator::new());
    let storage = Arc::new(MemoryStorageAdapter::new());

    let alice =
        DmailClient::new(WalletKeyPair::generate(), chain.clone(), storage.clone()).unwrap();
    let bob = DmailClient::new(WalletKeyPair::generate(), chain.clone(), storage.clone()).unwrap();
    let charlie =
        DmailClient::new(WalletKeyPair::generate(), chain.clone(), storage.clone()).unwrap();

    bob.register_on_chain().unwrap();
    charlie.register_on_chain().unwrap();

    // Alice sends mail specifically to Bob
    alice
        .send_mail(
            &bob.wallet_address(),
            "Top Secret",
            "Confidential Data",
            vec![],
        )
        .unwrap();

    // Charlie checks his inbox: he receives 0 messages
    let charlie_inbox = charlie.fetch_inbox().unwrap();
    assert_eq!(charlie_inbox.len(), 0);

    // Now suppose Charlie is a malicious node that intercepts Bob's on-chain message
    let bobs_records = chain.fetch_inbox(&bob.wallet_address()).unwrap();
    assert_eq!(bobs_records.len(), 1);
    let record = &bobs_records[0];

    // Charlie has the ciphertext from storage and the wrapped CEK from the chain
    let ciphertext_blob = storage.get(&record.storage_uri).unwrap();
    assert!(!ciphertext_blob.is_empty());

    // Charlie attempts to unwrap Bob's CEK using Charlie's private encryption key
    // We access Charlie's internal encryption keypair through unwrap_cek
    let charlie_signature = WalletKeyPair::generate().sign(b"DMAIL-ENCRYPTION-KEY-V1");
    let charlie_enc =
        dmail_crypto::EncryptionKeyPair::derive_from_signature(&charlie_signature).unwrap();

    let attack = unwrap_cek(&charlie_enc, &record.ephemeral_pubkey, &record.wrapped_cek);

    // Mathematical guarantee: Charlie fails completely
    assert_eq!(
        attack.unwrap_err(),
        dmail_crypto::CryptoError::DecryptionFailed
    );
}

#[test]
fn test_security_tampered_ciphertext_fails() {
    let chain = Arc::new(LocalChainSimulator::new());
    let storage = Arc::new(MemoryStorageAdapter::new());

    let alice_wallet = WalletKeyPair::generate();
    let bob_wallet = WalletKeyPair::generate();
    let bob_seed = bob_wallet.to_bytes();

    let alice = DmailClient::new(alice_wallet, chain.clone(), storage.clone()).unwrap();
    let bob = DmailClient::new(bob_wallet, chain.clone(), storage.clone()).unwrap();

    bob.register_on_chain().unwrap();

    alice
        .send_mail(
            &bob.wallet_address(),
            "Integrity Test",
            "Original Body",
            vec![],
        )
        .unwrap();

    // Tamper with the ciphertext blob in storage
    let bobs_records = chain.fetch_inbox(&bob.wallet_address()).unwrap();
    let record = &bobs_records[0];

    let mut blob = storage.get(&record.storage_uri).unwrap();
    blob[15] ^= 0xAA; // Flip bits in ciphertext

    // To simulate direct storage tampering at the same URI:
    let tampered_storage = Arc::new(TamperedStorage {
        inner: storage.clone(),
        target_uri: record.storage_uri.clone(),
        tampered_data: blob,
    });

    let bob_with_tampered_store = DmailClient::new(
        WalletKeyPair::from_seed(&bob_seed),
        chain.clone(),
        tampered_storage,
    )
    .unwrap();

    // Bob attempting to fetch inbox must detect payload hash mismatch
    let result = bob_with_tampered_store.fetch_inbox();
    assert!(matches!(result, Err(ClientError::PayloadHashMismatch)));
}

#[test]
fn test_security_unregistered_recipient_fails_closed() {
    let chain = Arc::new(LocalChainSimulator::new());
    let storage = Arc::new(MemoryStorageAdapter::new());

    let alice =
        DmailClient::new(WalletKeyPair::generate(), chain.clone(), storage.clone()).unwrap();
    let unregistered_wallet = WalletKeyPair::generate();
    let unregistered_addr =
        dmail_types::WalletAddress::from_bytes(unregistered_wallet.public_key_bytes());

    // Alice attempts to send to unregistered recipient
    let result = alice.send_mail(&unregistered_addr, "Subject", "Body", vec![]);

    // Must fail closed with RecipientNotRegistered error
    match result {
        Err(ClientError::RecipientNotRegistered(addr)) => {
            assert_eq!(addr, unregistered_addr.to_base58());
        }
        _ => panic!("Expected RecipientNotRegistered error"),
    }
}

struct TamperedStorage {
    inner: Arc<dyn StorageAdapter>,
    target_uri: String,
    tampered_data: Vec<u8>,
}

impl StorageAdapter for TamperedStorage {
    fn put(&self, data: &[u8]) -> Result<String, dmail_storage::StorageError> {
        self.inner.put(data)
    }

    fn get(&self, uri: &str) -> Result<Vec<u8>, dmail_storage::StorageError> {
        if uri == self.target_uri {
            Ok(self.tampered_data.clone())
        } else {
            self.inner.get(uri)
        }
    }
}
