# Test Matrix

## Positive

- `first-sync-empty-reader`: empty reader syncs from `minimal-valid`
- `three-peer-consistency`: two readers converge on the same verified object set
  Reference JSON: `sim/tests/three-peer-consistency.example.json`

## Negative

- `reject-hash-mismatch`: reject invalid object body hash
  Reference JSON: `sim/tests/hash-mismatch.example.json`
- `reject-signature-mismatch`: reject invalid object or wire signature
  Reference JSON: `sim/tests/signature-mismatch.example.json`

## Recovery

- `recover-missing-objects`: recover missing objects via `WANT`
  Reference JSON: `sim/tests/partial-want-recovery.example.json`

## Deferred

- snapshot-assisted catch-up
- localhost multi-process runs
- accepted-head comparison reports
