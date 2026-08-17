from __future__ import annotations

import json
import os
import re
import subprocess
from pathlib import Path
from typing import Any


LATEST_CI_GH_FIELDS = [
    "databaseId",
    "status",
    "conclusion",
    "workflowName",
    "displayTitle",
    "headSha",
    "updatedAt",
]
LATEST_CI_WORKFLOW_NAME = "CI"


class BootstrapCiError(Exception):
    pass


def _run_json_array_command(root_dir: Path, command: list[str]) -> list[dict[str, Any]]:
    proc = subprocess.run(
        command,
        cwd=root_dir,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        message = proc.stderr.strip() or proc.stdout.strip() or "command failed"
        raise BootstrapCiError(message)
    try:
        payload = json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        raise BootstrapCiError(f"expected JSON output from {' '.join(command)}") from exc
    if not isinstance(payload, list):
        raise BootstrapCiError(f"expected JSON array output from {' '.join(command)}")
    return [item for item in payload if isinstance(item, dict)]


def _git_ref_sha(root_dir: Path, ref: str) -> str | None:
    proc = subprocess.run(
        ["git", "rev-parse", "--verify", ref],
        cwd=root_dir,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        return None
    sha = proc.stdout.strip()
    return sha if re.fullmatch(r"[0-9a-f]{40}", sha) else None


def _resolve_previous_push_head(root_dir: Path) -> str | None:
    origin = subprocess.run(
        ["git", "remote", "get-url", "origin"],
        cwd=root_dir,
        text=True,
        capture_output=True,
        check=False,
    )
    if origin.returncode == 0 and origin.stdout.strip():
        fetch = subprocess.run(
            [
                "git",
                "fetch",
                "--quiet",
                "--no-tags",
                "origin",
                "+refs/heads/main:refs/remotes/origin/main",
            ],
            cwd=root_dir,
            text=True,
            capture_output=True,
            check=False,
            env={**os.environ, "GIT_TERMINAL_PROMPT": "0"},
        )
        if fetch.returncode != 0:
            message = fetch.stderr.strip() or fetch.stdout.strip() or "git fetch failed"
            raise BootstrapCiError(f"unable to refresh origin/main before CI lookup: {message}")
        sha = _git_ref_sha(root_dir, "origin/main")
        if sha is None:
            raise BootstrapCiError("origin/main is unavailable after refreshing it for the CI lookup")
        return sha

    return _git_ref_sha(root_dir, "HEAD")


def _changed_paths_for_commit(root_dir: Path, commit_sha: str) -> list[str] | None:
    proc = subprocess.run(
        [
            "git",
            "diff-tree",
            "--root",
            "--no-commit-id",
            "--name-only",
            "-r",
            "-m",
            commit_sha,
        ],
        cwd=root_dir,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        return None
    return [line.strip() for line in proc.stdout.splitlines() if line.strip()]


def _ci_push_path_patterns(root_dir: Path) -> list[str] | None:
    workflow_path = root_dir / ".github" / "workflows" / "ci.yml"
    try:
        text = workflow_path.read_text(encoding="utf-8")
    except FileNotFoundError:
        return None

    in_push = False
    in_paths = False
    patterns: list[str] = []
    for line in text.splitlines():
        if line == "  push:":
            in_push = True
            in_paths = False
            continue
        if in_push and line == "    paths:":
            in_paths = True
            continue
        if not in_paths:
            if in_push and line and not line.startswith("    "):
                break
            continue
        if not line.startswith("      - "):
            break
        value = line.strip()[2:].strip().strip("'\"")
        if value:
            patterns.append(value)
    return patterns or None


def _path_matches_ci_trigger(path: str, pattern: str) -> bool | None:
    if pattern.endswith("/**") and not any(char in pattern[:-3] for char in "*?["):
        return path.startswith(pattern[:-2])
    if not any(char in pattern for char in "*?["):
        return path == pattern
    return None


def _commit_triggers_ci(root_dir: Path, commit_sha: str) -> bool | None:
    changed_paths = _changed_paths_for_commit(root_dir, commit_sha)
    patterns = _ci_push_path_patterns(root_dir)
    if changed_paths is None or patterns is None:
        return None
    unsupported_pattern = False
    for path in changed_paths:
        for pattern in patterns:
            matched = _path_matches_ci_trigger(path, pattern)
            if matched is True:
                return True
            if matched is None:
                unsupported_pattern = True
    return None if unsupported_pattern else False


def _ci_run_summary(run: dict[str, Any]) -> dict[str, Any]:
    return {
        "databaseId": run.get("databaseId"),
        "workflowName": run.get("workflowName"),
        "displayTitle": run.get("displayTitle"),
        "conclusion": run.get("conclusion"),
        "headSha": run.get("headSha"),
        "updatedAt": run.get("updatedAt"),
    }


def _completed_ci_result(run: dict[str, Any]) -> dict[str, Any]:
    return {
        "checked": True,
        "status": "completed",
        **_ci_run_summary(run),
    }


def _not_applicable_ci_result(
    previous_push_head: str,
    completed_ci_runs: list[dict[str, Any]],
) -> dict[str, Any]:
    baseline = _ci_run_summary(completed_ci_runs[0]) if completed_ci_runs else None
    return {
        "checked": True,
        "status": "not-applicable",
        "headSha": previous_push_head,
        "message": (
            f"previous push {previous_push_head} does not change a path that triggers "
            f"the {LATEST_CI_WORKFLOW_NAME} workflow"
        ),
        "baseline": baseline,
    }


def lookup_latest_completed_ci(root_dir: Path, role: str) -> dict[str, Any] | None:
    if role not in {"coding", "delivery"}:
        return None
    try:
        previous_push_head = _resolve_previous_push_head(root_dir)
    except BootstrapCiError as exc:
        return {
            "checked": False,
            "status": "unavailable",
            "message": str(exc),
        }

    fields = ",".join(LATEST_CI_GH_FIELDS)
    command = [
        "gh",
        "run",
        "list",
        "--branch",
        "main",
        "--workflow",
        LATEST_CI_WORKFLOW_NAME,
        "--status",
        "completed",
        "--limit",
        "20",
        "--json",
        fields,
    ]
    try:
        runs = _run_json_array_command(root_dir, command)
    except BootstrapCiError as exc:
        return {
            "checked": False,
            "status": "unavailable",
            "message": str(exc),
        }

    completed_ci_runs = [
        run
        for run in runs
        if run.get("status") == "completed" and run.get("workflowName") == LATEST_CI_WORKFLOW_NAME
    ]
    if previous_push_head is not None:
        for run in completed_ci_runs:
            if run.get("headSha") == previous_push_head:
                return _completed_ci_result(run)

        triggers_ci = _commit_triggers_ci(root_dir, previous_push_head)
        if triggers_ci is False:
            return _not_applicable_ci_result(previous_push_head, completed_ci_runs)

        if completed_ci_runs:
            latest_head = completed_ci_runs[0].get("headSha")
            suffix = (
                f"; latest completed {LATEST_CI_WORKFLOW_NAME} run was for {latest_head}"
                if isinstance(latest_head, str) and latest_head.strip()
                else ""
            )
            return {
                "checked": True,
                "status": "missing",
                "message": (
                    f"no completed {LATEST_CI_WORKFLOW_NAME} workflow run found for previous push "
                    f"{previous_push_head}{suffix}"
                ),
            }

        return {
            "checked": True,
            "status": "missing",
            "message": (
                f"no completed {LATEST_CI_WORKFLOW_NAME} workflow runs found on main "
                f"for previous push {previous_push_head}"
            ),
        }

    if completed_ci_runs:
        return _completed_ci_result(completed_ci_runs[0])

    return {
        "checked": True,
        "status": "missing",
        "message": f"no completed {LATEST_CI_WORKFLOW_NAME} workflow runs found on main",
    }


def _ci_state(latest_ci: dict[str, Any] | None) -> tuple[Any, Any, dict[str, Any] | None, Any]:
    status = latest_ci.get("status") if isinstance(latest_ci, dict) else None
    conclusion = latest_ci.get("conclusion") if isinstance(latest_ci, dict) else None
    raw_baseline = latest_ci.get("baseline") if isinstance(latest_ci, dict) else None
    baseline = raw_baseline if isinstance(raw_baseline, dict) else None
    baseline_conclusion = baseline.get("conclusion") if baseline is not None else None
    return status, conclusion, baseline, baseline_conclusion


def _coding_ci_action(latest_ci: dict[str, Any] | None, *, locale: str | None) -> str:
    status, conclusion, baseline, baseline_conclusion = _ci_state(latest_ci)
    if locale == "zh-TW":
        if status in {"unavailable", "missing"}:
            return "因為 bootstrap 還沒有成功確認最新 completed CI，先重新查一次，再決定下一個 implementation slice"
        if status == "not-applicable":
            if baseline_conclusion == "success":
                return "上一個 push 未觸發 CI；以最近一次適用且成功的 completed CI 作為基線，再決定下一個 implementation slice"
            if baseline is not None:
                return "上一個 push 未觸發 CI，但最近一次適用的 completed CI 並未成功；先分流處理該結果，再開始新的 implementation slice"
            return "上一個 push 未觸發 CI，且目前沒有可用的 completed CI 基線；先完成適當的本機驗證，再決定下一個 implementation slice"
        if conclusion != "success":
            return "上一個 completed CI 並未成功；先分流處理該結果，再開始新的 implementation slice"
        return "以上一個已完成的 CI 結果作為基線，再決定下一個 implementation slice"

    if status in {"unavailable", "missing"}:
        return (
            "re-run the latest completed CI lookup before choosing the next implementation slice "
            "because bootstrap could not confirm it"
        )
    if status == "not-applicable":
        if baseline_conclusion == "success":
            return (
                "the previous push did not trigger CI; use the latest applicable successful completed "
                "CI as the baseline before choosing the next implementation slice"
            )
        if baseline is not None:
            return (
                "the previous push did not trigger CI, but the latest applicable completed CI was not "
                "successful; triage that result before starting a new implementation slice"
            )
        return (
            "the previous push did not trigger CI and no completed CI baseline is available; complete "
            "appropriate local validation before choosing the next implementation slice"
        )
    if conclusion != "success":
        return (
            "the previous completed CI was not successful; triage that result before starting a new "
            "implementation slice"
        )
    return "use the latest completed CI result above as the baseline before choosing the next implementation slice"


def _delivery_ci_action(latest_ci: dict[str, Any] | None, *, locale: str | None) -> str:
    status, conclusion, baseline, baseline_conclusion = _ci_state(latest_ci)
    if locale == "zh-TW":
        if status in {"unavailable", "missing"}:
            return "因為 bootstrap 還沒有成功確認最新 completed CI，先重新查一次，再決定下一個 delivery 後續工作"
        if status == "not-applicable":
            if baseline_conclusion == "success":
                return "上一個 push 未觸發 CI；以最近一次適用且成功的 completed CI 作為 delivery 基線"
            if baseline is not None:
                return "上一個 push 未觸發 CI，但最近一次適用的 completed CI 並未成功；先分流處理該結果"
            return "上一個 push 未觸發 CI，且目前沒有可用的 completed CI 基線；先完成適當的本機驗證"
        if conclusion != "success":
            return "上一個 completed CI 並未成功；先分流處理該結果，再進行 delivery 後續工作"
        return "以上一個已完成的 CI 結果作為基線，再決定下一個 delivery 後續工作"

    if status in {"unavailable", "missing"}:
        return "re-run the latest completed CI lookup before delivery follow-up because bootstrap could not confirm it"
    if status == "not-applicable":
        if baseline_conclusion == "success":
            return (
                "the previous push did not trigger CI; use the latest applicable successful completed "
                "CI as the delivery baseline"
            )
        if baseline is not None:
            return (
                "the previous push did not trigger CI, but the latest applicable completed CI was not "
                "successful; triage that result before delivery follow-up"
            )
        return (
            "the previous push did not trigger CI and no completed CI baseline is available; complete "
            "appropriate local validation before delivery follow-up"
        )
    if conclusion != "success":
        return "the previous completed CI was not successful; triage that result before delivery follow-up"
    return "use the latest completed CI result above as the baseline before triaging delivery work"


def next_ci_action(
    role: str,
    latest_ci: dict[str, Any] | None,
    *,
    locale: str | None = None,
) -> str | None:
    if role == "coding":
        return _coding_ci_action(latest_ci, locale=locale)
    if role == "delivery":
        return _delivery_ci_action(latest_ci, locale=locale)
    return None
