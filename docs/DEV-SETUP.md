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

Additional tools required for `Status: ready` and the `--full` CI-parity pass:

- `cargo-nextest`
- `ast-grep`

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

Install the CI-parity tools:

```bash
cargo install cargo-nextest --locked
cargo install ast-grep --locked
```

Use `scripts/check-dev-env.py` for a quick base-tool check. Use `scripts/check-dev-env.py --full` to require the CI toolchain and run the CI-parity validation gates. The generated local readiness file uses the full pass, so `Status: ready` means this workspace can reproduce the current CI gates locally.

## 1.1 Local Ready File For New Chats

Use `.agent-local/dev-setup-status.md` as the local readiness record for this workspace.

New chats should:

- read `.agent-local/dev-setup-status.md` first if it exists
- skip repeated setup checks when the file says `Status: ready`
- re-run the necessary checks when the file is missing or does not say `Status: ready`
- keep the file detailed enough to show both tool presence and repo validation coverage

Use [`.agent-local/DEV-SETUP-STATUS.example.md`](../.agent-local/DEV-SETUP-STATUS.example.md) as the format reference.

The local status file should at minimum record:

- overall status
- checked-at timestamp
- checked-by actor
- tool checks for `cargo`, `rustup`, `rustc`, `gh`, `rg`, `cargo-nextest`, and `ast-grep`
- Rust component checks for `rustfmt` and `clippy`
- whether the full repo validation pass was run
- the exact validation commands and whether they passed

Recommended tools for populating the file:

- `scripts/check-dev-env.py` to gather the repo-local environment and validation result
- `scripts/update-dev-setup-status.py` to refresh the local readiness record
- `scripts/check-runtime-preflight.py` to verify the current shell session before a specific test or validation command

Treat `Status: ready` as valid only when the recorded checks cover the CI-parity tools and validation surface for the current workspace.

`Status: ready` does not guarantee that the current shell session has the right `PATH` or helper utilities for the exact verification command you are about to run. Before commands such as `cargo test ... | grep ...` or repo scripts that rely on extra shell tools, run a lightweight runtime preflight:

```bash
scripts/check-runtime-preflight.py
scripts/check-runtime-preflight.py --require grep --require tail
```

Treat missing commands, or follow-on command exits such as `126` and `127`, as environment blockers first rather than product failures.

## 2. Clone and Enter the Repo

```bash
git clone https://github.com/MycelLayer/Mycel.git
cd Mycel
```

## 2.1 Enable Repo-local Hooks

Enable the checked-in git hooks for this clone:

```bash
git config core.hooksPath .githooks
```

The current pre-commit hook runs `npm run lint:pages` whenever staged changes touch `pages/`.

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
cargo clippy --workspace --all-targets -- -D warnings
cargo check
cargo nextest run --workspace
cargo test --workspace --doc
./sim/negative-validation/smoke.sh --summary-only
ast-grep scan --config sgconfig.yml --report-style short --format github
```

These commands confirm:

- formatting is available
- Clippy passes across all workspace targets with warnings denied
- the workspace compiles
- all workspace tests pass through the same `cargo-nextest` runner used by CI
- doctests pass
- simulator negative-validation smoke coverage works
- the CI-backed ast-grep structural quality rules pass

## 5. What “Setup Complete” Looks Like

Treat setup as complete if all of the following are true:

- `cargo fmt --all --check` succeeds
- `cargo clippy --workspace --all-targets -- -D warnings` succeeds
- `cargo check` succeeds
- `cargo nextest run --workspace` succeeds
- `cargo test --workspace --doc` succeeds
- `./sim/negative-validation/smoke.sh --summary-only` succeeds
- `ast-grep scan --config sgconfig.yml --report-style short --format github` succeeds

The repo-local shortcut for the full pass is `scripts/check-dev-env.py --full`.

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
```

Useful repo-local tools:

- `scripts/check-dev-env.py` for environment validation
- `scripts/agent_workflow_bundle.py` for exporting or importing the repo's agent workflow docs, templates, and scripts as a portable bundle
- `scripts/check-labels.py` for tracked-label verification
- `scripts/check-plan-refresh.py` for planning-refresh cadence checks
- `scripts/codespaces_storage_gc.py` for safe dry-run or apply-based Codespaces storage cleanup; see [`CODESPACES-STORAGE-GC.md`](./CODESPACES-STORAGE-GC.md)

## 8. If You Are a New AI Agent

Recommended next step after setup:

1. read [`docs/PROGRESS.md`](./PROGRESS.md)
2. read [`docs/MULTI-AGENT-COORDINATION.md`](./MULTI-AGENT-COORDINATION.md)
3. look for an `ai-ready` issue or a narrow checklist gap
4. verify the exact file boundary before editing

The repository is easiest to contribute to when work is narrow, deterministic, and directly tied to one spec or checklist closure item.
