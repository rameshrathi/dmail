# ADR 0004: Identity Ownership, Device Lifecycle, and Recovery

## Status
Proposed (Baseline for Phase 3)

## Context
DMN identities correspond to standard email addresses (`user@domain.tld`). The system must differentiate domain ownership, user identity authorization, and device authorization without centralizing private keys or requiring cryptocurrency wallets.

## Decision
1. **Domain Authorization:**
   - Domains prove namespace control through DNS TXT challenges or well-known HTTPS domain challenges before publishing identities.
2. **Key Separation & Hierarchy:**
   - **Identity Authority Key:** User-held primary signing key authorizing identity records and device additions.
   - **Device Signing Key:** Local per-device Ed25519 key used for signing client actions.
   - **Device Encryption Key:** Local per-device X25519 key used for HPKE recipient wrapping.
   - Private keys never leave user endpoints and are never stored in the database.
3. **Device Lifecycle:**
   - Adding a device requires an authorization signature from the existing identity authority or an authorized recovery credential.
   - Revoking a device emits an immutable signed revocation event in the key transparency log. Future messages exclude revoked devices.
4. **Recovery Policy:**
   - At identity creation, the client generates an offline recovery keypair (or mnemonic).
   - Domain-assisted recovery is configurable per domain policy but cannot silently replace keys without creating an auditable transparency entry.

## Consequences
- Independent devices can decrypt incoming messages in parallel.
- Malicious servers cannot silently enroll attacker devices without triggering transparency alerts.
