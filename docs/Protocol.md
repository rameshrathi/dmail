# Decentralized Mail Network (DMN)

## Protocol Specification --- Draft v0.1

**Status:** Design draft; not production-ready\
**Date:** 17 November 2026\
**Audience:** Client, mail-server, identity-service, and federation
implementers

------------------------------------------------------------------------

## 1. Scope and Conventions

This document defines a first protocol draft for:

-   Mail identity records and device keys.
-   Public-key discovery and key transparency.
-   Native end-to-end encrypted message envelopes.
-   SMTP carriage and federation.
-   Delivery identifiers, replay handling, and error behaviour.
-   Optional blockchain anchoring.

This draft is not a cryptographic standard and must not be implemented
for production without a security review, exact algorithm-suite
selection, canonical serialization rules, and interoperable test
vectors.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD
NOT**, and **MAY** are normative.

DMN MUST use established cryptographic libraries and standards. It MUST
NOT define custom cryptographic primitives.

------------------------------------------------------------------------

## 2. Protocol Components

DMN consists of four independently deployable protocol functions:

1.  **Identity Protocol (DID-MAIL):** Registers identities, signing
    keys, encryption keys, devices, and key lifecycle events.
2.  **Key Discovery Protocol (KDP):** Resolves an email address to a
    signed identity record and transparency proof.
3.  **Secure Message Envelope (SME):** Encodes encrypted message
    content, recipient key envelopes, and sender authentication.
4.  **Federation Protocol (DFP):** Discovers DMN-capable domains and
    carries messages over SMTP or an optional native transport.

SMTP and MIME remain the baseline interoperability layer.

------------------------------------------------------------------------

## 3. Identifiers and Encoding

### 3.1 Email identity

The canonical identity is an email address: `local-part@domain`.

Implementations MUST follow the relevant email address syntax and domain
normalization rules. Domain names MUST be normalized using IDNA rules
before lookup. Local-part case handling MUST follow the receiving
domain's policy; clients MUST NOT assume all providers treat local parts
identically.

An email address is a locator, not proof of identity.

### 3.2 Opaque identifiers

Protocol objects use random opaque identifiers to avoid exposing raw
email addresses unnecessarily.

-   `identity_id`: 32-byte random or collision-resistant identifier.
-   `device_id`: 16-byte random identifier.
-   `message_id`: 16 or 32 bytes of cryptographically secure randomness.
-   `key_id`: stable identifier derived from or assigned to a public key
    record.
-   `log_id`: identifier for a transparency log.

Identifiers MUST NOT be derived from email addresses in a way that
allows trivial enumeration.

### 3.3 Serialization

The initial API representation may use JSON, but cryptographic
signatures MUST NOT be calculated over arbitrary JSON serialization.

The protocol MUST select a deterministic canonical encoding before
interoperability release. Deterministic CBOR is a candidate. If JSON is
retained, a precisely specified canonicalization profile is required.

All binary values MUST use a single specified encoding, such as
base64url without padding, consistently across implementations.

Unknown critical fields MUST cause rejection. Unknown non-critical
extension fields MAY be ignored according to the versioning rules.

------------------------------------------------------------------------

## 4. Cryptographic Profile

### 4.1 Separation of key purposes

A key MUST be associated with exactly one declared purpose:

-   `identity-signing`
-   `device-signing`
-   `device-encryption`
-   `recovery-authorization`

A signing key MUST NOT be treated as an encryption key unless an
explicitly standardized scheme defines that use.

### 4.2 Candidate primitives

The initial profile should evaluate:

  Purpose                           Candidate
  --------------------------------- -------------------------------------
  Digital signatures                Ed25519
  Recipient public-key encryption   HPKE, RFC 9180
  HPKE KEM                          DHKEM(X25519, HKDF-SHA256)
  HPKE KDF                          HKDF-SHA256
  HPKE AEAD                         A precisely selected RFC 9180 suite
  Content encryption                XChaCha20-Poly1305 or AES-256-GCM
  Hash                              SHA-256
  Password-protected backup         Argon2id + authenticated encryption

