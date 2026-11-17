# Decentralized Mail Network (DMN)

## High-Level Design Document

**Status:** Draft v0.1\
**Date:** 17 November 2026\
**Purpose:** Define a practical architecture for an interoperable,
federated email network with end-to-end encryption (E2EE), portable
identity, and optional blockchain-backed trust.

------------------------------------------------------------------------

## 1. Executive Summary

The Decentralized Mail Network (DMN) is a federated email system that
preserves compatibility with traditional Internet email while adding
portable cryptographic identities and end-to-end encrypted
communication.

DMN is not intended to replace SMTP, MIME, IMAP, or JMAP. It uses
established email protocols for interoperability and delivery, while
adding a native identity, key-discovery, and encrypted-message protocol.

The system separates four responsibilities:

1.  **Identity and trust:** Bind email identities and domains to
    authenticated signing and encryption keys.
2.  **Message security:** Encrypt message content on sender devices and
    decrypt only on authorized recipient devices.
3.  **Mail transport and storage:** Use interoperable mail servers and
    standard protocols to route and store ciphertext.
4.  **Trust anchoring:** Optionally use Solana to anchor identity
    ownership and key-transparency commitments. The blockchain is not
    the message bus or primary mail store.

### 1.1 Design goals

-   End-to-end confidentiality for native DMN messages.
-   Sender authentication and message integrity.
-   Interoperability with conventional SMTP/IMAP/JMAP email.
-   Independent operation of mail domains and infrastructure providers.
-   Portable identities and multi-device support.
-   Key rotation, revocation, recovery, and auditable key history.
-   No requirement for users to hold cryptocurrency or operate
    validators.
-   Graceful operation during blockchain or identity-service outages.

### 1.2 Non-goals for the initial release

-   Hiding all traffic metadata or providing network-level anonymity.
-   Guaranteeing that a recipient cannot copy, forward, or disclose
    decrypted content.
-   Replacing the global DNS and SMTP ecosystem.
-   Storing complete email bodies or attachments on a public blockchain.
-   Inventing a new cryptographic primitive or replacing established
    MIME/OpenPGP formats.
-   Guaranteeing permanent availability of content-addressed storage.

------------------------------------------------------------------------

## 2. Terminology

  -----------------------------------------------------------------------
  Term                                Meaning
  ----------------------------------- -----------------------------------
  Identity                            A verified mail identity, normally
                                      an email address at a domain

  Identity provider (IdP)             Service or domain authority that
                                      publishes identity records

  Mail node                           Independently operated mail server
                                      and associated services

  Device                              A user endpoint with its own
                                      cryptographic keys

  Signing key                         Private key used to sign protocol
                                      objects

  Encryption key                      Public/private key material used to
                                      establish or wrap encryption keys

  Envelope                            Recipient-specific metadata and
                                      encrypted content-key material

  Ciphertext object                   Encrypted MIME message or native
                                      message payload

  Federation                          Communication between independently
                                      operated DMN mail nodes

  Key transparency log                Append-only authenticated record of
                                      identity-key changes

  Blockchain anchor                   Commitment to identity or
                                      transparency state recorded on a
                                      blockchain
  -----------------------------------------------------------------------

------------------------------------------------------------------------

## 3. System Context and Actors

### 3.1 Actors

-   **User:** Sends, receives, searches, and manages email.
-   **Client:** Web, desktop, or mobile application. Performs encryption
    and decryption locally.
-   **Domain operator:** Controls a mail domain and its identity
    namespace.
-   **Mail node:** Accepts, queues, routes, and stores messages.
-   **Identity resolver:** Resolves an email identity to authenticated
    public-key records.
-   **Transparency service:** Publishes append-only key history and
    inclusion/consistency proofs.
-   **Storage provider:** Stores encrypted message objects and
    attachments.
-   **Blockchain network:** Optionally anchors identity ownership and
    transparency checkpoints.
-   **Traditional mail provider:** Gmail, Outlook, or any
    standards-compliant SMTP/IMAP provider.

### 3.2 Trust boundaries

The system assumes mail servers, storage providers, relays, RPC
providers, and blockchain observers may be curious or compromised.

