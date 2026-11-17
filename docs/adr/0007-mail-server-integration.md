# ADR 0007: Mail Server Integration Architecture (Stalwart)

## Status
Proposed (Baseline for Phase 6)

## Context
DMN requires a standards-compliant, high-performance mail server to handle SMTP routing, queue management, delivery retries, and mailbox storage. Forking or modifying mail server internals increases maintenance burden and risks breaking email standard conformance.

## Decision
1. **Foundation Server:**
   - Use **Stalwart** as the baseline mail server.
2. **Integration Boundaries:**
   - **No Forks / Internal Modifications:** DMN integrates strictly through standard protocols:
     - Outbound Submission: SMTP submission on port 587 with STARTTLS/TLS and standard authentication.
     - Inbound Federation: Server-to-server SMTP on port 25.
     - Mailbox Retrieval: IMAP (port 993/143) and JMAP (HTTPS).
3. **Confidentiality Invariant:**
   - Stalwart processes native DMN messages as standard MIME messages (`multipart/encrypted` or custom application media type).
   - Stalwart never holds private keys or CEKs and stores only ciphertext envelopes for native DMN communications.
4. **Error Handling & Retries:**
   - Stalwart handles network-level retries, exponential backoff, and MX fallback using its native SMTP engine.

## Consequences
- Preserves 100% compatibility with conventional email infrastructure.
- Zero maintenance overhead from maintaining a custom mail daemon fork.