The final specification MUST select one mandatory suite for each
operation. It MUST define algorithm identifiers, key lengths, nonce
handling, tag lengths, serialization, and test vectors.

Implementations MUST NOT negotiate weaker algorithms silently. Unknown
or unsupported mandatory suites MUST fail closed.

### 4.3 Randomness

All private keys, content-encryption keys, nonces where required, and
random identifiers MUST be generated by a cryptographically secure
random-number generator.

A CEK MUST be unique per message. Implementations MUST follow the
selected AEAD's nonce uniqueness requirements.

### 4.4 Content encryption

The sender:

1.  Creates a canonical message payload.
2.  Generates a fresh CEK.
3.  Encrypts the payload with an authenticated encryption algorithm.
4.  Computes a ciphertext hash.
5.  Creates a recipient envelope for every authorized recipient device.
6.  Signs the canonical manifest and ciphertext commitment.

The payload MUST be authenticated before plaintext is released to the
application.

------------------------------------------------------------------------

## 5. Identity Record

### 5.1 Logical schema

The following is a logical schema, not a final wire encoding:

``` json
{
  "protocol": "did-mail/1",
  "identity_id": "base64url-32-bytes",
  "address": "alice@example.org",
  "domain": "example.org",
  "authority_key_id": "key-id",
  "identity_signing_keys": [
    {
      "key_id": "identity-signing-key-1",
      "algorithm": "Ed25519",
      "public_key": "base64url",
      "status": "active"
    }
  ],
  "devices": [
    {
      "device_id": "base64url-16-bytes",
      "device_signing_key": {
        "key_id": "device-signing-1",
        "algorithm": "Ed25519",
        "public_key": "base64url"
      },
      "device_encryption_key": {
        "key_id": "device-encryption-1",
        "algorithm": "HPKE-X25519",
        "public_key": "base64url"
      },
      "status": "active"
    }
  ],
  "version": 1,
  "previous_record_hash": null,
  "created_at": "UTC timestamp",
  "expires_at": "UTC timestamp",
  "authority_signature": "base64url"
}
```

### 5.2 Registration

A domain MUST prove control of its domain namespace before publishing
identities under that namespace.

A user identity MUST be bound to an authenticated account or domain
policy. Device public keys MUST be accompanied by proof of possession
and authorization.

The identity record MUST be signed by the appropriate identity
authority. A domain's authorization of an identity MUST be
distinguishable from a user's authorization of a device.

### 5.3 Device addition

To add a device:

1.  Generate device signing and encryption keys locally.
2.  Authenticate the user to the identity authority.
3.  Authorize the new device using an existing trusted device, recovery
    key, or domain policy.
4.  Publish a new signed identity-record version.
5.  Append the update to the transparency log.
6.  Notify existing devices of the change.

If the identity authority cannot establish authorization, the new device
MUST NOT become trusted automatically.

### 5.4 Key rotation

A rotation record MUST identify:

-   Identity and device.
-   Old key identifier.
-   New key identifier and public key.
-   Key purpose and algorithm.
-   Effective time.
-   Reason, if supplied.
-   Authorizing signature(s).
-   Previous record hash.

Clients MUST validate the authorization chain and transparency proof.
Unexpected key changes MUST be surfaced to users.

### 5.5 Revocation

Revocation MUST be represented as a signed, append-only event. A revoked
key MUST NOT be used for new encryption after clients learn of the
revocation.

Previously encrypted messages may remain decryptable by the revoked
device. Revocation cannot erase already obtained plaintext or private
keys.

------------------------------------------------------------------------

## 6. Key Discovery Protocol (KDP)

### 6.1 Resolution inputs

A client supplies:

-   Email address.
-   Required protocol profile.
-   Optional device or capability constraints.
-   Freshness policy.

### 6.2 Resolution sequence

1.  Normalize the domain.
2.  Query DNS for DMN capability discovery.
3.  Resolve the identity directory endpoint.
4.  Fetch the identity record.
5.  Validate domain authorization and record signatures.
6.  Fetch transparency inclusion and consistency proofs.
7.  Check key status, expiry, and revocation.
8.  Cache the verified record according to its validity and local
    policy.

### 6.3 DNS discovery