The client is trusted with plaintext and private keys. The cryptographic
design must not require mail servers or storage providers to see
plaintext.

A compromised endpoint can expose plaintext and keys. E2EE does not
protect against malware, screen capture, malicious browser extensions,
or an authorized recipient intentionally disclosing content.

------------------------------------------------------------------------

## 4. Logical Architecture

``` text
+-----------------------------------------------------------+
|                       User Clients                         |
|       Web / Desktop / Mobile / Third-party clients         |
|                                                           |
|  Identity UI | Key vault | Encrypt/Sign | Decrypt/Verify   |
+-------------------------------+---------------------------+
                                |
                     HTTPS / JMAP / SMTP submission
                                |
+-------------------------------v---------------------------+
|                    DMN Mail Node                           |
|                                                           |
|  SMTP MTA | IMAP/JMAP | Mailbox | Queue | Policy/Spam      |
|                                                           |
|  +------------------- DMN Services ---------------------+  |
|  | Identity resolver | Key cache | Federation adapter  |  |
|  | E2EE MIME handler | Delivery state | Notifications   |  |
|  +------------------------------------------------------+  |
+-------------+------------------+--------------------------+
              |                  |
       +------v------+    +------v----------------+
       | PostgreSQL  |    | Encrypted object store|
       | metadata    |    | MIME blobs/attachments|
       +-------------+    +-----------------------+
              |
       +------v-----------------------------------+
       | Identity & trust infrastructure           |
       | DNS / DNSSEC / key directory / Merkle log |
       | Optional Solana identity registry         |
       +-------------------------------------------+
```

### 4.1 Component responsibilities

  -----------------------------------------------------------------------
  Component               Responsibility          Initial implementation
  ----------------------- ----------------------- -----------------------
  Mail server             SMTP, IMAP, JMAP,       Stalwart
                          queue, mailbox          

  DMN client SDK          Identity, encryption,   Rust core with
                          signing, MIME envelope  TypeScript bindings

  Identity service        Registration, lookup,   Rust service +
                          device/key status       PostgreSQL

  Key transparency        Append-only key history Merkle transparency
                          and proofs              service

  Federation adapter      Domain discovery and    Rust service integrated
                          DMN delivery metadata   with mail node

  Object storage          Ciphertext and          S3-compatible storage
                          attachments             

  Mailbox index           Message-to-user         Mail server database /
                          indexing and            PostgreSQL
                          notifications           

  Blockchain program      Optional identity and   Solana program in Rust
                          log checkpoint anchors  

  Event delivery          Notifications and       NATS or an equivalent
                          asynchronous work       queue
  -----------------------------------------------------------------------

Stalwart is the proposed initial mail-server foundation. Integration
should use supported APIs, SMTP/JMAP, webhooks, and documented extension
points. Do not fork or modify its internal mail-processing pipeline
until a concrete requirement justifies it.

------------------------------------------------------------------------

## 5. Identity Architecture

### 5.1 Address and identity model

A DMN identity is normally represented by an email address, such as
`alice@example.org`. The address is a human-readable identifier, not a
cryptographic key.

The domain controls the namespace and proves domain ownership. A user or
organization controls one or more signing keys and device encryption
keys.

The system must distinguish:

-   Domain ownership.
-   Identity ownership.
-   Account authentication.
-   Device authorization.
-   Signing-key validity.
-   Encryption-key validity.

A Solana wallet must not be mandatory for users. Blockchain ownership
may be represented by a domain-controlled authority or optional
user-controlled key.

### 5.2 Key separation

Use distinct key purposes:

-   **Identity signing key:** Ed25519 or another supported signing
    scheme.
-   **Device signing key:** Signs device-specific protocol objects.
-   **Device encryption key:** X25519/HPKE-compatible recipient key.
-   **Storage/backup key:** Protects encrypted local key backups; never
    published.
-   **Account login credential:** Authentication to a service; not used
    directly for message encryption.

Do not assume an Ed25519 key can be used directly as an encryption key.
Use separately generated keys and bind them through a signed identity
record.

### 5.3 Registration

