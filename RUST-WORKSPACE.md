# Rust Workspace

This note describes the first Rust workspace cut for Mycel.

## Goals

- keep protocol-facing logic in Rust
- keep simulator logic in a reusable Rust library
- expose a thin CLI before any Flutter UI work starts

## Layout

- `crates/mycel-core/`: shared protocol-facing Rust library
- `crates/mycel-sim/`: simulator-facing Rust library
- `apps/mycel-cli/`: initial CLI binary crate

## Current Scope

The current Rust workspace is only a scaffold.

It does not yet implement:

- schema loading
- schema validation
- fixture/topology/test/report cross-check execution
- wire sync
- object parsing or replay

## Recommended Next Step

Implement one narrow CLI command first:

- `mycel info`: verify the workspace boots and can read scaffold defaults

Then move to:

- `mycel validate`
- `mycel sim run`
