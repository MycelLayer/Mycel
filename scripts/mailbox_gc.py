#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import re
import shutil
import sys
from collections.abc import Iterable
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any


ROOT_DIR = Path(__file__).resolve().parent.parent
REGISTRY_PATH = ROOT_DIR / ".agent-local" / "agents.json"
MAILBOX_DIR = ROOT_DIR / ".agent-local" / "mailboxes"
ARCHIVE_DIR = MAILBOX_DIR / "archive"
EXCLUDED_NAMES = {"EXAMPLE-planning-sync-handoff.md"}
TAIPEI_TIMEZONE = timezone(timedelta(hours=8))
DEFAULT_PRUNE_AGE_DAYS = 10


class MailboxGcError(Exception):
    pass


def format_timestamp(dt: datetime) -> str:
    return dt.astimezone(TAIPEI_TIMEZONE).replace(microsecond=0).strftime("%Y-%m-%dT%H:%M:%S%z")


def relative_to_root(path: Path) -> str:
    return str(path.relative_to(ROOT_DIR))


def load_registry() -> dict[str, Any]:
    try:
        payload = json.loads(REGISTRY_PATH.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise MailboxGcError(f"missing registry file: {REGISTRY_PATH}") from exc
    except json.JSONDecodeError as exc:
        raise MailboxGcError(f"invalid registry JSON: {exc}") from exc

    if not isinstance(payload, dict):
        raise MailboxGcError("invalid registry: top-level JSON value must be an object")
    agents = payload.get("agents")
    if not isinstance(agents, list):
        raise MailboxGcError("invalid registry: agents must be an array")
    return payload


def registry_mailboxes(registry: dict[str, Any]) -> dict[str, dict[str, Any]]:
    mapping: dict[str, dict[str, Any]] = {}
    for entry in registry["agents"]:
        if not isinstance(entry, dict):
            continue
        mailbox = entry.get("mailbox")
        if not isinstance(mailbox, str) or not mailbox.strip():
            continue
        mailbox_path = ROOT_DIR / mailbox
        mapping[str(mailbox_path.resolve())] = {
            "agent_uid": entry.get("agent_uid"),
            "status": entry.get("status"),
            "mailbox": mailbox,
        }
    return mapping


def live_mailbox_files() -> list[Path]:
    if not MAILBOX_DIR.exists():
        return []
    return sorted(
        path
        for path in MAILBOX_DIR.glob("*.md")
        if path.is_file() and path.name not in EXCLUDED_NAMES
    )


def archived_mailbox_files() -> list[Path]:
    if not ARCHIVE_DIR.exists():
        return []
    return sorted(path for path in ARCHIVE_DIR.rglob("*.md") if path.is_file())


def section_chunks(text: str) -> list[tuple[str, str]]:
    sections: list[tuple[str, str]] = []
    current_heading = ""
    current_lines: list[str] = []

    for line in text.splitlines():
        if line.startswith("## "):
            if current_heading or current_lines:
                sections.append((current_heading, "\n".join(current_lines)))
            current_heading = line[3:].strip()
            current_lines = []
            continue
        current_lines.append(line)

    if current_heading or current_lines:
        sections.append((current_heading, "\n".join(current_lines)))
    return sections


def mailbox_has_unresolved_planning_handoff(path: Path) -> bool:
    text = path.read_text(encoding="utf-8")
    planning_impact_pattern = re.compile(r"^- Planning impact:\s*(.+)$", re.MULTILINE | re.IGNORECASE)
    status_pattern = re.compile(r"^- Status:\s*(.+)$", re.MULTILINE | re.IGNORECASE)

    for heading, body in section_chunks(text):
        heading_lower = heading.lower()
        status_match = status_pattern.search(body)
        status = status_match.group(1).strip().lower() if status_match else ""
        planning_match = planning_impact_pattern.search(body)
        planning_value = planning_match.group(1).strip().lower() if planning_match else ""
        is_planning_section = "planning sync handoff" in heading_lower or (
            bool(planning_value) and planning_value != "`none`" and planning_value != "none"
        )
        if not is_planning_section:
            continue
        if status not in {"resolved", "superseded"}:
            return True

    return False


def mailbox_record(
    path: Path,
    *,
    extra: dict[str, Any] | None = None,
    include_planning_state: bool = False,
    now: datetime | None = None,
) -> dict[str, Any]:
    stat = path.stat()
    modified_at = datetime.fromtimestamp(stat.st_mtime, tz=timezone.utc)
    record = {
        "path": relative_to_root(path),
        "mtime": format_timestamp(modified_at),
        "size_bytes": stat.st_size,
    }
    if include_planning_state:
        current_time = now or datetime.now(timezone.utc)
        age_days = int((current_time - modified_at).total_seconds() // 86400)
        record["age_days"] = age_days
        record["has_unresolved_planning_handoff"] = mailbox_has_unresolved_planning_handoff(path)
    if extra:
        record.update(extra)
    return record


def prune_candidates_for_archived(
    archived_records: list[dict[str, Any]], *, min_age_days: int
) -> list[dict[str, Any]]:
    return [
        record
        for record in archived_records
        if record["age_days"] >= min_age_days and not record["has_unresolved_planning_handoff"]
    ]


def scan_mailboxes(*, prune_age_days: int = DEFAULT_PRUNE_AGE_DAYS) -> dict[str, Any]:
    registry = load_registry()
    referenced = registry_mailboxes(registry)
    live_files = live_mailbox_files()
    archived_files = archived_mailbox_files()
    now = datetime.now(timezone.utc)

    referenced_existing: list[dict[str, Any]] = []
    missing_referenced: list[dict[str, Any]] = []
    orphaned: list[dict[str, Any]] = []
    archived = [mailbox_record(path, include_planning_state=True, now=now) for path in archived_files]
    prune_candidates = prune_candidates_for_archived(archived, min_age_days=prune_age_days)

    live_by_resolved = {str(path.resolve()): path for path in live_files}

    for resolved_path, entry in sorted(referenced.items(), key=lambda item: item[1]["mailbox"]):
        live_path = live_by_resolved.get(resolved_path)
        if live_path is None:
            missing_referenced.append(
                {
                    "mailbox": entry["mailbox"],
                    "agent_uid": entry["agent_uid"],
                    "status": entry["status"],
                }
            )
            continue
        referenced_existing.append(
            mailbox_record(
                live_path,
                extra={
                    "agent_uid": entry["agent_uid"],
                    "status": entry["status"],
                },
            )
        )

    for path in live_files:
        resolved_path = str(path.resolve())
        if resolved_path in referenced:
            continue
        orphaned.append(mailbox_record(path))

    return {
        "status": "ok",
        "mailbox_dir": relative_to_root(MAILBOX_DIR),
        "archive_dir": relative_to_root(ARCHIVE_DIR),
        "referenced_count": len(referenced_existing),
        "missing_referenced_count": len(missing_referenced),
        "orphaned_count": len(orphaned),
        "archived_count": len(archived),
        "prune_candidate_count": len(prune_candidates),
        "prune_age_days": prune_age_days,
        "referenced": referenced_existing,
        "missing_referenced": missing_referenced,
        "orphaned": orphaned,
        "archived": archived,
        "prune_candidates": prune_candidates,
    }


def archive_bucket(now: datetime) -> Path:
    return ARCHIVE_DIR / now.astimezone(TAIPEI_TIMEZONE).strftime("%Y-%m")


def unique_destination(dest_dir: Path, filename: str) -> Path:
    candidate = dest_dir / filename
    if not candidate.exists():
        return candidate

    stem = Path(filename).stem
    suffix = Path(filename).suffix
    index = 1
    while True:
        candidate = dest_dir / f"{stem}-{index}{suffix}"
        if not candidate.exists():
            return candidate
        index += 1


def archive_mailboxes(*, dry_run: bool) -> dict[str, Any]:
    scan = scan_mailboxes()
    now = datetime.now(timezone.utc)
    dest_dir = archive_bucket(now)
    archived: list[dict[str, Any]] = []

    for record in scan["orphaned"]:
        src = ROOT_DIR / record["path"]
        dest = unique_destination(dest_dir, src.name)
        archived.append(
            {
                "source": record["path"],
                "destination": relative_to_root(dest),
            }
        )
        if dry_run:
            continue
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.move(str(src), str(dest))

    result = {
        "status": "ok",
        "dry_run": dry_run,
        "archive_dir": relative_to_root(dest_dir),
        "archived_count": len(archived),
        "archived": archived,
    }
    return result


def prune_archived_mailboxes(*, dry_run: bool, min_age_days: int) -> dict[str, Any]:
    scan = scan_mailboxes(prune_age_days=min_age_days)
    deleted: list[dict[str, Any]] = []

    for record in scan["prune_candidates"]:
        path = ROOT_DIR / record["path"]
        deleted.append(
            {
                "path": record["path"],
                "age_days": record["age_days"],
            }
        )
        if dry_run:
            continue
        path.unlink()

    return {
        "status": "ok",
        "dry_run": dry_run,
        "min_age_days": min_age_days,
        "deleted_count": len(deleted),
        "deleted": deleted,
    }


def print_scan(data: dict[str, Any]) -> None:
    print(f"mailbox_dir: {data['mailbox_dir']}")
    print(f"archive_dir: {data['archive_dir']}")
    print(f"referenced_mailboxes: {data['referenced_count']}")
    print(f"missing_referenced_mailboxes: {data['missing_referenced_count']}")
    print(f"orphaned_mailboxes: {data['orphaned_count']}")
    print(f"archived_mailboxes: {data['archived_count']}")
    print(f"prune_candidates: {data['prune_candidate_count']}")
    print(f"prune_age_days: {data['prune_age_days']}")
    if data["missing_referenced"]:
        print("missing_referenced:")
        for record in data["missing_referenced"]:
            print(f"  - {record['mailbox']} ({record['agent_uid']}, {record['status']})")
    if data["orphaned"]:
        print("orphaned:")
        for record in data["orphaned"]:
            print(f"  - {record['path']}")
    if data["prune_candidates"]:
        print("prune_candidates:")
        for record in data["prune_candidates"]:
            print(f"  - {record['path']} ({record['age_days']} days)")


def print_archive(data: dict[str, Any]) -> None:
    print(f"archive_dir: {data['archive_dir']}")
    print(f"dry_run: {data['dry_run']}")
    print(f"archived_mailboxes: {data['archived_count']}")
    for record in data["archived"]:
        print(f"- {record['source']} -> {record['destination']}")


def print_prune(data: dict[str, Any]) -> None:
    print(f"dry_run: {data['dry_run']}")
    print(f"min_age_days: {data['min_age_days']}")
    print(f"deleted_mailboxes: {data['deleted_count']}")
    for record in data["deleted"]:
        print(f"- {record['path']} ({record['age_days']} days)")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="scripts/mailbox_gc.py",
        description="Inspect and archive orphaned uid-based agent mailboxes.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    scan = subparsers.add_parser("scan", add_help=False)
    scan.add_argument("--prune-age-days", type=int, default=DEFAULT_PRUNE_AGE_DAYS)
    scan.add_argument("--json", action="store_true")
    scan.add_argument("-h", "--help", action="help")
    scan.set_defaults(func=cmd_scan)

    archive = subparsers.add_parser("archive", add_help=False)
    archive.add_argument("--dry-run", action="store_true")
    archive.add_argument("--json", action="store_true")
    archive.add_argument("-h", "--help", action="help")
    archive.set_defaults(func=cmd_archive)

    prune = subparsers.add_parser("prune", add_help=False)
    prune.add_argument("--dry-run", action="store_true")
    prune.add_argument("--min-age-days", type=int, default=DEFAULT_PRUNE_AGE_DAYS)
    prune.add_argument("--json", action="store_true")
    prune.add_argument("-h", "--help", action="help")
    prune.set_defaults(func=cmd_prune)

    return parser


def cmd_scan(args: argparse.Namespace) -> int:
    result = scan_mailboxes(prune_age_days=args.prune_age_days)
    if args.json:
        print(json.dumps(result))
    else:
        print_scan(result)
    return 0


def cmd_archive(args: argparse.Namespace) -> int:
    result = archive_mailboxes(dry_run=args.dry_run)
    if args.json:
        print(json.dumps(result))
    else:
        print_archive(result)
    return 0


def cmd_prune(args: argparse.Namespace) -> int:
    result = prune_archived_mailboxes(dry_run=args.dry_run, min_age_days=args.min_age_days)
    if args.json:
        print(json.dumps(result))
    else:
        print_prune(result)
    return 0


def main(argv: Iterable[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(list(argv) if argv is not None else None)
    try:
        return args.func(args)
    except MailboxGcError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
