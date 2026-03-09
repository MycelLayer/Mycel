# Mycel Signer Activity Model

Status: design draft

This note describes how a Mycel-based custody deployment should evaluate signer activity rather than treating signer availability as a binary guess.

The main design principle is:

- a signer may be validly enrolled but still operationally unavailable
- a signer may be reachable but still unsafe to count as fully active
- signer activity should be measured explicitly, not inferred from casual communication
- effective custody safety depends on the currently active signer set, not only the configured signer set

## 0. Goal

Enable a Mycel-based m-of-n custody system to distinguish:

- who is enrolled
- who is currently allowed to sign
- who is operationally able to sign
- who is still safe to count toward practical availability

This note complements consent and custody-policy notes.

It does not redefine:

- signer consent
- policy scope
- execution authorization

## 1. Why Activity Must Be Modeled Separately

A signer can fail in ways that do not fit simple `active / revoked` state:

- the signer still consents, but has lost device access
- the signer is reachable, but misses signing windows
- the signer key material exists, but recovery procedures no longer work
- the signer is still enrolled, but has become operationally dependent on another signer

If a system ignores these differences, a nominal `3-of-7` set may behave like a fragile `3-of-4` set in practice.

## 2. Activity Dimensions

Signer activity should be evaluated across at least four dimensions.

### 2.1 Technical Readiness

Can the signer still complete the technical path required for safe signing?

Examples:

- signed heartbeat succeeds
- challenge-response succeeds
- signer node, HSM, or MPC endpoint is reachable
- credential and device state are current

### 2.2 Operational Responsiveness

Can the signer respond within expected operational windows?

Examples:

- responds within SLA for critical contact
- participates in drills and rotations
- does not repeatedly miss execution windows
- can still use the approved communication path

### 2.3 Governance Participation

Is the signer still part of the accepted operating structure?

Examples:

- still belongs to the accepted signer set
- still accepts the active policy scope
- has not exited the role
- understands current pause, revoke, and rotation state

### 2.4 Security Hygiene and Independence

Is the signer still independently and safely controlled?

Examples:

- no evidence of uncontrolled delegation
- no unsafe device sharing
- no obvious account or credential compromise
- no hidden dependency that collapses signer independence

## 3. Activity States

This note recommends at least these operational states:

- `active`
- `degraded`
- `inactive`
- `retired`
- `revoked`

These should not be treated as synonyms for consent state.

### 3.1 Active

The signer:

- passes recent technical checks
- responds within expected operational windows
- remains inside accepted governance scope
- shows acceptable security hygiene

### 3.2 Degraded

The signer is still part of the system, but reliability has weakened.

Typical causes:

- repeated late responses
- missed drills
- unstable runtime endpoint
- incomplete recovery path
- increased operational dependency on others

`degraded` signers should not be ignored.

They are early warning indicators that the effective signer set may be shrinking.

### 3.3 Inactive

The signer is no longer practically available for dependable participation.

Typical causes:

- repeated failed heartbeat or challenge checks
- no reliable contact path
- long-term device loss
- inability to complete required recovery procedure

`inactive` signers should not be counted toward practical signing capacity.

### 3.4 Retired

The signer has exited the role in an orderly way.

The signer should remain in history, but future activity should not be expected.

### 3.5 Revoked

The signer has been removed from valid future participation.

Revocation is a governance and eligibility event, not merely an activity observation.

## 4. Minimum Evidence for Activity Assessment

Activity status should be based on explicit evidence rather than operator intuition.

Recommended evidence classes:

- recent signed heartbeat
- recent challenge-response result
- recent successful drill participation
- recent verified contact-path confirmation
- recent rotation or policy acknowledgment
- recent security-hygiene review

## 5. Suggested Evaluation Rules

Exact thresholds depend on deployment size and risk, but a practical baseline may include:

- at least one signed heartbeat within 30 days
- at least one recovery or signing drill within 90 days
- successful response to critical contact within 7 days
- downgrade to `degraded` after repeated missed checks
- downgrade to `inactive` after sustained inability to verify readiness

The system should preserve the underlying evidence, not only the resulting label.

## 6. Effective Signer Set

Custody safety should be evaluated against both:

- configured signer set
- currently active signer set

Example:

- configured set: `3-of-7`
- currently active signers: `4`

This may still satisfy policy on paper, but it behaves much closer to `3-of-4` in operational risk.

A conforming deployment should surface that difference explicitly.

## 7. Required Client and Runtime Behavior

### 7.1 Client

A conforming client should show:

- configured signer count
- currently `active`, `degraded`, `inactive`, `retired`, and `revoked` counts
- last successful verification time per signer
- last drill or recovery-check time
- whether rotation is recommended

The client should not imply that enrollment alone equals practical readiness.

### 7.2 Runtime

A conforming runtime should:

- record explicit activity evidence
- distinguish failed technical checks from governance revocation
- emit warnings before a signer becomes operationally unavailable
- refuse to treat stale activity data as current

## 8. Failure Cases

### 8.1 Silent inactivity

A signer stops participating, but no explicit checks exist.

Result:

- the system overestimates real signing capacity

### 8.2 Enrollment mistaken for readiness

A signer remains enrolled, but the device path is broken.

Result:

- custody planners misread actual resilience

### 8.3 Correlated degradation

Multiple signers appear separately enrolled but share one hidden dependency.

Result:

- activity labels overstate independence

### 8.4 No rotation after inactivity

Inactive signers remain in the set too long.

Result:

- nominal redundancy decays into operational fragility

## 9. Recommended Controls

- require periodic signed heartbeat
- require periodic challenge-response checks
- require periodic recovery drills
- record signer-activity evidence as auditable events
- track effective signer capacity separately from configured capacity
- trigger review or rotation when degraded or inactive counts cross thresholds

## 10. Practical Rule

For custody safety, the important question is not:

- "Is this signer still enrolled?"

It is:

- "Can this signer still participate safely, independently, and on time?"

If a deployment cannot answer that with evidence, it does not actually know its current m-of-n resilience.
