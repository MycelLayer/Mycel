# Object Sets

This directory contains named fixture sets for simulator and verification tests.

Each subdirectory should represent one scenario.

Recommended contents for each scenario:

- `README.md`: scenario purpose and expected outcomes
- `fixture.json`: language-neutral metadata about peers, documents, and expected results
- `../fixture.schema.json`: JSON Schema for validating `fixture.json`
- optional object files if we later decide to store canonical examples separately

## Schema

- `fixture.schema.json` is the formal contract for `fixture.json`.
- New fixture sets should validate against this schema.
- The schema keeps the contract narrow but leaves room for optional `metadata` and document-level extension fields.

## Current Scenarios

- `minimal-valid/`
- `hash-mismatch/`
- `signature-mismatch/`
- `partial-want-recovery/`
