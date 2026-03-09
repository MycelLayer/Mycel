# Mycel Roadmap

Status: draft

This roadmap turns the current README priorities, implementation checklist, and design-note planning guidance into one repo-level build sequence.

It is intentionally narrow:

- build the first interoperable client first
- keep protocol-core changes conservative
- move mature ideas into profiles, schemas, and tests before expanding scope

## Current Position

The repository already has:

- a growing v0.1 protocol and wire-spec document set
- a Rust CLI suitable for internal validation and deterministic simulator workflows
- early `mycel-core` support for object schema metadata, object-envelope parsing, object inspection, object verification, and accepted-head inspection
- simulator fixtures, topologies, tests, and reports for regression coverage

The repository does not yet have:

- a complete interoperable node implementation
- a finished object-authoring and storage-write path
- end-to-end wire sync
- a production-ready public CLI or app

## Planning Levels

The roadmap follows the planning split already suggested in the design notes:

1. `minimal`
2. `reader-plus-governance`
3. `full-stack`

Each later phase assumes the earlier one is already stable.

## Phase 1: Minimal

Goal: reach a narrow first client that can parse, verify, store, replay, and inspect Mycel objects deterministically.

### Deliverables

1. Shared protocol object model for all v0.1 object families
2. Canonical serialization, derived ID recomputation, and signature verification
3. Replay-based revision verification and `state_hash` checking
4. Local object store and rebuildable indexes
5. Stable internal CLI/API for validation, object verification, object inspection, and accepted-head inspection
6. Interop fixtures and negative tests for object and simulator validation

### Exit Criteria

1. Required object types parse and validate reproducibly
2. Canonical IDs and signatures are deterministic
3. Revision replay passes on stored objects alone
4. Accepted-head selection is deterministic for fixed profiles
5. The local store can be rebuilt from canonical objects alone

### Current Status

Partially underway.

Already in progress or partially implemented:

1. Shared object schema metadata
2. Shared object-envelope parsing
3. Object inspection and verification
4. Accepted-head inspection
5. Internal validation and simulator harness CLI

Still missing or incomplete:

1. Full typed object model across all object families
2. Canonical serialization as a fully shared protocol layer
3. Revision replay engine and complete `state_hash` verification
4. Storage-write and object-authoring path
5. Formal store-rebuild workflow

## Phase 2: Reader-Plus-Governance

Goal: add a usable reader-oriented client layer with deterministic accepted-head behavior and governance-aware reading state.

### Deliverables

1. Verified View ingestion as governance signal input
2. Stable accepted-head selection for fixed reader profiles
3. Reader-first text rendering from replayed revision state
4. Clear separation between reader workflows and governance publication workflows
5. CLI/API support for inspecting accepted heads, views, and governance decision detail

### Exit Criteria

1. A fixed reader profile produces stable accepted heads across repeated runs
2. Governance inputs are separated from discretionary local policy
3. A reader can reconstruct and inspect accepted text state from stored objects
4. Decision summaries and typed arrays are stable enough for tooling and tests

### Current Status

Early partial progress.

Already in progress or partially implemented:

1. Accepted-head inspection
2. Structured decision output with typed machine-readable arrays
3. Early simulator workflows around peer and topology validation

Still missing or incomplete:

1. Full reader rendering path
2. View publication workflow
3. Stable reader-facing profile selection surface
4. Complete storage and retrieval path for governance inputs

## Phase 3: Full-Stack

Goal: extend from local verification and governed reading into interoperable replication, richer profiles, and selective app-layer support.

### Deliverables

1. Canonical wire envelope implementation
2. `HELLO`, `MANIFEST`, `HEADS`, `WANT`, `OBJECT`, `BYE`, and `ERROR`
3. Optional `SNAPSHOT_OFFER` and `VIEW_ANNOUNCE` for supported profiles
4. End-to-end sync workflow between peers
5. Merge-generation profile support for local authoring tools
6. Selective app-layer profiles on top of a stable protocol core

### Exit Criteria

1. Minimal sync succeeds end-to-end between peers
2. Received objects are verified before indexing and exposure
3. Merge generation can emit replayable patch operations
4. Profile-specific extensions remain outside the protocol core unless clearly justified

### Current Status

Mostly not started.

Already in progress or partially implemented:

1. Simulator topology and report scaffolding
2. CLI workflows for report inspection, listing, stats, and diffing

Still missing or incomplete:

1. Real wire implementation
2. Object fetch and sync state machine
3. Snapshot-assisted catch-up
4. Production replication behavior
5. App-layer runtime support

## Cross-Cutting Priorities

These priorities apply across all phases:

1. Keep the first client deliberately narrow
2. Prefer profiles and schemas over frequent protocol-core expansion
3. Keep machine-readable CLI output stable where tests rely on it
4. Add regression coverage whenever a new protocol rule or CLI contract is introduced
5. Preserve the separation between protocol state, governance state, and local discretionary policy

## Near-Term Work

The highest-value near-term work is:

1. finish the Phase 1 core object and storage path
2. complete replay and `state_hash` verification
3. keep strengthening interop fixtures and negative tests
4. turn mature governance behavior into fixed reader-profile workflows

## Not Yet the Target

The roadmap does not currently treat these as near-term targets:

1. rich editor UX
2. production network deployment
3. generalized app runtime
4. broad plugin systems
5. rapid protocol-core expansion driven by speculative design notes
