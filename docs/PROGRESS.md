# Mycel Progress View

Status: draft, refreshed after the recent wire-envelope parsing, signature-verification, and session-sequencing groundwork batch

This page turns [`ROADMAP.md`](../ROADMAP.md) and [`IMPLEMENTATION-CHECKLIST.en.md`](../IMPLEMENTATION-CHECKLIST.en.md) into one quick progress view.

## Current Lane

The current build lane is:

1. close `M1` parsing and canonicalization debt
2. finish `M2` replay, rebuild, and merge-authoring closure
3. expand `M3` reader workflows carefully on top of the now-usable accepted-head inspection, render, and editor-admission-aware profile base
4. keep `M4` wire groundwork narrow until `OBJECT` body verification and end-to-end sync can land cleanly

## Milestone Timeline

```mermaid
flowchart LR
  subgraph Minimal
    M1["M1<br/>Core Object and Validation Base<br/>Mostly complete"]
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
| `M1` | Mostly complete | shared parsing, canonical helpers, and stricter parser/replay/CLI proof coverage | malformed field-shape depth closure, remaining semantic-edge closure, final shared-helper adoption for `state_hash`/wire, milestone-close proof points |
| `M2` | Substantially underway | replay, `state_hash`, store rebuild, ancestry-aware render/store verification, narrow write path, and conservative merge authoring with broader structural coverage | stronger replay/store fixtures, broader core reuse, and richer nested/reparenting conflict classification |
| `M3` | Early partial | accepted-head reader workflows, bundle/store rendering with clearer ancestry context, named fixed-profile reading, editor-admission-aware inspect/render flows, initial filtered/sorted/projected `view` governance inspect/list/publish workflows, and persisted reverse governance indexes | broader governance persistence, richer governance tooling, reader profile ergonomics |
| `M4` | Early partial | wire envelope validation, signature checks, session sequencing, and peer-interop groundwork | full fetch/verify/store sync, capability-gated optional flows, and `OBJECT` body-derived verification wiring remain |
| `M5` | Later | selective app-layer growth | depends on stable protocol core and sync |

## Implementation Matrix

Legend:

- `Done`: current checklist section is substantially closed for the minimal client
- `Mostly done`: only closure or follow-up work remains
- `Partial`: meaningful implementation exists, but the section is not closeable yet
- `Not started`: still mostly future work

| Area | Status | Primary milestone | Current read |
|---|---|---|---|
| 1. Repo and Build Setup | Mostly done | `M1` | the build/test base is stable, and shared canonical helper ownership is converging on a dedicated core module; final utility closure still remains |
| 2. Object Types and IDs | Partial | `M1` | all required v0.1 families are typed, and parser / verify / CLI dependency-proof coverage is broader; malformed field-shape depth, remaining semantic-edge closure, and role modeling remain |
| 3. Canonical Serialization and Hashing | Partial | `M1` | core rules and reproducibility coverage exist, and canonical helper ownership is more centralized in a shared core module; final adoption across replay-derived `state_hash` and wire remains open |
| 4. Signature Verification | Partial | `M1` / `M4` | object signature rules are in place, and generic wire-envelope signature verification now exists; remaining closure is broader shared-helper adoption plus end-to-end sync plumbing |
| 5. Patch and Revision Engine | Mostly done | `M2` | replay and `state_hash` are in place; dependency verification, wrong-type and multi-hop ancestry proofs, sibling declared-ID determinism, and render-path ancestry context are stronger |
| 6. Local State and Storage | Mostly done | `M2` | store ingest, rebuild, and indexes exist; rebuild smoke now preserves nested ancestry context in summary reporting, but local transport/safety separation remains |
| 7. Wire Protocol | Partial | `M4` | canonical wire-envelope parsing, field validation, RFC 3339 checks, minimal-message payload validation, sender checks, and inbound sequencing now exist in `mycel-core`; `OBJECT` body-derived verification and optional capability gating remain |
| 8. Sync Workflow | Partial | `M4` | advertised-head tracking, WANT root checks, requested-object gating, and closed-session enforcement exist, but first-time/incremental sync still do not run end-to-end |
| 9. Views and Head Selection | Mostly done | `M3` | deterministic selector core, named fixed-profile selection, and editor-admission-aware inspect/render flows exist; dual-role closure remains |
| 10. Merge Generation | Partial | `M2` | replay verification and a conservative local merge-authoring profile exist, including structural move/reorder, new-parent reparenting, simple composed parent-chain coverage, and a broader nested structural matrix, but richer nested/reparenting conflict cases still fall back to manual curation |
| 11. CLI or API Surface | Partial | `M2` / `M3` | verification, authoring, conservative merge authoring, editor-admission-aware reader inspection/render, governance inspect/list/publish, and persisted governance index query surfaces exist; accepted-head render now pre-verifies selected heads with clearer ancestry-context reporting, while sync remains open |
| 12. Interop Test Minimum | Partial | `M1` / `M2` / `M4` | fixture isolation, reproducibility, stricter parser/replay smoke coverage, and direct wire-envelope/signature/session tests exist, but several end-to-end sync and `OBJECT` behavior checks remain |
| 13. Ready-to-Build Gate | Partial | whole plan | replay, head selection, rebuild, and conservative merge authoring are green; parse closure and wire sync are not |

## Suggested Reading Path

1. Read [`ROADMAP.md`](../ROADMAP.md) for build order and milestone intent.
2. Read [`IMPLEMENTATION-CHECKLIST.en.md`](../IMPLEMENTATION-CHECKLIST.en.md) for section-by-section closure items.
3. Read [`DEV-SETUP.md`](./DEV-SETUP.md) if you are starting from a fresh environment or onboarding a new agent.
4. Use [`progress.html`](./progress.html) for the public visual summary.
