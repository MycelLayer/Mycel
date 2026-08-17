# Mycel Progress View

Status: draft, refreshed for a planning-sync pass so the summary now reflects `M3` / `M4` as the active planning lane; `M2` replay/storage/rebuild closure remains landed at the current narrow scope, and the next `M3` work now centers on broader governance persistence, richer governance tooling, and reader profile ergonomics beyond the current relationship/current-governance summaries, store-index governance source/context summaries, deduplicated policy catalog, point-to-point View diff, and copyable profile-reuse baseline, while `M4` now also records advertised Snapshot/View metadata binding and document-scoped `HEADS replace=true` behavior on top of the existing production replication, optional-message, sequencing, reachability, sender-validation, and negative/warning proof set; broader `M3` governance work and broader `M4` session/capability/error-path interop remain open, and the current open task queue still maps cleanly to those gaps

This page turns [`ROADMAP.md`](../ROADMAP.md) and [`IMPLEMENTATION-CHECKLIST.en.md`](../IMPLEMENTATION-CHECKLIST.en.md) into one quick progress view.

## Current Lane

The current build lane is:

1. keep `M2` closed at the current narrow replay/storage/rebuild scope while later work grows around it
2. expand `M3` reader-plus-governance workflows carefully on top of the now-usable accepted-head inspection/render base, clearer profile discovery/error feedback and copyable reuse hints, editor-admission-aware flows, bounded viewer score surfaces, governance inspect/list/publish/current surfaces, persisted relationship/current-governance summaries, store-index governance source/context summaries, a deduplicated policy catalog, and point-to-point View diff, while keeping broader persistence, richer history/range and policy-aware tooling, and further profile ergonomics explicit
3. keep `M4` narrow while peer-store sync proof grows toward the remaining session/capability/error-path interop closure now that the current production replication sub-items, capability-gated optional-message rejection, broader sequencing/root gating, advertised Snapshot/View metadata binding, document-scoped `HEADS replace=true`, reachability and sender-validation faults, and messages-after-BYE plus missing-BYE warning proof all exist

## Milestone Timeline

```mermaid
flowchart LR
  subgraph Minimal
    M1["M1<br/>Core Object and Validation Base<br/>Closed gate"]
    M2["M2<br/>Replay, Storage, and Rebuild<br/>Closed at current narrow scope"]
  end

  subgraph ReaderPlusGovernance["Reader-plus-governance"]
    M3["M3<br/>Reader and Governance Surface<br/>Early partial"]
  end

  subgraph FullStack["Full-stack"]
    M4["M4<br/>Wire Sync and Peer Interop<br/>Early partial"]
    M5["M5<br/>Selective App-Layer Expansion<br/>Later"]
  end

  M1 --> M2 --> M3 --> M4 --> M5
```

## Milestone Snapshot

| Milestone | Status | Main focus now | Main gaps |
|---|---|---|---|
| `M1` | Closed gate | minimal-client proof retained as a completed checklist section | no longer the active lane; follow-up work moved into `M2` / `M3` / `M4` tracking |
| `M2` | Closed at current narrow scope | replay, `state_hash`, store rebuild, ancestry-aware render/store verification, explicit CLI proof that multi-document indexes rebuild cleanly after index loss from stored canonical objects, narrow write path, conservative merge authoring with broader structural coverage plus manual-curation smoke for nested parent-choice, nested sibling-choice, and composed-branch placement conflicts, richer direct and anchor-based competing placement reasons, richer mixed content/metadata competing-branch classification with matching CLI smoke coverage, and rebuild-after-index-loss proof for the richer metadata multi-variant merge case | no remaining narrow-scope closure gaps; future follow-up can stay outside active `M2` debt |
| `M3` | Early partial | accepted-head reader workflows, bundle/store rendering, named fixed-profile reading with clearer discovery/error feedback and copyable reuse hints, editor-admission-aware inspect/render flows, distinct human/debug head text modes, bounded viewer score surfaces, filtered/sorted/projected `view` governance inspect/list/publish/current workflows, persisted reverse indexes, relationship/current-governance summaries, store-index governance source/context summaries, a deduplicated policy catalog, and point-to-point View diff | broader governance persistence beyond the current catalog and summaries, richer history/range and policy-aware governance tooling, and reader profile ergonomics beyond this initial polish |
| `M4` | Early partial | wire envelope and `OBJECT` body validation, session reachability, store-backed bootstrap, peer-store-driven first-time/incremental sync, capability-gated optional messages, localhost multi-process and production-replication proofs, broader sequencing/root gating, advertised Snapshot/View metadata binding, document-scoped `HEADS replace=true`, unadvertised `WANT` and unrequested `OBJECT` rejection, sender validation, explicit `ERROR` handling, reachability faults, and messages-after-BYE plus missing-BYE warning proof | remaining broader session/capability/error-path interop coverage such as other advertised-root/root-set violations, unaffected-document in-flight dependency provenance, and other post-`HELLO` protocol-state faults |
| `M5` | Later | selective app-layer growth | depends on stable protocol core and sync |

## Implementation Matrix

Legend:

- `Done`: current checklist section is substantially closed for the minimal client
- `Mostly done`: only closure or follow-up work remains
- `Partial`: meaningful implementation exists, but the section is not closeable yet
- `Not started`: still mostly future work

