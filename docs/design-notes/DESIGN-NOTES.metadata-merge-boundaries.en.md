# Metadata Merge Boundaries

Status: design draft

This note records the current `M2` metadata-merge boundary for the conservative
merge-authoring flow.

It is intentionally narrow. The goal is to make the currently implemented
metadata behavior easy to cite before the remaining `content-variant` work or
any later patch-model expansion.

## 0. Scope

This note covers only three metadata cases:

1. adopting a non-primary metadata addition
2. keeping the primary variant over a non-primary metadata addition
3. removing metadata from the resolved state

It does not try to redesign the full merge profile.

## 1. Current Rule

The current conservative metadata-merge rule is:

1. if the resolved state chooses a non-primary parent metadata variant, report
   `multi-variant`
2. if the resolved state keeps the primary metadata variant while a
   non-primary competing alternative exists, still report `multi-variant`
3. if the resolved state removes primary metadata, report
   `manual-curation-required`

This keeps competing metadata choices visible even when the final resolved state
matches the primary parent.

## 2. Adopt Non-primary Add

Case:

- the primary parent does not contain a metadata key
- a non-primary parent adds that key
- the resolved state adopts the non-primary value

Current treatment:

- outcome: `multi-variant`
- materialization: one `set_metadata` op

Reason:

- this is a real competing parent choice
- the current patch model can express the chosen result directly

## 3. Keep Primary Over Add

Case:

- the primary parent does not contain a metadata key
- a non-primary parent adds that key
- the resolved state keeps the primary variant and leaves the key absent

Current treatment:

- outcome: `multi-variant`
- materialization: zero-op merge patch

Reason:

- the merge still made a competing parent choice
- the choice should remain visible in the merge assessment even though the
  resolved state already matches the primary parent

The important design point is that "no generated patch op" does not mean
"no competing variant existed".

## 4. Metadata Removal Boundary

Case:

- the primary parent contains a metadata key
- the resolved state removes that key

Current treatment:

- outcome: `manual-curation-required`

Reason:

- Mycel v0.1 currently exposes `set_metadata`, but not a metadata-deletion op
- replay can insert or overwrite metadata keys, but it does not have a formal
  deletion primitive
- because the resolved state cannot be materialized faithfully through the
  current patch op set, this is a patch-model boundary rather than just a
  missing competing-variant classification

## 5. Implication For `M2`

This gives the current metadata story a stable three-branch shape:

1. adopt a non-primary metadata add: implemented
2. keep primary over a non-primary metadata add: implemented
3. remove metadata: explicitly out of automatic materialization scope for v0.1

That means the remaining metadata follow-up work should not treat removal as an
ordinary unclassified merge hole.

The next real decision is:

1. keep metadata deletion out of scope for v0.1
2. or add a narrowly-scoped metadata deletion op before widening removal cases
