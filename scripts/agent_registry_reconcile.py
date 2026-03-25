#!/usr/bin/env python3
"""Reconcile stale active registry entries using persisted Codex liveness evidence."""

from __future__ import annotations

import argparse
import json
import sqlite3
from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import Any


ROOT_DIR = Path(__file__).resolve().parent.parent
DEFAULT_REGISTRY = ROOT_DIR / ".agent-local" / "agents.json"
DEFAULT_CODEX_HOME = Path.home() / ".codex"
TAIPEI_OFFSET = timedelta(hours=8)


class ReconcileError(Exception):
    pass


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="scripts/agent_registry_reconcile.py",
        description=(
            "Inspect active registry entries for stale liveness evidence and "
            "optionally downgrade them to inactive or paused."
        ),
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    for name in ("scan", "reconcile"):
        sub = subparsers.add_parser(name, add_help=False)
        sub.add_argument("--registry", default=str(DEFAULT_REGISTRY), help="registry JSON path")
        sub.add_argument("--codex-home", default=str(DEFAULT_CODEX_HOME), help="Codex home directory")
        sub.add_argument("--state-db", help="Explicit state_*.sqlite path override")
        sub.add_argument(
            "--stale-after-minutes",
            type=int,
            default=15,
            help="mark an active agent stale when its newest evidence is older than this many minutes",
        )
        sub.add_argument(
            "--downgrade-to",
            choices=("inactive", "paused"),
            default="inactive",
            help="target status when reconcile applies a stale-active downgrade",
        )
        sub.add_argument("--json", action="store_true", help="emit JSON")
        sub.add_argument("-h", "--help", action="help")

    return parser.parse_args()


def parse_timestamp(value: Any) -> datetime | None:
    if isinstance(value, (int, float)):
        if value > 1_000_000_000_000:
            return datetime.fromtimestamp(value / 1000, UTC)
        return datetime.fromtimestamp(value, UTC)
    if not isinstance(value, str):
        return None
    text = value.strip()
    if not text:
        return None
    if text.endswith("Z"):
        text = text[:-1] + "+00:00"
    if text.endswith("+0800"):
        text = text[:-5] + "+08:00"
    try:
        parsed = datetime.fromisoformat(text)
    except ValueError:
        return None
    if parsed.tzinfo is None:
        return parsed.replace(tzinfo=UTC)
    return parsed.astimezone(UTC)


def format_taipei_timestamp(ts: datetime) -> str:
    local = ts.astimezone(UTC) + TAIPEI_OFFSET
    return local.strftime("%Y-%m-%dT%H:%M:%S+0800")


def load_registry(path: Path) -> dict[str, Any]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise ReconcileError(f"registry not found: {path}") from exc
    except json.JSONDecodeError as exc:
        raise ReconcileError(f"invalid registry JSON: {path}") from exc
    if not isinstance(payload, dict):
        raise ReconcileError(f"registry must contain a top-level object: {path}")
    return payload


def save_registry(path: Path, payload: dict[str, Any]) -> None:
    path.write_text(json.dumps(payload, ensure_ascii=True, indent=2) + "\n", encoding="utf-8")


def discover_state_db(codex_home: Path) -> Path | None:
    candidates: list[tuple[int, Path]] = []
    for path in codex_home.glob("state_*.sqlite"):
        suffix = path.stem.removeprefix("state_")
        if not suffix.isdigit():
            continue
        candidates.append((int(suffix), path))
    if not candidates:
        return None
    candidates.sort(key=lambda item: (item[0], item[1].name))
    return candidates[-1][1]


def latest_snapshot_for_agent(agent_uid: str) -> dict[str, Any] | None:
    workcycles_dir = ROOT_DIR / ".agent-local" / "agents" / agent_uid / "workcycles"
    if not workcycles_dir.exists():
        return None
    latest_payload: dict[str, Any] | None = None
    latest_time: datetime | None = None
    for path in sorted(workcycles_dir.glob("token-usage*.json")):
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            continue
        if not isinstance(payload, dict):
            continue
        snapshot_time = parse_timestamp(payload.get("timestamp"))
        if snapshot_time is None:
            snapshot_time = datetime.fromtimestamp(path.stat().st_mtime, UTC)
        if latest_time is None or snapshot_time > latest_time:
            latest_time = snapshot_time
            latest_payload = payload
    return latest_payload


def load_thread_updated_at(state_db: Path | None, thread_id: str | None) -> tuple[datetime | None, str | None]:
    if state_db is None or thread_id is None:
        return None, None
    if not state_db.exists():
        return None, f"state db not found: {state_db}"
    query = "SELECT updated_at FROM threads WHERE id = ?"
    try:
        with sqlite3.connect(state_db) as conn:
            row = conn.execute(query, (thread_id,)).fetchone()
    except sqlite3.Error as exc:
        return None, f"sqlite query failed: {exc}"
    if row is None:
        return None, None
    return parse_timestamp(row[0]), None


