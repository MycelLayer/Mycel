# Mycel Protocol Upgrade Philosophy

Status: design draft

This note describes how Mycel should evolve its protocol, profiles, and design notes without turning the core specification into a fast-moving target.

The main design principle is:

- the protocol core should evolve conservatively
- profiles should carry narrower deployment and app-level commitments
- design notes should absorb exploration before promotion
- new capability should prefer outer layers before core-layer change

## 0. Goal

Enable Mycel to grow without making independent implementations drift, without invalidating existing object history, and without forcing every experimental idea into the protocol core.

This note defines:

- the preferred upgrade posture for the protocol core
- the role of profiles
- the role of design notes
- the promotion path between layers
- compatibility expectations for each layer

## 1. Layered Upgrade Model

Mycel should evolve across three layers.

### 1.1 Protocol

The protocol layer defines rules that independent implementations must share.

Examples:

- object model
- canonical serialization
- hashing and derived IDs
- signature rules
- revision replay
- wire envelope and required message semantics

Changes at this layer should be rare and conservative.

### 1.2 Profile

A profile defines a narrower, versioned operating subset built on top of the protocol.

Examples:

- a Tor-oriented deployment profile
- a strict Q&A profile
- a fund auto-disbursement profile

Profiles should lock concrete parameters and behaviors without redefining protocol validity.

### 1.3 Design Note

A design note is the exploration layer.

It should be used for:

- candidate models
- unresolved tradeoffs
- app-layer experiments
- deployment ideas not yet ready for strict conformance

Design notes are intentionally easier to change than profiles or protocol text.

## 2. Core Stability Rule

Mycel should prefer a stable protocol core.

This means:

- avoid frequent changes to object validity rules
- avoid changing canonicalization rules once adopted
- avoid redefining existing wire message meaning unless necessary
- avoid requiring old valid history to become invalid

The protocol should move slowly because interoperability depends on shared invariants, not just shared intent.

## 3. Profile-first Expansion

New capability should generally enter Mycel as a profile or design note before it enters the core protocol.

Recommended order:

1. write the idea as a design note
2. narrow it into a versioned profile
3. observe whether the profile is stable and useful
4. only then consider whether any part belongs in the core protocol

This keeps experimentation away from the most expensive layer.

## 4. Promotion Rules

### 4.1 Design Note -> Profile

Promotion is appropriate when:

- the model has a clear scope
- required records and behaviors are known
- major tradeoffs are already documented
- a first implementation target is realistic

### 4.2 Profile -> Protocol

Promotion is appropriate when:

- multiple profiles depend on the same shared rule
- independent implementations need the same invariant to interoperate
- the rule is no longer app-specific or deployment-specific
- the cost of leaving it outside the protocol is higher than the cost of freezing it

The default answer should still be:

- keep it outside the protocol unless there is a clear interoperability reason

## 5. Compatibility by Layer

Compatibility expectations should differ by layer.

### 5.1 Protocol Compatibility

The protocol should prefer historical validity preservation.

Recommended posture:

- once an object is valid under an adopted protocol version, later editorial cleanup should not silently invalidate it
- future protocol revisions should prefer additive clarification over incompatible reinterpretation

This is the strictest compatibility zone in Mycel.

### 5.2 Profile Compatibility

Profiles do not need strong backward compatibility by default.

Recommended posture:

- profiles should be explicitly versioned
- a new profile version may supersede an older version
- deployments may choose to support only one profile version at a time

Profiles should be stable enough to implement, but flexible enough to replace.

### 5.3 Design Note Compatibility

Design notes should assume low compatibility guarantees.

They are for structured exploration, not deployment promises.

## 6. Versioning Posture

Mycel should prefer:

- slow protocol version movement
- explicit profile versioning
- fast design-note iteration

This implies:

- protocol versions should be rare and meaningful
- profile versions may be more numerous and use-case-specific
- design notes may change freely as long as they remain clearly non-normative

## 7. Change Placement Checklist

When deciding where a change belongs, ask:

1. does this affect object validity or wire interoperability?
2. do independent implementations need this exact rule to agree?
3. is the rule app-specific, deployment-specific, or governance-specific?
4. is the model already proven enough to freeze?

Recommended outcomes:

- if the answer is mostly interoperability and invariants, it may belong in protocol
- if the answer is mostly deployment or app constraints, it likely belongs in a profile
- if the answer is still exploratory, it belongs in a design note

## 8. Deprecation and Replacement

Mycel should prefer replacement over silent mutation.

Recommended rule:

- do not silently rewrite an old design note into a different model without marking the change
- do not silently change a profile's meaning without a new profile version
- do not reinterpret protocol validity rules without explicit versioning and migration discussion

This keeps the document set auditable.

## 9. First-client Guidance

For a first client, the safest implementation target is:

- protocol core as written
- a narrow set of explicit profiles
- no dependency on unresolved design notes

This reduces implementation drift and gives the repo a clearer stabilization path.

## 10. Recommended Mycel Upgrade Posture

The recommended posture for Mycel is:

- conservative protocol core
- versioned profile evolution
- active design-note experimentation

In short:

- protocol should behave like the stable base
- profiles should carry practical commitments
- design notes should absorb ongoing invention

This is the cleanest way for Mycel to remain evolvable without becoming unstable.