A domain may publish a DNSSEC-protected DMN capability record or a
well-known HTTPS endpoint.

A proposed HTTPS path is:

`https://example.org/.well-known/dmn-identity`

The exact DNS record name, syntax, signature model, and fallback rules
MUST be finalized before protocol freeze.

DNS discovery MUST NOT be treated as sufficient proof of key ownership
without validating the signed identity record.

### 6.4 Cache rules

Caches MUST record:

-   Record version.
-   Verification time.
-   Expiry.
-   Transparency checkpoint.
-   Revocation freshness.
-   Source endpoint.

If a key is stale or cannot be checked, the client MUST apply an
explicit policy. It MUST NOT silently treat stale data as current.

------------------------------------------------------------------------

## 7. Key Transparency

### 7.1 Purpose

The transparency service records identity and key changes in an
append-only Merkle structure. It helps clients detect unauthorized key
substitution or inconsistent views.

### 7.2 Log entries

A log entry SHOULD commit to:

-   Identity identifier.
-   Identity-record hash.
-   Version.
-   Previous-record hash.
-   Event type.
-   Timestamp.
-   Authorization signature hash.

Avoid putting unnecessary personal data or raw recipient relationships
into the log.

### 7.3 Proofs

The service MUST support:

-   Inclusion proofs.
-   Consistency proofs between checkpoints.
-   Signed tree heads/checkpoints.
-   Monitoring for conflicting views.

Clients SHOULD compare checkpoints through independent monitors or
gossip channels.

### 7.4 Blockchain anchoring

An optional Solana program may store a transparency checkpoint hash and
authority state.

The chain MUST NOT be required for each message. A mail node MUST be
able to continue delivery using previously verified records within its
configured freshness policy.

A blockchain transaction proves only that a commitment was recorded
under the chain's consensus rules. It does not prove that the underlying
identity data is truthful.

------------------------------------------------------------------------

## 8. Secure Message Envelope (SME)

### 8.1 Message model

A message contains:

-   A public or transport-visible header.
-   A signed manifest.
-   One or more recipient-specific encrypted CEK envelopes.
-   Encrypted content.
-   Optional encrypted attachments or attachment references.

For SMTP compatibility, the SME is carried as a MIME body part or an
established OpenPGP/MIME representation.

### 8.2 Logical envelope schema

``` json
{
  "protocol": "dmn-sme/1",
  "message_id": "base64url-random-id",
  "created_at": "UTC timestamp",
  "sender": {
    "identity_id": "opaque-id",
    "key_id": "sender-signing-key-id"
  },
  "crypto": {
    "content_algorithm": "selected-aead-id",
    "key_wrap_algorithm": "selected-hpke-suite"
  },
  "content": {
    "content_type": "application/vnd.dmn.mime",
    "nonce": "base64url",
    "ciphertext": "base64url",
    "ciphertext_hash": "base64url-sha256"
  },
  "recipient_envelopes": [
    {
      "recipient_identity_id": "opaque-id",
      "device_id": "opaque-device-id",
      "key_id": "recipient-encryption-key-id",
      "encapsulation": "base64url",
      "wrapped_cek": "base64url"
    }
  ],
  "signature": {
    "algorithm": "Ed25519",
    "key_id": "sender-signing-key-id",
    "value": "base64url"
  }
}
```

This schema is illustrative. Production implementations MUST use the
finalized canonical encoding and must not sign the displayed JSON
serialization directly.

### 8.3 Payload

The encrypted payload SHOULD contain a canonical MIME message with:

-   Subject.
-   Text and HTML alternatives.
-   MIME attachments.
-   Message references such as In-Reply-To and References.
-   Optional application metadata.

The protocol MUST define whether transport headers are authenticated,
encrypted, or duplicated inside the protected payload.

### 8.4 Recipient envelopes

For each authorized recipient device:

1.  Obtain and validate its current encryption key.
2.  Use the selected HPKE profile to encapsulate/wrap the CEK.
3.  Bind the envelope to the message identifier, recipient identity,
    device identifier, key identifier, and protocol version through
    authenticated context.
4.  Include the resulting encapsulation and wrapped CEK.

