# Blockchain End-to-End Encrypted Mail Protocol (D-Mail)
## Protocol Specification — v1.0 POC

---

## 1. Executive Overview

**D-Mail** is a decentralized, blockchain-native email and messaging protocol. It eliminates centralized email servers, third-party certificate authorities, and cleartext mail transfer agents (MTAs). 

In D-Mail:
- **Identity is cryptographic:** Every user account is identified by their blockchain address (e.g., Solana public key or Ethereum address).
- **Encryption is end-to-end (E2EE):** All mail contents (subject, body, attachments, metadata) are encrypted on the sender's client device before transmission.
- **Only the intended recipient can read the mail:** The Content Encryption Key (CEK) is wrapped exclusively for the recipient's public encryption key using modern asymmetric cryptography (HPKE / X25519).
- **Public keys are tamper-proof:** The blockchain serves as the immutable, decentralized Public Key Infrastructure (PKI), preventing Man-In-The-Middle (MITM) key substitution.
- **Authentication is non-repudiable:** Senders sign message commitments using their blockchain wallet private key.

---

## 2. Public / Private Key Architecture

```text
+---------------------------------------------------------------------------------+
|                                USER WALLET / CLIENT                             |
|                                                                                 |
|   +------------------------------------+   +--------------------------------+   |
|   |         BLOCKCHAIN WALLET          |   |       E2EE ENCRYPTION KEY      |   |
|   |         (Ed25519 / Secp256k1)      |   |            (X25519)            |   |
|   |                                    |   |                                |   |
|   | - Address: 7xK3... (Identity)      |   | - Public Key:  Pk_enc (Share)  |   |
|   | - Private: Sk_sig (Signs tx/mail)  |   | - Private Key: Sk_enc (SECRET) |   |
|   +-----------------+------------------+   +---------------+----------------+   |
|                     |                                      |                    |
+---------------------|--------------------------------------|--------------------+
                      | Signs Registration                   | Publishes Public Key
                      v                                      v
+---------------------------------------------------------------------------------+
|                      ON-CHAIN IDENTITY & KEY REGISTRY                           |
|                                                                                 |
|   Mapping: Wallet Address (7xK3...) ===> Public Encryption Key (Pk_enc)        |
+---------------------------------------------------------------------------------+
```

### 2.1 Separation of Key Roles

In cryptographic engineering, signing keys and encryption keys must be strictly separated:
1. **Wallet Signing Key (`Sk_sig` / `Pk_sig`):**
   - **Algorithm:** Ed25519 (Solana) or Secp256k1 (EVM).
   - **Role:** Controls blockchain funds, signs on-chain transactions, authorizes key registration, and signs outgoing mail manifests for authenticity.
2. **Device Encryption Key (`Sk_enc` / `Pk_enc`):**
   - **Algorithm:** Curve25519 / X25519 (RFC 7748).
   - **Role:** Asymmetric Diffie-Hellman / HPKE recipient key. Used strictly to encapsulate and decapsulate symmetric Content Encryption Keys (CEKs).
   - **Confidentiality:** The private key `Sk_enc` NEVER leaves the client device. It is never stored on the blockchain or sent over the network.

### 2.2 Key Generation and Derivation

Clients support two methods of establishing the encryption keypair:
- **Method A (Deterministic Wallet Derivation):**
  The user's blockchain wallet signs a specific domain separation message:
  ```text
  seed = HKDF-SHA256(wallet.sign("D-MAIL-ENCRYPTION-KEY-V1"), salt="dmail-salt", info="x25519-seed")
  (Sk_enc, Pk_enc) = X25519_DeriveKeyPair(seed)
  ```
  *Benefit:* Allows the user to restore their encryption key on any device simply by connecting their wallet.
- **Method B (Independent Ephemeral/Device Keypair):**
  The client generates a random X25519 keypair using OS CSPRNG, and signs a registration payload with the wallet to link `Pk_enc` to the wallet address on-chain.

---

## 3. On-Chain Smart Contract Specification

The blockchain acts as the immutable registry and mailbox notification bus.

### 3.1 Identity & Key Registry

The Registry contract maintains the mapping from wallet addresses to public encryption keys:

```rust
pub struct UserKeyRecord {
    pub wallet_address: Pubkey,       // 32-byte blockchain address
    pub encryption_pubkey: [u8; 32],  // 32-byte X25519 public key
    pub registered_at: i64,           // Unix timestamp
    pub key_version: u32,             // Increments on key rotation
    pub status: u8,                   // 0 = Active, 1 = Revoked
}
```

