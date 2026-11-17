# ADR 0002: Mandatory Cryptographic Suites

## Status
Proposed (Baseline for Phase 2)

## Context
DMN requires standardized, audited cryptographic primitives for digital signatures, public-key encapsulation of content encryption keys (CEKs), and authenticated payload encryption. Custom cryptography is strictly forbidden.

Candidate algorithms identified in the HLD and protocol specification:
- Digital Signatures: Ed25519 (RFC 8032)
- Key Encapsulation: HPKE (RFC 9180)
- Symmetric Content Encryption: ChaCha20-Poly1305 (RFC 8439) or AES-256-GCM
- Digest: SHA-256

## Decision
1. **Digital Signatures:**
   - Algorithm: **Ed25519** (PureEd25519, RFC 8032).
   - Key lengths: 32-byte public key, 32-byte private seed, 64-byte signature.
   - Purpose: Signing identity records, device authorization events, and message manifests.

2. **Public-Key Key Encapsulation (HPKE):**
   - Profile: **RFC 9180 Base Mode**
   - KEM: `DHKEM(X25519, HKDF-SHA256)` (KEM ID `0x0020`)
   - KDF: `HKDF-SHA256` (KDF ID `0x0001`)
   - AEAD: `ChaCha20Poly1305` (AEAD ID `0x0003`)
   - Context Binding: The HPKE info string binds protocol version, message ID, recipient identity ID, device ID, and recipient key ID.

3. **Content Encryption (AEAD):**
   - Algorithm: **ChaCha20-Poly1305** (RFC 8439) with unique 32-byte Content Encryption Key (CEK) and 12-byte random/counter nonce per message.
   - Associated Data: Binds canonical message manifest digest and protocol version.
   - Secondary supported cipher: **AES-256-GCM** (NIST SP 800-38D).

4. **Hashing & Commitments:**
   - Algorithm: **SHA-256** (FIPS 180-4) for ciphertext integrity, Merkle tree nodes, and manifest digests.

5. **Random Number Generation:**
   - All ephemeral CEKs, nonces, and keypairs must be generated via OS CSPRNG (e.g. `getrandom` / `ring::rand`).

6. **Downgrade Protection:**
   - Any unsupported or unrecognized suite identifier fails closed immediately.

## Consequences
- Fast, secure, constant-time software implementations across x86_64 and ARM64.
- Clear separation between signing keys (Ed25519) and encryption keys (X25519).
- Tested against RFC 9180 and RFC 8032 test vectors.