The recipient device MUST verify that the envelope is addressed to its
identity/device and that all required context matches before decrypting.

### 8.5 Multiple recipients

The same ciphertext MAY be shared among To and CC recipients by wrapping
the same CEK independently for each authorized device.

BCC handling MUST avoid revealing BCC identities to other recipients.
Implementations SHOULD use separate recipient-specific transport
envelopes or separate message copies where required by SMTP semantics.

A shared public blockchain transaction MUST NOT contain a plaintext
recipient list.

### 8.6 Signature

The sender signature MUST cover a deterministic manifest that includes:

-   Protocol version.
-   Message identifier.
-   Sender identity and signing-key identifier.
-   Creation time.
-   Crypto-suite identifiers.
-   Ciphertext hash.
-   Recipient-envelope commitments or the complete canonical envelope
    list, subject to BCC privacy requirements.

The protocol MUST specify how BCC envelopes are signed without
disclosing their identities to other recipients. A likely implementation
is a signed per-recipient envelope plus a common signed content
manifest.

A signature MUST be verified against a trusted identity record and
transparency state. A valid signature alone is not sufficient to
establish the sender's real-world identity.

### 8.7 Replay and duplicate handling

Receiving nodes MUST maintain an idempotency record for message
identifiers within a defined retention window.

A repeated message identifier with identical authenticated content
SHOULD be treated as a duplicate. The same identifier with different
content MUST be rejected and logged as a protocol violation.

Clients SHOULD display duplicate messages only once unless the user
explicitly requests otherwise.

### 8.8 Message expiry

A message MAY include an expiry policy. Expiry is advisory and cannot
guarantee deletion from recipient devices, backups, mail archives, or
blockchain history.

------------------------------------------------------------------------

## 9. SMTP and MIME Binding

### 9.1 Baseline transport

DMN federation MUST support SMTP transport. The receiving node stores
the native encrypted object without decrypting it.

### 9.2 MIME representation

The final protocol must register or specify a versioned MIME media type.
A provisional media type for development is:

`application/vnd.dmn.message+json`

This is not a registered media type and MUST be treated as provisional.
The production format should use a binary canonical encoding or a
carefully specified MIME structure.

### 9.3 Header policy

Transport headers may expose sender, recipient, routing, timestamp, and
message size. Implementations MUST document this metadata exposure.

Subject confidentiality requires the subject to be inside the encrypted
payload and a neutral outer subject. This may reduce compatibility with
ordinary mail clients and server-side filtering.

### 9.4 Downgrade protection

If the sender selects E2EE, a failure to resolve or validate recipient
keys MUST NOT cause automatic plaintext delivery.

The client MUST present the reason and require explicit user action to
send an unencrypted alternative.

### 9.5 Traditional OpenPGP interoperability

Implementations SHOULD support OpenPGP/MIME for communication with
compatible clients. Key discovery may use WKD and Autocrypt where
appropriate.

DMN-native envelopes and OpenPGP/MIME are distinct profiles. A client
MUST NOT assume that a native DMN envelope can be decrypted by a generic
OpenPGP client.

------------------------------------------------------------------------

## 10. Federation Protocol (DFP)

### 10.1 Domain capability discovery

A receiving domain advertises:

-   DMN protocol versions.
-   Supported message profiles.
-   Identity directory endpoint.
-   Transparency log endpoint.
-   Optional native delivery endpoint.
-   Key and certificate information.

The discovery mechanism MUST be authenticated using DNSSEC and/or HTTPS
with domain validation and signed protocol metadata.

### 10.2 SMTP delivery sequence

1.  Sender client resolves recipient identity and keys.
2.  Sender client creates and signs the encrypted envelope.
3.  Sending mail node accepts the MIME message.
4.  Sending node resolves recipient MX.
5.  Sending node establishes SMTP with TLS according to standard mail
    security policy.
6.  Receiving node validates syntax, size, policy, and idempotency.
7.  Receiving node stores ciphertext and acknowledges SMTP acceptance.
8.  Recipient mailbox becomes available through IMAP/JMAP.
9.  Client fetches and decrypts locally.

