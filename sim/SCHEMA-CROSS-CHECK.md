# Schema Cross-check

This note explains how the simulator schemas relate to one another and which fields should be checked for consistency across files.

The goal is to keep:

- fixture data
- peer definitions
- topology definitions
- test cases
- run reports

aligned without forcing one implementation language.

## Overview

The simulator data flow is:

1. a fixture defines the test data scenario
2. a peer definition defines one peer shape
3. a topology assembles peers into a bounded network
4. a test case selects a topology and fixture and declares expected outcomes
5. a report records what actually happened

## Schema Roles

| Schema | Main purpose | Typical file location |
| --- | --- | --- |
| `fixture.schema.json` | defines one fixture set | `fixtures/object-sets/*/fixture.json` |
| `peer.schema.json` | defines one peer config | `sim/peers/*.json` |
| `topology.schema.json` | defines one peer graph | `sim/topologies/*.json` |
| `test-case.schema.json` | defines one runnable test case | `sim/tests/*.json` |
| `report.schema.json` | defines one run result | `sim/reports/*.json` or `sim/reports/out/*.json` |

## Cross-check Rules

### 1. Fixture -> Topology

These values should agree:

- `fixture.seed_peer` should match one topology peer role or node mapping that is intended to load the seed data.
- `fixture.reader_peers[]` should map to reader peers in the chosen topology.
- `fixture.fault_peer`, if present, should map to a fault-capable peer in the chosen topology.
- `fixture.expected_outcomes[]` should be compatible with the topology's intended scenario.

Minimum loader check:

- the selected topology includes enough peers to satisfy the fixture role references

## 2. Peer -> Topology

`topology.schema.json` reuses `peer.schema.json`.

That means every entry in `topology.peers[]` should already satisfy the standalone peer contract.

Cross-check focus:

- `node_id` values are unique inside one topology
- `bootstrap_peers[]` only reference peer IDs that exist in the same topology unless the test explicitly allows external peers
- peer `role` values are coherent with the selected fixture and test case

Minimum loader check:

- all bootstrap references resolve

## 3. Topology -> Test Case

These values should agree:

- `test_case.topology` should point to one topology file
- `test_case.execution_mode` should match or be compatible with `topology.execution_mode` when the topology declares one
- `test_case.expected_outcomes[]` should be a subset of, or compatible with, the topology and fixture scenario

Minimum loader check:

- the referenced topology file exists and parses

## 4. Fixture -> Test Case

These values should agree:

- `test_case.fixture_set` should point to one fixture directory
- the chosen fixture should support the `test_case.category`
- `test_case.expected_result` should be consistent with the fixture intent

Examples:

- `minimal-valid` normally pairs with `expected_result: "pass"`
- `hash-mismatch` may still produce an overall pass if the test expects correct rejection behavior
- a malformed negative test should not automatically imply `expected_result: "fail"` unless the harness defines failure that way

Minimum loader check:

- the referenced fixture directory exists and contains a valid `fixture.json`

## 5. Test Case -> Report

These values should agree:

- `report.test_id` should equal `test_case.test_id`
- `report.topology_id` should equal the loaded topology's `topology_id`
- `report.fixture_id` should equal the loaded fixture's `fixture_id`
- `report.execution_mode` should equal the actual mode used for the run

Minimum runner check:

- report identity fields are copied from resolved inputs, not retyped manually

## 6. Report -> Expected Outcomes

These values should be compared after execution:

- `report.result` versus `test_case.expected_result`
- `report.summary.matched_expected_outcomes[]` versus `test_case.expected_outcomes[]`
- `report.failures[]` versus the assertions defined in the test case

Minimum validator check:

- every required test-case assertion is either satisfied or represented by a report failure entry

## Identity Map

These identifiers should stay stable across the simulator data model:

| Identifier | Source of truth | Reused by |
| --- | --- | --- |
| `fixture_id` | fixture | test case, report |
| `node_id` | peer / topology peer entry | fixture role references, report peer entries |
| `topology_id` | topology | report |
| `test_id` | test case | report |
| `run_id` | report runtime | report only |

## Minimum Loader Order

The recommended resolution order is:

1. load test case
2. load referenced topology
3. load referenced fixture
4. validate topology peer entries against `peer.schema.json`
5. run cross-checks between fixture, topology, and test case
6. execute the run
7. emit a report
8. validate the report against `report.schema.json`

This order keeps reference resolution explicit and avoids hidden defaults.

## Minimum Validation Checklist

A first implementation should at least reject:

- missing referenced topology file
- missing referenced fixture file
- duplicate `node_id` values in one topology
- unresolved `bootstrap_peers` references
- fixture role references that do not map to topology peers
- report identity fields that do not match the resolved test inputs

## Non-goals

This cross-check note does not require:

- a global registry of all peer IDs across the repo
- one universal role taxonomy for all future profiles
- fully automatic semantic validation of every expected outcome string

It defines only the narrow consistency checks needed for simulator v0.
