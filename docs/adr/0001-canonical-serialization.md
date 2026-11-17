# ADR 0001: Canonical Serialization for Cryptographic Commitments

## Status
Proposed (Baseline for Phase 1)

## Context
DMN requires computing digital signatures over message manifests, identity records, and key transparency entries. In standard JSON serialization (e.g. `serde_json::to_vec`), key ordering, whitespace, number formatting, and Unicode representations can vary across implementations and platforms, leading to signature verification failures or malleability vulnerabilities.

Two primary candidates exist:
1. **Canonical JSON (e.g. RFC 8785 / JCS):** Human-readable, but lacks native byte strings, requiring extra base64 encoding layers for all cryptographic binary data.
2. **Deterministic CBOR (RFC 8949 Section 4.2.1 / dCBOR):** Compact, unambiguous binary format with native byte arrays, deterministic integer encodings, and strict map key ordering rules.

## Decision
1. **Wire and Signature Format:** Use **Deterministic CBOR (RFC 8949 Section 4.2.1)** as the canonical serialization format for all cryptographic signatures and digest computations.
2. **Key Ordering:** Map keys must be sorted deterministically according to the canonical CBOR ordering rules (shorter keys precede longer keys; keys of identical length are sorted lexicographically by byte value). Duplicate keys are strictly prohibited and cause decoding rejection.
3. **Integer and Timestamp Representation:** Integers must be encoded using the shortest valid CBOR representation. Timestamps are encoded as standard 64-bit integer UTC seconds or milliseconds since Unix epoch.
4. **Binary Data:** Cryptographic keys, nonces, ciphertext, and hashes are encoded as native CBOR byte strings (major type 2).
5. **API Presentation Layer:** External HTTP REST APIs may accept and return standard JSON for ease of client consumption, but all signatures and hash commitments are computed strictly over the canonical CBOR byte representation.

## Consequences
- Guarantees byte-for-byte reproducibility across languages and platforms.
- Eliminates JSON floating-point, whitespace, and Unicode normalization ambiguities.
- Unit and cross-implementation test vectors must be committed for all canonical representations.
