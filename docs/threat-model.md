# Threat Model for Decentralized Mail Network (DMN)

## 1. Overview and Trust Boundaries

The Decentralized Mail Network (DMN) federates email with client-side End-to-End Encryption (E2EE), portable identity, and auditable key transparency.

### Trust Assumptions
- **User Client (Endpoint):** Trusted with local plaintext and private keys. Compromise of the endpoint compromises confidentiality of messages decrypted on or sent from that endpoint.
- **Mail Nodes (Stalwart / SMTP MTAs):** Untrusted for confidentiality. Mail servers route and store encrypted MIME envelopes. They must never see plaintext content or private Content Encryption Keys (CEKs).
- **Storage Providers (S3 / MinIO):** Untrusted for confidentiality. They store ciphertext blobs and encrypted attachments.
- **Identity Directory & Transparency Log:** Semi-trusted. An untrusted or rogue operator could attempt to publish a replacement key; however, the append-only Merkle transparency log allows clients and independent monitors to detect unauthorized key substitutions.
- **Blockchain / Solana:** Untrusted for privacy. Transactions are public and immutable. Blockchain availability is not required for SMTP delivery.

---

## 2. Threats Addressed

1. **Mail Server Confidentiality Violation:** Mail servers (both sender and recipient nodes) cannot decrypt message payloads or access private keys.
2. **Storage Provider Compromise:** Compromised object stores cannot decrypt message bodies or attachments.
3. **Ciphertext Tampering in Transit:** Network adversaries cannot alter ciphertext without detection (guaranteed by AEAD authentication tags).
4. **Sender Impersonation & Manifest Modification:** Digital signatures (Ed25519) over deterministic canonical manifests bind the sender identity to the ciphertext commitment.
5. **Unauthorized Public Key Substitution:** Key transparency logs and client-side history tracking detect rogue key insertions or rollbacks.
6. **Message Replay and Duplication:** Recipient-side idempotency tracking and message ID uniqueness prevent replay attacks.
7. **Unauthorized Device Enrollment:** Identity records require authorization from an existing trusted identity authority before newly added devices are trusted.
8. **Silent Downgrade Attacks:** If an encrypted send is requested, clients fail closed if recipient keys cannot be validated or resolved.

---

## 3. Threats Not Fully Addressed (Residual Risks)

1. **Endpoint Compromise:** Malware, screen capture, keyloggers, or hardware compromise on the sender or recipient device.
2. **Traffic Analysis & Metadata Exposure:** Standard SMTP routing metadata (envelope sender, recipient, domain names, IP addresses, message size, and timestamps) remains visible to mail relays.
3. **Authorized Recipient Malfeasance:** A legitimate recipient who can decrypt the message may disclose or forward the plaintext.
4. **Denial of Service (DoS):** Upstream network or SMTP blockades, DNS tampering, or resource exhaustion.
5. **Irrevocable Delivered Plaintext:** Revoking a device or key prevents future messages from being decrypted; it cannot claw back or erase already received/decrypted data.

---

## 4. Required Security Controls

- **No Custom Cryptography:** Rely strictly on RFC 9180 (HPKE), audited AEAD implementations (XChaCha20-Poly1305 / AES-256-GCM), and Ed25519.
- **Secure Memory & Secret Redaction:** Private keys and CEKs must be erased from memory when dropped and redacted from all `Debug` representations, logs, and telemetry.
- **Deterministic Canonical Encoding:** Cryptographic signatures must only be calculated over canonical byte streams (e.g., deterministic CBOR).
- **Fail-Closed Policy:** Any cryptographic verification failure or key validation failure must abort message processing and never release partial plaintext.
