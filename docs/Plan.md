# DMN Implementation Plan
## Agent-Executable Engineering Plan

**Project:** Decentralized Mail Network (DMN)  
**Plan version:** 0.1  
**Status:** Implementation baseline  
**Date:** 17 November 2026  
**Audience:** Software engineers and coding agents implementing the system  
**Normative references:** `decentralized_mail_high_level_design.md` and `decentralized_mail_protocol_specification.md`

---

# 0. Instructions for an Implementing LLM

This file is an execution plan. Implement the project in small, verifiable increments. Do not attempt to generate the entire system in one pass.

## 0.1 Mandatory working rules

1. Read this plan and both design documents before changing code.
2. Treat the HLD and protocol specification as the architecture baseline. If they conflict, stop and report the conflict; do not silently choose.
3. Do not invent cryptographic protocols, algorithms, wire formats, DNS records, or security semantics.
4. Do not use custom cryptography. Use maintained, reviewed libraries and standard protocols.
5. Do not claim E2EE until tests demonstrate that mail servers and storage services cannot decrypt native messages.
6. Do not silently downgrade an explicitly encrypted message to plaintext.
7. Do not put plaintext messages, private keys, CEKs, full recipient lists, or unnecessary personal data on-chain or in logs.
8. Do not make Solana availability a dependency for SMTP delivery.
9. Do not fork Stalwart or modify its internals in the MVP. Integrate through supported protocols and documented APIs.
10. Keep all services runnable locally with reproducible commands.
11. Every feature must include unit tests. Protocol, cryptographic, and interoperability features also require integration and negative tests.
12. Update `README.md`, architecture records, protocol documents, and this plan when implementation decisions change.
13. Never mark a task complete unless its acceptance criteria pass.
14. If an essential design choice is unresolved, create an ADR (Architecture Decision Record), explain options and risks, and request human approval before implementing security-sensitive behaviour.
15. Prefer deterministic builds, pinned dependencies, reproducible test fixtures, and explicit versioning.

## 0.2 Execution loop

For each task:

1. Inspect the existing repository and identify related code.
2. State the intended change and files to be modified.
3. Implement only the task's scope.
4. Run formatting, linting, unit tests, and relevant integration tests.
5. Inspect the diff for secrets, plaintext leakage, unsafe defaults, and unrelated changes.
6. Update documentation.
7. Report files changed, commands run, test results, known limitations, and next task.

Do not proceed past a phase gate if acceptance criteria fail.

## 0.3 Human approval required

A human reviewer must approve these decisions before production use:

- Cryptographic suite and canonical serialization.
- Identity ownership, recovery, and key revocation policy.
- BCC privacy design.
- Public DNS discovery format.
- Native MIME media type and envelope wire format.
- Solana account/program authority model.
- Any fallback from E2EE to plaintext.
- Production key backup and recovery implementation.

---

# 1. Product Scope and MVP

## 1.1 MVP outcome

Build a working federated email proof of concept with:

- Two independently configured mail domains/nodes.
- A web or CLI client for two users.
- Local generation and storage of device keys.
- Authenticated identity/key records.
- Native E2EE message creation and decryption.
- Signed message manifests.
- SMTP transport between the two mail nodes.
- IMAP or JMAP retrieval.
- Encrypted attachment support.
- PostgreSQL-backed identity metadata.
- S3-compatible object storage for encrypted objects where needed.
- Automated tests proving that mail nodes and storage cannot decrypt native messages.

The MVP must work without Solana. Blockchain anchoring is a later phase.

## 1.2 Explicitly out of scope for MVP

- Production-grade anonymity or traffic-analysis resistance.
- Global deletion or message recall.
- Mandatory blockchain transactions for sending messages.
- Running a public validator network.
- A custom SMTP server.
- A custom cryptographic primitive.
- Full Gmail/Outlook account integration beyond standards-based email exchange.
- Mobile apps.
- Production multi-region active-active deployment.
- A promise of forward secrecy unless a ratcheting/session protocol is separately specified and reviewed.

## 1.3 MVP demonstration

The final MVP demonstration must show:

1. Alice on Node A registers an identity and device.
2. Bob on Node B registers an identity and device.
3. Alice discovers and validates Bob's public encryption key.
4. Alice creates an encrypted message and signs its manifest.
5. Node A submits the MIME message through SMTP to Node B.
6. Node B stores the message without decrypting it.
7. Bob retrieves the message through IMAP/JMAP.
8. Bob verifies the sender and decrypts the message locally.
9. A modified ciphertext, invalid signature, unknown key, or replayed message is rejected.
10. An ordinary unencrypted email can still pass through the mail server as conventional email.

