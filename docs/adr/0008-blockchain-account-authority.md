# ADR 0008: Blockchain Account Authority and Transparency Commitments (Solana)

## Status
Proposed (Baseline for Phase 9)

## Context
DMN seeks optional public trust anchoring to detect rogue identity authorities or split-view attacks. The blockchain must act as an independent public witness without compromising message privacy, user anonymity, or mail network availability.

## Decision
1. **Decoupled Architecture:**
   - Solana is strictly an **optional trust anchor**. SMTP delivery and mailbox retrieval must continue uninterrupted during Solana RPC outages or network congestion.
2. **On-Chain Scope:**
   - The Solana program manages:
     - Domain authority registrations and public key commitments.
     - Periodic Merkle tree root checkpoints from the Key Transparency Log.
   - The Solana program **never** stores:
     - Plaintext email bodies or attachments.
     - Recipient lists or communication graphs.
     - Per-message hashes that enable correlation attacks.
     - Unencrypted personally identifiable information (PII).
3. **Authority Model:**
   - Program Derived Addresses (PDAs) derived from domain seeds: `[b"domain", domain_bytes]`.
   - Identity authority commitments are signed by the domain's authorized key or governance multisig.

## Consequences
- Protects user privacy while providing decentralized tamper-evidence for key history.
- Eliminates reliance on cryptocurrency holding for everyday mail users.
