# Head Inspect Fixtures

This directory holds local input bundles for the minimal `mycel head inspect` CLI surface.

Recommended contents:

- `head-inspect.schema.json`: formal schema for one local input bundle
- `<fixture-name>/bundle.json`: repo-native fixture directory layout for smoke tests and manual inspection
- optional flat `*.json` / `*.example.json` bundles for one-off inputs

Current example bundles:

- `minimal-head-selection/bundle.json`: selects one accepted head from two eligible revisions using three signed View objects
