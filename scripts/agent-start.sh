#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Confirm an assigned agent role before starting tracked work.

Usage:
  scripts/agent-start.sh <agent-id> [--json]

Examples:
  scripts/agent-start.sh coding-1
  scripts/agent-start.sh doc-1 --json

Behavior:
  - reads .agent-local/agents.json
  - finds the matching agent entry
  - verifies role assignment fields are present
  - marks the agent as confirmed
  - sets status to active
  - creates the mailbox file if missing

This command fails if the agent entry is missing or not properly assigned.
EOF
}

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
REGISTRY_PATH="$ROOT_DIR/.agent-local/agents.json"
JSON_MODE=0
AGENT_ID=""

while [[ $# -gt 0 ]]; do
  case "$1" in
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
  AGENT_ID="$AGENT_ID" REGISTRY_PATH="$REGISTRY_PATH" ROOT_DIR="$ROOT_DIR" python3 <<'PY'
import json
import os
from datetime import datetime, timezone
from pathlib import Path

agent_id = os.environ["AGENT_ID"]
registry_path = Path(os.environ["REGISTRY_PATH"])
root_dir = Path(os.environ["ROOT_DIR"])

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

required_fields = [
    "role",
    "assigned_by",
    "assigned_at",
    "scope",
    "mailbox",
]
for field in required_fields:
    value = entry.get(field)
    if not isinstance(value, str) or not value.strip():
        raise SystemExit(f"agent {agent_id} is missing required field: {field}")

status = entry.get("status")
if not isinstance(status, str) or not status.strip():
    raise SystemExit(f"agent {agent_id} is missing required field: status")
if status == "done":
    raise SystemExit(f"agent {agent_id} cannot start because status is done")
if status == "blocked":
    raise SystemExit(f"agent {agent_id} cannot start because status is blocked")

files = entry.get("files")
if not isinstance(files, list):
    raise SystemExit(f"agent {agent_id} is missing required field: files")

now = datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
entry["confirmed_by_agent"] = True
entry["confirmed_at"] = now
entry["status"] = "active"
registry["updated_at"] = now

mailbox_value = entry["mailbox"]
mailbox_path = Path(mailbox_value)
if not mailbox_path.is_absolute():
    mailbox_path = root_dir / mailbox_path
mailbox_path.parent.mkdir(parents=True, exist_ok=True)
if not mailbox_path.exists():
    mailbox_path.write_text(
        f"# Mailbox for {agent_id}\n\n",
        encoding="utf-8",
    )

registry_path.write_text(json.dumps(registry, indent=2) + "\n", encoding="utf-8")

result = {
    "status": "ok",
    "agent_id": agent_id,
    "role": entry["role"],
    "scope": entry["scope"],
    "mailbox": str(mailbox_path.relative_to(root_dir)),
    "confirmed_at": now,
}
print(json.dumps(result))
PY
)

if (( JSON_MODE )); then
  printf '%s\n' "$PYTHON_OUTPUT"
  exit 0
fi

START_ROLE=$(python3 -c 'import json,sys; data=json.loads(sys.argv[1]); print(data["role"])' "$PYTHON_OUTPUT")
START_SCOPE=$(python3 -c 'import json,sys; data=json.loads(sys.argv[1]); print(data["scope"])' "$PYTHON_OUTPUT")
START_MAILBOX=$(python3 -c 'import json,sys; data=json.loads(sys.argv[1]); print(data["mailbox"])' "$PYTHON_OUTPUT")
START_TIME=$(python3 -c 'import json,sys; data=json.loads(sys.argv[1]); print(data["confirmed_at"])' "$PYTHON_OUTPUT")

printf 'agent confirmed: %s\n' "$AGENT_ID"
printf 'role: %s\n' "$START_ROLE"
printf 'scope: %s\n' "$START_SCOPE"
printf 'mailbox: %s\n' "$START_MAILBOX"
printf 'confirmed_at: %s\n' "$START_TIME"