SMTP acceptance means the receiving server accepted responsibility for
delivery; it does not mean the user read or decrypted the message.

### 10.3 Native delivery endpoint

A future optional endpoint may support direct encrypted-envelope
submission between DMN nodes. It MUST provide:

-   Mutual domain/service authentication.
-   Message idempotency.
-   Explicit acceptance/rejection.
-   Rate limits and quotas.
-   Retry-safe semantics.
-   Version negotiation.
-   No plaintext access requirement.

SMTP remains the mandatory fallback.

### 10.4 Retry and failure

Mail nodes MUST implement queueing, retry with backoff, expiry, bounce
handling, and duplicate suppression.

Identity lookup failures, stale keys, unsupported protocol versions, and
invalid signatures MUST have distinct error states.

------------------------------------------------------------------------

## 11. Mailbox and Client Protocol Behaviour

### 11.1 Mailbox storage

A server may store the encrypted MIME envelope as a normal mailbox
message. The server may index transport metadata but MUST NOT index
decrypted subject or body for native E2EE messages.

### 11.2 Client processing

A receiving client MUST:

1.  Fetch the message.
2.  Parse and validate the envelope.
3.  Check message version and supported algorithms.
4.  Resolve sender identity and signing key.
5.  Verify transparency and key status according to policy.
6.  Verify signature and ciphertext hash.
7.  Locate the matching recipient-device envelope.
8.  Decrypt the CEK.
9.  Authenticate and decrypt the payload.
10. Parse MIME safely and display content.

A client MUST NOT render untrusted HTML or attachments without normal
content-safety controls.

### 11.3 Read receipts and delivery receipts

Read receipts and delivery acknowledgements are separate protocol
events. They MUST be optional and user-controlled. A receipt is not
cryptographic proof that a human read the content unless a carefully
defined, consent-based mechanism exists.

------------------------------------------------------------------------

## 12. Security and Privacy Requirements

### 12.1 Required

-   Private keys MUST remain on user-controlled devices or explicitly
    trusted secure hardware.
-   Servers MUST NOT receive plaintext CEKs or private encryption keys.
-   AEAD authentication MUST succeed before plaintext is released.
-   Signatures MUST be checked against authenticated identity records.
-   Key changes MUST be transparent and auditable.
-   E2EE send operations MUST fail closed on untrusted keys.
-   Secrets MUST NOT be written to logs, telemetry, crash dumps, or
    analytics.
-   Recovery flows MUST not silently weaken key trust.
-   Implementations MUST enforce message and attachment size limits.

### 12.2 Metadata limitations

The protocol does not inherently hide:

-   SMTP sender and recipient addresses.
-   Routing domains and IP addresses.
-   Message timing and size.
-   Mailbox access patterns.
-   Storage access patterns.
-   Blockchain transaction timing and account relationships.

Operators SHOULD minimize retention and avoid publishing linkable
identity/message metadata on public chains.

### 12.3 Compromise and recovery

A lost or compromised device must be revocable. The identity authority
must publish a revocation event and rotate keys.

Recovery MUST be defined before deployment. Options include user-held
recovery keys, multi-party recovery, or domain-admin recovery. Each
option has different trust implications and must be visible to users.

------------------------------------------------------------------------

## 13. Versioning and Extensibility

Protocol identifiers use major versions, such as `dmn-sme/1`.

-   Backward-compatible optional fields may be added within a major
    version.
-   Changes to cryptographic semantics require a new profile or major
    version.
-   Unknown mandatory features cause rejection.
-   Algorithm downgrade is prohibited.
-   Implementations must advertise supported versions and profiles.

------------------------------------------------------------------------

## 14. Conformance Profiles

### Profile 1 --- DMN Core

-   Identity records.
-   Device keys.
-   Signed key discovery.
-   Native encrypted envelope.
-   SMTP carriage.
-   IMAP/JMAP mailbox access.

### Profile 2 --- OpenPGP Interoperability

-   OpenPGP/MIME send and receive.
-   WKD and/or Autocrypt discovery as applicable.
-   Explicit key trust and downgrade UX.

### Profile 3 --- Transparency

