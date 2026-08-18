# Mycel App-signing Model

Status: design draft

This note describes how a Mycel-based system should treat application signing
as a set of separate but explicitly linked trust boundaries, rather than as one
signature decision.

The main design principles are:

- application-state signing is not the same as transport-envelope signing
- transport-envelope signing is not the same as release signing
- release signing is not the same as execution-evidence signing
- each signing layer authenticates a different claim
- signatures at separate layers need explicit bindings before they describe the
  same app, release, request, or execution
- a secure deployment should not assume one layer automatically replaces the
  others

Related notes and specifications:

- `PROTOCOL.*` for the normative v0.1 object-signature matrix
- `WIRE-PROTOCOL.*` for transport-envelope signatures
- `DESIGN-NOTES.signature-role-matrix.*` for app-layer signer roles
- `DESIGN-NOTES.signature-priority.*` for app-layer signing priorities
- `DESIGN-NOTES.mycel-app-layer.*` for effect requests and receipts

## 0. Goal

Enable a Mycel-based application system to distinguish at least four signing
needs:

- signing authority-bearing app records and accepted-state inputs
- signing transport envelopes
- signing released software artifacts
- signing execution claims, receipts, or runtime attestations

This note does not define one mandatory signing toolchain.

It defines the trust boundaries, signer lifecycle, and cross-layer bindings that
a signing model should preserve.

## 1. What a Signature Proves

A cryptographically valid signature proves that the holder of a particular key
signed particular bytes.

It does not by itself prove:

- that the key holder was authorized for the claimed role
- that the signed statement is true
- that an external side effect actually happened
- that the signed object is current rather than an older valid object replayed
  after a policy or release change
- that two independently signed objects refer to the same app, release, request,
  or execution

Each layer therefore needs verification rules for:

- canonical signed bytes and signature-domain separation
- trust roots and role authorization
- key rotation, revocation, and compromise recovery
- the governance state or policy under which the signer was authorized
- freshness, version, and replay or rollback handling
- references that bind the signed claim to adjacent layers

## 2. Layer 1: App-layer Record Signing

This layer protects Mycel-carried application records and the protocol objects
that carry them.

Typical authority-bearing records include:

- `app_manifest`
- app-governance records
- policy objects
- proposal and approval records
- effect requests or execution intents
- effect receipts when represented as app-layer records

Primary purpose:

- attribute a record to an authorized app or governance role
- protect record integrity
- preserve governance and state history
- provide verifiable inputs to accepted-state derivation

App-layer records are not automatically new Mycel protocol object types. In
Mycel v0.1, the normative top-level object-signature rules apply to `patch`,
`revision`, `view`, and `snapshot`; `document` and `block` do not carry a
top-level protocol signature.

An app profile should therefore choose and specify one of these mechanisms:

1. rely on a signed protocol container whose signed fields and signer authority
   cover the app record adequately
2. define an embedded app-record signature, including its signer field,
   canonical payload, domain separator, and verification rules

The profile should also define which records require signatures, which derived
or cache-like records can be recomputed from signed inputs, and whether any
co-signature or threshold rule applies. Derived summaries should not outrank
canonical signed source records merely because the summaries are signed.

This layer does not by itself authenticate the peer that transported the object
or prove that a downloaded software artifact is authentic.

## 3. Layer 2: Transport-envelope Signing

This layer authenticates wire-message metadata and the sending peer according
to the active transport profile.

Typical signed material includes:

- wire message type and message ID
- sender identity
- timestamp
- message payload or payload reference, as defined by the wire protocol

Primary purpose:

- detect wire-message substitution or tampering
- attribute a transport message to its sender key
- support session-level sender and sequencing checks

The transport-envelope signature does not replace object-level or embedded
app-record signatures. A peer that legitimately forwards an object does not
become the author of that object, and a valid object signature does not prove
which peer delivered it in a particular session.

## 4. Layer 3: Release-artifact Signing

This layer protects distributed software artifacts.

Typical signed artifacts or release metadata include:

- CLI binaries
- application packages
- installers
- container images
- release manifests
- provenance attestations and software bills of materials

Primary purpose:

- protect the software supply chain
- attribute an artifact digest and release manifest to an authorized release
  signer
- detect substituted or tampered releases
- support version, platform, provenance, and rollback checks

This layer is important even if all authority-bearing Mycel records are signed.
A user may still download a malicious client that verifies Mycel objects
incorrectly or leaks secrets locally.

Where an `app_manifest` identifies executable software, it should reference a
signed release manifest or immutable artifact digest. The release manifest
should bind at least the app identity, version, target platform, artifact
digest, and relevant provenance. Without this binding, a valid app signature
and a valid release signature may describe different software.

Release verification also needs an authorized release-signer policy, rotation
and revocation handling, and a rule for rejecting unintended downgrade or
rollback to an older but validly signed release.

This layer belongs in the build, release, and distribution pipeline rather than
in the protocol core.

## 5. Layer 4: Execution-evidence Signing

This layer protects attributable claims about what a runtime did or observed.

Typical signed claims or evidence include:

- execution receipts
- settlement receipts
- runtime attestations
- external-effect confirmations

Primary purpose:

- attribute a receipt or attestation to an authorized runtime or executor key
- preserve post-event auditability
- distinguish an intended action from a runtime's claim about the completed
  side effect

A signed receipt is not, by itself, proof that the claimed external event
happened. It proves that the signer made the claim. High-risk profiles should
also require independently verifiable evidence appropriate to the effect, such
as a ledger transaction reference and proof, a counterparty acknowledgement,
an independent observer signature, or a hardware-backed attestation.

An execution receipt should bind at least:

- `app_id` and applicable app version
- the release manifest or artifact digest used by the runtime
- `effect_request_id` or `execution_intent_id`
- the accepted state, policy, and signer-set versions that authorized execution
- an idempotency key or execution nonce
- executor or runtime identity
- start and finish times, outcome, and any external evidence references

These bindings prevent a valid receipt for one request, policy, release, or app
from being reused as evidence for another.

This layer is especially important for:

- payment execution
- custody systems
- external-effect systems
- disputes and incident review

It should not be confused with release signing. An authentic runtime artifact
can still produce a false, incomplete, or unrelated receipt, while an authentic
receipt can have been produced by an obsolete or unauthorized runtime release.

## 6. Core, Profile, Transport, Release, and Runtime Boundaries

The protocol core should define its normative object-signature rules and expose
the cryptographic verification primitives needed by higher layers. It should
not silently treat arbitrary app-level record families as new protocol object
types.

The app and profile layer should define:

- which app records require embedded signatures or signed protocol containers
- the canonical signed payload and domain separator for embedded signatures
- whose signatures are authorized for each record family
- how authorization changes, rotation, and revocation are evaluated
- how signed records participate in accepted-state selection

The transport layer should define:

- which envelope fields are signed
- how sender keys map to peer identities
- session sequencing, timestamp, and replay checks

The release pipeline should define:

- how artifact digests, release manifests, and provenance are signed
- which release keys are trusted
- how downgrade, rotation, revocation, and transparency are handled

The runtime and execution profile should define:

- how execution receipts and attestations are signed
- how runtime identities and authorized releases are managed
- which request, state, policy, release, and evidence references a receipt must
  bind
- what independent evidence is required before a signed claim is treated as a
  verified external result

This keeps the protocol core stable while allowing higher-layer signing models
to evolve without weakening the links between them.

## 7. Common Failure Cases

### 7.1 App-record Signing Without Release Signing

Authority-bearing governance records are signed, but users download unsigned
binaries.

Result:

- governance history may be valid
- the client supply chain may still be compromised

### 7.2 Release Signing Without App-record Signing

The shipped binary is authentic, but governance and app-state inputs have weak
authorship or authorization rules.

Result:

- the artifact is attributable to an authorized release signer
- app-layer authority and history remain weak

### 7.3 Authentic Runtime Without Attributable Receipts

The runtime artifact is trusted, but execution claims are unsigned or cannot be
attributed to an authorized runtime identity.

Result:

- post-event audit and dispute handling become weak

### 7.4 Signed Receipt Without Independent Evidence

An authorized runtime signs a receipt claiming that an external action
succeeded, but the profile has no independent evidence requirement.

Result:

- the claim is attributable
- a lying or compromised runtime can still report a false outcome

### 7.5 Valid Signatures Without Cross-layer Binding

An app manifest, release artifact, and execution receipt are each validly
signed, but they do not reference one another through stable IDs and digests.

Result:

- valid but unrelated or obsolete artifacts can be combined into a misleading
  verification result

### 7.6 Envelope Signing Treated as Object Authorship

A receiving node treats the peer that signed a transport envelope as the author
or authority for the carried object.

Result:

- relays can accidentally acquire authority they do not possess
- transport authentication can mask missing object-level verification

### 7.7 Valid Signature From a Revoked or Wrong-role Key

Signature bytes verify, but the system does not evaluate whether the key was
authorized for that role at the relevant state and time.

Result:

- retired, compromised, or unrelated keys can create apparently valid records
  or releases

## 8. Recommended Baseline

A practical deployment should usually provide:

1. signed authority-bearing app and governance records, using either a defined
   embedded signature or an adequate signed protocol container
2. transport-envelope verification that remains distinct from object authorship
3. signed release manifests and artifact digests linked to the app identity
4. signed execution receipts for high-risk runtimes, bound to the request,
   accepted state, policy, runtime release, and relevant external evidence
5. explicit trust-root, signer-authorization, rotation, revocation, freshness,
   and rollback rules for every enabled layer

Minimal deployments may begin with the first layer, but should explicitly state
which other guarantees they do not provide.

Security-sensitive deployments should not stop there.

## 9. Practical Rule

The right question is not:

- "Is the app signed?"

It is:

- "Which bytes are signed, by which authorized role and key, under which policy
  and time, what adjacent object or artifact do they bind, and what claim does
  that signature actually prove?"

If a deployment cannot answer that clearly for every enabled layer, its signing
model is underspecified.
