#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
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


def mailbox_record(path: Path, *, extra: dict[str, Any] | None = None) -> dict[str, Any]:
    stat = path.stat()
    record = {
        "path": relative_to_root(path),
        "mtime": format_timestamp(datetime.fromtimestamp(stat.st_mtime, tz=timezone.utc)),
        "size_bytes": stat.st_size,
    }
    if extra:
        record.update(extra)
    return record


def scan_mailboxes() -> dict[str, Any]:
    registry = load_registry()
    referenced = registry_mailboxes(registry)
    live_files = live_mailbox_files()
    archived_files = archived_mailbox_files()

    referenced_existing: list[dict[str, Any]] = []
    missing_referenced: list[dict[str, Any]] = []
    orphaned: list[dict[str, Any]] = []

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
        "archived_count": len(archived_files),
        "referenced": referenced_existing,
        "missing_referenced": missing_referenced,
        "orphaned": orphaned,
        "archived": [mailbox_record(path) for path in archived_files],
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


def print_scan(data: dict[str, Any]) -> None:
    print(f"mailbox_dir: {data['mailbox_dir']}")
    print(f"archive_dir: {data['archive_dir']}")
    print(f"referenced_mailboxes: {data['referenced_count']}")
    print(f"missing_referenced_mailboxes: {data['missing_referenced_count']}")
    print(f"orphaned_mailboxes: {data['orphaned_count']}")
    print(f"archived_mailboxes: {data['archived_count']}")
    if data["missing_referenced"]:
        print("missing_referenced:")
        for record in data["missing_referenced"]:
            print(f"  - {record['mailbox']} ({record['agent_uid']}, {record['status']})")
    if data["orphaned"]:
        print("orphaned:")
        for record in data["orphaned"]:
            print(f"  - {record['path']}")


def print_archive(data: dict[str, Any]) -> None:
    print(f"archive_dir: {data['archive_dir']}")
    print(f"dry_run: {data['dry_run']}")
    print(f"archived_mailboxes: {data['archived_count']}")
    for record in data["archived"]:
        print(f"- {record['source']} -> {record['destination']}")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="scripts/mailbox_gc.py",
        description="Inspect and archive orphaned uid-based agent mailboxes.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    scan = subparsers.add_parser("scan", add_help=False)
    scan.add_argument("--json", action="store_true")
    scan.add_argument("-h", "--help", action="help")
    scan.set_defaults(func=cmd_scan)

    archive = subparsers.add_parser("archive", add_help=False)
    archive.add_argument("--dry-run", action="store_true")
    archive.add_argument("--json", action="store_true")
    archive.add_argument("-h", "--help", action="help")
    archive.set_defaults(func=cmd_archive)

    return parser


def cmd_scan(args: argparse.Namespace) -> int:
    result = scan_mailboxes()
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