def evidence_for_agent(entry: dict[str, Any], *, state_db: Path | None) -> dict[str, Any]:
    agent_uid = entry.get("agent_uid")
    if not isinstance(agent_uid, str) or not agent_uid.strip():
        raise ReconcileError("registry entry missing agent_uid")

    registry_touched = parse_timestamp(entry.get("last_touched_at"))
    snapshot = latest_snapshot_for_agent(agent_uid)
    snapshot_time = parse_timestamp(snapshot.get("timestamp")) if isinstance(snapshot, dict) else None
    thread_id = snapshot.get("thread_id") if isinstance(snapshot, dict) and isinstance(snapshot.get("thread_id"), str) else None
    rollout_path = Path(snapshot["rollout_path"]) if isinstance(snapshot, dict) and isinstance(snapshot.get("rollout_path"), str) else None
    rollout_time = None
    if rollout_path is not None and rollout_path.exists():
        rollout_time = datetime.fromtimestamp(rollout_path.stat().st_mtime, UTC)
    sqlite_time, sqlite_error = load_thread_updated_at(state_db, thread_id)

    candidates = [
        ("registry_last_touched_at", registry_touched),
        ("workcycle_snapshot", snapshot_time),
        ("rollout_mtime", rollout_time),
        ("sqlite_thread_updated_at", sqlite_time),
    ]
    freshest_source = None
    freshest_time = None
    for source, ts in candidates:
        if ts is None:
            continue
        if freshest_time is None or ts > freshest_time:
            freshest_time = ts
            freshest_source = source

    return {
        "agent_uid": agent_uid,
        "thread_id": thread_id,
        "state_db": None if state_db is None else str(state_db),
        "snapshot_timestamp": None if snapshot_time is None else snapshot_time.isoformat(),
        "rollout_path": None if rollout_path is None else str(rollout_path),
        "rollout_mtime": None if rollout_time is None else rollout_time.isoformat(),
        "sqlite_thread_updated_at": None if sqlite_time is None else sqlite_time.isoformat(),
        "sqlite_error": sqlite_error,
        "freshest_source": freshest_source,
        "freshest_timestamp": None if freshest_time is None else freshest_time.isoformat(),
    }


def scan_registry(
    payload: dict[str, Any], *, state_db: Path | None, stale_after_minutes: int
) -> list[dict[str, Any]]:
    now = datetime.now(UTC)
    agents = payload.get("agents")
    if not isinstance(agents, list):
        raise ReconcileError("registry agents field must be a list")

    report: list[dict[str, Any]] = []
    threshold = timedelta(minutes=stale_after_minutes)
    for entry in agents:
        if not isinstance(entry, dict):
            continue
        if entry.get("status") != "active":
            continue
        evidence = evidence_for_agent(entry, state_db=state_db)
        freshest_text = evidence.get("freshest_timestamp")
        freshest = parse_timestamp(freshest_text)
        stale = freshest is None or (now - freshest) > threshold
        report.append(
            {
                "agent_uid": entry.get("agent_uid"),
                "display_id": entry.get("current_display_id"),
                "role": entry.get("role"),
                "scope": entry.get("scope"),
                "registry_status": entry.get("status"),
                "stale_active": stale,
                "age_minutes": None if freshest is None else round((now - freshest).total_seconds() / 60, 1),
                "evidence": evidence,
            }
        )
    return report


def apply_reconcile(
    payload: dict[str, Any],
    *,
    state_db: Path | None,
    stale_after_minutes: int,
    downgrade_to: str,
) -> dict[str, Any]:
    report = scan_registry(payload, state_db=state_db, stale_after_minutes=stale_after_minutes)
    stale_uids = {item["agent_uid"] for item in report if item.get("stale_active")}
    if not stale_uids:
        return {"updated": 0, "agents": report}

    now = datetime.now(UTC)
    agents = payload.get("agents")
    assert isinstance(agents, list)
    updated = 0
    for entry in agents:
        if not isinstance(entry, dict):
            continue
        agent_uid = entry.get("agent_uid")
        if agent_uid not in stale_uids:
            continue
        entry["status"] = downgrade_to
        if downgrade_to == "inactive":
            entry["inactive_at"] = format_taipei_timestamp(now)
            entry["paused_at"] = None
        else:
            entry["paused_at"] = format_taipei_timestamp(now)
            entry["inactive_at"] = None
        updated += 1
    payload["updated_at"] = format_taipei_timestamp(now)
    payload["agent_count"] = len(agents)
    return {"updated": updated, "agents": report}


def print_text_scan(report: list[dict[str, Any]]) -> None:
    if not report:
        print("no active agents found")
        return
    for item in report:
        state = "stale-active" if item.get("stale_active") else "active-ok"
        agent_uid = item.get("agent_uid")
        display_id = item.get("display_id")
        role = item.get("role")
        age = item.get("age_minutes")
        print(f"{display_id or agent_uid} ({role}): {state}")
        if age is not None:
            print(f"  age_minutes: {age}")
        evidence = item.get("evidence", {})
        if isinstance(evidence, dict):
            print(f"  freshest_source: {evidence.get('freshest_source')}")
            print(f"  freshest_timestamp: {evidence.get('freshest_timestamp')}")
            if evidence.get("thread_id") is not None:
                print(f"  thread_id: {evidence.get('thread_id')}")
            if evidence.get("sqlite_error") is not None:
                print(f"  sqlite_error: {evidence.get('sqlite_error')}")


def main() -> int:
    args = parse_args()
    registry_path = Path(args.registry).expanduser()
    codex_home = Path(args.codex_home).expanduser()
    state_db = Path(args.state_db).expanduser() if args.state_db else discover_state_db(codex_home)
    payload = load_registry(registry_path)

    if args.command == "scan":
        report = scan_registry(payload, state_db=state_db, stale_after_minutes=args.stale_after_minutes)
        if args.json:
            print(json.dumps({"agents": report}, ensure_ascii=True, indent=2))
        else:
            print_text_scan(report)
        return 0

    result = apply_reconcile(
        payload,
        state_db=state_db,
        stale_after_minutes=args.stale_after_minutes,
        downgrade_to=args.downgrade_to,
    )
    if result["updated"]:
        save_registry(registry_path, payload)
    if args.json:
        print(json.dumps(result, ensure_ascii=True, indent=2))
    else:
        print(f"updated: {result['updated']}")
        print_text_scan(result["agents"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
