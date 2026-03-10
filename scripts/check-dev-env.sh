#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Check whether the local machine is ready for Mycel development.

Usage:
  scripts/check-dev-env.sh [--full] [--json]

Examples:
  scripts/check-dev-env.sh
  scripts/check-dev-env.sh --full
  scripts/check-dev-env.sh --json
  scripts/check-dev-env.sh --full --json

Modes:
  default   Check required tools, toolchain metadata, and required Rust components.
  --full    Also run the first-pass validation commands from docs/DEV-SETUP.md.
            This checks current workspace health as well as local tool readiness.
  --json    Emit machine-readable JSON instead of human-oriented log lines.
EOF
}

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
MODE="quick"
JSON_MODE=0
RESULTS=()
ERROR_MESSAGE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --full)
      MODE="full"
      shift
      ;;
    --json)
      JSON_MODE=1
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

json_escape() {
  local value=$1
  value=${value//\\/\\\\}
  value=${value//\"/\\\"}
  value=${value//$'\n'/\\n}
  value=${value//$'\r'/\\r}
  value=${value//$'\t'/\\t}
  printf '%s' "$value"
}

append_result() {
  local kind=$1
  local name=$2
  local status=$3
  local detail=${4:-}

  RESULTS+=("${kind}"$'\t'"${name}"$'\t'"${status}"$'\t'"${detail}")
}

emit_json() {
  local overall_status=$1
  local first=1

  printf '{'
  printf '"status":"%s",' "$(json_escape "$overall_status")"
  printf '"mode":"%s",' "$(json_escape "$MODE")"
  printf '"repo_root":"%s",' "$(json_escape "$ROOT_DIR")"
  printf '"required_toolchain_channel":"%s",' "$(json_escape "${toolchain_channel:-unknown}")"
  printf '"minimum_rust":"%s",' "$(json_escape "${minimum_rust:-unknown}")"
  printf '"checks":['
  for entry in "${RESULTS[@]}"; do
    IFS=$'\t' read -r kind name status detail <<<"$entry"
    if (( first )); then
      first=0
    else
      printf ','
    fi
    printf '{'
    printf '"kind":"%s",' "$(json_escape "$kind")"
    printf '"name":"%s",' "$(json_escape "$name")"
    printf '"status":"%s",' "$(json_escape "$status")"
    printf '"detail":"%s"' "$(json_escape "$detail")"
    printf '}'
  done
  printf ']'
  if [[ -n "$ERROR_MESSAGE" ]]; then
    printf ',"error":"%s"' "$(json_escape "$ERROR_MESSAGE")"
  fi
  printf '}\n'
}

fail() {
  local message=$1
  ERROR_MESSAGE=$message
  if (( JSON_MODE )); then
    emit_json "failed"
  else
    echo "$message" >&2
  fi
  exit 1
}

log_line() {
  if (( ! JSON_MODE )); then
    printf '%s\n' "$1"
  fi
}

require_cmd() {
  local cmd=$1
  local version_arg=${2:---version}

  if ! command -v "$cmd" >/dev/null 2>&1; then
    append_result "command" "$cmd" "missing" ""
    fail "missing required command: $cmd"
  fi

  local version
  version=$("$cmd" "$version_arg" 2>/dev/null | head -n 1 || true)
  append_result "command" "$cmd" "found" "$version"
  if [[ -n "$version" ]]; then
    log_line "$(printf 'found %-8s %s' "$cmd" "$version")"
  else
    log_line "$(printf 'found %-8s' "$cmd")"
  fi
}

require_component() {
  local toolchain=$1
  local component=$2

  if ! rustup component list --toolchain "$toolchain" --installed | grep -Eq "^${component}(-|$)"; then
    append_result "component" "$component" "missing" "$toolchain"
    fail "missing required Rust component on $toolchain: $component"
  fi

  append_result "component" "$component" "found" "$toolchain"
  log_line "found component $component on $toolchain"
}

run_check() {
  local label=$1
  shift
  local output

  log_line "$(printf 'running %-18s %s' "$label" "$*")"
  if output=$(
    cd "$ROOT_DIR"
    "$@" 2>&1
  ); then
    append_result "validation" "$label" "passed" "$*"
    if [[ -n "$output" && $JSON_MODE -eq 0 ]]; then
      printf '%s\n' "$output"
    fi
    return 0
  fi

  append_result "validation" "$label" "failed" "$*"
  if (( JSON_MODE )); then
    fail "validation step failed: $label"
  fi

  if [[ -n "$output" ]]; then
    printf '%s\n' "$output"
  fi
  fail "validation step failed: $label"
}

if [[ ! -f "$ROOT_DIR/rust-toolchain.toml" ]]; then
  fail "missing rust-toolchain.toml in repo root"
fi

if [[ ! -f "$ROOT_DIR/Cargo.toml" ]]; then
  fail "missing Cargo.toml in repo root"
fi

toolchain_channel=$(awk -F '"' '/^channel = / { print $2; exit }' "$ROOT_DIR/rust-toolchain.toml")
minimum_rust=$(awk -F '"' '/rust-version = / { print $2; exit }' "$ROOT_DIR/Cargo.toml")

log_line "checking Mycel development environment"
log_line "repo root: $ROOT_DIR"
log_line "required toolchain channel: ${toolchain_channel:-unknown}"
log_line "workspace minimum Rust: ${minimum_rust:-unknown}"

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
  log_line "running full validation pass"
  run_check "fmt" cargo fmt --all --check
  run_check "core-tests" cargo test -p mycel-core
  run_check "cli-tests" cargo test -p mycel-cli
  run_check "cli-info" cargo run -p mycel-cli -- info
  run_check "fixture-validate" cargo run -p mycel-cli -- validate fixtures/object-sets/minimal-valid/fixture.json --json
  run_check "sim-smoke" ./sim/negative-validation/smoke.sh --summary-only
fi

if (( JSON_MODE )); then
  emit_json "passed"
else
  echo "dev environment check passed"
fi
