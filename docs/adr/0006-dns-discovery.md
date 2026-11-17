# ADR 0006: DNS and Well-Known Capability Discovery

## Status
Proposed (Baseline for Phase 8)

## Context
When sending an email to `user@example.org`, the sender's client or mail node must discover whether `example.org` supports DMN protocols, and locate its Identity Directory, Key Transparency, and optional native federation endpoints.

## Decision
1. **Primary DNS Record:**
   - Record Name: `_dmn.<domain>`
   - Record Type: `TXT`
   - Format: Semicolon-delimited key-value tags:
     ```text
     v=DMN1; idp=https://identity.example.org/v1; ktl=https://ktl.example.org/v1
     ```
   - Tags:
     - `v`: Protocol version (`DMN1`).
     - `idp`: Identity Directory service base URL.
     - `ktl`: Key Transparency Log service base URL.
     - `fed`: (Optional) Native federation endpoint URL.

2. **Well-Known HTTPS Fallback:**
   - If DNS TXT lookup is unavailable or blocked, clients query `https://<domain>/.well-known/dmn` via HTTPS.
   - The response is a JSON capability document:
     ```json
     {
       "version": "1.0",
       "identity_endpoint": "https://identity.example.org/v1",
       "transparency_endpoint": "https://ktl.example.org/v1"
     }
     ```

3. **Fallback to Standard SMTP:**
   - If neither DNS nor HTTPS discovery succeeds, the client treats the recipient domain as conventional email. If E2EE was mandatory, the client halts with a clear error and refuses silent downgrade.

## Consequences
- Clean, non-intrusive DNS integration compatible with existing mail configurations.
- Clear fallback to well-known HTTPS endpoints.
