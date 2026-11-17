# ADR 0003: Native Secure Message Envelope (SME) and MIME Binding

## Status
Proposed (Baseline for Phase 4 & Phase 5)

## Context
DMN messages must traverse standard SMTP mail transfer agents (MTAs) and be stored by standard mailboxes (Stalwart) without modification and without leaking plaintext content. DMN also requires coexistence with OpenPGP/MIME.

## Decision
1. **Envelope Structure (SME):**
   - The native Secure Message Envelope consists of:
     - `manifest`: Canonical metadata (protocol version, message ID, timestamp, sender identity and key ID, ciphertext SHA-256 hash).
     - `recipient_envelopes`: Array of recipient entries, each specifying `recipient_identity_id`, `device_id`, `key_id`, HPKE `encapsulation` (public ephemeral key), and `wrapped_cek`.
     - `ciphertext`: Encrypted payload with AEAD authentication tag.
     - `signature`: Ed25519 signature by the sender over the canonical manifest.

2. **MIME Wire Representation:**
   - Transport format: MIME message with `Content-Type: multipart/encrypted; protocol="application/vnd.dmn.envelope"`.
   - Outer headers: Contain only the required RFC 5322 headers for SMTP delivery (`From`, `To`, `Message-ID`, `Date`). Outer `Subject` is set to a neutral placeholder (e.g., `Encrypted Message`).
   - Protected inner payload: Fully encrypted canonical MIME tree holding the confidential `Subject`, `Body` (text/plain and text/html), and attachments.

3. **OpenPGP/MIME Interoperability:**
   - DMN native envelope and OpenPGP/MIME (`multipart/encrypted; protocol="application/pgp-encrypted"`) are treated as distinct profiles.
   - If a recipient only supports OpenPGP, the client formats an RFC 3156 / RFC 9580 message.
   - Downgrade from requested E2EE to unencrypted plaintext is strictly forbidden without explicit user intervention.

## Consequences
- Mail nodes can forward and store messages via ordinary SMTP without parsing private payload data.
- Confidential subjects and attachments are completely protected from server inspection.
