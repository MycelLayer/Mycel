#!/usr/bin/env python3
"""Detect likely Codex context compaction points from session JSONL."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


CODEX_HOME = Path.home() / ".codex"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Read a Codex rollout JSONL and flag likely context compaction "
            "points based on sharp drops in per-turn input token usage."
        )
    )
    parser.add_argument(
        "--cwd",
        default=str(Path.cwd()),
        help="Working directory to match against Codex turn_context cwd.",
    )
    parser.add_argument(
        "--codex-home",
        default=str(CODEX_HOME),
        help="Codex home directory. Defaults to ~/.codex.",
    )
    parser.add_argument(
        "--thread-id",
        help="Specific thread_id to inspect. If omitted, use the latest thread matching --cwd.",
    )
    parser.add_argument(
        "--min-drop-tokens",
        type=int,
        default=8000,
        help="Minimum input-token drop between turns to flag as likely compaction.",
    )
    parser.add_argument(
        "--min-drop-ratio",
        type=float,
        default=0.15,
        help="Minimum fractional input-token drop between turns to flag as likely compaction.",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=20,
        help="Maximum number of detected compaction points to print.",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Emit JSON instead of a table.",
    )
    parser.add_argument(
        "--full-numbers",
        action="store_true",
        help="Render full token counts instead of compact K units.",
    )
    return parser.parse_args()


def find_latest_rollout_path(sessions_dir: Path, cwd: str) -> Path:
    latest_path: Path | None = None
    latest_ts = ""
    for path in sorted(sessions_dir.rglob("rollout-*.jsonl")):
        with path.open("r", encoding="utf-8") as handle:
            for raw_line in handle:
                line = raw_line.strip()
                if not line:
                    continue
                try:
                    entry = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if entry.get("type") != "turn_context":
                    continue
                payload = entry.get("payload", {})
                if payload.get("cwd") != cwd:
                    continue
                timestamp = entry.get("timestamp", "")
                if timestamp >= latest_ts:
                    latest_ts = timestamp
                    latest_path = path
    if latest_path is None:
        raise SystemExit(
            f"Could not find any rollout JSONL under {sessions_dir} for cwd {cwd!r}."
        )
    return latest_path


def find_rollout_path_by_thread_id(sessions_dir: Path, thread_id: str) -> Path:
    matches = sorted(sessions_dir.rglob(f"rollout-*{thread_id}.jsonl"))
    if not matches:
        raise SystemExit(
            f"Could not find any rollout JSONL under {sessions_dir} for thread_id {thread_id!r}."
        )
    return matches[-1]


def parse_rollout(
    rollout_path: Path,
) -> tuple[str, list[dict[str, Any]], list[dict[str, Any]]]:
    thread_id = "-".join(rollout_path.stem.rsplit("-", 5)[-5:])
    current_turn_id: str | None = None
    current_context: dict[str, Any] = {}
    turns: dict[str, dict[str, Any]] = {}
    turn_order: list[str] = []
    token_rows: list[dict[str, Any]] = []

    for raw_line in rollout_path.open("r", encoding="utf-8"):
        line = raw_line.strip()
        if not line:
            continue
        try:
            entry = json.loads(line)
        except json.JSONDecodeError:
            continue

        entry_type = entry.get("type")
        payload = entry.get("payload", {})

        if entry_type == "task_started":
            current_turn_id = payload.get("turn_id")
            if current_turn_id and current_turn_id not in turns:
                turns[current_turn_id] = {
                    "turn_id": current_turn_id,
                    "task_started_at": entry.get("timestamp"),
                    "first_token_usage": None,
                    "last_token_usage": None,
                    "cwd": current_context.get("cwd"),
                }
                turn_order.append(current_turn_id)
            continue

        if entry_type == "turn_context":
            current_turn_id = payload.get("turn_id") or current_turn_id
            current_context = payload
            if current_turn_id and current_turn_id not in turns:
                turns[current_turn_id] = {
                    "turn_id": current_turn_id,
                    "task_started_at": entry.get("timestamp"),
                    "first_token_usage": None,
                    "last_token_usage": None,
                    "cwd": payload.get("cwd"),
                }
                turn_order.append(current_turn_id)
            if current_turn_id:
                turns[current_turn_id]["cwd"] = payload.get("cwd")
                turns[current_turn_id]["turn_context_at"] = entry.get("timestamp")
            continue

        if entry_type != "event_msg" or payload.get("type") != "token_count":
            continue

        info = payload.get("info")
        if not info or current_turn_id is None:
            continue

        last = info.get("last_token_usage")
        total = info.get("total_token_usage")
        if not last or not total:
            continue

        usage = {
            "timestamp": entry.get("timestamp"),
            "turn_id": current_turn_id,
            "input_tokens": int(last.get("input_tokens", 0)),
            "cached_input_tokens": int(last.get("cached_input_tokens", 0)),
            "output_tokens": int(last.get("output_tokens", 0)),
            "reasoning_output_tokens": int(last.get("reasoning_output_tokens", 0)),
            "total_tokens": int(last.get("total_tokens", 0)),
            "cumulative_total_tokens": int(total.get("total_tokens", 0)),
            "model_context_window": int(info.get("model_context_window", 0)),
        }
        token_rows.append(usage)
        turn = turns.setdefault(
            current_turn_id,
            {
                "turn_id": current_turn_id,
                "task_started_at": None,
                "first_token_usage": None,
                "last_token_usage": None,
                "cwd": current_context.get("cwd"),
            },
        )
        if current_turn_id not in turn_order:
            turn_order.append(current_turn_id)
        if turn["first_token_usage"] is None:
            turn["first_token_usage"] = usage
        turn["last_token_usage"] = usage

    ordered_turns = [turns[turn_id] for turn_id in turn_order if turn_id in turns]
    return thread_id, ordered_turns, token_rows


def detect_compactions(
    turns: list[dict[str, Any]],
    token_rows: list[dict[str, Any]],
    min_drop_tokens: int,
    min_drop_ratio: float,
) -> list[dict[str, Any]]:
    detections: list[dict[str, Any]] = []
    for prev_turn, turn in zip(turns, turns[1:]):
        prev_last = prev_turn.get("last_token_usage")
        current_first = turn.get("first_token_usage")
        if not prev_last or not current_first:
            continue

        prev_input = int(prev_last["input_tokens"])
        current_input = int(current_first["input_tokens"])
        drop_tokens = prev_input - current_input
        drop_ratio = (drop_tokens / prev_input) if prev_input > 0 else 0.0

        if drop_tokens < min_drop_tokens or drop_ratio < min_drop_ratio:
            continue

        detections.append(
            {
                "kind": "between_turns",
                "previous_turn_id": prev_turn["turn_id"],
                "previous_timestamp": prev_last["timestamp"],
                "previous_input_tokens": prev_input,
                "previous_cached_input_tokens": int(prev_last["cached_input_tokens"]),
                "current_turn_id": turn["turn_id"],
                "current_timestamp": current_first["timestamp"],
                "current_input_tokens": current_input,
                "current_cached_input_tokens": int(
                    current_first["cached_input_tokens"]
                ),
                "drop_tokens": drop_tokens,
                "drop_ratio": drop_ratio,
                "current_context_window": int(current_first["model_context_window"]),
                "note": "Likely context compaction before the current turn.",
            }
        )

    for previous_row, current_row in zip(token_rows, token_rows[1:]):
        prev_input = int(previous_row["input_tokens"])
        current_input = int(current_row["input_tokens"])
        drop_tokens = prev_input - current_input
        drop_ratio = (drop_tokens / prev_input) if prev_input > 0 else 0.0

        if drop_tokens < min_drop_tokens or drop_ratio < min_drop_ratio:
            continue

        detections.append(
            {
                "kind": "row_drop",
                "previous_turn_id": previous_row.get("turn_id"),
                "previous_timestamp": previous_row["timestamp"],
                "previous_input_tokens": prev_input,
                "previous_cached_input_tokens": int(
                    previous_row["cached_input_tokens"]
                ),
                "current_turn_id": current_row.get("turn_id"),
                "current_timestamp": current_row["timestamp"],
                "current_input_tokens": current_input,
                "current_cached_input_tokens": int(
                    current_row["cached_input_tokens"]
                ),
                "drop_tokens": drop_tokens,
                "drop_ratio": drop_ratio,
                "current_context_window": int(current_row["model_context_window"]),
                "note": "Likely compaction or meter reset within the token_count stream.",
            }
        )

    detections.sort(
        key=lambda row: (
            row.get("current_timestamp") or "",
            row.get("kind") or "",
        )
    )
    return detections


def format_int(value: int) -> str:
    return f"{value:,}"


def format_compact_k(value: int) -> str:
    return f"{value / 1000:.1f}K"


def format_value(value: int, full_numbers: bool) -> str:
    if full_numbers:
        return format_int(value)
    return format_compact_k(value)


def render_table(
    rows: list[dict[str, Any]], rollout_path: Path, full_numbers: bool
) -> str:
    headers = [
        "current_timestamp",
        "prev_input",
        "current_input",
        "drop",
        "drop_pct",
        "kind",
        "prev_cached",
        "current_cached",
        "current_turn_id",
    ]
    body: list[list[str]] = []
    for row in rows:
        body.append(
            [
                str(row["current_timestamp"]),
                format_value(row["previous_input_tokens"], full_numbers),
                format_value(row["current_input_tokens"], full_numbers),
                format_value(row["drop_tokens"], full_numbers),
                f"{row['drop_ratio'] * 100:.1f}%",
                str(row["kind"]),
                format_value(row["previous_cached_input_tokens"], full_numbers),
                format_value(row["current_cached_input_tokens"], full_numbers),
                str(row["current_turn_id"]),
            ]
        )

    widths = [len(header) for header in headers]
    for line in body:
        for idx, cell in enumerate(line):
            widths[idx] = max(widths[idx], len(cell))

    lines = [f"rollout_path: {rollout_path}", ""]
    lines.append("  ".join(header.ljust(widths[idx]) for idx, header in enumerate(headers)))
    lines.append("  ".join("-" * widths[idx] for idx in range(len(headers))))
    for line in body:
        lines.append("  ".join(cell.ljust(widths[idx]) for idx, cell in enumerate(line)))
    return "\n".join(lines)


def main() -> int:
    args = parse_args()
    codex_home = Path(args.codex_home).expanduser()
    sessions_dir = codex_home / "sessions"
    rollout_path = (
        find_rollout_path_by_thread_id(sessions_dir, args.thread_id)
        if args.thread_id
        else find_latest_rollout_path(sessions_dir, args.cwd)
    )
    resolved_thread_id, turns, token_rows = parse_rollout(rollout_path)
    detections = detect_compactions(
        turns, token_rows, args.min_drop_tokens, args.min_drop_ratio
    )
    if args.limit > 0:
        detections = detections[-args.limit :]

    result = {
        "cwd": args.cwd,
        "thread_id": args.thread_id or resolved_thread_id,
        "rollout_path": str(rollout_path),
        "min_drop_tokens": args.min_drop_tokens,
        "min_drop_ratio": args.min_drop_ratio,
        "detection_count": len(detections),
        "detections": detections,
    }

    if args.json:
        print(json.dumps(result, ensure_ascii=True, indent=2))
        return 0

    print(render_table(detections, rollout_path, args.full_numbers))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