#### Registry Instructions:
1. `register_key(encryption_pubkey: [u8; 32], signature: [u8; 64])`:
   - Validates that the transaction was signed by `wallet_address`.
   - Validates proof of possession of the encryption key.
   - Stores or updates `UserKeyRecord` in the account state (e.g. PDA on Solana).
2. `rotate_key(new_encryption_pubkey: [u8; 32])`:
   - Increments `key_version` and updates the active public encryption key.
3. `revoke_key()`:
   - Sets status to revoked, preventing future mail from being encrypted to this key.

### 3.2 Mail Dispatch & On-Chain Notification

When an encrypted mail is dispatched, a transaction records the delivery metadata:

```rust
pub struct MailDispatchEvent {
    pub message_id: [u8; 32],         // Unique 256-bit message identifier
    pub sender: Pubkey,               // Sender's blockchain address
    pub recipient: Pubkey,            // Recipient's blockchain address
    pub ephemeral_pubkey: [u8; 32],   // Sender's ephemeral X25519 public key
    pub wrapped_cek: Vec<u8>,         // HPKE-encapsulated Content Encryption Key
    pub payload_hash: [u8; 32],       // SHA-256 hash of encrypted ciphertext
    pub storage_uri: String,          // URI to ciphertext blob (IPFS / Arweave / local node)
    pub timestamp: i64,               // Dispatch timestamp
}
```

*Note: For short messages (e.g., < 512 bytes), the ciphertext can optionally be stored directly on-chain within transaction calldata / account data.*

---

## 4. End-to-End Encryption (E2EE) Wire Specification

```text
===================================================================================
                             ENCRYPTION WORKFLOW (SENDER)
===================================================================================

[ Plaintext Mail ] : { subject, body, attachments, timestamp, sender, recipient }
        |
        |  1. Generate random 256-bit CEK (Content Encryption Key)
        v
[ Symmetric Cipher: ChaCha20-Poly1305 ] ===> [ Ciphertext Payload + 16-byte Poly1305 Tag ]
        |                                                          |
        |                                                          v
        |                                                [ SHA-256 Payload Hash ]
        |
        |  2. Encapsulate CEK using Recipient's X25519 Public Key
        |     via HPKE (DHKEM(X25519) + HKDF-SHA256 + ChaCha20Poly1305)
        v
[ Wrapped CEK Envelope ] : { ephemeral_pubkey, ciphertext_cek }
        |
        |  3. Construct Manifest & Sign with Sender's Wallet Private Key (Ed25519)
        v
[ Sender Signature ] = Ed25519_Sign(Sk_sig, SHA-256(message_id || recipient || payload_hash))

===================================================================================
```

### 4.1 Symmetric Payload Encryption

- **Algorithm:** ChaCha20-Poly1305 AEAD (RFC 8439).
- **Key:** Unique 256-bit cryptographically secure random `CEK`.
- **Nonce:** 96-bit unique random nonce per message.
- **Associated Data (AAD):** Binds the protocol version, `message_id`, `sender_address`, and `recipient_address`:
  ```text
  AAD = "BMAIL/1.0" || message_id (32B) || sender_address (32B) || recipient_address (32B)
  ```
- **Plaintext Payload Structure:**
  ```json
  {
    "version": "1.0",
    "message_id": "0x...",
    "sender": "7xK3...",
    "recipient": "9bA4...",
    "timestamp": 1731866400,
    "subject": "Confidential Project Proposal",
    "body": "Hello Bob, here are the private keys...",
    "attachments": [
      {
        "filename": "document.pdf",
        "mime_type": "application/pdf",
        "data_base64": "..."
      }
    ]
  }
  ```

### 4.2 Content Encryption Key (CEK) Wrapping (HPKE)

- **Standard:** Hybrid Public Key Encryption (HPKE, RFC 9180) Base Mode.
  - **KEM:** `DHKEM(X25519, HKDF-SHA256)` (ID: `0x0020`).
  - **KDF:** `HKDF-SHA256` (ID: `0x0001`).
  - **AEAD:** `ChaCha20Poly1305` (ID: `0x0003`).