---

# 2. Architecture Baseline

## 2.1 Initial technology choices

| Area | Initial choice |
|---|---|
| Primary language | Rust |
| Workspace | Cargo workspace |
| Mail server | Stalwart |
| Mail transport | SMTP |
| Mailbox access | IMAP initially; JMAP adapter can follow |
| Message interoperability | MIME and OpenPGP/MIME |
| Native envelope | Versioned DMN envelope; exact wire format requires ADR |
| Signing | Ed25519 candidate |
| Public-key encryption | HPKE with X25519 candidate |
| Content encryption | One selected AEAD suite after ADR |
| Hash | SHA-256 |
| Database | PostgreSQL |
| Object storage | S3-compatible API, MinIO locally |
| Event queue | NATS, only when asynchronous service boundaries require it |
| Blockchain | Solana/Agave, deferred |
| Smart contracts | Rust/Anchor or native Solana program, decision deferred |
| Local orchestration | Docker Compose |
| CI | GitHub Actions or equivalent |

Do not add Redis, Kubernetes, IPFS, or NATS merely because they appear in the HLD. Add them only when a concrete requirement exists.

## 2.2 Proposed repository layout

```text
decentral-mail/
├── README.md
├── CONTRIBUTING.md
├── LICENSE
├── Cargo.toml
├── rust-toolchain.toml
├── .gitignore
├── .env.example
│
├── docs/
│   ├── Plan.md
│   ├── HLD.md
│   ├── Protocol.md
│   ├── threat-model.md
│   ├── deployment.md
│   ├── adr/
│   └── test-vectors/
│
├── crates/
│   ├── dmn-types/
│   ├── dmn-encoding/
│   ├── dmn-crypto/
│   ├── dmn-identity/
│   ├── dmn-transparency/
│   ├── dmn-envelope/
│   ├── dmn-mime/
│   ├── dmn-key-directory/
│   ├── dmn-federation/
│   ├── dmn-mail-adapter/
│   ├── dmn-storage/
│   ├── dmn-db/
│   ├── dmn-api/
│   └── dmn-client-sdk/
│
├── services/
│   ├── identity-api/
│   ├── transparency-api/
│   ├── federation-worker/
│   └── mailbox-notifier/
│
├── clients/
│   ├── cli/
│   └── web/
│
├── blockchain/
│   └── identity-registry/
│
├── migrations/
├── deploy/
│   ├── docker/
│   ├── compose/
│   └── monitoring/
│
├── scripts/
└── tests/
    ├── integration/
    ├── interoperability/
    ├── security/
    └── fixtures/
```

The repository may start smaller. Do not create empty crates or services without an implementation need. Keep protocol/domain logic in libraries and network/database adapters at the edges.

---

# 3. Phase 0 — Repository Bootstrap and Engineering Controls

## Goal

Create a reproducible Rust workspace and local development environment.

## Tasks

### P0.1 Repository inspection

- Inspect current files and git history.
- Preserve existing code and naming where sensible.
- Confirm whether the HLD and protocol documents are present.
- Create `docs/` and place the approved documents there.
- Record existing technical debt and unresolved decisions.

**Acceptance criteria**
- Repository inventory documented.
- No existing work overwritten.
- Build instructions documented.

### P0.2 Rust workspace

- Create Cargo workspace.
- Pin Rust toolchain.
- Configure formatting and linting.
- Configure dependency policy and lockfile.
- Add CI workflow for formatting, Clippy, build, and tests.

