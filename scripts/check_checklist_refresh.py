#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

from item_id_checklist import (
    AGENTS_BOOTSTRAP_TITLE,
    AGENTS_WORKCYCLE_TITLE,
    CHECKLIST_ITEM_RE,
    HEADING_RE,
    ItemIdChecklistError,
    build_agents_section_body,
    collect_relevant_lines,
    relative_to_root,
    render_checklist_document,
    resolve_path,
    split_heading_blocks,
)


SOURCE_LINE_RE = re.compile(r"^- Source: `(?P<source>.+)`$")
AGENT_UID_LINE_RE = re.compile(r"^- Agent UID: `(?P<agent_uid>.+)`$")
DISPLAY_ID_LINE_RE = re.compile(r"^- Display ID: `(?P<display_id>.+)`$")


class ChecklistRefreshCheckError(Exception):
    pass


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="scripts/check_checklist_refresh.py",
        description="Report whether an agent-local checklist copy needs to be refreshed from its source.",
    )
    parser.add_argument("checklist_md", help="existing agent-local checklist copy")
    parser.add_argument("--json", action="store_true", help="emit JSON instead of human-readable lines")
    return parser.parse_args()


def extract_metadata(checklist_path: Path) -> tuple[str, str | None, Path]:
    lines = checklist_path.read_text(encoding="utf-8").splitlines()
    agent_uid = None
    display_id = None
    source_rel = None

    for line in lines[:12]:
        if agent_uid is None:
            match = AGENT_UID_LINE_RE.match(line)
            if match:
                agent_uid = match.group("agent_uid")
                continue
        if display_id is None:
            match = DISPLAY_ID_LINE_RE.match(line)
            if match:
                value = match.group("display_id")
                display_id = None if value == "none" else value
                continue
        if source_rel is None:
            match = SOURCE_LINE_RE.match(line)
            if match:
                source_rel = match.group("source")

    if agent_uid is None or source_rel is None:
        raise ChecklistRefreshCheckError(
            f"could not read checklist metadata from {relative_to_root(checklist_path)}"
        )
    return agent_uid, display_id, resolve_path(source_rel)


def classify_agents_section(checklist_path: Path) -> str:
    name = checklist_path.name
    if name.endswith("-bootstrap-checklist.md"):
        return "bootstrap"
    if re.fullmatch(r".+-workcycle-checklist-\d+\.md", name):
        return "workcycle"
    raise ChecklistRefreshCheckError(
        "split checklist must be *-bootstrap-checklist.md or *-workcycle-checklist-<n>.md"
    )


def render_expected_document(
    *,
    checklist_path: Path,
    agent_uid: str,
    display_id: str | None,
    source_path: Path,
) -> str:
    source_text = source_path.read_text(encoding="utf-8")
    normalized_lines, item_count = collect_relevant_lines(source_text.splitlines())
    if item_count == 0:
        raise ChecklistRefreshCheckError(f"source file has no item-id markers: {relative_to_root(source_path)}")

    if source_path.name == "AGENTS.md":
        section = classify_agents_section(checklist_path)
        root_lines, source_blocks = split_heading_blocks(normalized_lines)
        source_block_map = {title: lines for title, lines in source_blocks}
        if section == "bootstrap":
            block = source_block_map.get(AGENTS_BOOTSTRAP_TITLE)
            if block is None:
                raise ChecklistRefreshCheckError("AGENTS.md source is missing the New chat bootstrap section")
        else:
            block = source_block_map.get(AGENTS_WORKCYCLE_TITLE)
            if block is None:
                raise ChecklistRefreshCheckError("AGENTS.md source is missing the Work Cycle Workflow section")
        body_lines = build_agents_section_body(root_lines, block)
    else:
        body_lines = normalized_lines

    return render_checklist_document(
        agent_uid=agent_uid,
        display_id=display_id,
        source_path=source_path,
        body_lines=body_lines,
        generated_at="CHECK",
    )


def normalize_for_comparison(text: str) -> list[str]:
    normalized: list[str] = []
    for line in text.splitlines():
        if line.startswith("- Generated at: "):
            continue
        if line.startswith("  - Problem: "):
            continue
        match = CHECKLIST_ITEM_RE.match(line)
        if match is not None:
            normalized.append(line.replace(f"[{match.group('mark')}]", "[ ]", 1))
            continue
        normalized.append(line)
    while normalized and normalized[-1] == "":
        normalized.pop()
    return normalized


def item_id_set(text: str) -> set[str]:
    return {
        match.group("item_id").strip()
        for line in text.splitlines()
        if (match := CHECKLIST_ITEM_RE.match(line)) is not None
    }


def compare_documents(actual_text: str, expected_text: str) -> dict[str, object]:
    actual_normalized = normalize_for_comparison(actual_text)
    expected_normalized = normalize_for_comparison(expected_text)
    actual_ids = item_id_set(actual_text)
    expected_ids = item_id_set(expected_text)

    missing = sorted(expected_ids - actual_ids)
    stale = sorted(actual_ids - expected_ids)
    content_changed = actual_normalized != expected_normalized

    reasons: list[str] = []
    if missing:
        reasons.append("new item-id entries exist in the source")
    if stale:
        reasons.append("checklist still contains item-id entries removed from the source")
    if content_changed and not missing and not stale:
        reasons.append("source checklist wording or structure changed")
    status = "refresh-needed" if reasons else "up-to-date"

    return {
        "status": status,
        "reasons": reasons,
        "missing_item_ids": missing,
        "stale_item_ids": stale,
    }


def print_human(payload: dict[str, object]) -> None:
    print(f"path: {payload['checklist']}")
    print(f"source: {payload['source']}")
    print(f"status: {payload['status']}")
    reasons = payload.get("reasons", [])
    if isinstance(reasons, list) and reasons:
        print("reasons:")
        for reason in reasons:
            print(f"  - {reason}")


def main() -> int:
    args = parse_args()
    try:
        checklist_path = resolve_path(args.checklist_md).resolve()
        if not checklist_path.exists():
            raise ChecklistRefreshCheckError(f"checklist file not found: {relative_to_root(checklist_path)}")
        agent_uid, display_id, source_path = extract_metadata(checklist_path)
        expected_text = render_expected_document(
            checklist_path=checklist_path,
            agent_uid=agent_uid,
            display_id=display_id,
            source_path=source_path,
        )
        actual_text = checklist_path.read_text(encoding="utf-8")
        comparison = compare_documents(actual_text, expected_text)
    except (ChecklistRefreshCheckError, ItemIdChecklistError) as exc:
        print(str(exc), file=sys.stderr)
        return 1

    payload = {
        "checklist": relative_to_root(checklist_path),
        "source": relative_to_root(source_path),
        **comparison,
    }
    if args.json:
        print(json.dumps(payload))
    else:
        print_human(payload)
    return 0


if __name__ == "__main__":
    sys.exit(main())