1.  Verify control of the email address or domain through a challenge.
2.  Generate signing and encryption keypairs on the client/device.
3.  Prove possession of the signing private key.
4.  Publish a signed identity record.
5.  Add the record to the key-transparency log.
6.  Optionally anchor a commitment to the log checkpoint or identity
    record on Solana.
7.  Publish discovery information through DNS and/or the identity
    directory.

### 5.4 Key rotation and revocation

Key updates must be authorized by a currently trusted identity authority
or by a recovery process established at registration.

Each key record includes:

-   Key identifier and version.
-   Key purpose and algorithm.
-   Validity interval.
-   Device identifier.
-   Status: active, retired, revoked, or compromised.
-   Previous-record hash or transparency-log sequence.
-   Authorization signature.

A client must detect unexpected key changes and show a warning rather
than silently accepting a replacement key.

### 5.5 Multi-device

Each device has independent signing and encryption keys. A sender
encrypts the content-encryption key to every authorized recipient
device. A user may also have a designated recovery device or offline
recovery key.

Device removal prevents future messages from being encrypted to that
device. It cannot revoke plaintext or keys already obtained by that
device.

------------------------------------------------------------------------

## 6. Message Security Architecture

### 6.1 Cryptographic approach

Use standardized, reviewed cryptographic constructions and established
libraries.

Recommended design candidates:

-   HPKE (RFC 9180) for public-key encryption of recipient key material.
-   XChaCha20-Poly1305 or AES-256-GCM for authenticated content
    encryption.
-   Ed25519 for signatures where supported by the selected protocol
    profile.
-   SHA-256 for content and manifest commitments.
-   HKDF-SHA-256 for key derivation where required.
-   Argon2id for password-derived protection of local backups, with
    parameters selected through security review.

The exact HPKE ciphersuite and algorithm identifiers must be fixed in
the protocol specification. Avoid ad hoc conversion between key types.

### 6.2 Hybrid encryption

For each message:

1.  Generate a cryptographically random content-encryption key (CEK).
2.  Encrypt the canonical MIME payload once using an AEAD cipher.
3.  For each authorized recipient device, encapsulate/wrap the CEK using
    that device's public encryption key.
4.  Construct a manifest containing the ciphertext hash, algorithm
    identifiers, sender identity/key reference, and recipient envelopes.
5.  Sign the canonical manifest and ciphertext commitment.
6.  Deliver the encrypted object and recipient envelope through SMTP or
    native federation.

The recipient decrypts the CEK with its device private key and then
decrypts the payload.

### 6.3 Signatures and sender authentication

A valid signature proves that the holder of a signing key signed the
covered object. It does not, by itself, prove that the key belongs to a
particular person. Clients must validate the identity-to-key binding and
transparency history.

The signature must cover a deterministic canonical representation of the
manifest and ciphertext hash. The protocol must specify canonical
serialization and reject ambiguous encodings.

### 6.4 Forward secrecy

A static long-term recipient key does not provide forward secrecy for
stored messages: compromise of that private key may expose captured
ciphertexts encrypted to it.

The initial MVP may use per-message CEKs wrapped to recipient device
keys, with explicit disclosure of this limitation. A later profile
should evaluate ratcheting/session protocols or ephemeral recipient key
mechanisms. Do not claim forward secrecy unless the implemented protocol
and key lifecycle actually provide it.

### 6.5 BCC and recipient privacy

BCC recipients must not appear in publicly visible headers, public
blockchain data, shared recipient lists, or logs accessible to other
recipients.

Use recipient-specific envelopes and transport handling that does not
disclose BCC addresses to other recipients. SMTP transport metadata and
receiving servers may still learn delivery addresses. The design does
not provide full traffic-analysis resistance.

------------------------------------------------------------------------

## 7. Mail Transport and Interoperability

### 7.1 Standards

-   SMTP for server-to-server delivery and client submission.
-   MIME for message structure and attachments.
-   IMAP and JMAP for mailbox access.
-   OpenPGP/MIME for interoperability with existing encrypted-email
    clients.
-   DNS MX for mail routing.
-   SPF, DKIM, and DMARC for domain-level email authentication and abuse
    reduction.