**Acceptance criteria**
- `cargo fmt --all -- --check` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` passes.
- `cargo test --workspace` passes.
- CI runs the same commands.

### P0.3 Local infrastructure

Create Docker Compose configuration for:

- PostgreSQL.
- Stalwart.
- MinIO.
- Optional local SMTP test sink.
- Optional Solana local validator only when blockchain work begins.

Provide health checks, persistent development volumes, non-default credentials, and `.env.example`.

**Acceptance criteria**
- One documented command starts the local environment.
- Services report healthy.
- No secrets are committed.
- Environment can be reset without manual cleanup.

### P0.4 Architecture decision records

Create ADRs for:

- Canonical serialization.
- Crypto suite.
- Native envelope and MIME binding.
- Identity ownership/recovery.
- BCC privacy.
- DNS discovery.
- Mail-server integration.
- Blockchain account authority.

**Gate 0:** Reproducible build, CI, and local infrastructure work.

---

# 4. Phase 1 — Protocol Types and Canonical Encoding

## Goal

Create stable, validated protocol types before implementing network services.

## Tasks

### P1.1 Define core types

Implement types for:

- `IdentityId`
- `DeviceId`
- `KeyId`
- `MessageId`
- `DomainName`
- `EmailAddress`
- `KeyPurpose`
- `KeyStatus`
- `DeviceStatus`
- `ProtocolVersion`
- `AlgorithmSuiteId`
- `Timestamp`
- `Hash32`

Requirements:

- Validate lengths and syntax at construction.
- Avoid using arbitrary strings for typed identifiers.
- Avoid logging secret-bearing structures.
- Implement safe redacted `Debug` output for secret types.

**Acceptance criteria**
- Unit tests cover valid and invalid values.
- Invalid values cannot be constructed through public constructors.
- Serialization is deterministic under the selected encoding.

### P1.2 Canonical encoding decision

Evaluate deterministic CBOR and canonical JSON.

The decision must define:

- Field ordering/canonical representation.
- Integer and timestamp representation.
- Binary encoding.
- Duplicate-field handling.
- Unknown field handling.
- Version negotiation.
- Signature input bytes.

Do not implement signatures over ordinary `serde_json::to_vec()` output unless a canonical JSON profile is formally selected.

**Acceptance criteria**
- ADR approved.
- Canonical encoding test vectors committed.
- Two independent implementations can produce identical bytes.

### P1.3 Protocol versioning

Define version parsing and compatibility rules.

- Unknown major version: reject.
- Unknown critical field: reject.
- Unknown optional field: ignore only where specified.
- Unknown mandatory crypto suite: reject.
- No automatic crypto downgrade.

**Gate 1:** Protocol types and canonical encoding are frozen for the MVP.

---

# 5. Phase 2 — Cryptographic Core

## Goal

Implement local cryptographic operations with no network, blockchain, or mail-server dependency.

## Security constraint

This phase requires an approved ADR selecting exact algorithms and libraries. The HLD's candidate algorithms are not sufficient to define a wire-compatible cryptographic profile.

## Tasks

### P2.1 Key generation and key types

Implement:

- Identity signing key generation.
- Device signing key generation.
- Device encryption key generation.
- Public-key serialization.
- Key identifiers.
- Key-purpose validation.
- Secure secret-key handling.

Private key material must not implement ordinary serializable traits. Do not persist private keys in PostgreSQL.

**Acceptance criteria**
- Key generation uses OS-backed CSPRNG.
- Public and private key encodings have test vectors.
- Secret material is redacted from logs/debug output.
- Key purpose mismatch is rejected.

### P2.2 Sign and verify

Implement a narrow API:

```rust
trait Signer {
    fn public_key(&self) -> PublicSigningKey;
    fn sign(&self, message: &[u8]) -> Result<Signature>;
}

