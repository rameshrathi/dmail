# ADR 0005: BCC Privacy and Recipient Confidentiality

## Status
Proposed (Baseline for Phase 4)

## Context
In standard SMTP, `Bcc` headers are stripped prior to delivery, but multi-recipient E2EE messages wrap the CEK to all recipient public keys. If all recipient device envelopes or addresses are bundled into a single manifest, `To` and `Cc` recipients can inspect the list of recipient keys and deduce the existence and identity of `Bcc` recipients. Furthermore, public blockchain transactions must never leak recipient lists.

## Decision
1. **No Shared Recipient List for BCC:**
   - A single shared message manifest must never include BCC recipient envelopes or identifiers alongside To and CC recipients.
2. **Client-Side Split Delivery for BCC:**
   - When a user includes BCC recipients, the sender client splits the dispatch into separate cryptographic deliveries:
     - Delivery 1: Multi-recipient envelope containing `To` and `Cc` recipients with their shared CEK and manifest.
     - Delivery N: For each `Bcc` recipient (or group), an independent message envelope with a distinct random CEK is generated and transmitted via standard SMTP to that recipient only.
3. **Outer SMTP Headers:**
   - Outer MIME headers for BCC copies omit the BCC address in the headers, relying solely on SMTP envelope (`RCPT TO`) routing.
4. **On-Chain Commitments:**
   - No recipient identifiers or recipient counts may ever be published to the blockchain or public key transparency logs.

## Consequences
- Completely eliminates BCC address leakage to other recipients.
- Requires multiple encryption passes when BCC is used, which is acceptable given the security and privacy guarantee.