- **Encapsulation:**
  1. Sender generates ephemeral keypair `(Sk_eph, Pk_eph)`.
  2. Computes shared secret via `X25519(Sk_eph, Recipient_Pk_enc)`.
  3. Derives encryption key via `HKDF-SHA256` with info string `"D-MAIL-CEK-WRAP-V1"`.
  4. Encrypts the 32-byte `CEK` using the derived key.
- **Output:** `(Pk_eph [32 bytes], Wrapped_CEK [48 bytes])`.

### 4.3 Decryption Workflow (Recipient Only)

```text
===================================================================================
                             DECRYPTION WORKFLOW (RECIPIENT)
===================================================================================

[ On-Chain Event / Mailbox ] ===> Fetches { Pk_eph, Wrapped_CEK, Payload_URI, Payload_Hash, Signature }
        |
        +---> 1. Verify Sender Signature using Sender's On-Chain Public Key (Pk_sig)
        |
        +---> 2. Download Ciphertext from Storage URI and verify SHA-256 == Payload_Hash
        |
        +---> 3. Decapsulate CEK using Recipient's Private Encryption Key (Sk_enc):
        |        shared_secret = X25519(Sk_enc, Pk_eph)
        |        CEK = HPKE_Decrypt(shared_secret, Wrapped_CEK)
        |
        v
[ Decrypted CEK ]
        |
        v  4. Decrypt Ciphertext Payload with CEK and verify Poly1305 Tag
[ Plaintext Mail ] (Subject, Body, Attachments)
===================================================================================
```

If anyone other than the recipient attempts decryption:
- They do not possess `Sk_enc`.
- Computing the shared secret from `Pk_eph` and `Recipient_Pk_enc` without `Sk_enc` is mathematically infeasible (Computational Diffie-Hellman problem on Curve25519).
- Decryption of the CEK fails completely, rendering the ciphertext payload impossible to decrypt.

---

## 5. Decentralized Storage & Transport Layer

To ensure scalability and low gas costs while preserving decentralization:
1. **Lightweight Messages (< 512 bytes):** Can be transmitted directly on-chain within transaction memos or program logs.
2. **Standard & Large Messages:**
   - The sender encrypts the payload locally.
   - The encrypted ciphertext blob is uploaded to decentralized content-addressed storage:
     - **IPFS:** Referenced by Content Identifier (CID).
     - **Arweave:** Permanent decentralized storage.
     - **Local / Peer-to-Peer Relay Node:** For high-speed local testing and private networks.
   - The content hash (`SHA-256`) is stored on-chain. The recipient verifies that `SHA-256(downloaded_blob) == on_chain_payload_hash`.
   - Even if the storage provider or IPFS node is malicious or public, the data stored is 100% encrypted ciphertext. No party without the recipient's private key can read it.

---

## 6. Security Analysis & Guarantees

| Security Property | Mechanism | Threat Prevented |
|---|---|---|
| **End-to-End Confidentiality** | X25519 HPKE + ChaCha20-Poly1305 | Miners, validators, RPC nodes, storage providers, and network sniffers cannot read email content. |
| **Sender Authenticity** | Wallet digital signature (Ed25519/Secp256k1) | Prevents email spoofing, forging, or impersonation. |
| **Message Integrity** | AEAD Poly1305 tag + SHA-256 payload digest | Any alteration of ciphertext or storage payload is detected and rejected. |
| **Tamper-Proof PKI** | Blockchain consensus state for public keys | Prevents Man-in-the-Middle (MITM) key substitution attacks. |
| **Replay Protection** | On-chain nonce, unique message ID, block height validation | Replayed messages are rejected by the client and smart contract. |
| **Zero Plaintext Leakage** | All sensitive headers (Subject, Attachments, Body) are encrypted inside the payload | Observers only see sender address, recipient address, and timestamp. |

---

## 7. Protocol Error Handling & Edge Cases

1. **Unregistered Recipient:**
   If a sender attempts to email an address that has not registered a public encryption key on-chain, the client halts immediately: `Error: RecipientNotRegisteredOnChain`. The system strictly fails closed and never sends unencrypted data.
2. **Mismatched Payload Hash:**
   If the downloaded ciphertext does not match `on_chain_payload_hash`, the client rejects the message: `Error: StorageTamperingDetected`.
3. **Invalid Signature:**
   If the sender's signature over the manifest is invalid, the client flags the message as forged: `Error: InvalidSenderSignature`.
4. **Key Revocation:**
   If a recipient key was revoked before the message block timestamp, the message is marked untrusted.
