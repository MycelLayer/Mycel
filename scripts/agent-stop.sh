#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Stop an active or paused agent by updating its local registry status.

Usage:
  scripts/agent-stop.sh <agent-id> [--status paused|done] [--json]

Examples:
  scripts/agent-stop.sh coding-1
  scripts/agent-stop.sh doc-1 --status done
  scripts/agent-stop.sh coding-1 --json

Behavior:
  - reads .agent-local/agents.json
  - finds the matching agent entry
  - sets status to paused or done
  - updates registry timestamp

This command does not delete the mailbox or remove the agent entry.
EOF
}

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
REGISTRY_PATH="$ROOT_DIR/.agent-local/agents.json"
JSON_MODE=0
AGENT_ID=""
TARGET_STATUS="paused"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --status)
      if [[ $# -lt 2 ]]; then
        echo "missing value for --status" >&2
        exit 1
      fi
      TARGET_STATUS=$2
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
    -*)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
    *)
      if [[ -n "$AGENT_ID" ]]; then
        echo "unexpected extra argument: $1" >&2
        usage >&2
        exit 1
      fi
      AGENT_ID=$1
      shift
      ;;
  esac
done

if [[ -z "$AGENT_ID" ]]; then
  usage >&2
  exit 1
fi

if [[ ! -f "$REGISTRY_PATH" ]]; then
  echo "missing registry file: $REGISTRY_PATH" >&2
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "missing required command: python3" >&2
  exit 1
fi

PYTHON_OUTPUT=$(
  AGENT_ID="$AGENT_ID" TARGET_STATUS="$TARGET_STATUS" REGISTRY_PATH="$REGISTRY_PATH" python3 <<'PY'
import json
import os
from datetime import datetime, timezone
from pathlib import Path

agent_id = os.environ["AGENT_ID"]
target_status = os.environ["TARGET_STATUS"]
registry_path = Path(os.environ["REGISTRY_PATH"])

if target_status not in {"paused", "done"}:
    raise SystemExit(f"unsupported stop status: {target_status}")

try:
    registry = json.loads(registry_path.read_text(encoding="utf-8"))
except FileNotFoundError:
    raise SystemExit(f"missing registry file: {registry_path}")
except json.JSONDecodeError as exc:
    raise SystemExit(f"invalid registry JSON: {exc}")

if not isinstance(registry, dict):
    raise SystemExit("invalid registry: top-level JSON value must be an object")

agents = registry.get("agents")
if not isinstance(agents, list):
    raise SystemExit("invalid registry: agents must be an array")

agent_count = registry.get("agent_count")
if agent_count != len(agents):
    raise SystemExit(
        f"invalid registry: agent_count={agent_count!r} does not match agents length {len(agents)}"
    )

matches = [entry for entry in agents if isinstance(entry, dict) and entry.get("id") == agent_id]
if not matches:
    raise SystemExit(f"agent entry not found: {agent_id}")
if len(matches) > 1:
    raise SystemExit(f"invalid registry: duplicate agent id {agent_id}")

entry = matches[0]
current_status = entry.get("status")
if not isinstance(current_status, str) or not current_status.strip():
    raise SystemExit(f"agent {agent_id} is missing required field: status")

now = datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
entry["status"] = target_status
registry["updated_at"] = now

registry_path.write_text(json.dumps(registry, indent=2) + "\n", encoding="utf-8")

result = {
    "status": "ok",
    "agent_id": agent_id,
    "previous_status": current_status,
    "current_status": target_status,
    "updated_at": now,
}
print(json.dumps(result))
PY
)

if (( JSON_MODE )); then
  printf '%s\n' "$PYTHON_OUTPUT"
  exit 0
fi

STOP_PREVIOUS=$(python3 -c 'import json,sys; data=json.loads(sys.argv[1]); print(data["previous_status"])' "$PYTHON_OUTPUT")
STOP_CURRENT=$(python3 -c 'import json,sys; data=json.loads(sys.argv[1]); print(data["current_status"])' "$PYTHON_OUTPUT")
STOP_TIME=$(python3 -c 'import json,sys; data=json.loads(sys.argv[1]); print(data["updated_at"])' "$PYTHON_OUTPUT")

printf 'agent stopped: %s\n' "$AGENT_ID"
printf 'previous_status: %s\n' "$STOP_PREVIOUS"
printf 'current_status: %s\n' "$STOP_CURRENT"
printf 'updated_at: %s\n' "$STOP_TIME"
