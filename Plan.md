# Blockchain End-to-End Encrypted Mail POC (D-Mail)
## Master Implementation & Execution Plan

**Project:** Blockchain E2EE Mail POC (D-Mail)  
**Plan Version:** 1.0  
**Status:** Ready for Execution  
**Normative Reference:** `Protocol.md`  

---

## 1. Execution Directives for Coding Agents

1. **Strict Cryptographic Integrity:** Use only audited, maintained cryptographic libraries (`x25519-dalek`, `ed25519-dalek`, `chacha20poly1305`, `sha2`, `hkdf`). Do not write custom cryptographic primitives.
2. **Zero-Plaintext Invariant:** The blockchain, storage relays, and RPC providers must NEVER receive or store plaintext subjects, bodies, attachments, or Content Encryption Keys (CEKs).
3. **Fail-Closed Security:** If key lookup fails or the recipient is not registered, the system must immediately abort and reject sending. Unencrypted fallback is strictly forbidden.
4. **Verifiable Increments:** Implement crate by crate with passing unit and integration tests at each phase gate.
5. **Demonstrable Proof of E2EE:** The final integration test must mathematically demonstrate that:
   - Alice encrypts for Bob.
   - Bob decrypts successfully.
   - An unauthorized party (Charlie / node operator / observer) possessing all on-chain data and storage blobs cannot decrypt the message.

---

## 2. Proposed Architecture & Repository Layout

```text
dmail/
├── README.md                      # High-level overview, architecture, quickstart
├── Protocol.md                    # Normative protocol specification
├── Plan.md                        # This execution plan
├── rust-toolchain.toml            # Pinned Rust compiler toolchain
├── Cargo.toml                     # Cargo workspace definition
│
├── crates/
│   ├── dmail-crypto/              # X25519, Ed25519, HPKE CEK wrapping, ChaCha20-Poly1305
│   ├── dmail-types/               # Protocol identifiers, envelopes, serialization
│   ├── dmail-storage/             # Content-addressed encrypted payload storage (IPFS / local)
│   ├── dmail-chain/               # Smart contract / blockchain interface & local simulator
│   └── dmail-sdk/                 # High-level client SDK orchestrating E2EE workflow
│
├── programs/                      # On-chain smart contract programs
│   └── dmail-registry/            # Smart contract for key registration and mail dispatch
│
├── clients/
│   └── dmail-cli/                 # Reference CLI client for wallet setup, compose, send, inbox
│
└── tests/
    └── e2e_test.rs                # End-to-end integration test verifying two-user E2EE flow
```

---

## 3. Phase Breakdown & Execution Order

### Phase 0: Workspace Bootstrap & Environment Controls
**Goal:** Establish reproducible Rust workspace, linting, toolchain, and CI definitions.

- **Task 0.1:** Initialize Cargo workspace root `Cargo.toml` with workspace dependencies.
- **Task 0.2:** Pin Rust toolchain (`rust-toolchain.toml`) and formatting rules (`rustfmt.toml`).
- **Task 0.3:** Configure CI workflow (`.github/workflows/ci.yml`) for `fmt`, `clippy`, and `test`.

**Acceptance Criteria:**
- `cargo check --workspace` passes cleanly.
- Toolchain and formatting configurations verified.

---

### Phase 1: Cryptographic Engine (`dmail-crypto`)
**Goal:** Build pure, self-contained cryptographic operations for key derivation, HPKE encapsulation, and AEAD payload encryption.

- **Task 1.1: Key Generation & Derivation:**
  - Generate Ed25519 wallet signing keypair.
  - Generate / derive X25519 device encryption keypair via HKDF from signature seed.
  - Implement secret key zeroization on drop.
  - Redact secret keys from `Debug` and `Display` outputs.
- **Task 1.2: Symmetric Payload Encryption (ChaCha20-Poly1305):**
  - Generate cryptographically secure 256-bit random CEK.
  - Encrypt payload bytes with authenticated AAD binding.
  - Decrypt payload bytes and verify Poly1305 tag.
  - Reject tampered ciphertext or modified AAD.
- **Task 1.3: Asymmetric Key Wrapping (HPKE / X25519):**
  - Encapsulate CEK using recipient's X25519 public key.
  - Decapsulate CEK using recipient's X25519 private key.
  - Negative test: Decapsulation with wrong private key fails.
- **Task 1.4: Manifest Signing:**
  - Compute Ed25519 digital signature over canonical message commitment.
  - Verify signature against sender's wallet public key.

**Acceptance Criteria:**
- 100% test pass rate for `cargo test -p dmail-crypto`.
- Known-answer test vectors verify roundtrip encryption and decryption.
- Unauthorized key attempts return explicit cryptographic errors.

---

### Phase 2: Protocol Types & Wire Serialization (`dmail-types`)
**Goal:** Define stable, strongly-typed data structures for mail envelopes, manifests, and registry records.

- **Task 2.1: Identity & Key Records:**
  - `WalletAddress`: 32-byte public key wrapper with Base58 / Hex formatting.
  - `EncryptionPublicKey`: 32-byte X25519 public key wrapper.
  - `UserKeyRecord`: On-chain registration record with timestamps and versioning.
- **Task 2.2: Message Structures:**
  - `MailPayload`: Subject, body, timestamp, sender, recipient, optional attachments.
  - `EncryptedPayload`: Ciphertext bytes, 12-byte nonce, 16-byte tag, SHA-256 payload hash.
  - `WrappedCekEnvelope`: Ephemeral X25519 public key + encapsulated CEK bytes.
  - `MailManifest`: Metadata header signed by the sender.
