# Code Quality Hotspots Triage

Date: 2026-03-23

This note captures the CQH triage that reviewed issue `#29` and the current
hotspot scan output from:

```bash
python3 scripts/check_code_quality_hotspots.py apps crates scripts
```

## Summary

The highest-value follow-up is to refactor production hotspots before spending
time on the largest smoke-test files.

That does not mean the biggest test files are healthy. It means the best
near-term return comes from shrinking mixed or overly-central production
surfaces first, especially where one module now carries too much policy,
decision-making, or protocol behavior.

## Priority Order

1. Split `crates/mycel-core/src/wire.rs` tests out of the production module.
   - Why it ranked first:
     - it mixed production logic with a large inlined test surface
     - it carried repeated test literals inside a protocol-facing module
     - the refactor was likely to be behavior-preserving and low risk
   - Intended outcome:
     - keep `wire.rs` focused on production behavior
     - move unit tests into a dedicated sibling module

2. Extract `assess_merge_resolution` and adjacent helpers from `crates/mycel-core/src/author/merge.rs`.
   - Why it ranked second:
     - the merge-decision path is core product logic, not just test weight
     - the current function is long enough that future merge-rule changes will
       be harder to review and validate confidently
   - Intended outcome:
     - break the decision path into named helpers with clearer review
       boundaries

3. Split sync transcript generation and verification flow in `crates/mycel-core/src/sync.rs`.
   - Focus area:
     - `generate_sync_pull_transcript_filtered`
     - `sync_pull_from_transcript_with_policy`
   - Why it ranked third:
     - the sync path has meaningful CQH pressure and mixes several concerns
       that would be easier to evolve independently after decomposition

## Recommendation

Pick one production-code slice and land a narrow refactor with unchanged
behavior, then rerun the hotspot scan before choosing the next slice.

This triage intentionally favored reviewability, behavior safety, and future
edit clarity over simply chasing the single largest file by line count.