fn verify(
    public_key: &PublicSigningKey,
    message: &[u8],
    signature: &Signature,
) -> Result<()>;
```

The exact API may differ, but signing must accept explicit bytes and verification must fail closed.

**Acceptance criteria**
- Valid signature verifies.
- Modified message fails.
- Modified signature fails.
- Wrong public key fails.
- Malformed key/signature fails safely.

### P2.3 HPKE recipient wrapping

Implement the approved HPKE suite for wrapping a CEK to a device public key.

Bind protocol context to:

- Protocol version.
- Message ID.
- Recipient identity ID.
- Device ID.
- Recipient key ID.
- Crypto-suite ID.

**Acceptance criteria**
- Intended device unwraps the CEK.
- Wrong private key fails.
- Context substitution fails.
- Tampering fails.
- Test vectors are committed.

### P2.4 Content encryption

Implement the selected AEAD for message payloads.

- Generate a unique CEK per message.
- Generate nonce according to selected AEAD requirements.
- Authenticate the protocol context as associated data.
- Reject invalid tags.
- Do not return partial plaintext.

**Acceptance criteria**
- Encrypt/decrypt round trip passes.
- Ciphertext modification fails.
- Associated-data modification fails.
- Nonce misuse constraints are documented and tested where testable.

### P2.5 Key backup

Do not implement production recovery in this phase. Define a backup format only after a separate review.

A local development key store may use OS keychain integration or an encrypted test-only file. Clearly label test-only storage.

**Gate 2:** Cryptographic unit tests and test vectors pass. Independent review is required before production claims.

---

# 6. Phase 3 — Identity Records and Local Identity Service

## Goal

Register identities and devices, publish signed public-key records, and resolve keys.

## Tasks

### P3.1 Identity schema

Implement database migrations and domain types for:

- Identity.
- Domain.
- Identity authority.
- Device.
- Public key.
- Key lifecycle event.
- Identity record version.

Store public key material and metadata only. Private keys remain client-side.

Add unique constraints and foreign keys. Use transactions for key lifecycle changes.

### P3.2 Domain verification

Implement a development verification method first, such as a DNS TXT challenge or an HTTP well-known challenge.

The production DNS record format must be defined in an ADR.

**Acceptance criteria**
- Unverified domains cannot register arbitrary identities.
- Challenge expiry and replay are tested.
- Verification is auditable.

### P3.3 Identity registration API

Implement authenticated endpoints or commands to:

- Create identity.
- Add initial device.
- Publish signed identity record.
- Fetch identity record.
- Fetch device keys.
- Fetch key status.

The service must verify signatures and proof of possession before accepting records.

### P3.4 Device lifecycle

Implement:

- Add device.
- List devices.
- Retire device.
- Revoke device.
- Rotate device keys.

Every change must create an immutable lifecycle event. A current-state table may be maintained as a projection.

### P3.5 Identity resolution

Implement lookup by email address and return:

- Identity record.
- Domain authorization.
- Device public keys.
- Key versions/status.
- Record expiry.
- Transparency state when available.

Do not return private account data or unnecessary device metadata.

**Acceptance criteria**
- Two users can register and resolve each other's keys.
- Unauthorized key changes fail.
- Revoked keys are not returned as active.
- Expired and malformed records are rejected.

**Gate 3:** Local identity registration and resolution work end-to-end.

---

# 7. Phase 4 — Native Message Envelope

## Goal

Create, sign, validate, encrypt, and decrypt native DMN messages.

## Tasks

### P4.1 Envelope data model

Implement versioned types for:

- Common message manifest.
- Ciphertext payload.
- Recipient envelope.
- Sender signature.
- Optional attachment descriptors.
- Optional protocol extensions.

The final wire format must follow the Phase 1 canonical encoding decision.

### P4.2 Message creation

Client SDK operation should:

1. Validate sender identity and signing key.
2. Resolve recipient identity and active devices.
3. Verify recipient key records and freshness policy.
4. Create canonical MIME payload.
5. Generate CEK.
6. Encrypt payload.
7. Wrap CEK for every authorized recipient device.
8. Build canonical manifest.
9. Sign the required manifest/commitments.
10. Return a transport-ready envelope.

### P4.3 Message verification and decryption

Recipient operation should:

1. Parse envelope with strict size and field limits.
2. Validate version and algorithms.
3. Validate recipient identity/device/key IDs.
4. Resolve sender signing key and trust state.
5. Verify manifest signature.
6. Verify ciphertext hash.
7. Unwrap CEK.
8. Authenticate and decrypt payload.
9. Parse MIME safely.
10. Return plaintext only after all checks succeed.

### P4.4 Multi-recipient support

Support To and CC by wrapping the same CEK independently to each authorized device.

Do not implement BCC using a public shared recipient list. BCC requires a separately approved privacy design. Until approved, explicitly mark BCC unsupported in native DMN mode rather than leaking recipients.

### P4.5 Replay protection

Implement message ID uniqueness and recipient-side duplicate tracking.

- Same ID and same authenticated content: duplicate.
- Same ID and different content: reject.
- Expired replay record: behaviour follows documented retention policy.

### P4.6 Attachments

Encrypt attachments before upload or MIME inclusion. The server receives ciphertext only.

Start with attachments embedded in the encrypted MIME payload. Add chunked large-object support only after message-size requirements are established.

**Acceptance criteria**
- Single-recipient round trip passes.
- Multi-recipient round trip passes.
- Wrong recipient cannot decrypt.
- Invalid signature fails.
- Modified ciphertext fails.
- Replayed messages are detected.
- Server-side test cannot recover plaintext or CEK.

**Gate 4:** Native E2EE message path works locally without SMTP.

---

# 8. Phase 5 — MIME and OpenPGP Interoperability

## Goal

Carry native DMN messages over standards-based email and exchange compatible OpenPGP/MIME messages.

## Tasks

### P5.1 MIME parser and generator

Use a maintained MIME library. Do not write a MIME parser from scratch.

Implement:

- Text/plain.
- Text/html.
- Multipart/alternative.
- Attachments.
- Content transfer encoding.
- Safe filename handling.
- Header normalization.
- Message size limits.

### P5.2 Native envelope MIME binding

Select and document the final MIME media type and encoding.

The provisional development media type in the protocol document is not automatically a production registration.

The outer MIME message should contain only the metadata necessary for SMTP delivery. The protected payload contains private subject/body/attachment data according to the selected profile.

### P5.3 OpenPGP/MIME adapter

Use a maintained OpenPGP implementation/library.

Implement:
- Parse compatible encrypted OpenPGP/MIME.
- Create encrypted OpenPGP/MIME where recipient key and policy allow.
- Verify signatures.
- Import/export public keys.
- WKD lookup if included in the approved scope.

Do not conflate native DMN envelope encryption with OpenPGP. They are separate profiles.

### P5.4 Downgrade policy

If the sender selects E2EE and recipient key resolution or encryption fails:

- Return a clear error.
- Do not send plaintext automatically.
- Require an explicit user action to create an ordinary email.
- Record the selected send mode.

**Acceptance criteria**
- MIME round-trip preserves body and attachments.
- Native envelope survives SMTP/MIME transport unchanged.
- OpenPGP test vectors interoperate with at least one independent client.
- E2EE downgrade tests pass.

**Gate 5:** Message formats are interoperable and downgrade-safe.

---

# 9. Phase 6 — Stalwart Integration and SMTP Delivery

## Goal

Use Stalwart for mail transport and mailbox access without giving it plaintext access to native messages.

## Tasks

### P6.1 Local Stalwart environment

- Configure two separate domains.
- Configure SMTP submission and server-to-server SMTP.
- Configure mailbox accounts.
- Configure IMAP or JMAP access.
- Configure TLS for development and production profiles.
- Document DNS records for a real deployment.

### P6.2 SMTP sender adapter

Implement a Rust adapter that submits a complete MIME message to Stalwart.

Responsibilities:
- SMTP authentication.
- TLS policy.
- Envelope sender/recipient.
- Message submission.
- Error classification.
- Retryable vs permanent failures.

Do not embed cryptographic private keys in the mail adapter.

### P6.3 SMTP receiving path

The receiving mail server stores native envelope MIME without decrypting it.

Validate:
- Message size.
- MIME structure.
- Required native envelope syntax.
- Sender/domain policy.
- Spam/abuse policy.
- Duplicate message identifier.

Cryptographic verification may occur at the client and optionally at a server-side metadata-validation service, but the server must not need plaintext or private keys.

### P6.4 Mailbox retrieval

Implement a client adapter for IMAP first. JMAP can be added after the core path is stable.

The client retrieves the MIME message and passes it to the DMN SDK for validation/decryption.

### P6.5 Two-node integration test

Run Node A and Node B as separate service configurations, preferably separate Docker networks or isolated containers.

**Acceptance criteria**
- Alice sends from Node A to Bob on Node B.
- SMTP delivery succeeds.
- Bob retrieves via IMAP/JMAP.
- Bob decrypts locally.
- Node A and Node B cannot decrypt.
- A normal plaintext test email still works.

**Gate 6:** Two-node E2EE email federation works over SMTP.

---

# 10. Phase 7 — Key Transparency

## Goal

Detect unauthorized identity/key changes through an append-only Merkle log.

## Tasks

### P7.1 Select transparency design

Create an ADR selecting:
- Tree construction.
- Leaf encoding.
- Hash algorithm.
- Tree-head signature format.
- Inclusion proof format.
- Consistency proof format.
- Monitor/gossip model.

Review Certificate Transparency concepts, but do not claim RFC 9162 compatibility unless the implementation actually conforms.

### P7.2 Append-only log

Implement:
- Append identity-record hash.
- Generate signed checkpoint.
- Fetch checkpoint.
- Fetch inclusion proof.
- Fetch consistency proof.
- Verify proofs in client SDK.

### P7.3 Client enforcement

Clients must:
- Verify signed checkpoints.
- Verify inclusion proofs.
- Verify consistency between previously seen and current checkpoints.
- Detect rollback or conflicting tree heads.
- Warn or block on suspicious key changes according to policy.

### P7.4 Independent monitor

Implement a small monitor that periodically fetches checkpoints and detects inconsistent views.

**Acceptance criteria**
- Valid inclusion and consistency proofs pass.
- Altered proof fails.
- Rollback is detected.
- Conflicting tree heads are reported.
- Key replacement without authorization is detectable.

**Gate 7:** Identity key history is verifiable independently of the identity API operator.

---

# 11. Phase 8 — Federation Discovery and Delivery Hardening

## Goal

Make independently operated domains discover one another and exchange native messages reliably.

## Tasks

### P8.1 Domain capability discovery

Define and approve:
- DNS record name and format.
- DNSSEC expectations.
- HTTPS well-known fallback.
- Supported protocol versions.
- Identity endpoint.
- Transparency endpoint.
- Optional native federation endpoint.

### P8.2 Federation resolver

Implement:
- MX lookup.
- DMN capability lookup.
- TLS verification.
- Endpoint timeout.
- Cache and expiry.
- DNS failure classification.
- Fallback to SMTP when native endpoint is unavailable.

### P8.3 Delivery state machine

Use explicit states:

```text
CREATED
  -> ENCRYPTED
  -> QUEUED
  -> SUBMITTED
  -> ACCEPTED_BY_REMOTE
  -> DELIVERED_TO_MAILBOX (if observable)
  -> FAILED_PERMANENTLY
  -> EXPIRED
