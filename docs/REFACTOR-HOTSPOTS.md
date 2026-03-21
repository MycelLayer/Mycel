# Refactor Hotspots

Status: draft backlog note derived from `scripts/check_code_quality_hotspots.py`

This note turns the current hotspot scan into a short refactor backlog so we can
pick structural cleanup work without re-reading the full scanner output each
time.

Current scan command:

```bash
python3 scripts/check_code_quality_hotspots.py apps crates scripts
```

Current warning thresholds:

- file size: over `800` lines
- function size: over `100` lines
- same non-trivial literal: `3+` repeats

## Prioritization Rule

Treat these as the best split candidates first:

1. high score plus active roadmap relevance
2. high score plus multiple long functions in one file
3. repeated literals or helper drift that suggest hidden submodules
4. very large test files that behave like a second implementation surface

## Highest-Value Candidates

### 1. `crates/mycel-core/src/store.rs`

- Current hotspot score: `48`
- Why now: this is one of the largest core files and it mixes rebuild, ingest,
  replay-adjacent store behavior, and document-loading concerns in the same
  surface.
- Roadmap fit: `M2` replay, storage, and rebuild follow-up
- Minimal safe split:
  - separate ingest/write paths from rebuild/index maintenance
  - isolate document-loading / object-loading helpers from mutation paths

### 2. `crates/mycel-core/src/author/merge.rs`

- Current hotspot score: `32`
- Why now: the file stays on the critical `M2` lane and still contains a very
  large `assess_merge_resolution` function plus repeated conflict-reason text.
- Roadmap fit: `M2` merge-authoring competing-variant closure
- Minimal safe split:
  - move competing placement/content/metadata classification into smaller
    helper modules
  - keep the top-level authoring flow thin and orchestration-only

### 3. `crates/mycel-core/src/head.rs`

- Current hotspot score: `21`
- Why now: `M3` work keeps landing here, and the file still mixes selector
  logic, inspect/render support, governance persistence queries, and viewer
  scoring.
- Roadmap fit: `M3` governance persistence and reader-plus-governance follow-up
- Minimal safe split:
  - split selector scoring from inspect/render surfaces
  - isolate governance-persistence query helpers from head selection logic

### 4. `apps/mycel-cli/src/view.rs`

- Current hotspot score: `11`
- Why now: the score is lower than the core hotspots, but this file sits
  directly on the current `M3` issue queue and is already over the file-size
  threshold with multiple long command handlers.
- Roadmap fit: `M3` governance persistence and tooling follow-up
- Minimal safe split:
  - separate `view inspect`, `view list`, and `view publish` output/rendering
  - isolate shared formatting and summary helpers

### 5. `apps/mycel-cli/src/report/mod.rs` and `apps/mycel-cli/src/report/diff.rs`

- Current hotspot score: `6` for `report/mod.rs`, `9` for `report/diff.rs`
- Why now: the report area is not the main roadmap lane, but it shows a clear
  command-dispatch plus formatting split opportunity and has multiple large
  functions.
- Roadmap fit: supporting tooling quality, not a primary milestone gate
- Minimal safe split:
  - move report command dispatch away from diff/render implementation details
  - reduce repeated report-format strings behind shared render helpers

## Large Test-Surface Candidates

These are worth splitting when we want safer follow-up work with lower product
risk:

1. `crates/mycel-core/src/author/tests/structural.rs` (`score=53`)
2. `apps/mycel-cli/tests/store_author_smoke/variants.rs` (`score=49`)
3. `apps/mycel-cli/tests/store_author_smoke/structural.rs` (`score=34`)
4. `apps/mycel-cli/tests/object_verify_smoke/revision.rs` (`score=22`)
5. `apps/mycel-cli/tests/head_inspect_smoke/selector.rs` (`score=20`)

Preferred split pattern:

- group by behavior family instead of keeping one giant matrix file
- extract shared builders/helpers before moving assertions
- keep fixture-backed coverage intact while reducing individual test length

## Notable Non-Primary Hotspots

These are useful backlog items, but not first-wave candidates:

- `apps/mycel-cli/src/store/index.rs` (`score=27`): repeated literal-heavy CLI
  summary/output surface
- `crates/mycel-sim/src/run.rs` (`score=41`): large simulator orchestration
  file that may deserve a follow-up once `M4` interop work resumes
- `scripts/agent_registry.py` (`score=6`): tooling hotspot, but not on the main
  product lane

## Suggested Use

When choosing the next refactor slice:

1. prefer one hotspot that overlaps the active roadmap lane
2. pick the smallest split that reduces future edit pressure
3. keep behavior-preserving refactors separate from lane-closing feature work
4. rerun the hotspot scanner after the split so the backlog note can be updated