-   DNSSEC where available for authenticated DNS records.

### 7.2 Two message profiles

**Profile A --- OpenPGP/MIME interoperability**

Use established OpenPGP/MIME formats for communication with compatible
traditional email clients. Support key discovery through established
mechanisms such as WKD and Autocrypt where applicable. Do not assume all
conventional email clients support E2EE.

**Profile B --- Native DMN envelope**

Use a versioned native envelope for richer multi-device recipient
wrapping, key transparency references, and federation metadata. Carry it
inside MIME for SMTP delivery. Native messages remain opaque to mail
servers.

### 7.3 Traditional email behaviour

  -----------------------------------------------------------------------
  Sender                  Recipient               Behaviour
  ----------------------- ----------------------- -----------------------
  DMN client              DMN client              Native E2EE

  OpenPGP client          DMN client              OpenPGP/MIME if keys
                                                  and algorithms are
                                                  compatible

  DMN client              OpenPGP recipient       OpenPGP/MIME if
                                                  supported

  Ordinary client         DMN client              Ordinary email; no E2EE
                                                  guarantee

  DMN client              Gmail/Outlook ordinary  Ordinary email or
                          mailbox                 explicitly configured
                                                  secure-message workflow
  -----------------------------------------------------------------------

A gateway must never silently downgrade an explicitly requested
encrypted message to plaintext. If the recipient cannot receive the
selected encryption profile, the sender must be warned and must
explicitly choose an alternative.

### 7.4 Mail server visibility

Mail servers necessarily process routing and delivery metadata. For
native E2EE messages, the server must not have access to plaintext or
private encryption keys.

Subject lines, sender/recipient headers, timestamps, IP addresses,
message sizes, and SMTP routing information may remain visible unless
specifically protected. Even encrypted subject handling can create
interoperability and filtering limitations.

------------------------------------------------------------------------

## 8. Federation Design

Each operator runs a mail node for one or more domains.

### 8.1 Discovery

1.  Resolve the recipient domain using DNS MX.
2.  Determine whether the domain advertises DMN capabilities through a
    DNSSEC-protected record or a well-known HTTPS endpoint.
3.  Fetch the recipient's signed identity/key record.
4.  Validate the record and transparency proof.
5.  Submit using SMTP with the native MIME envelope, or use an optional
    native delivery endpoint.

### 8.2 Federation principles

-   SMTP remains the baseline transport.
-   Native federation is an optimization, not a requirement.
-   Operators authenticate endpoints using TLS and domain identity.
-   Messages are idempotently identified to prevent duplicate
    processing.
-   Receiving nodes return standard SMTP acceptance/rejection status.
-   Asynchronous notifications and delivery receipts are not treated as
    proof that a human read a message.
-   Nodes must support retry, queue expiry, bounce handling, and abuse
    controls.

### 8.3 Availability

If the identity or blockchain service is unavailable:

-   Use cached, previously verified, unexpired key records according to
    local policy.
-   Queue messages requiring fresh key resolution.
-   Do not silently accept an unverified key replacement.
-   Continue delivery of already encrypted messages where policy
    permits.
-   Record the trust state used at send time.

------------------------------------------------------------------------

## 9. Storage and Data Model

### 9.1 On-chain data

Store only minimal commitments and authority state:

-   Opaque identity identifier.
-   Domain or authority commitment.
-   Identity authority/public-key commitment.
-   Key version or transparency checkpoint.
-   Revocation/status information.
-   Recovery-policy commitment.

Do not store plaintext, private keys, full email addresses, contact
lists, recipient lists, message subjects, or full message bodies
on-chain.

Public-chain transactions are permanent and observable. Even opaque
identifiers can become linkable through correlation.

### 9.2 Off-chain data

  Data                       Storage
  -------------------------- ------------------------------
  Encrypted message object   Object storage
  Encrypted attachments      Object storage
  Mailbox metadata           Mail server database
  Identity cache             PostgreSQL/cache
  Transparency tree nodes    Transparency service storage
  Delivery queue             Mail server queue
  Private keys               Client secure storage only

