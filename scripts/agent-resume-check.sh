#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Check whether a reopened chat may safely resume tracked work under its existing agent id.

Usage:
  scripts/agent-resume-check.sh <agent-id> [--json]

Examples:
  scripts/agent-resume-check.sh coding-2
  scripts/agent-resume-check.sh doc-1 --json

Behavior:
  - reads .agent-local/agents.json
  - validates the selected agent entry
  - succeeds only when the agent is still confirmed and active
  - fails closed if the old chat must stop instead of resuming work
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

set +e
PYTHON_OUTPUT=$(
  AGENT_ID="$AGENT_ID" REGISTRY_PATH="$REGISTRY_PATH" ROOT_DIR="$ROOT_DIR" python3 <<'PY'
import json
import os
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

selected = [entry for entry in agents if isinstance(entry, dict) and entry.get("id") == agent_id]
if not selected:
    raise SystemExit(f"agent entry not found: {agent_id}")
if len(selected) > 1:
    raise SystemExit(f"invalid registry: duplicate agent id {agent_id}")

entry = selected[0]
role = entry.get("role")
status = entry.get("status")
scope = entry.get("scope")
confirmed_by_agent = entry.get("confirmed_by_agent", False)
confirmed_at = entry.get("confirmed_at")
mailbox_value = entry.get("mailbox")

if not isinstance(role, str) or not role.strip():
    raise SystemExit(f"agent {agent_id} is missing required field: role")
if not isinstance(status, str) or not status.strip():
    raise SystemExit(f"agent {agent_id} is missing required field: status")
if not isinstance(scope, str) or not scope.strip():
    raise SystemExit(f"agent {agent_id} is missing required field: scope")
if not isinstance(mailbox_value, str) or not mailbox_value.strip():
    raise SystemExit(f"agent {agent_id} is missing required field: mailbox")

mailbox_path = Path(mailbox_value)
if not mailbox_path.is_absolute():
    mailbox_path = root_dir / mailbox_path
mailbox_display = str(mailbox_path.relative_to(root_dir))

safe_to_resume = True
reason = "agent is still active and confirmed"
exit_code = 0

if status != "active":
    safe_to_resume = False
    reason = f"agent status is {status}; do not resume tracked work under {agent_id}"
    exit_code = 2
elif confirmed_by_agent is not True or not isinstance(confirmed_at, str) or not confirmed_at.strip():
    safe_to_resume = False
    reason = f"agent {agent_id} is not fully confirmed; do not resume tracked work"
    exit_code = 2

result = {
    "status": "ok" if safe_to_resume else "stop",
    "agent_id": agent_id,
    "role": role,
    "current_status": status,
    "scope": scope,
    "confirmed_by_agent": bool(confirmed_by_agent),
    "confirmed_at": confirmed_at,
    "mailbox": mailbox_display,
    "safe_to_resume": safe_to_resume,
    "reason": reason,
}
print(json.dumps(result))
raise SystemExit(exit_code)
PY
)
PYTHON_EXIT=$?
set -e

if (( JSON_MODE )); then
  printf '%s\n' "$PYTHON_OUTPUT"
  exit "$PYTHON_EXIT"
fi

python3 -c '
import json
import sys

data = json.loads(sys.argv[1])
print(f"agent_id: {data['\''agent_id'\'']}")
print(f"role: {data['\''role'\'']}")
print(f"current_status: {data['\''current_status'\'']}")
print(f"scope: {data['\''scope'\'']}")
print(f"confirmed_by_agent: {data['\''confirmed_by_agent'\'']}")
print(f"confirmed_at: {data['\''confirmed_at'\'']}")
print(f"mailbox: {data['\''mailbox'\'']}")
print(f"safe_to_resume: {data['\''safe_to_resume'\'']}")
print(f"reason: {data['\''reason'\'']}")
' "$PYTHON_OUTPUT"

exit "$PYTHON_EXIT"
