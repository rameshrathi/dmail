# DMN Architecture & Specification Documentation

This directory contains the normative architecture documents, protocol specifications, engineering plan, security threat model, and Architecture Decision Records (ADRs) for the Decentralized Mail Network (DMN).

---

## Document Index

| Document | Purpose |
|---|---|
| [Plan.md](Plan.md) | Agent-executable master implementation plan, phase gates, and acceptance criteria. |
| [HLD.md](HLD.md) | High-Level Design: system context, trust boundaries, technology choices, and roadmap. |
| [Protocol.md](Protocol.md) | Protocol Specification: DID-MAIL identities, KDP discovery, SME envelopes, and DFP federation. |
| [threat-model.md](threat-model.md) | Threat model, trust assumptions, addressed risks, and required cryptographic controls. |
| [deployment.md](deployment.md) | Local development setup with Docker Compose (PostgreSQL, MinIO, Stalwart). |

---

## Architecture Decision Records (ADRs)

Key architectural decisions frozen for the implementation:

1. [ADR 0001: Canonical Serialization](adr/0001-canonical-serialization.md) — Deterministic CBOR (RFC 8949) for all cryptographic digests and signatures.
2. [ADR 0002: Mandatory Cryptographic Suites](adr/0002-cryptographic-suite.md) — Ed25519, HPKE (DHKEM X25519 / HKDF-SHA256 / ChaCha20Poly1305), and AEAD.
3. [ADR 0003: Native Secure Message Envelope and MIME Binding](adr/0003-native-envelope-and-mime-binding.md) — SME wire format and standard MIME carriage.
4. [ADR 0004: Identity Ownership and Device Lifecycle](adr/0004-identity-ownership-and-recovery.md) — Identity authority keys, per-device keys, and rotation/revocation.
5. [ADR 0005: BCC Privacy and Recipient Confidentiality](adr/0005-bcc-privacy.md) — Separate cryptographic passes to prevent recipient enumeration.
6. [ADR 0006: DNS and Well-Known Capability Discovery](adr/0006-dns-discovery.md) — `_dmn.<domain>` DNS TXT record and well-known HTTPS fallback.
7. [ADR 0007: Mail Server Integration Architecture](adr/0007-mail-server-integration.md) — Unmodified Stalwart mail server integration via SMTP/IMAP/JMAP.
8. [ADR 0008: Blockchain Account Authority and Transparency Commitments](adr/0008-blockchain-account-authority.md) — Decoupled Solana trust anchor for Merkle roots.
