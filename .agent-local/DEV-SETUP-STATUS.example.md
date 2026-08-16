# Dev Setup Status

- Status: ready
- Checked at: 2026-03-12 15:00 UTC+8
- Checked by: doc-<n>
- Workspace: <workspace-root>
- Evidence source:
  - `scripts/update-dev-setup-status.py --actor <role-id>`
- Notes:
  - Update this file whenever tool availability changes or the workspace is reprovisioned.
  - New chats may skip bootstrap dev-setup checks only when this file says `Status: ready`.
  - This file is normally generated rather than edited by hand.

## Tool Checks

| Item | Status | Detail |
|---|---|---|
| `cargo` | passed | `cargo --version` |
| `rustup` | passed | `rustup --version` |
| `rustc` | passed | `rustc --version` |
| `gh` | passed | `gh --version` |
| `rg` | passed | `rg --version` |
| `cargo-nextest` | passed | `cargo-nextest --version` |
| `ast-grep` | passed | `ast-grep --version` |

## Rust Components

| Item | Status | Detail |
|---|---|---|
| `rustfmt` | passed | `rustup component list --toolchain stable --installed` |
| `clippy` | passed | `rustup component list --toolchain stable --installed` |

## Repo Validation

- Full validation run: yes

| Check | Status | Command |
|---|---|---|
| format | passed | `cargo fmt --all --check` |
| clippy | passed | `cargo clippy --workspace --all-targets -- -D warnings` |
| compile | passed | `cargo check` |
| workspace tests | passed | `cargo nextest run --workspace` |
| doctests | passed | `cargo test --workspace --doc` |
| sim smoke | passed | `./sim/negative-validation/smoke.sh --summary-only` |
| ast-grep quality | passed | `ast-grep scan --config sgconfig.yml --report-style short --format github` |
