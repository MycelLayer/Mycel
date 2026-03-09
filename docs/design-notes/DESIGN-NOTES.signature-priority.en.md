# Mycel Signature-priority Note

Status: design draft

This note organizes which object families in the current Mycel repo should require signatures first, rather than treating every possible object family as equally urgent.

The main design principle is:

- sign the objects that define authority before the objects that merely summarize outcomes
- sign the objects that participate in accepted-state derivation before convenience or cache-like objects
- sign the objects whose forgery would silently change governance meaning
- expand outward in layers rather than trying to sign everything at once

## 0. Goal

Provide a practical prioritization for signature requirements across the current repo documents.

This note focuses on Mycel-carried objects.

It does not replace:

- release artifact signing
- transport envelope signing
- local-only audit artifacts

## 1. Priority Levels

This note uses three priority levels:

- `P1`: must sign first
- `P2`: should sign next
- `P3`: useful later, but not the first signature boundary to implement

## 2. P1: Must Sign First

These objects define app identity, governance authority, or execution authorization.

If they are unsigned or weakly signed, accepted-state reasoning becomes unsafe.

### 2.1 `app_manifest`

Why first:

- defines what the application is
- declares which documents and state families belong to it
- anchors app-level authority and scope

Likely signer:

- app author or app maintainer

### 2.2 Governance proposals and approvals

Examples in current repo vocabulary:

- allocation proposal
- signer approval
- governance signal
- accepted resolution inputs

Why first:

- these records change what the system is allowed to treat as accepted
- forged governance records can silently rewrite history meaning

Likely signer:

- governance actor, maintainer, or signer role defined by the profile

### 2.3 `signer_enrollment`

Why first:

- proves that a signer knowingly joined a custody system
- anchors later consent and signer-activity reasoning

Likely signer:

- the enrolling signer
- optionally a confirming governance or operator role

### 2.4 `consent_scope`

Why first:

- defines what bounded policy scope a signer accepted
- automatic signing becomes weak or invalid without clear authorship here

Likely signer:

- the signer granting consent

### 2.5 `signer_set`

Why first:

- defines the membership and m-of-n rule that later execution depends on
- forged signer-set state can redirect approval power

Likely signer:

- governance or custody-maintainer role

### 2.6 `policy_bundle`

Why first:

- defines what execution is allowed to do
- is a direct authorization boundary for automated behavior

Likely signer:

- governance or policy-authorizing role

### 2.7 `trigger_record` when it is an accepted execution trigger

Why first:

- this is where policy-bound execution may begin
- forged triggers can create unauthorized execution intents

Likely signer:

- trusted runtime, governance source, or trigger-author role, depending on profile

### 2.8 `execution_intent`

Why first:

- binds a concrete action to fund, policy, signer-set, and amount context
- is the exact object that later signing and execution refer back to

Likely signer:

- derived system authority, authorized runtime, or governance-derived executor role

## 3. P2: Should Sign Next

These objects are highly valuable for auditability, dispute handling, and high-integrity operation, but the authority boundary usually begins slightly earlier.

### 3.1 `signer_attestation`

Why second:

- proves one signer-side evaluation and result
- important for audit, threshold counting, and mismatch analysis

Likely signer:

- signer runtime or signer identity

### 3.2 `execution_receipt`

Why second:

- proves what the executor or runtime actually did
- required for post-event audit and external settlement linkage

Likely signer:

- runtime or executor identity

### 3.3 `pause_or_revoke_record`

Why second:

- changes future execution eligibility
- should be signed to avoid silent local override or forged emergency state

Likely signer:

- governance authority, signer, or emergency-control role defined by profile

### 3.4 Effect receipts in generic app-layer systems

Why second:

- important whenever a runtime performs external side effects
- needed to attribute runtime behavior correctly

Likely signer:

- runtime identity

## 4. P3: Useful Later

These may still need signatures in some deployments, but they are not the first boundary to stabilize.

### 4.1 Derived summaries and snapshots

Examples:

- balance snapshots
- resource summaries
- replayed indexes

Why later:

- they are useful, but ideally can be recomputed from already signed source history

### 4.2 Cache-like or convenience state documents

Examples:

- local status summaries
- convenience mirrors of accepted state

Why later:

- they should not be allowed to outrank canonical signed source objects

### 4.3 Optional monitoring and operator annotations

Why later:

- useful for operations, but not the first authority boundary

## 5. Minimal First-pass Set for the Current Repo

If the repo had to choose a narrow first signature set now, the strongest practical starting point is:

1. `app_manifest`
2. governance proposals and approvals
3. `signer_enrollment`
4. `consent_scope`
5. `signer_set`
6. `policy_bundle`
7. `trigger_record`
8. `execution_intent`
9. `execution_receipt`

This set covers:

- app identity
- governance authority
- signer legitimacy
- execution authorization
- final runtime evidence

## 6. Practical Rule

If forging an object could silently answer one of these questions incorrectly, it should be signed early:

- who is allowed to decide?
- what was actually approved?
- who is allowed to sign?
- what action was authorized?
- what action actually happened?

If an object only summarizes already signed truth, it is usually not the first signature priority.
