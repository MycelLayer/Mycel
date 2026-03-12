#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Check whether planning-surface refresh is due.

Usage:
  scripts/check-plan-refresh.sh [--doc-threshold N] [--issue-threshold N] [--web-threshold N] [--json]

Examples:
  scripts/check-plan-refresh.sh
  scripts/check-plan-refresh.sh --json
  scripts/check-plan-refresh.sh --doc-threshold 10 --issue-threshold 10 --web-threshold 20
EOF
}

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
DOC_THRESHOLD=10
ISSUE_THRESHOLD=10
WEB_THRESHOLD=20
JSON_MODE=0
RESULTS=()
SURFACE_RESULTS=()
ERROR_MESSAGE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --doc-threshold)
      if [[ $# -lt 2 ]]; then
        echo "missing value for $1" >&2
        usage >&2
        exit 1
      fi
      DOC_THRESHOLD="$2"
      shift 2
      ;;
    --issue-threshold)
      if [[ $# -lt 2 ]]; then
        echo "missing value for $1" >&2
        usage >&2
        exit 1
      fi
      ISSUE_THRESHOLD="$2"
      shift 2
      ;;
    --web-threshold)
      if [[ $# -lt 2 ]]; then
        echo "missing value for $1" >&2
        usage >&2
        exit 1
      fi
      WEB_THRESHOLD="$2"
      shift 2
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

emit_threshold_error() {
  local field_name=$1
  local field_value=$2
  ERROR_MESSAGE="${field_name} must be a non-negative integer"
  if (( JSON_MODE )); then
    printf '{'
    printf '"status":"failed",'
    printf '"repo_root":"%s",' "$(json_escape "$ROOT_DIR")"
    printf '"checks":[],'
    printf '"surfaces":[],'
    printf '"error":"%s"' "$(json_escape "$ERROR_MESSAGE")"
    printf '}\n'
  else
    echo "$ERROR_MESSAGE: $field_value" >&2
  fi
  exit 1
}

for threshold_name in DOC_THRESHOLD ISSUE_THRESHOLD WEB_THRESHOLD; do
  threshold_value=${!threshold_name}
  if ! [[ "$threshold_value" =~ ^[0-9]+$ ]]; then
    emit_threshold_error "$threshold_name" "$threshold_value"
  fi
done

append_result() {
  local file=$1
  local status=$2
  local count=$3
  local short_commit=$4

  RESULTS+=("${file}"$'\t'"${status}"$'\t'"${count}"$'\t'"${short_commit}")
}

append_surface_result() {
  local name=$1
  local threshold=$2
  local status=$3
  local remaining=$4

  SURFACE_RESULTS+=("${name}"$'\t'"${threshold}"$'\t'"${status}"$'\t'"${remaining}")
}

emit_json() {
  local overall_status=$1
  local remaining=$2
  local first=1
  local surface_first=1
  local due_surface_first=1

  printf '{'
  printf '"status":"%s",' "$(json_escape "$overall_status")"
  printf '"repo_root":"%s",' "$(json_escape "$ROOT_DIR")"
  printf '"highest_commit_distance":%s,' "$(json_escape "$max_count")"
  printf '"remaining_commits":%s,' "$(json_escape "$remaining")"
  printf '"checks":['
  for entry in "${RESULTS[@]}"; do
    IFS=$'\t' read -r file status count short_commit <<<"$entry"
    if (( first )); then
      first=0
    else
      printf ','
    fi
    printf '{'
    printf '"file":"%s",' "$(json_escape "$file")"
    printf '"status":"%s",' "$(json_escape "$status")"
    printf '"commit_count":%s,' "$(json_escape "$count")"
    printf '"last_commit":"%s"' "$(json_escape "$short_commit")"
    printf '}'
  done
  printf '],'
  printf '"surfaces":['
  for entry in "${SURFACE_RESULTS[@]}"; do
    IFS=$'\t' read -r name threshold status surface_remaining <<<"$entry"
    if (( surface_first )); then
      surface_first=0
    else
      printf ','
    fi
    printf '{'
    printf '"name":"%s",' "$(json_escape "$name")"
    printf '"threshold":%s,' "$(json_escape "$threshold")"
    printf '"status":"%s",' "$(json_escape "$status")"
    printf '"remaining_commits":%s' "$(json_escape "$surface_remaining")"
    printf '}'
  done
  printf '],'
  printf '"due_surfaces":['
  for entry in "${SURFACE_RESULTS[@]}"; do
    IFS=$'\t' read -r name _threshold status _surface_remaining <<<"$entry"
    if [[ "$status" != "due" ]]; then
      continue
    fi
    if (( due_surface_first )); then
      due_surface_first=0
    else
      printf ','
    fi
    printf '"%s"' "$(json_escape "$name")"
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
    emit_json "failed" 0
  else
    echo "$message" >&2
  fi
  exit 1
}

if ! command -v git >/dev/null 2>&1; then
  fail "git is required"
fi

if ! git -C "$ROOT_DIR" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  fail "not inside a git worktree: $ROOT_DIR"
fi

tracked_files=(
  "ROADMAP.md"
  "ROADMAP.zh-TW.md"
  "IMPLEMENTATION-CHECKLIST.en.md"
  "IMPLEMENTATION-CHECKLIST.zh-TW.md"
)

max_count=0

for file in "${tracked_files[@]}"; do
  path="${ROOT_DIR}/${file}"
  if [[ ! -f "$path" ]]; then
    fail "tracked file not found: $file"
  fi

  last_commit=$(git -C "$ROOT_DIR" log -n 1 --format='%H' -- "$file")
  if [[ -z "$last_commit" ]]; then
    fail "no git history found for tracked file: $file"
  fi

  commit_count=$(git -C "$ROOT_DIR" rev-list --count "${last_commit}..HEAD")
  if (( commit_count > max_count )); then
    max_count=$commit_count
  fi

  status="ok"
  short_commit=$(git -C "$ROOT_DIR" rev-parse --short "$last_commit")
  append_result "$file" "$status" "$commit_count" "$short_commit"
  if (( ! JSON_MODE )); then
    printf '%s\t%s commits since %s\t%s\n' "$status" "$commit_count" "$short_commit" "$file"
  fi
done

overall_due=0
smallest_remaining=-1

for surface_name in doc issue web; do
  case "$surface_name" in
    doc)
      threshold=$DOC_THRESHOLD
      ;;
    issue)
      threshold=$ISSUE_THRESHOLD
      ;;
    web)
      threshold=$WEB_THRESHOLD
      ;;
  esac

  if (( max_count >= threshold )); then
    surface_status="due"
    surface_remaining=0
    overall_due=1
  else
    surface_status="ok"
    surface_remaining=$((threshold - max_count))
    if (( smallest_remaining < 0 || surface_remaining < smallest_remaining )); then
      smallest_remaining=$surface_remaining
    fi
  fi

  append_surface_result "$surface_name" "$threshold" "$surface_status" "$surface_remaining"
  if (( ! JSON_MODE )); then
    if [[ "$surface_status" == "due" ]]; then
      printf 'due\t%s refresh\tthreshold %s\t0 commits remain\n' "$surface_name" "$threshold"
    else
      printf 'ok\t%s refresh\tthreshold %s\t%s commits remain\n' \
        "$surface_name" "$threshold" "$surface_remaining"
    fi
  fi
done

remaining=0
if (( overall_due == 0 )); then
  remaining=$smallest_remaining
fi

if (( overall_due )); then
  if (( JSON_MODE )); then
    emit_json "due" "$remaining"
  else
    due_surfaces=()
    for entry in "${SURFACE_RESULTS[@]}"; do
      IFS=$'\t' read -r name _threshold status _surface_remaining <<<"$entry"
      if [[ "$status" == "due" ]]; then
        due_surfaces+=("$name")
      fi
    done
    printf 'plan refresh due: %s\n' "$(IFS=', '; echo "${due_surfaces[*]}")"
    printf 'highest commit distance across tracked files: %s\n' "$max_count"
  fi
  exit 1
fi

if (( JSON_MODE )); then
  emit_json "ok" "$remaining"
else
  printf 'plan refresh not due: %s commits remain before the next threshold\n' "$remaining"
fi
