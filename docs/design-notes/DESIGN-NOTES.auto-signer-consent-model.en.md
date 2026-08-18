# Auto-signer Consent Model

Status: design draft

This note describes how a Mycel-based system should model consent for signers
that participate in automated m-of-n signing.

In this note, `m-of-n` means a signer set with `n` members and a threshold of
`m`. The underlying mechanism may use independent multisignatures, partial
signatures, or a cryptographic threshold-signature scheme. This note defines
consent and eligibility semantics, not the cryptographic construction.

The main design principles are:

- a signer principal explicitly consents to enrollment and one bounded policy
  scope in advance
- a signer does not manually approve each later transaction unless the active
  profile requires per-intent approval
- automatic signing is valid only against the exact accepted policy,
  signer-set, and consent versions to which the signer agreed
- policy widening, signer-set replacement, or key rotation must not silently
  inherit old consent
- pause, revoke, expiry, and synchronization failure must fail closed
- every signing result must remain attributable, replay-resistant, and
  auditable without treating a runtime operator as the consenting signer

Related notes and profiles:

- `DESIGN-NOTES.app-signing-model.*` for signature trust boundaries
- `DESIGN-NOTES.signature-role-matrix.*` for signer and runtime roles
- `DESIGN-NOTES.signature-priority.*` for record-signing priorities
- `DESIGN-NOTES.policy-driven-threshold-custody.*` for the custody flow
- `PROFILE.fund-auto-disbursement-v0.1.*` for the narrow interoperable profile
- `DESIGN-NOTES.blind-address-threat-model.*` for limited-disclosure signing

## 0. Goal and Scope

Enable automated m-of-n signing without pretending that "no per-transaction
human approval" means "no human consent."

This model separates:

- signer-principal consent
- signer-runtime eligibility checks
- governance-defined policy and signer-set state
- coordinator or executor behavior
- later execution and settlement evidence

It does not:

- treat enrollment as unlimited future authorization
- let a runtime operator consent on behalf of a signer principal
- define one mandatory key-custody or threshold-signature implementation
- claim that a signer attestation proves an external settlement occurred

## 1. Actors and Authority Boundaries

### 1.1 Signer Principal

The signer principal is the person, organization, or autonomous authority that
controls participation in the signer set.

The signer principal:

- signs or otherwise verifiably authorizes `signer_enrollment`
- signs one exact `consent_scope`
- may exercise signer-authorized pause or revoke controls when the profile
  permits them
- is the source of consent, even when a separate runtime holds the signing
  share

### 1.2 Signer Runtime

The signer runtime evaluates accepted state and produces a signature, partial
signature, or explicit refusal.

The signer runtime:

- proves possession of, or authorized access to, the enrolled signing key or
  share
- evaluates the exact consent, policy, signer-set, and intent versions
- signs a `signer_attestation` that binds its result to those versions
- must not widen the signer's consent locally

### 1.3 Governance Authority

The governance authority defines signer-set membership, policy bundles, and any
governance-controlled pause or revoke records.

Governance can narrow or disable what the system will execute. It cannot create
signer-principal consent merely by adding a key to a signer set.

### 1.4 Coordinator and Executor

A coordinator may distribute intents and collect signer attestations. An
executor may assemble the threshold result and submit the external action.

Neither role may:

- enroll a signer without signer-principal authorization
- substitute a different intent, policy, signer-set, or consent version
- count an attestation for a different `intent_hash`
- treat infrastructure operation as consent authority

## 2. Consent Boundary

The signer's consent happens when the signer knowingly authorizes both:

1. enrollment of a particular key or share commitment into a particular signer
   pool
2. one canonical, bounded consent scope

The signer does not need to:

- inspect or approve each transaction synchronously
- be online when every eligible intent is evaluated
- receive real-time notice whenever its share participates

The signer must still be able to know and verify:

- the fund, app, or signer pool in which it is enrolled
- the enrolled key or share commitment
- the signer-set ID and version
- the policy ID, version, and canonical digest
- the trigger, amount, rate, asset, destination, and time constraints
- the effective and expiry times
- how to pause, revoke, rotate, and inspect later signer attestations

A user-interface summary is not the consent object. The signer should see a
human-readable rendering that is unambiguously tied to the same canonical
consent-scope digest that the signer authorizes.

## 3. Core Definitions

### 3.1 Enrollment Consent

Enrollment consent means the signer principal knowingly binds one signer
identity and one key or share commitment to a defined fund and signer-set
context.

It minimally means:

- "this is my signer identity and enrolled key or share commitment"
- "this identity may participate in the named signer pool"
- "the associated signer runtime may sign automatically only under a separate,
  valid consent scope"

Enrollment should include proof of possession or another profile-defined proof
that the signer controls the enrolled key or share. Possession alone does not
grant policy consent.

