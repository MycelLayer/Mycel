#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


DEFAULT_LOCALE = "en"
SUPPORTED_LOCALES = frozenset({"en", "zh-TW"})
LOCALE_LABELS: dict[str, dict[str, str]] = {
    "en": {
        "highest_value_marker": "(最有價值)",
        "tradeoff_label": "Tradeoff",
        "roadmap_label": "Roadmap",
    },
    "zh-TW": {
        "highest_value_marker": "(最有價值)",
        "tradeoff_label": "取捨",
        "roadmap_label": "路線圖",
    },
}
DEFAULT_COMPACTION_MESSAGES: dict[str, str] = {
    "en": "compaction detected, we better open a new chat.",
    "zh-TW": "偵測到 compact context，我們最好開一個新聊天再繼續。",
}
DEFAULT_COMPACTION_TRADEOFFS: dict[str, str] = {
    "en": "safest follow-up after compaction, but it pauses immediate work until a fresh chat is open.",
    "zh-TW": "這是 compact context 後最安全的下一步，但會先暫停眼前工作，直到新聊天開好為止。",
}
ROLE_DEFAULT_ITEMS: dict[str, dict[str, list[dict[str, str]]]] = {
    "en": {
        "coding": [
            {
                "text": "review ROADMAP.md and identify the highest-value next coding work",
                "tradeoff": "best roadmap alignment, but it spends a little time on prioritization before implementation",
                "roadmap": "ROADMAP.md / next coding slice",
            },
            {
                "text": "review the latest CQH issue and identify high-value work items",
                "tradeoff": "usually cheaper to land quickly, but it may be less directly tied to the main roadmap lane",
            },
        ],
        "delivery": [
            {
                "text": "review the latest completed CI result before choosing the next delivery follow-up",
                "tradeoff": "safest delivery baseline, but it may delay action if CI context needs re-reading",
            },
            {
                "text": "review the current delivery workflow or process follow-up with the freshest CI evidence",
                "tradeoff": "good for stabilizing delivery flow, but it is less directly product-facing than coding work",
            },
        ],
        "doc": [
            {
                "text": "review the freshest planning or documentation follow-up before choosing the next doc item",
                "tradeoff": "keeps doc work aligned with current repo state, but it adds a short review step first",
            },
            {
                "text": "check whether planning-sync or issue-sync follow-up is due before writing the next doc update",
                "tradeoff": "helps avoid drift in planning surfaces, but it may defer narrower writing work briefly",
            },
        ],
    },
    "zh-TW": {
        "coding": [
            {
                "text": "檢查 ROADMAP.md，找出最高價值的下一個 coding 工作",
                "tradeoff": "和 roadmap 對齊最好，但在開始實作前需要先花一點時間做優先順序判斷",
                "roadmap": "ROADMAP.md / next coding slice",
            },
            {
                "text": "檢查最新的 CQH issue，整理高價值工作項目",
                "tradeoff": "通常比較快能落地，但可能沒有那麼直接貼近主要 roadmap 軌道",
            },
        ],
        "delivery": [
            {
                "text": "檢查上一個已完成的 CI 結果，再決定下一個 delivery 後續工作",
                "tradeoff": "delivery 基線最安全，但如果 CI 脈絡需要重讀，行動會稍微延後",
            },
            {
                "text": "依照最新的 CI 證據檢查目前的 delivery workflow 或流程後續項",
                "tradeoff": "很適合穩定 delivery 流程，但不像 coding 工作那麼直接面向產品功能",
            },
        ],
        "doc": [
            {
                "text": "檢查最新的規劃或文件後續項，再決定下一個 doc 工作",
                "tradeoff": "能讓文件工作保持和目前 repo 狀態同步，但會先多一個短暫的檢查步驟",
            },
            {
                "text": "先確認 planning-sync 或 issue-sync 的後續是否到期，再撰寫下一份文件更新",
                "tradeoff": "有助於避免規劃面漂移，但可能會先稍微延後較窄範圍的寫作工作",
            },
        ],
    },
}


class NextWorkItemsError(Exception):
    """Raised when the next-work-items spec is invalid."""


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="scripts/render_next_work_items.py",
        description="Render Markdown next-work-item options from a JSON spec.",
    )
    parser.add_argument(
        "spec_path",
        nargs="?",
        default="-",
        help="JSON spec path, or '-' to read the spec from stdin",
    )
    return parser.parse_args()


def load_spec(spec_path: str) -> dict[str, object]:
    if spec_path == "-":
        raw = sys.stdin.read()
    else:
        raw = Path(spec_path).read_text(encoding="utf-8")
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise NextWorkItemsError(f"invalid JSON spec: {exc.msg}") from exc
    if not isinstance(payload, dict):
        raise NextWorkItemsError("JSON spec must be an object")
    return payload


