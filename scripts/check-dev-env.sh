#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Check whether the local machine is ready for Mycel development.

Usage:
  scripts/check-dev-env.sh [--full]

Examples:
  scripts/check-dev-env.sh
  scripts/check-dev-env.sh --full

Modes:
  default   Check required tools, toolchain metadata, and required Rust components.
  --full    Also run the first-pass validation commands from docs/DEV-SETUP.md.
            This checks current workspace health as well as local tool readiness.
EOF
}

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
MODE="quick"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --full)
      MODE="full"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

require_cmd() {
  local cmd=$1
  local version_arg=${2:---version}

  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "missing required command: $cmd" >&2
    exit 1
  fi

  local version
  version=$("$cmd" "$version_arg" 2>/dev/null | head -n 1 || true)
  if [[ -n "$version" ]]; then
    printf 'found %-8s %s\n' "$cmd" "$version"
  else
    printf 'found %-8s\n' "$cmd"
  fi
}

require_component() {
  local toolchain=$1
  local component=$2

  if ! rustup component list --toolchain "$toolchain" --installed | grep -Eq "^${component}(-|$)"; then
    printf 'missing required Rust component on %s: %s\n' "$toolchain" "$component" >&2
    exit 1
  fi

  printf 'found component %s on %s\n' "$component" "$toolchain"
}

run_check() {
  local label=$1
  shift

  printf 'running %-18s %s\n' "$label" "$*"
  (
    cd "$ROOT_DIR"
    "$@"
  )
}

if [[ ! -f "$ROOT_DIR/rust-toolchain.toml" ]]; then
  echo "missing rust-toolchain.toml in repo root" >&2
  exit 1
fi

if [[ ! -f "$ROOT_DIR/Cargo.toml" ]]; then
  echo "missing Cargo.toml in repo root" >&2
  exit 1
fi

toolchain_channel=$(awk -F '"' '/^channel = / { print $2; exit }' "$ROOT_DIR/rust-toolchain.toml")
minimum_rust=$(awk -F '"' '/rust-version = / { print $2; exit }' "$ROOT_DIR/Cargo.toml")

echo "checking Mycel development environment"
printf 'repo root: %s\n' "$ROOT_DIR"
printf 'required toolchain channel: %s\n' "${toolchain_channel:-unknown}"
printf 'workspace minimum Rust: %s\n' "${minimum_rust:-unknown}"

require_cmd cargo
require_cmd rustup
require_cmd rustc
require_cmd gh
require_cmd rg

if [[ -n "${toolchain_channel:-}" ]]; then
  require_component "$toolchain_channel" rustfmt
  require_component "$toolchain_channel" clippy
fi

if [[ "$MODE" == "full" ]]; then
  echo "running full validation pass"
  run_check "fmt" cargo fmt --all --check
  run_check "core-tests" cargo test -p mycel-core
  run_check "cli-tests" cargo test -p mycel-cli
  run_check "cli-info" cargo run -p mycel-cli -- info
  run_check "fixture-validate" cargo run -p mycel-cli -- validate fixtures/object-sets/minimal-valid/fixture.json --json
  run_check "sim-smoke" ./sim/negative-validation/smoke.sh --summary-only
fi

echo "dev environment check passed"