Content-addressed storage provides integrity addressing, not guaranteed
persistence or privacy. Encrypt before upload and ensure
pinning/retention arrangements if IPFS is used.

### 9.3 Deletion

Deleting a mailbox entry does not delete a public blockchain transaction
or copies held by recipients and storage providers. The product must
distinguish:

-   Remove from inbox.
-   Delete local copy.
-   Request storage-object deletion.
-   Expire storage access/reference.
-   Revoke future device access.

The UI must not promise global deletion or recall after delivery.

------------------------------------------------------------------------

## 10. Solana Integration

Solana is an optional trust anchor, not a dependency for SMTP delivery.

### 10.1 Program responsibilities

A minimal identity-registry program may support:

-   Register domain/identity authority.
-   Update authorized key commitment.
-   Rotate or revoke key version.
-   Publish transparency checkpoint commitment.
-   Transfer or recover identity authority under an explicit policy.

### 10.2 Program non-responsibilities

-   Store email content.
-   Store attachments.
-   Deliver messages.
-   Resolve every mailbox query synchronously.
-   Store private keys.
-   Act as the sole identity database.

### 10.3 Operational model

Use a local validator for development and test environments. In
production, initially use multiple independent RPC providers. Running a
validator is not required for a mail-node operator.

The identity service should verify chain state and cache authenticated
results. It should handle RPC outages, reorg/finality policy, stale
data, and checkpoint verification.

------------------------------------------------------------------------

## 11. Security Threat Model

### 11.1 Threats addressed

-   Mail server reads message content.
-   Storage provider reads message content.
-   Network attacker modifies ciphertext.
-   Unauthorized party substitutes a public key.
-   Mail node replays or duplicates a message.
-   Unauthorized device is added to an identity.
-   Spam and resource exhaustion.
-   Key rotation is hidden from users.

### 11.2 Threats not fully addressed

-   Compromised sender or recipient endpoint.
-   Malicious authorized recipient.
-   Traffic analysis and metadata correlation.
-   Coercion or endpoint seizure.
-   Denial of service by mail providers, DNS, storage, or RPC providers.
-   Global deletion of previously delivered messages.
-   Availability of unpinned decentralized storage content.

### 11.3 Required security controls

-   Use maintained cryptographic libraries.
-   No custom cryptographic algorithms.
-   Secure random-number generation.
-   Strict algorithm/version negotiation.
-   Canonical serialization.
-   Signature and AEAD verification before displaying content.
-   Key transparency and change warnings.
-   Rate limits, quotas, and abuse controls.
-   Encrypted local key backups and recovery testing.
-   Dependency scanning, fuzzing, and protocol test vectors.
-   Independent cryptographic and application security audit before
    production.

------------------------------------------------------------------------

## 12. Operational Architecture

### 12.1 MVP deployment

A single development environment may run:

-   Stalwart mail server.
-   DMN identity and federation services.
-   PostgreSQL.
-   S3-compatible local object store.
-   NATS or an equivalent queue.
-   Solana local validator.
-   Web client and test clients.

Use Docker Compose for reproducible local development.

### 12.2 Production deployment

Separate independently scalable services:

-   Mail nodes per operator/domain.
-   Identity and key-directory replicas.
-   Transparency-log replicas and monitors.
-   Object storage with replication and retention policy.
-   PostgreSQL backups and tested recovery.
-   Multiple RPC providers.
-   Observability and abuse monitoring.

Kubernetes is optional and should be introduced only when scale and
operations justify it.

### 12.3 Monitoring

Monitor:

-   SMTP queue age and delivery success.
-   Mailbox storage growth.
-   Identity lookup latency and cache age.
-   Key verification failures.
-   Transparency-log consistency.
-   Object-storage availability.
-   Federation errors and retries.
-   Spam and abuse rates.
-   Blockchain/RPC health.

Never log plaintext, CEKs, private keys, or decrypted attachments.

------------------------------------------------------------------------

## 13. Implementation Roadmap

### Phase 0 --- Standards and threat-model validation

-   Finalize identity and message profiles.
-   Select exact cryptographic suites.
-   Define canonical serialization.
-   Document metadata leakage and downgrade rules.
-   Review OpenPGP/MIME and Autocrypt interoperability.