### 3.2 Policy-scope Consent

Policy-scope consent means the signer principal authorizes one immutable scope
identified by both a version and a canonical digest.

Minimum boundaries:

- profile ID and version
- `fund_id` or equivalent app scope
- `signer_set_id` and signer-set version
- `policy_id`, policy version, and policy digest
- allowed trigger types
- maximum amount per execution
- cumulative amount per defined time window
- allowed assets
- allowed destination classes or allowlist digest
- cooldown, timelock, or delay requirements
- effective start and end time
- any required disclosure or blind-signing mode

### 3.3 Operational State

Operational state answers whether the enrolled signer may produce new signing
results under a particular consent scope.

Typical values:

- `pending`
- `active`
- `paused`
- `revoked`
- `expired`
- `rotating`
- `superseded`

Operational state is derived from accepted enrollment, consent, pause, revoke,
expiry, and rotation records. A mutable local status field must not override
that history.

### 3.4 Execution Intent

An execution intent is one concrete action derived from accepted trigger and
policy state. It is not a new consent grant.

The intent must commit to the exact outputs or action, total amount, asset,
policy version, signer-set version, and any nonce or idempotency key used for
replay prevention.

### 3.5 Signer Attestation

A signer attestation is an attributable signer-runtime claim that it evaluated
one exact intent under one exact authorization snapshot and either produced a
signing result or refused.

It does not prove that threshold was reached, that an executor broadcast the
transaction, or that external settlement succeeded.

## 4. What Counts as Valid Consent

Automatic signing is inside consent only when all of the following are true:

1. the signer principal verifiably authorized the enrollment record
2. the enrollment proves the required key or share possession
3. the signer principal verifiably authorized the exact consent-scope digest
4. the consent scope names the applicable fund, signer-set version, and policy
   version or digest
5. enrollment, consent, policy, and signer-set records are accepted under the
   active profile
6. no effective pause, revoke, expiry, rotation, or superseding record blocks
   the signer or scope
7. the intent matches every scope constraint
8. cumulative limits, cooldowns, and nonce or idempotency rules are evaluated
   against a deterministic accepted-state snapshot
9. the signer runtime is synchronized to that snapshot and fails closed if it
   cannot establish freshness
10. the signer attestation binds the evaluation result to the exact intent and
    authorization snapshot

If any condition fails, the runtime must refuse to sign and preserve an
auditable outcome.

## 5. What Does Not Count as Consent

The following are not valid consent:

- silent inclusion in a signer pool
- a governance record that names a signer without signer authorization
- inferred consent from general app or service usage
- an operator configuring a runtime with a signer's public key
- access to a policy document without signing its canonical digest
- unlimited automatic signing without policy bounds
- reuse outside the accepted fund, signer set, profile, or policy scope
- reuse of old consent after an incompatible policy or signer-set update
- signing after an effective pause, revoke, expiry, or supersession
- hidden widening of local runtime policy
- a click-through summary that is not tied to the signed consent object

## 6. Policy Changes and Re-consent

Consent applies to an exact policy scope, not to an unversioned policy name.

The system may continue using existing consent only when the new execution is
still evaluated against the exact policy digest and signer-set version named by
that consent.

A new signer-principal consent is required before automatic signing under a
change that:

- raises an amount or rate limit
- adds a trigger type, asset, destination, or capability
- weakens a cooldown, timelock, review, or disclosure requirement
- extends the effective period
- changes the fund or app scope
- changes the signer-set version or enrolled key or share commitment
- changes how an intent digest or signing payload is derived

A profile may allow a governance change that only narrows scope to take effect
without new signer consent, but it must preserve the old and new policy digests
and must not later restore the wider scope without re-consent.

## 7. Consent Lifecycle

### 7.1 Join

The signer principal authorizes enrollment into one signer pool.

Required outputs:

- signed or otherwise verifiably authorized `signer_enrollment`
- signer identity and consent-key reference
- signing-key or share commitment plus proof of possession
- fund and signer-set reference
- initial `pending` operational state

### 7.2 Activate

The signer becomes eligible only after the enrollment and exact consent scope
are accepted and mutually consistent.

Required outputs:

- consent-scope ID, version, and canonical digest
- effective policy and signer-set references
- effective and expiry times
- transition to `active`

### 7.3 Pause

Pause temporarily blocks new signing results without deleting enrollment or
history.

Two pause channels may coexist:

- a local emergency pause that the signer runtime applies immediately and
  fail-closed
- an accepted pause record that other participants can verify and enforce

A local pause must never be interpreted as permission to resume automatically.
Resumption requires an explicit profile-defined action and a fresh accepted
state check.

### 7.4 Revoke

Revocation permanently removes future eligibility for the referenced enrollment,
key, or consent scope.

Revocation:

- blocks new signer attestations once effective
- does not erase historical intents, attestations, or receipts
- cannot retract a signature already released or undo an external settlement
- should trigger key or signer-set rotation when compromise is suspected

### 7.5 Expire

Expiry blocks signing at or after `effective_until` according to the profile's
canonical time and clock-skew rules. Runtimes that cannot establish trustworthy
time must fail closed for time-bounded consent.

### 7.6 Rotate or Supersede

Rotation creates a new enrollment, key or share commitment, signer-set version,
or consent scope.

Old and new records remain linked for audit, but future intents bind only to the
new effective versions. Rotation never causes old consent to authorize a new
key, signer set, or widened policy implicitly.

### 7.7 Resume

Resume is not the absence of a pause record. It is an explicit transition that
identifies the enrollment and consent scope being reactivated.

Before resuming, the runtime must reload accepted state and re-evaluate expiry,
revocation, rotation, and policy compatibility.

## 8. Effective-state and In-flight Rules

Every signer attestation must identify one accepted authorization snapshot,
including the relevant enrollment, consent-scope, policy, signer-set, and
pause-or-revoke state.

The default safety rule is:

- a pause or revoke effective before a signer emits its attestation blocks that
  attestation
- attestations already released remain historical facts but may be rejected by
  the threshold assembler if the active profile invalidates in-flight intents
- a coordinator must not combine attestations from different authorization
  snapshots, signer-set versions, policy digests, or intent hashes
- an executor must re-check intent status and profile-defined cancellation rules
  immediately before broadcast

Profiles must state whether an accepted pause or revoke cancels an in-flight
intent that already collected some valid attestations. If a profile does not
state this, fail closed and do not broadcast.

## 9. Deterministic Limits and Concurrency

Per-window caps and cooldowns are unsafe if each signer evaluates only the
current intent in isolation. Two concurrent intents can each appear valid while
their combined amount exceeds the policy limit.

A conforming profile should define one deterministic reservation or ordering
rule, such as:

- accepted intent sequence numbers
- a per-fund nonce chain
- serialized accepted reservations
- another conflict rule that makes cumulative-budget consumption replayable

Each signer runtime must evaluate the same reserved budget state and bind its
attestation to that state. Coordinators must not reuse attestations across
competing intents or count the same signer twice for one threshold.

## 10. Recommended Records

All authority-bearing records should follow the signing mechanism defined by
the active app profile and `DESIGN-NOTES.app-signing-model.*`. In the default
role split, the signer principal signs enrollment and consent scope, governance
signs signer-set and policy state, and the signer runtime signs attestations.

### 10.1 `signer_enrollment`

Suggested fields:

- `enrollment_id`
- `profile_id` and `profile_version`
- `signer_id`
- `consent_key_ref`
- `signing_key_or_share_commitment`
- `proof_of_possession_ref`
- `fund_id`
- `signer_set_id` and signer-set version
- `status`
- `joined_at`
- signer-principal signature or authorization reference

### 10.2 `consent_scope`

Suggested fields:

- `consent_scope_id` and version
- `enrollment_id`
- `profile_id` and `profile_version`
- `fund_id`
- `signer_set_id` and signer-set version
- `policy_id`, policy version, and policy digest
- `max_amount_per_execution`
- cumulative amount and time-window definition
- `allowed_trigger_types`
- `allowed_assets`
- destination-allowlist digest or immutable reference
- cooldown and timelock requirements
- disclosure or blind-signing mode
- `effective_from` and `effective_until`
- consent nonce
- signer-principal signature or authorization reference

### 10.3 `pause_or_revoke_record`

Suggested fields:

- `control_id`
- target enrollment, consent-scope, policy, or signer-set reference
- `action`: `pause`, `resume`, or `revoke`
- reason code
- effective accepted-state reference
- `created_at`
- authorized signer or governance signature

### 10.4 `signer_attestation`

Suggested fields:

- `attestation_id`
- `intent_id` and `intent_hash`
- `signer_id`
- `enrollment_id`
- `consent_scope_id` and version
- `policy_id`, policy version, and policy digest
- `signer_set_id` and signer-set version
- accepted authorization-snapshot reference
- reservation, sequence, nonce, or idempotency reference
- `outcome` and reason code
- signing-result digest or commitment when `outcome` is `signed`
- `created_at`
- signer-runtime signature

Typical `outcome` values:

- `signed`
- `rejected`
- `skipped-paused`
- `skipped-revoked`
- `skipped-expired`
- `skipped-policy-mismatch`
- `skipped-stale-state`
- `skipped-budget-conflict`

The audit record should normally contain a digest or commitment to sensitive
signing material, not reusable secret shares.

### 10.5 Consent Evidence

Deployment-specific evidence may record how the signer reviewed and accepted
the scope.

Suggested fields:

- `evidence_id`
- `enrollment_id`
- `consent_scope_id` and canonical digest
- rendering or disclosure version shown to the signer
- `accepted_at`
- source or ceremony reference
- optional witness or device-attestation reference

Consent evidence supplements the signed consent object; it does not replace
cryptographic signer authorization. Sensitive ceremony evidence should use
access control, minimization, or sealed references rather than being replicated
publicly by default.

## 11. Client Responsibilities

A conforming signer client should let the signer inspect:

- enrollment identity and enrolled key or share commitment
- active signer-set and policy versions and digests
- human-readable consent scope tied to the canonical signed digest
- current operational state and its accepted-state source
- recent successful, blocked, and stale-state attestations
- local emergency-pause status and accepted pause or revoke status
- pending rotations, expiries, and changes that require re-consent

The client should provide explicit enroll, consent, pause, resume, revoke, and
rotation actions as permitted by the profile.

It should not present:

- hidden or implicit automatic signing
- ambiguous or unlimited policy scope
- an operator-controlled toggle as signer-principal consent
- a policy label without its version and digest
- a narrowed policy as permission to restore a previous wider scope later
- a false impression that automation removed the need for consent

## 12. Signer-runtime Responsibilities

Before producing a signing result, the signer runtime should:

1. verify accepted enrollment and proof of possession
2. verify the signer-principal authorization on the exact consent-scope digest
3. verify policy and signer-set identities, versions, and digests
4. verify pause, revoke, expiry, rotation, and supersession state
5. verify the complete intent and `intent_hash`
6. evaluate amount, cumulative budget, cooldown, destination, asset, time, and
   disclosure constraints against the deterministic accepted snapshot
7. verify nonce, idempotency, and reservation state
8. emit a signed attestation for either success or refusal

The runtime must not:

- widen policy locally
- accept unsigned coordinator summaries as authority
- silently ignore consent-state changes
- keep signing after loss of accepted-state freshness
- release the same share for a different intent hash or authorization snapshot
- expose reusable secret-share material in audit records

## 13. Failure Cases

### 13.1 Signer Never Knowingly Enrolled

Do not treat governance inclusion or operator configuration as consent. Disable
future use, preserve the disputed history, and begin compromise review if the
key or share was used.

### 13.2 Policy Scope Changed Without Re-consent

Reject signing under the new digest. Preserve a policy-mismatch outcome and show
the signer which field changed.

### 13.3 Pause or Revoke Was Not Observed

Mark the event as a stale-state or synchronization failure. Do not silently
continue or retroactively describe the signature as valid consent.

### 13.4 Rotated Signer Still Signs Under Old Scope

Reject the attestation for future execution, preserve it as evidence, and
investigate stale runtime state or key compromise.

### 13.5 Concurrent Intents Exceed a Cumulative Limit

Apply the deterministic reservation or ordering rule. At most the intents that
fit the accepted budget may proceed; the rest receive explicit budget-conflict
outcomes.

### 13.6 Coordinator Replays or Mixes Attestations

Reject any threshold set whose attestations disagree on intent hash,
authorization snapshot, consent scope, policy digest, signer-set version, or
nonce. Count each eligible signer at most once.

### 13.7 Consent Key or Signing Share Is Compromised

Pause locally, publish the appropriate revoke or rotation records, stop new
signing, and preserve evidence. A new key or share requires new enrollment and
new consent where the bindings change.

## 14. Minimal First-client Rules

A minimal interoperable client should support:

1. signer-authorized enrollment with proof of possession
2. signer-authorized consent bound to one profile, fund, policy digest, and
   signer-set version
3. explicit per-intent amount, cumulative amount, rate, trigger, asset,
   destination, and time bounds
4. visible `pending`, `active`, `paused`, `revoked`, `expired`, and `superseded`
   state
5. local emergency pause plus accepted pause, resume, revoke, and rotation
   records
6. fail-closed behavior when accepted state or trustworthy time is unavailable
7. signer attestations bound to the exact intent and authorization snapshot
8. deterministic nonce, idempotency, and cumulative-budget handling
9. visible successful and blocked attestation history
10. no implicit enrollment, policy widening, or consent inheritance

## 15. Profile Decisions That Must Be Explicit

Each concrete profile should decide:

- whether signer-local pause or revoke, governance controls, or both are valid
- when accepted controls become effective and how clock skew is handled
- whether pause or revoke cancels in-flight intents and already collected
  attestations
- which policy changes are provably narrowing and may avoid re-consent
- whether a signer may hold multiple concurrent consent scopes for one fund
- how cumulative limits are serialized across concurrent intents
- how consent evidence is stored without exposing sensitive signer information
- whether per-intent human approval is required above a risk threshold
- how recovery works when the consent key, signing share, runtime, or signer
  principal becomes unavailable

If a profile leaves one of these choices unspecified, implementations should
choose the fail-closed interpretation rather than silently expanding consent.
