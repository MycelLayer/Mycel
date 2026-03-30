#!/usr/bin/env python3

from __future__ import annotations

import json
import os
import re
import subprocess
from pathlib import Path
from typing import Mapping


ROOT_DIR = Path(__file__).resolve().parent.parent
GITHUB_SLUG_PATTERN = re.compile(r"github\.com[:/](?P<slug>[^/]+/[^/.]+?)(?:\.git)?$")


class RepoContextError(Exception):
    """Raised when the current GitHub repository context cannot be determined."""


def normalize_repo_slug(repo: str | None) -> str | None:
    if repo is None:
        return None
    value = repo.strip()
    return value or None


def parse_repo_slug_from_remote(url: str) -> str | None:
    match = GITHUB_SLUG_PATTERN.search(url.strip())
    if not match:
        return None
    return match.group("slug")


def repo_slug_from_git_remote(
    *,
    cwd: str | Path = ROOT_DIR,
) -> str | None:
    proc = subprocess.run(
        ["git", "remote", "get-url", "origin"],
        cwd=cwd,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        return None
    return parse_repo_slug_from_remote(proc.stdout)


def repo_slug_from_gh(
    *,
    cwd: str | Path = ROOT_DIR,
    env: Mapping[str, str] | None = None,
) -> str | None:
    proc = subprocess.run(
        ["gh", "repo", "view", "--json", "nameWithOwner"],
        cwd=cwd,
        env=dict(env or os.environ),
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        return None
    try:
        payload = json.loads(proc.stdout)
    except json.JSONDecodeError:
        return None
    if not isinstance(payload, dict):
        return None
    value = payload.get("nameWithOwner")
    return value.strip() if isinstance(value, str) and value.strip() else None


def resolve_repo_slug(
    repo: str | None = None,
    *,
    cwd: str | Path = ROOT_DIR,
    env: Mapping[str, str] | None = None,
) -> str:
    explicit = normalize_repo_slug(repo)
    if explicit is not None:
        return explicit

    detected = repo_slug_from_git_remote(cwd=cwd)
    if detected is not None:
        return detected

    detected = repo_slug_from_gh(cwd=cwd, env=env)
    if detected is not None:
        return detected

    raise RepoContextError(
        "could not determine the current GitHub repo; pass --repo explicitly or configure git/gh repo context"
    )