def require_string(entry: dict[str, object], key: str, *, item_index: int) -> str:
    value = entry.get(key)
    if not isinstance(value, str) or not value.strip():
        raise NextWorkItemsError(f"item {item_index} must provide a non-empty string '{key}'")
    return value.strip()


def parse_bool(payload: dict[str, object], key: str) -> bool:
    value = payload.get(key, False)
    if not isinstance(value, bool):
        raise NextWorkItemsError(f"'{key}' must be a boolean when provided")
    return value


def parse_optional_string(payload: dict[str, object], key: str) -> str | None:
    value = payload.get(key)
    if value is None:
        return None
    if not isinstance(value, str) or not value.strip():
        raise NextWorkItemsError(f"'{key}' must be a non-empty string when provided")
    return value.strip()


def parse_optional_role(payload: dict[str, object]) -> str | None:
    role = parse_optional_string(payload, "role")
    if role is None:
        return None
    if role not in ROLE_DEFAULT_ITEMS[DEFAULT_LOCALE]:
        raise NextWorkItemsError(
            f"'role' must be one of: {', '.join(sorted(ROLE_DEFAULT_ITEMS[DEFAULT_LOCALE]))}"
        )
    return role


def parse_append_role_defaults(payload: dict[str, object]) -> bool:
    return parse_bool(payload, "append_role_defaults")


def parse_locale(payload: dict[str, object]) -> str:
    locale = parse_optional_string(payload, "locale")
    if locale is None:
        return DEFAULT_LOCALE
    if locale not in SUPPORTED_LOCALES:
        raise NextWorkItemsError(
            f"'locale' must be one of: {', '.join(sorted(SUPPORTED_LOCALES))}"
        )
    return locale


def parse_items(payload: dict[str, object]) -> list[dict[str, str]]:
    raw_items = payload.get("items", [])
    if not isinstance(raw_items, list):
        raise NextWorkItemsError("'items' must be an array when provided")

    items: list[dict[str, str]] = []
    for index, entry in enumerate(raw_items, start=1):
        if not isinstance(entry, dict):
            raise NextWorkItemsError(f"item {index} must be an object")
        item = {
            "text": require_string(entry, "text", item_index=index),
            "tradeoff": require_string(entry, "tradeoff", item_index=index),
        }
        roadmap = entry.get("roadmap")
        if roadmap is not None:
            if not isinstance(roadmap, str) or not roadmap.strip():
                raise NextWorkItemsError(f"item {index} has an invalid 'roadmap' value")
            item["roadmap"] = roadmap.strip()
        items.append(item)
    return items


def role_default_items(role: str | None, *, locale: str) -> list[dict[str, str]]:
    if role is None:
        return []
    return [dict(entry) for entry in ROLE_DEFAULT_ITEMS[locale][role]]


def build_items(payload: dict[str, object]) -> list[dict[str, str]]:
    locale = parse_locale(payload)
    role = parse_optional_role(payload)
    explicit_items = parse_items(payload)
    role_items = role_default_items(role, locale=locale)
    if parse_append_role_defaults(payload):
        items = explicit_items + role_items
    else:
        items = role_items + explicit_items
    compaction_detected = parse_bool(payload, "compaction_detected")
    if compaction_detected:
        compaction_item = {
            "text": parse_optional_string(payload, "compaction_message")
            or DEFAULT_COMPACTION_MESSAGES[locale],
            "tradeoff": parse_optional_string(payload, "compaction_tradeoff")
            or DEFAULT_COMPACTION_TRADEOFFS[locale],
        }
        items.insert(0, compaction_item)
    if not items:
        raise NextWorkItemsError("spec must provide at least one item or set compaction_detected=true")
    return items


def render_payload(payload: dict[str, object]) -> str:
    return render_items(build_items(payload), locale=parse_locale(payload))


def render_items(items: list[dict[str, str]], *, locale: str = DEFAULT_LOCALE) -> str:
    labels = LOCALE_LABELS[locale]
    lines: list[str] = []
    for index, item in enumerate(items, start=1):
        prefix = f"{index}. "
        if index == 1:
            prefix += f"{labels['highest_value_marker']} "
        line = f"{prefix}{item['text']} {labels['tradeoff_label']}: {item['tradeoff']}"
        roadmap = item.get("roadmap")
        if roadmap:
            line += f" {labels['roadmap_label']}: {roadmap}"
        lines.append(line)
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    try:
        payload = load_spec(args.spec_path)
        rendered = render_payload(payload)
    except NextWorkItemsError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1
    print(rendered, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
