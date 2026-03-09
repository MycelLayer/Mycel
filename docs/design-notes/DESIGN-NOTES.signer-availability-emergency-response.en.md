# Mycel Signer-availability Emergency Response

Status: design draft

This note describes how a Mycel-based m-of-n custody deployment should detect dangerous signer-availability decline and trigger emergency rotation, address renewal, or other recovery paths before practical lockout occurs.

Related notes:

- `DESIGN-NOTES.policy-driven-threshold-custody.*` for the base m-of-n custody and rotation model
- `DESIGN-NOTES.auto-signer-consent-model.*` for signer eligibility and consent scope
- `DESIGN-NOTES.signer-activity-model.*` for how signer readiness and effective signer capacity are measured

The main design principle is:

- do not wait for complete signer failure before acting
- measure effective signer capacity continuously
- distinguish early warning from emergency lockout
- define recovery paths before they are needed

## 0. Goal

Enable a custody deployment to answer all of the following:

- how many signers are configured
- how many signers are effectively available now
- when the signer set has become operationally fragile
- what action should be triggered at each risk level

This note complements:

- signer activity evaluation
- signer consent modeling
- m-of-n custody rotation

## 1. Core Definitions

### 1.1 Configured Signer Set

The configured signer set is the nominal signer membership used by policy.

Example:

- configured set: `3-of-7`

### 1.2 Effective Signer Set

The effective signer set is the subset of signers that can still participate safely, independently, and on time.

This should be derived from explicit activity evidence, not from enrollment alone.

### 1.3 Lockout Risk

Lockout risk means the system is close to losing the practical ability to reach `m`.

### 1.4 Emergency Renewal

Emergency renewal means migrating future use away from the current custody target before continued decline causes practical loss of control.

Depending on the deployment, that may mean:

- signer-set rotation
- address rotation
- emergency move to a recovery address
- temporary execution freeze plus recovery workflow

## 2. Why Early Detection Matters

If a system waits until `effective_signers < m`, it may already be too late to move assets or renew the address under the normal path.

A practical system must therefore detect:

- declining signer availability
- shrinking safety margin above `m`
- correlated signer failure
- repeated degraded state that predicts future lockout

## 3. Required Inputs

Emergency response logic should use at least these inputs:

- configured `m` and `n`
- signer activity state per signer
- recent heartbeat results
- recent challenge-response results
- recent drill or recovery-check results
- contact-path success or failure
- evidence of correlated dependencies

## 4. Effective Signer Calculation

A deployment should calculate at least:

- `configured_signers`
- `active_signers`
- `degraded_signers`
- `inactive_signers`
- `revoked_or_retired_signers`
- `effective_signers`

Recommended practical rule:

- count `active` signers as effective
- count `degraded` signers separately and conservatively
- do not count `inactive`, `retired`, or `revoked` signers as effective

If a deployment chooses to count some `degraded` signers temporarily, that decision should be explicit and time-limited.

## 5. Risk States

This note recommends at least three escalating risk states:

- `warning`
- `critical`
- `emergency`

### 5.1 Warning

The signer set is still operable, but the safety margin is thinning.

Typical triggers:

- `effective_signers <= m + 1`
- repeated `degraded` outcomes
- repeated missed drills
- early evidence of correlated failure

### 5.2 Critical

The signer set is still technically operable, but one additional loss may cause lockout.

Typical triggers:

- `effective_signers = m`
- prolonged `warning` without recovery
- multiple key owners degraded at the same time

### 5.3 Emergency

The deployment has lost, or is at immediate risk of losing, practical signing capacity.

Typical triggers:

- `effective_signers < m`
- a common-mode failure removes multiple signers at once
- required recovery checks fail during a critical state

## 6. Recommended Actions by Risk State

### 6.1 Warning Actions

- increase verification frequency
- contact degraded signers
- run immediate recovery checks
- prepare signer rotation or address renewal
- restrict non-essential execution

### 6.2 Critical Actions

- freeze non-essential spending
- prioritize controlled migration to a new signer set or address
- activate higher-scrutiny review
- shorten monitoring intervals
- prepare emergency-only execution path if one exists

### 6.3 Emergency Actions

- invoke pre-authorized recovery path
- if `m` is still barely reachable, allow only migration to a recovery target
- if `m` is no longer reachable, move to governance, legal, or organizational recovery process
- preserve full incident evidence

## 7. Address Renewal and Other Recovery Paths

Emergency response should not assume one single mechanism.

Practical options include:

### 7.1 Normal Rotation Before Lockout

If `m` is still reachable, rotate early.

Preferred actions:

- create a new signer set
- create a new address or settlement target
- migrate funds before capacity falls below `m`

### 7.2 Pre-authorized Emergency Recovery Path

A deployment may define a special path that is narrower than normal execution.

Examples:

- move only to a fixed recovery address
- allow only one asset class
- impose stricter review or timelock

This path should never become a hidden bypass for ordinary spending.

### 7.3 Governance or Organizational Recovery

If practical signing capacity is already below `m`, normal cryptographic control may no longer be sufficient.

At that point, the system may need:

- governance escalation
- legal recovery process
- organizational reconstitution
- acceptance that the old target cannot be used further

## 8. Common Failure Cases

### 8.1 Waiting Too Long

The system detects degradation but takes no action until `effective_signers < m`.

Result:

- normal renewal path is lost

### 8.2 Counting Degraded Signers Optimistically

Operators keep assuming degraded signers are effectively available.

Result:

- the real safety margin is overstated

### 8.3 No Defined Emergency Path

The deployment notices a crisis but has no pre-agreed response.

Result:

- delay, panic, and inconsistent action

### 8.4 Common-mode Failure Ignored

Multiple signers fail through one shared dependency.

Result:

- configured redundancy was illusory

## 9. Required Controls

- define risk-state thresholds in advance
- compute effective signer capacity regularly
- trigger explicit review at `warning`
- require action plan at `critical`
- preserve sealed evidence at `emergency`
- keep emergency paths narrower than normal execution paths
- link signer rotation, address renewal, and policy updates explicitly

## 10. Practical Rule

The right time to renew an address or rotate a signer set is usually not after lockout.

It is when the system can still honestly say:

- "we still have enough signers to move safely, but we may not for much longer"

If a deployment lacks that decision point, it is operating too close to irreversible failure.