```

Do not represent SMTP acceptance as human delivery or read confirmation.

### P8.4 Retry and idempotency

- Exponential backoff with jitter.
- Maximum retry age.
- Permanent vs temporary error handling.
- Stable message ID.
- Duplicate suppression.
- Dead-letter or failed-message inspection.
- Operator-visible queue metrics.

### P8.5 Abuse protection

Implement:
- Per-domain and per-account rate limits.
- Message and attachment size limits.
- Sender authentication.
- Spam scoring integration.
- Quotas.
- Blocklists and allowlists.
- Abuse reporting and administrative controls.

**Acceptance criteria**
- Two independent operators can configure nodes without shared databases.
- DNS discovery and SMTP fallback work.
- Temporary outages recover through retry.
- Permanent failures generate correct bounce/status.
- Duplicate submissions do not create duplicate mailbox entries.

**Gate 8:** Federation is reliable under expected network failures.

---

# 12. Phase 9 — Solana Identity Anchoring

## Goal

Add optional blockchain-backed identity authority and transparency checkpoint commitments without coupling message delivery to chain availability.

## Prerequisites

Do not start until Phases 1–8 work and the identity authority model is approved.

## Tasks

### P9.1 Solana design ADR

Decide:
- Whether the program is Anchor-based or native Rust.
- Domain authority model.
- Identity authority model.
- PDA seeds and account layout.
- Key rotation/revocation authority.
- Recovery authority.
- Checkpoint publication cadence.
- Upgrade authority and governance.
- Mainnet cost and privacy analysis.

### P9.2 Local program

Implement only:
- Register authority/identity commitment.
- Update authorized key commitment.
- Revoke or rotate authority/key commitment.
- Publish transparency checkpoint hash.

Do not store email body, recipient lists, message hashes that reveal communication relationships, or private data.

### P9.3 Client verification

Implement a chain adapter that:
- Reads account state.
- Validates program ownership and expected account layout.
- Handles RPC errors and stale state.
- Applies the selected commitment/finality policy.
- Verifies that the on-chain commitment matches the transparency checkpoint.

### P9.4 Resilience

The identity service caches verified state. SMTP delivery must continue during RPC outages using the configured trust/freshness policy.

**Acceptance criteria**
- Local validator tests pass.
- Unauthorized state changes fail.
- Checkpoint mismatch is detected.
- RPC outage does not stop ordinary SMTP delivery.
- No message content or recipient relationship is written on-chain.

**Gate 9:** Blockchain anchoring adds independently verifiable trust without becoming a delivery dependency.

---

# 13. Phase 10 — Client Application

## Goal

Provide a usable client that owns keys and performs all native message cryptography locally.

## Initial client

Build a CLI first to exercise protocol operations. Add a web client after the SDK is stable.

## Tasks

### P10.1 CLI

Commands:
- `identity create`
- `identity show`
- `device add`
- `device revoke`
- `identity resolve <address>`
- `mail compose`
- `mail send`
- `mail inbox`
- `mail decrypt <message-id>`
- `mail verify <message-id>`

CLI output must not print private keys or plaintext unless the user explicitly requests display.

### P10.2 Web client

Features:
- Account setup.
- Device key generation.
- Identity verification.
- Compose and send.
- Inbox and message details.
- Signature/key trust status.
- Attachment handling.
- Key-change warnings.
- Explicit plaintext fallback confirmation.

Private keys must remain client-side and be protected by appropriate browser storage/security controls. Do not place long-lived private keys in localStorage. Evaluate WebCrypto, IndexedDB with wrapping, platform keystores, and hardware-backed options.

### P10.3 Multi-device UX

Show registered devices and their status. Make device addition and revocation explicit. Explain that removing a device does not erase messages or keys it already obtained.

**Acceptance criteria**
- CLI can complete the two-node demo.
- Web client can send and receive a native message.
- Key trust state is visible.
- Plaintext fallback requires explicit confirmation.

---

# 14. Phase 11 — Production Hardening

## Goal

Prepare the system for controlled pilot deployment.

## Tasks

- Threat-model review.
- Independent cryptographic review.
- Dependency and supply-chain audit.
- Fuzz canonical decoder, MIME parser, envelope parser, and protocol endpoints.
- Rate-limit and resource-exhaustion testing.
- Backup and restore exercises.
- Key rotation/recovery drills.
- Mail queue outage drills.
- Storage outage drills.
- RPC and transparency outage drills.
- Monitoring and alerting.
- Privacy review of logs and metrics.
- Data retention and deletion policy.
- Operator deployment guide.
- Incident response plan.
- Vulnerability disclosure process.

**Release gate**
No public production launch before independent security review and remediation of critical/high-severity findings.

---

# 15. Database Model — Initial Logical Schema

The final schema may change through migrations, but must preserve the following concepts.

## 15.1 `domains`

- `id`
- `domain_name`
- `verification_status`
- `verification_method`
- `verified_at`
- `created_at`
- `updated_at`

## 15.2 `identities`

- `id`
- `identity_id`
- `domain_id`
- `local_part`
- `status`
- `authority_key_id`
- `current_record_version`
- `created_at`
- `updated_at`

Unique constraint on normalized identity according to domain policy.

## 15.3 `devices`

- `id`
- `identity_id`
- `device_id`
- `status`
- `created_at`
- `revoked_at`

## 15.4 `public_keys`

- `id`
- `identity_id`
- `device_id` nullable for identity-level keys
- `key_id`
- `purpose`
- `algorithm`
- `public_key_bytes`
- `status`
- `version`
- `valid_from`
- `valid_until`
- `created_at`

Never store private keys in this table.

## 15.5 `identity_events`

- `id`
- `identity_id`
- `event_type`
- `record_version`
- `record_hash`
- `previous_record_hash`
- `authorization_signature`
- `created_at`

Append-only at the application level.

## 15.6 `message_delivery`

- `id`
- `message_id`
- `sender_domain`
- `recipient_domain`
- `status`
- `attempt_count`
- `next_attempt_at`
- `last_error_code`
- `created_at`
- `updated_at`

Do not store plaintext message bodies or CEKs here.

## 15.7 `message_objects`

- `id`
- `message_id`
- `storage_key`
- `ciphertext_hash`
- `size_bytes`
- `created_at`
- `expires_at`

Storage keys should be opaque and access-controlled.

---

# 16. API Surface — Initial Internal Contracts

These are logical endpoints. Exact HTTP paths and schemas must be finalized with the API implementation.

## Identity

- `POST /v1/domains/challenges`
- `POST /v1/domains/verify`
- `POST /v1/identities`
- `GET /v1/identities/resolve?address=...`
- `GET /v1/identities/{identity_id}`
- `POST /v1/identities/{identity_id}/devices`
- `POST /v1/identities/{identity_id}/keys/rotate`
- `POST /v1/identities/{identity_id}/devices/{device_id}/revoke`

All state-changing operations require authentication and signed authorization.

## Transparency

- `POST /v1/log/entries`
- `GET /v1/log/checkpoint`
- `GET /v1/log/proof/{entry_hash}`
- `GET /v1/log/consistency?from=...&to=...`

## Federation

- `GET /.well-known/dmn`
- `POST /v1/federation/messages` (optional future native transport)

SMTP remains the required delivery interface.

## API requirements

- Version every public API.
- Validate request size and field lengths.
- Use structured error codes.
- Apply authentication, authorization, rate limits, and audit events.
- Never return private key material.
- Avoid leaking whether an arbitrary address exists where enumeration is a concern.

---

# 17. Testing Strategy

## 17.1 Unit tests

Every crate must test:
- Valid input.
- Invalid input.
- Boundary lengths.
- Malformed encodings.
- Unsupported versions.
- Error propagation.
- Secret redaction.

## 17.2 Cryptographic tests

- Known-answer test vectors.
- Cross-implementation test vectors.
- Wrong key.
- Wrong context.
- Modified ciphertext.
- Modified signature.
- Invalid tag.
- Key rotation.
- Revocation.
- Multi-device and multi-recipient.
- Replay and duplicate handling.

## 17.3 Integration tests

- Identity service + PostgreSQL.
- Identity service + transparency log.
- Client SDK + identity resolver.
- Client SDK + MIME.
- Stalwart Node A + Stalwart Node B.
- SMTP send + IMAP retrieval.
- Object store upload/download.
- RPC unavailable after blockchain phase.

## 17.4 Security tests

- Malformed envelope fuzzing.
- MIME parser fuzzing.
- Canonical decoder fuzzing.
- Oversized messages and attachments.
- SMTP header injection.
- Address parsing edge cases.
- Key substitution.
- Downgrade attempts.
- BCC leakage checks.
- Plaintext/secret log scanning.
- Unauthorized device enrollment.

## 17.5 Required end-to-end scenarios

1. Alice and Bob exchange a native encrypted message.
2. Alice sends to Bob's two devices.
3. Alice sends to Bob and Charlie.
4. Bob's key rotates before Alice sends.
5. Bob's key is revoked.
6. An attacker substitutes Bob's key.
7. SMTP accepts the same message twice.
8. Node B is offline and later recovers.
9. Bob receives a conventional plaintext email.
10. A native E2EE send cannot silently downgrade.
11. Storage provider returns corrupted ciphertext.
12. Identity service is unavailable but a valid cached key remains within policy.
13. Blockchain RPC is unavailable after the blockchain phase.

---

# 18. Definition of Done

A task is complete only when:

- Code is implemented and formatted.
- Relevant tests pass.
- Negative/security tests are included.
- Errors are explicit and do not silently downgrade security.
- No secrets or plaintext are logged.
- Documentation is updated.
- Migration compatibility is considered.
- The change is scoped and reviewed.
- Known limitations are documented.

A phase is complete only when all its acceptance criteria pass and its gate is recorded.

---

# 19. Recommended Task Execution Order

The implementation agent should follow this exact order unless an approved ADR changes it:

1. Repository inspection and bootstrap.
2. Protocol type definitions.
3. Canonical serialization ADR and test vectors.
4. Cryptographic profile ADR.
5. Cryptographic core.
6. Identity records and local identity API.
7. Device lifecycle and key rotation.
8. Native envelope creation and verification.
9. MIME integration.
10. CLI client.
11. Stalwart local setup.
12. SMTP adapter and two-node delivery.
13. IMAP retrieval.
14. End-to-end and negative tests.
15. OpenPGP/MIME interoperability.
16. Key transparency log.
17. Federation discovery and retry hardening.
18. Solana anchoring.
19. Web client.
20. Production hardening and independent audit.

Do not start with Solana, a web UI, Kubernetes, or a custom mail server.

---

# 20. First Agent Task

The first implementation task is **P0.1 — Repository inspection**.

The agent must:

1. List the repository files and identify the current project state.
2. Locate the HLD and protocol documents.
3. Identify existing code, build tooling, and tests.
4. Propose the smallest safe workspace structure.
5. Create a short implementation status report.
6. Make no architectural or cryptographic assumptions.
7. Do not delete, rename, or overwrite existing files without approval.

After P0.1, proceed to P0.2 only if the repository state is understood.

---

# 21. Final Product Acceptance Criteria

The initial product is accepted when all of the following are demonstrated:

- A domain operator can deploy a DMN-compatible mail node.
- A user can register an identity and one or more devices.
- A client can resolve and verify another user's public keys.
- A client can encrypt a MIME message for multiple authorized devices.
- The sender can sign the message manifest.
- The recipient can verify and decrypt locally.
- Two independent Stalwart nodes can exchange native messages over SMTP.
- Conventional SMTP email remains functional.
- OpenPGP/MIME interoperability is tested for the supported profile.
- Key rotation and revocation are visible to clients.
- Key transparency detects unauthorized changes.
- Optional Solana anchoring can be disabled without breaking mail delivery.
- No mail node, storage service, or blockchain program receives plaintext or private encryption keys for native E2EE messages.
- Downgrade, replay, malformed-message, and key-substitution tests pass.
- Security limitations and metadata leakage are clearly documented.

**The core product is a federated, interoperable email network with client-side E2EE. Blockchain is an optional trust anchor, not the transport or storage layer.**
