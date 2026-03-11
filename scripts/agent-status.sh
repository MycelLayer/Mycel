#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Show local agent registry status.

Usage:
  scripts/agent-status.sh [<agent-id>] [--json]

Examples:
  scripts/agent-status.sh
  scripts/agent-status.sh coding-1
  scripts/agent-status.sh --json
  scripts/agent-status.sh doc-1 --json

Behavior:
  - reads .agent-local/agents.json
  - validates top-level registry structure
  - prints all agent entries or one selected agent
  - shows whether each agent assignment is confirmed

This command does not modify the registry.
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

selected = agents
if agent_id:
    selected = [entry for entry in agents if isinstance(entry, dict) and entry.get("id") == agent_id]
    if not selected:
        raise SystemExit(f"agent entry not found: {agent_id}")
    if len(selected) > 1:
        raise SystemExit(f"invalid registry: duplicate agent id {agent_id}")

normalized = []
for entry in selected:
    if not isinstance(entry, dict):
        raise SystemExit("invalid registry: agent entry must be an object")

    mailbox_value = entry.get("mailbox")
    mailbox_exists = False
    mailbox_display = None
    if isinstance(mailbox_value, str) and mailbox_value.strip():
        mailbox_path = Path(mailbox_value)
        if not mailbox_path.is_absolute():
            mailbox_path = root_dir / mailbox_path
        mailbox_exists = mailbox_path.exists()
        mailbox_display = str(mailbox_path.relative_to(root_dir))

    normalized.append(
        {
            "id": entry.get("id"),
            "role": entry.get("role"),
            "status": entry.get("status"),
            "scope": entry.get("scope"),
            "assigned_by": entry.get("assigned_by"),
            "assigned_at": entry.get("assigned_at"),
            "confirmed_by_agent": entry.get("confirmed_by_agent", False),
            "confirmed_at": entry.get("confirmed_at"),
            "files": entry.get("files", []),
            "mailbox": mailbox_display,
            "mailbox_exists": mailbox_exists,
        }
    )

result = {
    "status": "ok",
    "registry_path": str(registry_path),
    "updated_at": registry.get("updated_at"),
    "agent_count": len(normalized) if agent_id else len(agents),
    "agents": normalized,
}
print(json.dumps(result))
PY
)

if (( JSON_MODE )); then
  printf '%s\n' "$PYTHON_OUTPUT"
  exit 0
fi

python3 -c '
import json
import sys

data = json.loads(sys.argv[1])
print(f"registry: {data['\''registry_path'\'']}")
print(f"updated_at: {data['\''updated_at'\'']}")
print(f"agents: {data['\''agent_count'\'']}")
for entry in data["agents"]:
    print(f"id: {entry['\''id'\'']}")
    print(f"  role: {entry['\''role'\'']}")
    print(f"  status: {entry['\''status'\'']}")
    print(f"  scope: {entry['\''scope'\'']}")
    print(f"  assigned_by: {entry['\''assigned_by'\'']}")
    print(f"  assigned_at: {entry['\''assigned_at'\'']}")
    print(f"  confirmed_by_agent: {entry['\''confirmed_by_agent'\'']}")
    print(f"  confirmed_at: {entry['\''confirmed_at'\'']}")
    print(f"  mailbox: {entry['\''mailbox'\'']}")
    print(f"  mailbox_exists: {entry['\''mailbox_exists'\'']}")
    files = entry.get("files", [])
    if files:
        print("  files:")
        for path in files:
            print(f"    - {path}")
    else:
        print("  files: []")
' "$PYTHON_OUTPUT"
