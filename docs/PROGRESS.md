# Mycel Progress View

Status: draft, refreshed for a full planning-sync pass so the summary now reflects `M2` / `M3` / `M4` as the active planning lane; `M2` now records that the remaining closure work is no longer general nested/reparenting coverage or still-coarse content/metadata competing-branch classification, but the smaller replay/store fixture and reporting-alignment follow-up left after the richer mixed branch kinds and the multi-document rebuild-after-index-loss proof landed, `M3` still centers on broader governance persistence and reader-plus-governance follow-up beyond the current inspect/list/publish baseline, and `M4` records localhost transport proof, all three currently tracked production replication sub-proofs, and a first permanent messages-after-BYE session proof as landed while broader session/capability/error-path interop remains open

This page turns [`ROADMAP.md`](../ROADMAP.md) and [`IMPLEMENTATION-CHECKLIST.en.md`](../IMPLEMENTATION-CHECKLIST.en.md) into one quick progress view.

## Current Lane

The current build lane is:

1. finish `M2` replay, rebuild, merge-authoring, and narrow write-path closure, with the remaining focus now narrowed to final replay/store fixture closure and any last merge-authoring reporting follow-up after the richer mixed content/metadata competing-branch classification and multi-document rebuild-after-index-loss proof landed
2. expand `M3` reader-plus-governance workflows carefully on top of the now-usable accepted-head inspection, render, clearer available-profile and profile-error feedback, editor-admission-aware profile base, bounded viewer score surfaces, initial governance inspect/list/publish surfaces, and persisted governance relationship summaries exposed through `view inspect` / `view list` while keeping broader governance persistence, richer governance tooling beyond this initial base, profile ergonomics beyond this initial polish, and final independent dual-role role-assignment closure explicit
3. keep `M4` narrow while peer-store sync proof grows toward the remaining session/capability/error-path interop closure now that the current production replication sub-items are landed and a first messages-after-BYE proof exists

## Milestone Timeline

```mermaid
flowchart LR
  subgraph Minimal
    M1["M1<br/>Core Object and Validation Base<br/>Closed gate"]
    M2["M2<br/>Replay, Storage, and Rebuild<br/>Substantially underway"]
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
| `M2` | Substantially underway | replay, `state_hash`, store rebuild, ancestry-aware render/store verification, explicit CLI proof that multi-document indexes rebuild cleanly after index loss from stored canonical objects, narrow write path, conservative merge authoring with broader structural coverage plus manual-curation smoke for nested parent-choice, nested sibling-choice, and composed-branch placement conflicts, richer direct and anchor-based competing placement reasons, and richer mixed content/metadata competing-branch classification with matching CLI smoke coverage | stronger replay/store fixtures, final reporting/doc alignment, and any remaining narrow closure checks before the milestone can be treated as closeable |
| `M3` | Early partial | accepted-head reader workflows, bundle/store rendering with clearer ancestry context, named fixed-profile reading with clearer available-profile and profile-error feedback, editor-admission-aware inspect/render flows, distinct human/debug head text modes, bounded viewer score surfaces in head inspection, initial filtered/sorted/projected `view` governance inspect/list/publish workflows, persisted reverse governance indexes, and persisted governance relationship summaries exposed through `view inspect` / `view list` | broader governance persistence, richer governance tooling, reader profile ergonomics beyond this initial polish, and final independent dual-role role-assignment closure |
| `M4` | Early partial | wire envelope validation, `OBJECT` body verification, session reachability, store-backed bootstrap, peer-store-driven first-time / incremental sync proofs, capability-gated optional-message handling, localhost multi-process proof, re-sync idempotency proof, depth-N incremental catchup proof, partial-doc selective sync proof, and a messages-after-BYE simulator proof | remaining broader session/capability/error-path interop coverage |
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
| 7. Wire Protocol | Partial | `M4` | canonical wire-envelope parsing, field validation, RFC 3339 checks, minimal-message payload validation, sender checks, session sequencing/head-tracking, reachability gating, store-backed bootstrap, `OBJECT` body verification, capability-gated optional-message handling, a messages-after-BYE simulator proof, and a minimal peer-store sync driver now exist in `mycel-core`; the main remaining interop work is broader session/capability/error-path proof |
| 8. Sync Workflow | Partial | `M4` | peer-store-driven first-time and incremental sync now prove shared verify/store flows through `mycel-core`, the CLI, and simulator positive-path coverage, including snapshot-assisted catch-up, announced-view fetching, localhost multi-process transport proof, re-sync idempotency, depth-N incremental catchup, partial-doc selective sync, and messages-after-BYE rejection; remaining work is broader session/capability/error-path proof |
| 9. Views and Head Selection | Mostly done | `M3` | deterministic selector core, named fixed-profile selection with clearer available-profile summaries and profile-error feedback, separate editor/view admission-aware inspect/render flows, distinct human/debug head text modes, bounded viewer score channels in head inspection, and persisted governance relationship summaries through `view inspect` / `view list` exist; broader governance persistence and final independent dual-role role-assignment closure remain |
| 10. Merge Generation | Partial | `M2` | replay verification and a conservative local merge-authoring profile exist, including structural move/reorder, new-parent reparenting, simple composed parent-chain coverage, a broader nested structural matrix, manual-curation smoke for nested parent-choice, nested sibling-choice, and composed-branch placement conflicts, richer direct/anchor-based competing placement reasons, landed metadata competing-variant handling for adopting or keeping primary over non-primary metadata additions, richer mixed content/metadata competing-branch detail with matching CLI smoke coverage, and an explicit manual-curation boundary for metadata removal because v0.1 patch ops do not yet express deletion; the remaining `M2` work is now narrower replay/store closure and reporting/doc alignment rather than coarse competing-branch classification itself |
| 11. CLI or API Surface | Partial | `M2` / `M3` / `M4` | verification, authoring, conservative merge authoring, editor-admission-aware reader inspection/render, governance inspect/list/publish, persisted governance index query surfaces, persisted governance relationship summaries in `view inspect` / `view list`, transcript-backed `sync pull`, and internal `sync peer-store` all exist, including optional snapshot/view flows, localhost multi-process proof, re-sync idempotency, depth-N catchup, partial-doc selective sync, and messages-after-BYE rejection; the remaining `M4` gap is broader session/capability/error-path interop proof |
| 12. Interop Test Minimum | Partial | `M1` / `M2` / `M4` | fixture isolation, reproducibility, stricter parser/replay smoke coverage, direct wire-envelope/signature/session tests, peer-store first-time / incremental sync proofs, optional-message coverage, localhost multi-process proof, re-sync idempotency coverage, depth-N incremental catchup coverage, partial-doc selective sync coverage, and messages-after-BYE coverage exist, but broader session/capability/error-path cases are still open |
| 13. Ready-to-Build Gate | Done | `M1` | the minimal-client gate is closed; remaining work now lives in the post-`M1` follow-up checklist instead of this gate |

## Suggested Reading Path

1. Read [`ROADMAP.md`](../ROADMAP.md) for build order and milestone intent.
2. Read [`IMPLEMENTATION-CHECKLIST.en.md`](../IMPLEMENTATION-CHECKLIST.en.md) for section-by-section closure items.
3. Read [`DEV-SETUP.md`](./DEV-SETUP.md) if you are starting from a fresh environment or onboarding a new agent.
4. Use [`progress.html`](../pages/progress.html) for the public visual summary.