**Exit criteria:** Reviewed protocol draft and threat model.

### Phase 1 --- Local E2EE prototype

-   Two users and multiple devices.
-   Key generation and registration.
-   Native encrypted MIME envelope.
-   Signature verification.
-   Encrypted attachments.
-   Replay protection.

**Exit criteria:** Mail server and storage cannot decrypt test messages.

### Phase 2 --- Traditional email interoperability

-   Deploy Stalwart.
-   SMTP delivery.
-   IMAP/JMAP mailbox access.
-   OpenPGP/MIME interoperability tests.
-   Explicit plaintext fallback UX.

**Exit criteria:** Exchange test messages with conventional providers
and OpenPGP clients.

### Phase 3 --- Federation

-   Two independently operated mail nodes.
-   DNS capability discovery.
-   Identity resolution.
-   SMTP-carried native envelopes.
-   Queue, retry, bounce, and idempotency.

**Exit criteria:** Native E2EE communication between separate operators.

### Phase 4 --- Identity transparency and blockchain anchoring

-   Append-only Merkle log.
-   Inclusion and consistency proofs.
-   Key rotation and revocation.
-   Optional Solana checkpoint anchoring.
-   Multiple RPC providers.

**Exit criteria:** Clients can detect unauthorized key-history changes.

### Phase 5 --- Production hardening

-   Multi-region deployment.
-   Recovery and disaster-recovery tests.
-   Abuse prevention.
-   Independent audits.
-   Mobile/desktop clients.
-   Operator onboarding and documentation.

------------------------------------------------------------------------

## 14. Key Architecture Decisions

  -----------------------------------------------------------------------
  Decision                Proposed choice         Reason
  ----------------------- ----------------------- -----------------------
  Mail server             Stalwart                Mature Rust mail server
                                                  and standard protocols

  Transport               SMTP first              Maximum
                                                  interoperability

  Mailbox access          IMAP/JMAP               Existing client
                                                  compatibility

  Encryption              OpenPGP/MIME            Established email
  interoperability                                encryption format

  Native message security HPKE + AEAD             Hybrid encryption with
                                                  recipient envelopes

  Identity                Email/domain identity   Portable,
                          plus device keys        human-readable identity

  Key discovery           DNS + signed directory  Works without
                                                  blockchain availability

  Key history             Merkle transparency log Detect key substitution

  Blockchain              Optional Solana anchor  Publicly verifiable
                                                  commitments

  Storage                 S3-compatible first     Operational simplicity
                                                  and persistence control

  Backend                 Rust                    Shared language with
                                                  cryptographic and mail
                                                  components
  -----------------------------------------------------------------------

------------------------------------------------------------------------

## 15. Open Questions Requiring Decisions

1.  Should the first release support only user identities under
    DMN-operated domains, or also bind existing Gmail/Outlook addresses?
2.  Should native messages use a new MIME content type, or should the
    first profile use an OpenPGP-compatible container?
3.  Which exact HPKE ciphersuite and AEAD profile will be mandatory?
4.  Should domain operators control identity recovery, or should users
    control recovery independently?
5.  Is Solana a mandatory registry or an optional anchoring backend?
6.  What metadata privacy level is required for the first release?
7.  Is the product initially a hosted service, a self-hostable protocol,
    or both?

These decisions should be resolved before freezing protocol version 1.

------------------------------------------------------------------------

## 16. Recommended First Deliverable

Build a two-node proof of concept:

-   Two Stalwart instances on separate domains.
-   Two DMN clients with independent device keys.
-   One identity/key directory.
-   One encrypted MIME envelope implementation.
-   SMTP delivery between the nodes.
-   Local PostgreSQL and object storage.
-   No blockchain dependency in the first working path.

Demonstrate that Node A can deliver a message to Node B, both servers
store only ciphertext for native messages, and Bob can verify and
decrypt it locally. Then add key transparency and Solana anchoring.

**Success criterion:** A functioning federated, standards-compatible
mail system whose native messages remain end-to-end encrypted even when
every mail and storage server is considered untrusted.