- **Task 2.3: Deterministic Encoding:**
  - Implement canonical serialization for signatures and on-chain events.

**Acceptance Criteria:**
- Strongly-typed constructors validate lengths at boundaries.
- Serialization and deserialization roundtrip tests pass.

---

### Phase 3: Blockchain Interface & Smart Contract (`dmail-chain`)
**Goal:** Implement the on-chain Public Key Infrastructure (PKI) and Mail Dispatch notification engine.

- **Task 3.1: Blockchain Interface Trait:**
  - Define asynchronous / synchronous trait:
    ```rust
    pub trait BlockchainClient {
        fn register_key(&self, wallet: &WalletAddress, pubkey: &EncryptionPublicKey, sig: &[u8]) -> Result<()>;
        fn get_encryption_key(&self, wallet: &WalletAddress) -> Result<Option<EncryptionPublicKey>>;
        fn dispatch_mail(&self, event: &MailDispatchRecord) -> Result<[u8; 32]>;
        fn fetch_inbox(&self, recipient: &WalletAddress) -> Result<Vec<MailDispatchRecord>>;
    }
    ```
- **Task 3.2: Local In-Memory & File-Backed Simulator:**
  - Build `LocalChainSimulator` providing instant, deterministic local blockchain state for offline development, integration tests, and CLI demos.
- **Task 3.3: Smart Contract Specification & Implementation:**
  - Provide Rust / Anchor compatible smart contract logic for Solana / EVM deployment.

**Acceptance Criteria:**
- Users can register keys and query public keys.
- Mail dispatch events are queryable by recipient wallet address.
- Unauthorized key overwrite attempts are rejected.

---

### Phase 4: Content-Addressed Storage Adapter (`dmail-storage`)
**Goal:** Manage off-chain ciphertext storage while ensuring complete server blindness.

- **Task 4.1: Storage Engine Trait:**
  - Define `StorageAdapter` trait:
    ```rust
    pub trait StorageAdapter {
        fn put(&self, data: &[u8]) -> Result<String>; // Returns URI / CID
        fn get(&self, uri: &str) -> Result<Vec<u8>>;
    }
    ```
- **Task 4.2: Local / Content-Addressed File Storage:**
  - Implement local content-addressed storage indexed by SHA-256 / CID.
- **Task 4.3: Payload Verification:**
  - Downloaded blob MUST match on-chain `payload_hash`; reject with error if tampered.

**Acceptance Criteria:**
- Uploading ciphertext produces deterministic content-addressed URI.
- Retrieving ciphertext verifies integrity against on-chain hash.

---

### Phase 5: Client SDK (`dmail-sdk`)
**Goal:** Unify cryptography, blockchain PKI, and storage into a seamless developer API.

- **Task 5.1: Client Account Setup:**
  - `BmailClient::create_wallet()`
  - `BmailClient::register_on_chain()`
- **Task 5.2: Send Mail Flow:**
  - `BmailClient::send_mail(recipient, subject, body, attachments)`:
    1. Resolve recipient's `EncryptionPublicKey` from blockchain.
    2. Generate random 256-bit CEK.
    3. Encrypt payload via ChaCha20-Poly1305.
    4. Store ciphertext in storage adapter -> receive storage URI.
    5. Wrap CEK for recipient via HPKE-X25519.
    6. Sign message manifest with wallet.
    7. Dispatch notification transaction on-chain.
- **Task 5.3: Receive Mail Flow:**
  - `BmailClient::get_inbox()`:
    1. Query blockchain for incoming dispatch events.
    2. Download ciphertext from storage URI.
    3. Verify payload hash against on-chain record.
    4. Verify sender signature using sender's wallet address.
    5. Decapsulate CEK using local `Sk_enc`.
    6. Decrypt ciphertext using CEK.
    7. Return decrypted `MailPayload`.

**Acceptance Criteria:**
- Alice can send to Bob; Bob can retrieve and decrypt.
- All steps validate signatures, tags, and digests.

---

### Phase 6: Reference CLI & Automated E2E Security Test (`dmail-cli`)
**Goal:** Deliver a working CLI tool and an automated test suite proving end-to-end security.

- **Task 6.1: Command-Line Interface:**
  - `dmail init` — Generate wallet and encryption keys.
  - `dmail register` — Publish public key to blockchain registry.
  - `dmail send --to <address> --subject <text> --body <text>` — Encrypt and dispatch mail.
  - `dmail inbox` — List incoming messages.
  - `dmail read <message-id>` — Decrypt and display mail.
- **Task 6.2: Comprehensive E2E Verification Tests:**
  - **Happy Path:** Alice sends confidential mail to Bob -> Bob reads it.
  - **Attacker Test (Charlie):** Third party with access to blockchain and storage attempts to decrypt -> fails with cryptographic error.
  - **Tampering Test:** Modifying 1 byte in ciphertext causes decryption to fail.
  - **Spoofing Test:** Modifying sender signature is rejected as invalid.
  - **Unregistered Recipient Test:** Attempting to send to an unregistered address fails closed.

**Acceptance Criteria:**
- `cargo test --workspace` passes 100%.
- CLI commands work interactively and non-interactively.
- Mathematical proof of E2EE demonstrated in tests.

---

## 4. Definition of Done (DoD)

A task or phase is complete only when:
1. Code compiles without warnings (`RUSTFLAGS="-D warnings"`).
2. Unit and integration tests pass.
3. Cryptographic secret zeroization is implemented.
4. No sensitive private keys or plaintext data are printed in logs or debug statements.
5. All acceptance criteria for the phase are verified.