| Area | Status | Primary milestone | Current read |
|---|---|---|---|
| 1. Repo and Build Setup | Done | `M1` | this is now part of the closed minimal-client gate; no active follow-up remains here |
| 2. Object Types and IDs | Done | `M1` | the required v0.1 families and minimal-client role modeling are now retained as closed gate proof, not active checklist debt |
| 3. Canonical Serialization and Hashing | Done | `M1` | canonical rules and shared helper reuse needed for the minimal gate are closed; post-`M1` wire follow-up now belongs to the broader `M4` lane rather than this gate |
| 4. Signature Verification | Done | `M1` / `M4` | minimal object and wire signature verification are closed for the gate; broader interop/error-path follow-up remains in `M4` |
| 5. Patch and Revision Engine | Mostly done | `M2` | replay and `state_hash` are in place; dependency verification, wrong-type and multi-hop ancestry proofs, sibling declared-ID determinism, and render-path ancestry context are stronger |
| 6. Local State and Storage | Mostly done | `M2` | store ingest, rebuild, indexes, and explicit CLI proof that multi-document indexes recover after index loss from stored canonical objects all exist; local transport/safety policy now persists in a separate local policy file while rebuild smoke preserves both replicated indexes and local policy state |
| 7. Wire Protocol | Partial | `M4` | canonical wire-envelope parsing, field validation, RFC 3339 checks, sender checks, session sequencing/head-tracking, reachability gating, store-backed bootstrap, `OBJECT` body verification, capability-gated optional-message handling, broader pre-root gating, advertised Snapshot/View metadata binding, document-scoped `HEADS replace=true`, stale and unadvertised `WANT` rejection, unrequested `OBJECT` rejection, sender-validation faults, explicit `ERROR` handling, messages-after-BYE rejection, missing-BYE warning proof, and a minimal peer-store sync driver now exist in `mycel-core`; the main remaining interop work is broader session/capability/error-path proof |
| 8. Sync Workflow | Partial | `M4` | peer-store-driven first-time and incremental sync prove shared verify/store flows through `mycel-core`, the CLI, and simulator coverage, including snapshot-assisted catch-up, announced-view fetching, localhost multi-process transport, production-replication proofs, missing-capability rejection, broader sequencing/root gating, advertised Snapshot/View metadata binding before storage, document-scoped `HEADS replace=true`, stale/unadvertised `WANT` and unrequested `OBJECT` rejection, sender-validation and reachability faults, explicit `ERROR` handling, and messages-after-BYE plus missing-BYE warning handling; remaining work is broader session/capability/error-path proof |
| 9. Views and Head Selection | Mostly done | `M3` | deterministic selector core, named fixed-profile selection with clearer discovery/error feedback and copyable reuse hints, separate editor/view admission-aware inspect/render flows, distinct human/debug head text modes, bounded viewer score channels, persisted relationship/current-governance summaries, store-index governance source/context summaries, a deduplicated policy catalog, point-to-point View diff, and direct selector regressions all exist; broader persistence beyond the current catalog and summaries, richer history/range and policy-aware tooling, and further reader profile ergonomics are the remaining `M3` gaps |
| 10. Merge Generation | Partial | `M2` | replay verification and a conservative local merge-authoring profile exist, including structural move/reorder, new-parent reparenting, simple composed parent-chain coverage, a broader nested structural matrix, manual-curation smoke for nested parent-choice, nested sibling-choice, and composed-branch placement conflicts, richer direct/anchor-based competing placement reasons, landed metadata competing-variant handling for adopting or keeping primary over non-primary metadata additions, richer mixed content/metadata competing-branch detail with matching CLI smoke coverage, and an explicit manual-curation boundary for metadata removal because v0.1 patch ops do not yet express deletion; the current narrow `M2` closure is now landed, so future merge-authoring expansion is no longer active `M2` debt |
| 11. CLI or API Surface | Partial | `M2` / `M3` / `M4` | verification and authoring, editor-admission-aware reader inspection/render, governance inspect/list/publish/current/diff with persisted policy and relationship summaries, transcript-backed `sync pull`, and internal `sync peer-store` all exist, including optional snapshot/view flows, localhost and production-replication proofs, broader sequencing/root gating, advertised Snapshot/View metadata binding, document-scoped `HEADS replace=true`, stale/unadvertised `WANT` and unrequested `OBJECT` rejection, sender-validation and reachability faults, explicit `ERROR` handling, and messages-after-BYE plus missing-BYE warning handling; the remaining `M4` gap is broader session/capability/error-path interop proof |
| 12. Interop Test Minimum | Partial | `M1` / `M2` / `M4` | fixture isolation, reproducibility, parser/replay smoke coverage, direct wire-envelope/signature/session tests, peer-store first-time/incremental and production-replication proofs, optional-message and localhost coverage, broader sequencing/root gating, advertised Snapshot/View metadata binding, document-scoped `HEADS replace=true`, stale/unadvertised `WANT` and unrequested `OBJECT` coverage, sender-validation and reachability faults, explicit `ERROR` handling, and messages-after-BYE plus missing-BYE warning coverage exist, but broader session/capability/error-path cases are still open |
| 13. Ready-to-Build Gate | Done | `M1` | the minimal-client gate is closed; remaining work now lives in the post-`M1` follow-up checklist instead of this gate |

## Suggested Reading Path

1. Read [`ROADMAP.md`](../ROADMAP.md) for build order and milestone intent.
2. Read [`IMPLEMENTATION-CHECKLIST.en.md`](../IMPLEMENTATION-CHECKLIST.en.md) for section-by-section closure items.
3. Read [`DEV-SETUP.md`](./DEV-SETUP.md) if you are starting from a fresh environment or onboarding a new agent.
4. Read [`GITHUB-ADOPTION-PLAN.md`](./GITHUB-ADOPTION-PLAN.md) for repo workflow and security-adoption status, including the enabled GitHub CodeQL default setup and the already-fixed first workflow-permissions findings.
5. Use [`progress.html`](../pages/progress.html) for the public visual summary.