-   Merkle inclusion proofs.
-   Consistency proofs.
-   Signed checkpoints.
-   Independent monitors.

### Profile 4 --- Blockchain Anchor

-   Identity authority commitments.
-   Transparency checkpoint commitments.
-   Key lifecycle commitments.
-   No message content or public recipient lists.

A product may implement Core without Blockchain Anchor. This is
intentional.

------------------------------------------------------------------------

## 15. Interoperability and Security Test Plan

### 15.1 Identity tests

-   Valid identity registration.
-   Invalid domain ownership proof.
-   Invalid authority signature.
-   Unauthorized device addition.
-   Key rotation.
-   Key revocation.
-   Stale and expired key.
-   Conflicting transparency views.

### 15.2 Cryptographic tests

-   Correct encryption/decryption.
-   Wrong recipient key.
-   Modified ciphertext.
-   Modified manifest.
-   Invalid signature.
-   Invalid encapsulation.
-   Unsupported suite.
-   Replayed message.
-   Duplicate identifier with altered content.
-   Multi-device and multi-recipient delivery.

### 15.3 Email interoperability tests

-   Stalwart-to-Stalwart SMTP.
-   DMN-to-standard SMTP provider.
-   Standard provider-to-DMN.
-   OpenPGP/MIME clients.
-   IMAP and JMAP retrieval.
-   MIME attachment preservation.
-   BCC privacy.
-   Large message and attachment handling.
-   Spam filtering and queue retries.

### 15.4 Federation tests

-   Two independently administered domains.
-   DNS capability discovery.
-   TLS failure.
-   Identity service outage.
-   RPC outage.
-   Transparency service outage.
-   Retry and bounce behaviour.
-   Duplicate delivery.
-   Malformed envelope and resource-exhaustion attempts.

### 15.5 Required release gates

No production release until:

-   Cryptographic suite and canonical encoding are frozen.
-   Cross-implementation test vectors pass.
-   Downgrade and key-substitution tests pass.
-   Recovery and revocation have been tested.
-   Independent security review is completed.
-   Operational logging has been checked for secret leakage.

------------------------------------------------------------------------

## 16. Unresolved Protocol Decisions

The following must be decided before v1.0:

1.  Deterministic CBOR or canonical JSON.
2.  Exact HPKE ciphersuite and content AEAD.
3.  Whether message subject and selected headers are always encrypted.
4.  BCC envelope construction and SMTP delivery strategy.
5.  Identity authority and account-recovery model.
6.  DNS discovery record and well-known endpoint specification.
7.  Transparency-log protocol and checkpoint gossip.
8.  Solana account layout and authority model.
9.  Native MIME media type and binary encoding.
10. Key freshness and offline verification policy.
11. Message size limits and attachment chunking.
12. Forward-secrecy roadmap.

------------------------------------------------------------------------

## 17. Reference Standards

The implementation team should review the current versions and errata
of:

-   RFC 5321 --- Simple Mail Transfer Protocol.
-   RFC 5322 --- Internet Message Format.
-   RFC 2045--2049 --- MIME.
-   RFC 9051 --- IMAP4rev2.
-   RFC 8620 and related JMAP specifications.
-   RFC 9580 --- OpenPGP.
-   RFC 8551 --- S/MIME 4.0.
-   RFC 9180 --- Hybrid Public Key Encryption.
-   RFC 8616 --- OpenPGP Web Key Directory.
-   RFC 9162 --- Certificate Transparency Version 2.0.
-   W3C DID Core, if a DID representation is adopted.
-   DNSSEC specifications and current email authentication standards.

------------------------------------------------------------------------

## 18. Implementation Recommendation

The first interoperable prototype SHOULD avoid a blockchain dependency
in the message-delivery path.

Build two independent mail nodes, each running Stalwart, with two
clients and a shared test identity directory. Implement the native
encrypted envelope, signed key records, SMTP carriage, and local
decryption. Demonstrate that neither mail node nor object storage can
decrypt the message.

Then add a Merkle transparency log, followed by optional Solana
checkpoint anchoring.

This sequence validates the protocol's core security and
interoperability before introducing blockchain operational complexity.
