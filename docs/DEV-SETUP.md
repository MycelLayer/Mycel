# Dev Setup

This guide is the shortest path from a fresh checkout to a usable Mycel development environment.

Use it together with:

- [`README.md`](../README.md)
- [`CONTRIBUTING.md`](../CONTRIBUTING.md)
- [`BOT-CONTRIBUTING.md`](../BOT-CONTRIBUTING.md)
- [`RUST-WORKSPACE.md`](../RUST-WORKSPACE.md)

## 0. What You Need

Required tools:

- Rust toolchain manager: `rustup`
- Rust `stable` toolchain
- Rust components: `rustfmt`, `clippy`
- GitHub CLI: `gh`
- ripgrep: `rg`

The current workspace metadata declares:

- minimum Rust version: `1.79`
- current checked-in toolchain channel: `stable`

The current repository environment used by maintainers is compatible with:

- `cargo 1.94.0`
- `rustc 1.94.0`
- `gh 2.83.1`
- `rg 14.1.0`

## 1. Install or Verify Tools

Check the local environment:

```bash
cargo --version
rustup --version
rustc --version
gh --version
rg --version
```

Install required Rust components if needed:

```bash
rustup toolchain install stable
rustup component add rustfmt --toolchain stable
rustup component add clippy --toolchain stable
```

## 2. Clone and Enter the Repo

```bash
git clone https://github.com/ctf2090/Mycel.git
cd Mycel
```

## 3. First Read Order

Before changing anything, read in this order:

1. [`README.md`](../README.md)
2. [`ROADMAP.md`](../ROADMAP.md)
3. [`IMPLEMENTATION-CHECKLIST.en.md`](../IMPLEMENTATION-CHECKLIST.en.md)
4. [`PROTOCOL.en.md`](../PROTOCOL.en.md)
5. [`WIRE-PROTOCOL.en.md`](../WIRE-PROTOCOL.en.md)
6. [`BOT-CONTRIBUTING.md`](../BOT-CONTRIBUTING.md) if you are using an AI coding agent

## 4. First Commands To Run

From the repository root:

```bash
cargo fmt --all --check
cargo test -p mycel-core
cargo test -p mycel-cli
cargo run -p mycel-cli -- info
cargo run -p mycel-cli -- validate fixtures/object-sets/minimal-valid/fixture.json --json
./sim/negative-validation/smoke.sh --summary-only
```

These commands confirm:

- formatting is available
- core tests run
- CLI tests run
- repo-local CLI wiring works
- fixture validation works
- simulator negative-validation smoke coverage works

## 5. What “Setup Complete” Looks Like

Treat setup as complete if all of the following are true:

- `cargo fmt --all --check` succeeds
- `cargo test -p mycel-core` succeeds
- `cargo test -p mycel-cli` succeeds
- `mycel-cli -- info` runs from the repo root
- fixture validation succeeds on `fixtures/object-sets/minimal-valid/fixture.json`
- `./sim/negative-validation/smoke.sh --summary-only` succeeds

## 6. Common Working Rules

- Make narrow, reviewable changes.
- Keep protocol-core changes conservative.
- If you touch protocol or design concepts, update both English and Traditional Chinese docs when both exist.
- Prefer deterministic verification and fixture-backed changes.
- Read [`AGENTS.md`](../AGENTS.md) for repo-specific collaboration rules.

## 7. Useful Follow-up Commands

```bash
cargo run -p mycel-cli -- object inspect <path> --json
cargo run -p mycel-cli -- object verify <path> --json
cargo run -p mycel-cli -- sim run sim/tests/three-peer-consistency.example.json --json
scripts/check-labels.sh
scripts/check-doc-refresh.sh
```

## 8. If You Are a New AI Agent

Recommended next step after setup:

1. read [`docs/PROGRESS.md`](./PROGRESS.md)
2. read [`docs/MULTI-AGENT-COORDINATION.md`](./MULTI-AGENT-COORDINATION.md)
3. look for an `ai-ready` issue or a narrow checklist gap
4. verify the exact file boundary before editing

The repository is easiest to contribute to when work is narrow, deterministic, and directly tied to one spec or checklist closure item.
