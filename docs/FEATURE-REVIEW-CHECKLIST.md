# Mycel Feature Review Checklist

Status: working checklist

This document is a recurring review list for any proposed new Mycel feature.

Use it when we are deciding whether a feature belongs in:

- core protocol
- wire protocol
- profile layer
- app layer
- local tooling only

It is meant to keep feature growth conservative, auditable, and privacy-aware.

## 1. Fast Gate

Before spending much implementation effort, ask:

1. Does this feature clearly solve a real current problem?
2. Does it belong in the current milestone?
3. Can it stay out of core for now?
4. Does it preserve deterministic verification?
5. Does it weaken privacy or anonymity?

If the answer is unclear, slow down and narrow the feature first.

## 2. Core Review Concepts

Check these concepts every time.

### 2.1 Anonymity and Privacy

- Does this feature expose stable node identity more often?
- Does it make author, reader, maintainer, or relay correlation easier?
- Does it turn one-hop transport metadata into long-lived provenance?
- Does it add source-tracing that should remain optional?

Default bias:

- keep the core neutral
- avoid permanent origin-tracking in protocol objects
- prefer bounded, local, or opt-in audit surfaces

### 2.2 Layer Placement

Ask where this feature truly belongs:

- `core protocol` for interoperable signed object rules
- `wire protocol` for transport/session interoperability
- `profile layer` for deployment-specific policy
- `app layer` for domain semantics
- `local tooling` for operator convenience only

Default bias:

- if it is not required for cross-implementation interoperability, do not put it in core

### 2.3 Signed Truth vs Derived State

- Which fields must be signed?
- Which values must be recomputable?
- Which summaries are only caches or indexes?
- Could unsigned helper data accidentally outrank canonical signed objects?

Default bias:

- signed source objects first
- derived views, indexes, and reports second

### 2.4 Determinism

- Will different nodes compute the same IDs, signatures, replay results, and selected outputs?
- Does canonicalization remain stable?
- Does the feature depend on local clocks, ordering accidents, or environment differences?

Default bias:

- same input should produce the same verification result everywhere

### 2.5 Governance Boundary

- Does this feature affect accepted-head selection?
- Does it introduce a new governance signal?
- Does it let non-governance data influence view-maintainer authority?
- Does it blur the boundary between editor activity and selector weight?

Default bias:

- keep governance inputs narrow and explicit

### 2.6 Partial Replication

- Can a node still function without the full global dataset?
- Does the feature force all nodes to store all objects?
- Can missing data fail clearly instead of silently corrupting behavior?

Default bias:

- preserve bounded sync and partial replication

### 2.7 Auditability vs Traceability

- Can we audit correctness without permanently identifying who first relayed or hosted something?
- Is the feature adding verifiable history, or only more traceability?
- Could the same need be met with local logs or an opt-in profile?

Default bias:

- prefer auditable state over globally traceable origin trails

### 2.8 Failure Model

- What happens if data is missing?
- What happens if data is malformed?
- What happens if a dependency is unsigned, unverifiable, or unparseable?
- What happens if a peer or node lies?

Default bias:

- specify negative cases early
- add fixture-backed and CLI-visible failures where useful

### 2.9 User-Facing Surface Parity

- Will parser, verify, inspect, store, and reader surfaces stay aligned?
- If validation becomes stricter, will user-facing outputs explain it clearly?
- Are new failures exposed in tests at the CLI level?

Default bias:

- do not fix only the internal layer and leave user-facing surfaces behind

### 2.10 Scope Discipline

- Is this feature minimal?
- Does it quietly include a second problem?
- Can we ship the narrow version first?
- Are non-goals written down?

Default bias:

- narrow first, widen later

## 3. Repeated Review Questions

When we revisit a feature, ask these six questions again:

1. Does this weaken anonymity or privacy?
2. Is this in the right layer?
3. What is signed truth, and what is only derived?
4. Will all nodes still reach the same deterministic result?
5. Does this accidentally change governance behavior?
6. What is the failure mode under missing, malformed, or hostile input?

## 4. Decision Heuristics

Use these default heuristics unless there is a strong reason not to.

- Put domain-specific meaning into profiles or app layers.
- Keep core object rules small and stable.
- Keep wire focused on session interoperability, not social truth.
- Prefer local logs over protocol-wide provenance when privacy matters.
- Prefer explicit negative tests over informal assumptions.
- Prefer one narrow feature batch over one broad feature bundle.

## 5. Minimum Feature Write-Up

Before implementation, a new feature proposal should usually answer:

- Problem:
- Why now:
- Intended layer:
- Signed fields:
- Derived fields:
- Privacy impact:
- Governance impact:
- Failure modes:
- Verification plan:
- Non-goals:

## 6. Relation to Other Surfaces

Use this checklist together with:

- [ROADMAP.md](../ROADMAP.md)
- [IMPLEMENTATION-CHECKLIST.en.md](../IMPLEMENTATION-CHECKLIST.en.md)
- [PROTOCOL.en.md](../PROTOCOL.en.md)
- [WIRE-PROTOCOL.en.md](../WIRE-PROTOCOL.en.md)
- [AI-CO-WORKING-MODEL.md](./AI-CO-WORKING-MODEL.md)

If those surfaces disagree, follow the current planning-sync process in [PLANNING-SYNC-PLAN.md](./PLANNING-SYNC-PLAN.md).
