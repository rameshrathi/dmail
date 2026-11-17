# Decentralized Mail Network (DMN)

[![Status](https://img.shields.io/badge/status-implementation--baseline-blue.svg)](Plan.md)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

The **Decentralized Mail Network (DMN)** is an interoperable, federated email architecture that brings native client-side End-to-End Encryption (E2EE), portable cryptographic identities, and auditable key transparency to standard Internet email without breaking existing protocols (SMTP, MIME, IMAP, JMAP).

---

## 1. Executive Summary

Traditional email lacks native, modern, user-friendly end-to-end confidentiality and relies on centralized trust anchors. DMN bridges modern cryptography with established email infrastructure:

1. **Identity & Trust:** Binds human-readable mail identities (`user@domain.tld`) to authenticated device keys via signed identity records.
2. **Client-Side E2EE:** Content encryption keys (CEKs) are generated on user endpoints, payloads are encrypted with authenticated ciphers (AEAD), and CEKs are encapsulated for recipient devices using HPKE (RFC 9180). Mail servers and storage providers see only ciphertext.
3. **Transport Interoperability:** Uses standard SMTP for server-to-server routing and client submission, MIME for structure, and IMAP/JMAP for mailbox access. Supports both native DMN envelopes and OpenPGP/MIME.
4. **Key Transparency & Trust Anchoring:** Records key lifecycle events in an append-only Merkle tree to detect unauthorized key substitution, with optional checkpoint anchoring on Solana. Blockchain availability is strictly decoupled from SMTP mail transport.

---

## 2. Architecture & Design Principles

```text
+-----------------------------------------------------------+
|                       User Clients                         |
|       CLI / Web / Desktop / Third-Party Mail Clients       |
|                                                           |
|  Identity UI | Key Vault  | Encrypt/Sign | Decrypt/Verify  |
+-------------------------------+---------------------------+
                                |
                     HTTPS / JMAP / SMTP submission
                                |
+-------------------------------v---------------------------+
|                    DMN Mail Node                           |
|                                                           |
|  SMTP MTA | IMAP/JMAP | Mailbox | Queue | Policy/Spam      |
|                                                           |
|  +------------------- DMN Services ---------------------+  |
|  | Identity resolver | Key cache | Federation adapter  |  |
|  | E2EE MIME handler | Delivery state | Notifications   |  |
|  +------------------------------------------------------+  |
+-------------+------------------+--------------------------+
              |                  |
       +------v------+    +------v----------------+
       | PostgreSQL  |    | Encrypted Object Store|
       | metadata    |    | MIME blobs/attachments|
       +-------------+    +-----------------------+
              |
       +------v-----------------------------------+
       | Identity & Trust Infrastructure           |
       | DNS / DNSSEC / Key Directory / Merkle Log|
       | Optional Solana Identity Registry        |
       +-------------------------------------------+
```

### Core Tenets

- **Fail-Closed Security:** Messages explicitly marked for encryption never silently downgrade to plaintext.
- **Server Blindness:** Mail servers, relays, and storage providers never hold private keys or plaintext CEKs.
- **Decoupled Blockchain:** Solana provides optional public auditability. An outage or latency on Solana never halts SMTP message delivery.
- **Standards Compliant:** Integrates with Stalwart mail server via standard SMTP/IMAP/JMAP without custom forks.
- **Multi-Device Support:** Content is encrypted once and the CEK is wrapped independently to each authorized recipient device.

---

## 3. Technology Stack

| Layer | Component / Technology | Purpose |
|---|---|---|
| **Language** | Rust (2021 edition) | Memory safety, performance, cryptographic assurance |
| **Mail Server** | Stalwart | Standard SMTP, IMAP, JMAP, and queue management |
| **Public-Key Cryptography** | HPKE (RFC 9180) / X25519 | Recipient device key encapsulation |
| **Digital Signatures** | Ed25519 | Sender manifests and identity records |
| **Authenticated Encryption** | XChaCha20-Poly1305 / AES-256-GCM | Message payload and attachment encryption |
| **Integrity & Hashing** | SHA-256 / HKDF-SHA256 | Commitment hashes and key derivation |
| **Relational Database** | PostgreSQL | Domains, identities, devices, and delivery state |
| **Object Storage** | S3-compatible (MinIO locally) | Encrypted MIME blobs and large attachments |
| **Transparency Log** | Merkle Tree | Append-only public audit of key rotation/revocation |
| **Trust Anchor (Optional)** | Solana | On-chain authority commitments and checkpoints |

---

## 4. Repository Layout

```text
.
├── Plan.md                                    # Master engineering and execution plan
├── decentralized_mail_high_level_design.md    # High-level architecture specification
├── decentralized_mail_protocol_specification.md # Detailed protocol specification
├── docs/                                      # Architecture records, specifications, test vectors
│   ├── adr/                                   # Architecture Decision Records
│   ├── threat-model.md                        # Threat model and mitigation strategies
│   └── test-vectors/                          # Cross-implementation cryptographic fixtures
├── crates/                                    # Rust modular workspace crates
│   ├── dmn-types/                             # Core protocol and identifier types
│   ├── dmn-encoding/                          # Canonical deterministic serialization
│   ├── dmn-crypto/                            # HPKE, AEAD, Ed25519, hashing routines
│   ├── dmn-identity/                          # Identity records, verification, device lifecycle
│   ├── dmn-envelope/                          # Secure Message Envelope (SME) construction
│   ├── dmn-mime/                              # MIME bindings and OpenPGP adapters
│   ├── dmn-transparency/                      # Merkle log, inclusion & consistency proofs
│   └── dmn-client-sdk/                        # Unified client SDK for E2EE operations
├── services/                                  # Backend standalone services
│   ├── identity-api/                          # REST/gRPC identity and directory service
│   └── transparency-api/                      # Merkle key transparency log service
├── clients/                                   # Client applications
│   └── cli/                                   # Reference command-line client
├── deploy/                                    # Deployment and orchestration
│   └── compose/                               # Docker Compose for local development
└── tests/                                     # System, integration, and security tests
```

---

## 5. Implementation Roadmap & Phases

- [ ] **Phase 0:** Repository Bootstrap and Engineering Controls *(In Progress)*
- [ ] **Phase 1:** Protocol Types and Canonical Encoding
- [ ] **Phase 2:** Cryptographic Core (HPKE, AEAD, Signatures)
- [ ] **Phase 3:** Identity Records and Local Identity Service
- [ ] **Phase 4:** Native Message Envelope (SME)
- [ ] **Phase 5:** MIME and OpenPGP Interoperability
- [ ] **Phase 6:** Stalwart Integration and Two-Node SMTP Delivery
- [ ] **Phase 7:** Key Transparency Merkle Log
- [ ] **Phase 8:** Federation Discovery and Delivery Hardening
- [ ] **Phase 9:** Optional Solana Identity Anchoring
- [ ] **Phase 10:** Client Applications (CLI & Web)
- [ ] **Phase 11:** Production Hardening and Security Audit

Refer to [Plan.md](Plan.md) for detailed task definitions, phase gates, and acceptance criteria.

---

## 6. Getting Started

### Prerequisites

- **Rust:** Stable toolchain (`rustup install stable`)
- **Docker & Docker Compose:** For running local dependencies (PostgreSQL, MinIO, Stalwart)
- **Git**

### Building and Running Tests

```bash
# Verify formatting
cargo fmt --all -- --check

# Run linter
cargo clippy --workspace --all-targets -- -D warnings

# Run all unit and integration tests
cargo test --workspace
```

---

## 7. Security & Disclosure

Please refer to [threat-model.md](docs/threat-model.md) for full threat modeling details.

- **Zero-Plaintext Server Policy:** Mail nodes, network intermediaries, and storage providers must never have access to private keys or plaintext payloads.
- **Reporting Vulnerabilities:** Security concerns should be submitted via private vulnerability disclosure channels prior to public issue creation.

---

## 8. License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.
