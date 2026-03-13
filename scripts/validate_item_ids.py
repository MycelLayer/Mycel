#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any


ROOT_DIR = Path(__file__).resolve().parent.parent
ITEM_ID_COMMENT_RE = re.compile(r"<!--\s*item-id:\s*(?P<item_id>.*?)\s*-->")
ITEM_ID_VALUE_RE = re.compile(r"^[a-z0-9]+(?:[.-][a-z0-9]+)*$")
LIST_ITEM_WITH_ITEM_ID_RE = re.compile(
    r"^\s*(?:[-*+]\s|\d+\.\s).+<!--\s*item-id:\s*(?P<item_id>.*?)\s*-->\s*$"
)


class ItemIdValidationError(Exception):
    pass


def relative_to_root(path: Path) -> str:
    try:
        return str(path.relative_to(ROOT_DIR))
    except ValueError:
        return str(path)


def resolve_path(path_value: str) -> Path:
    candidate = Path(path_value)
    if not candidate.is_absolute():
        candidate = ROOT_DIR / candidate
    return candidate


def line_issue(path: Path, line_no: int, message: str) -> dict[str, Any]:
    return {
        "path": relative_to_root(path),
        "line": line_no,
        "message": message,
    }


def validate_file(path: Path) -> dict[str, Any]:
    if not path.exists():
        raise ItemIdValidationError(f"file not found: {relative_to_root(path)}")
    if not path.is_file():
        raise ItemIdValidationError(f"not a file: {relative_to_root(path)}")

    text = path.read_text(encoding="utf-8")
    issues: list[dict[str, Any]] = []
    seen: dict[str, int] = {}
    item_count = 0

    for line_no, line in enumerate(text.splitlines(), start=1):
        matches = list(ITEM_ID_COMMENT_RE.finditer(line))
        if not matches:
            continue

        for match in matches:
            item_count += 1
            item_id = match.group("item_id").strip()
            if not LIST_ITEM_WITH_ITEM_ID_RE.match(line):
                issues.append(line_issue(path, line_no, "item-id must be attached to a Markdown list item line"))
            if not item_id:
                issues.append(line_issue(path, line_no, "item-id must not be empty"))
                continue
            if not ITEM_ID_VALUE_RE.fullmatch(item_id):
                issues.append(
                    line_issue(
                        path,
                        line_no,
                        f"invalid item-id '{item_id}'; use lowercase letters, numbers, dots, and hyphens",
                    )
                )
                continue
            if item_id in seen:
                issues.append(
                    line_issue(
                        path,
                        line_no,
                        f"duplicate item-id '{item_id}' first seen on line {seen[item_id]}",
                    )
                )
            else:
                seen[item_id] = line_no

    if item_count == 0:
        issues.append(line_issue(path, 1, "no item-id markers found"))

    return {
        "path": relative_to_root(path),
        "item_count": item_count,
        "valid": not issues,
        "issues": issues,
    }


def print_human(results: list[dict[str, Any]]) -> None:
    for result in results:
        print(f"path: {result['path']}")
        print(f"valid: {result['valid']}")
        print(f"item_ids: {result['item_count']}")
        if result["issues"]:
            print("issues:")
            for issue in result["issues"]:
                print(f"  - line {issue['line']}: {issue['message']}")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="scripts/validate_item_ids.py",
        description="Validate Markdown files that use <!-- item-id: ... --> annotations.",
    )
    parser.add_argument("paths", nargs="+", help="Markdown file paths to validate")
    parser.add_argument("--json", action="store_true")
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    try:
        results = [validate_file(resolve_path(path_value)) for path_value in args.paths]
    except ItemIdValidationError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    if args.json:
        print(json.dumps({"status": "ok", "results": results}))
    else:
        print_human(results)

    return 0 if all(result["valid"] for result in results) else 1


if __name__ == "__main__":
    sys.exit(main())
