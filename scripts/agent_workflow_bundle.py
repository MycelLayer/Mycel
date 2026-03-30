#!/usr/bin/env python3

from __future__ import annotations

import argparse
import hashlib
import io
import json
import subprocess
import sys
import tarfile
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath
from typing import Any


ROOT_DIR = Path(__file__).resolve().parent.parent
BUNDLE_VERSION = 1
DEFAULT_EXPORT_CATEGORIES = ("docs", "templates", "scripts")
CATEGORY_PATHS: dict[str, tuple[str, ...]] = {
    "docs": (
        "AGENTS.md",
        "AGENTS-LOCAL.md",
        "docs/DEV-SETUP.md",
        "docs/AGENT-REGISTRY.md",
        "docs/ROLE-CHECKLISTS/README.md",
        "docs/ROLE-CHECKLISTS/coding.md",
        "docs/ROLE-CHECKLISTS/delivery.md",
        "docs/ROLE-CHECKLISTS/doc.md",
    ),
    "templates": (
        ".agent-local/DEV-SETUP-STATUS.example.md",
        ".agent-local/mailboxes/EXAMPLE-work-continuation-handoff.md",
        ".agent-local/mailboxes/EXAMPLE-delivery-continuation-note.md",
        ".agent-local/mailboxes/EXAMPLE-doc-continuation-note.md",
        ".agent-local/mailboxes/EXAMPLE-planning-sync-handoff.md",
        ".agent-local/mailboxes/EXAMPLE-planning-sync-resolution.md",
    ),
    "scripts": (
        "scripts/agent_bootstrap.py",
        "scripts/agent_workflow_bundle.py",
        "scripts/agent_work_cycle.py",
        "scripts/agent_registry.py",
        "scripts/agent_registry_reconcile.py",
        "scripts/agent_guard.py",
        "scripts/agent_timestamp.py",
        "scripts/agent_safe_commit.py",
        "scripts/agent_push.py",
        "scripts/check-runtime-preflight.py",
        "scripts/check-dev-env.py",
        "scripts/update-dev-setup-status.py",
        "scripts/item_id_checklist.py",
        "scripts/item_id_checklist_mark.py",
        "scripts/check_checklist_refresh.py",
        "scripts/mailbox_handoff.py",
        "scripts/mailbox_gc.py",
        "scripts/render_next_work_items.py",
        "scripts/render_files_changed_table.py",
        "scripts/render_files_changed_from_json.py",
        "scripts/codex_thread_metadata.py",
        "scripts/codex_token_usage_summary.py",
    ),
    "local-state": (
        ".agent-local/dev-setup-status.md",
        ".agent-local/agents.json",
    ),
}
TAR_GZ_SUFFIXES = (".tar.gz", ".tgz")


class AgentWorkflowBundleError(Exception):
    """Raised when bundle export/import fails."""


@dataclass(frozen=True)
class BundleFile:
    path: str
    categories: tuple[str, ...]
    sha256: str
    size: int
    data: bytes

    def manifest_entry(self) -> dict[str, Any]:
        return {
            "path": self.path,
            "categories": list(self.categories),
            "sha256": self.sha256,
            "size": self.size,
        }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="scripts/agent_workflow_bundle.py",
        description="Export or import the repository's agent workflow docs, templates, and scripts.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    export = subparsers.add_parser(
        "export",
        help="write a workflow bundle to a directory or tar archive",
    )
    export.add_argument(
        "output",
        help="bundle output path; directories create an unpacked bundle, .tar/.tar.gz/.tgz create an archive",
    )
    export.add_argument(
        "--include",
        action="append",
        choices=sorted(CATEGORY_PATHS),
        help="category to include; defaults to docs, templates, scripts",
    )
    export.add_argument("--json", action="store_true", help="emit machine-readable JSON output")

    import_cmd = subparsers.add_parser(
        "import",
        help="import a workflow bundle into a destination directory",
    )
    import_cmd.add_argument("bundle_path", help="bundle directory or tar archive created by export")
    import_cmd.add_argument(
        "--dest",
        default=str(ROOT_DIR),
        help="destination directory; defaults to the current repo root",
    )
    import_cmd.add_argument(
        "--overwrite",
        action="store_true",
        help="allow replacing existing files when bundle contents differ",
    )
    import_cmd.add_argument(
        "--dry-run",
        action="store_true",
        help="preview the import plan without writing any files",
    )
    import_cmd.add_argument("--json", action="store_true", help="emit machine-readable JSON output")

    return parser.parse_args()


def utc_now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run_git_head(root: Path) -> str | None:
    proc = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=root,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        return None
    value = proc.stdout.strip()
    return value or None


def bundle_format_for_output(path: Path) -> str:
    path_str = path.as_posix()
    if path_str.endswith(TAR_GZ_SUFFIXES) or path.suffix == ".tar":
        return "tar"
    return "dir"


def normalize_bundle_relpath(path_value: str) -> str:
    if not isinstance(path_value, str) or not path_value.strip():
        raise AgentWorkflowBundleError("bundle file path must be a non-empty string")
    raw = path_value.strip().replace("\\", "/")
    pure = PurePosixPath(raw)
    if pure.is_absolute():
        raise AgentWorkflowBundleError(f"bundle file path must be relative: {path_value}")
    if any(part in ("", ".", "..") for part in pure.parts):
        raise AgentWorkflowBundleError(f"bundle file path contains unsafe segments: {path_value}")
    return pure.as_posix()


def collect_bundle_files(root: Path, categories: list[str]) -> tuple[list[BundleFile], list[str]]:
    category_map: dict[str, set[str]] = {}
    missing_paths: list[str] = []
    for category in categories:
        for relative_path in CATEGORY_PATHS[category]:
            normalized = normalize_bundle_relpath(relative_path)
            file_path = root / normalized
            if not file_path.exists():
                missing_paths.append(normalized)
                continue
            if not file_path.is_file():
                raise AgentWorkflowBundleError(f"bundle path is not a file: {normalized}")
            category_map.setdefault(normalized, set()).add(category)

    bundle_files: list[BundleFile] = []
    for relative_path in sorted(category_map):
        file_path = root / relative_path
        data = file_path.read_bytes()
        bundle_files.append(
            BundleFile(
                path=relative_path,
                categories=tuple(sorted(category_map[relative_path])),
                sha256=sha256_bytes(data),
                size=len(data),
                data=data,
            )
        )
    return bundle_files, sorted(set(missing_paths))


def build_manifest(*, categories: list[str], bundle_files: list[BundleFile], missing_paths: list[str]) -> dict[str, Any]:
    manifest: dict[str, Any] = {
        "bundle_version": BUNDLE_VERSION,
        "created_at": utc_now_iso(),
        "source_repo_root": str(ROOT_DIR),
        "source_git_head": run_git_head(ROOT_DIR),
        "categories": categories,
        "missing_paths": missing_paths,
        "files": [bundle_file.manifest_entry() for bundle_file in bundle_files],
    }
    return manifest


def ensure_output_path_is_writable(path: Path, *, bundle_format: str) -> None:
    if not path.exists():
        return
    if bundle_format == "dir":
        if not path.is_dir():
            raise AgentWorkflowBundleError(f"output path exists and is not a directory: {path}")
        if any(path.iterdir()):
            raise AgentWorkflowBundleError(f"output directory must be empty: {path}")
        return
    raise AgentWorkflowBundleError(f"output path already exists: {path}")


def write_directory_bundle(output_dir: Path, manifest: dict[str, Any], bundle_files: list[BundleFile]) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    (output_dir / "manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    files_root = output_dir / "files"
    for bundle_file in bundle_files:
        target = files_root / bundle_file.path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(bundle_file.data)


def tar_mode_for_output(path: Path) -> str:
    path_str = path.as_posix()
    if path_str.endswith(TAR_GZ_SUFFIXES):
        return "w:gz"
    if path.suffix == ".tar":
        return "w"
    raise AgentWorkflowBundleError("tar output path must end with .tar, .tar.gz, or .tgz")


def write_tar_bundle(output_path: Path, manifest: dict[str, Any], bundle_files: list[BundleFile]) -> None:
    manifest_bytes = (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode("utf-8")
    with tarfile.open(output_path, tar_mode_for_output(output_path)) as archive:
        manifest_info = tarfile.TarInfo("manifest.json")
        manifest_info.size = len(manifest_bytes)
        archive.addfile(manifest_info, io.BytesIO(manifest_bytes))
        for bundle_file in bundle_files:
            archive_path = f"files/{bundle_file.path}"
            file_info = tarfile.TarInfo(archive_path)
            file_info.size = bundle_file.size
            archive.addfile(file_info, io.BytesIO(bundle_file.data))


def load_json_text(raw: str, *, source: str) -> dict[str, Any]:
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise AgentWorkflowBundleError(f"invalid JSON in {source}: {exc.msg}") from exc
    if not isinstance(payload, dict):
        raise AgentWorkflowBundleError(f"{source} must contain a JSON object")
    return payload


def validate_manifest(manifest: dict[str, Any]) -> list[dict[str, Any]]:
    version = manifest.get("bundle_version")
    if version != BUNDLE_VERSION:
        raise AgentWorkflowBundleError(
            f"unsupported bundle version {version!r}; expected {BUNDLE_VERSION}"
        )
    files = manifest.get("files")
    if not isinstance(files, list):
        raise AgentWorkflowBundleError("manifest must provide a 'files' array")
    validated: list[dict[str, Any]] = []
    for index, entry in enumerate(files, start=1):
        if not isinstance(entry, dict):
            raise AgentWorkflowBundleError(f"manifest file entry {index} must be an object")
        path = normalize_bundle_relpath(str(entry.get("path", "")))
        sha256 = entry.get("sha256")
        size = entry.get("size")
        if not isinstance(sha256, str) or len(sha256) != 64:
            raise AgentWorkflowBundleError(f"manifest file entry {index} has an invalid sha256")
        if not isinstance(size, int) or size < 0:
            raise AgentWorkflowBundleError(f"manifest file entry {index} has an invalid size")
        categories = entry.get("categories")
        if not isinstance(categories, list) or not categories or not all(
            isinstance(category, str) and category in CATEGORY_PATHS for category in categories
        ):
            raise AgentWorkflowBundleError(
                f"manifest file entry {index} must provide a non-empty valid categories list"
            )
        validated.append(
            {
                "path": path,
                "sha256": sha256,
                "size": size,
                "categories": list(categories),
            }
        )
    return validated


def load_bundle_directory(bundle_path: Path) -> tuple[dict[str, Any], dict[str, bytes]]:
    manifest_path = bundle_path / "manifest.json"
    if not manifest_path.is_file():
        raise AgentWorkflowBundleError(f"bundle directory is missing manifest.json: {bundle_path}")
    manifest = load_json_text(manifest_path.read_text(encoding="utf-8"), source=str(manifest_path))
    validated_files = validate_manifest(manifest)
    data_map: dict[str, bytes] = {}
    files_root = bundle_path / "files"
    for entry in validated_files:
        file_path = files_root / entry["path"]
        if not file_path.is_file():
            raise AgentWorkflowBundleError(f"bundle directory is missing file: {entry['path']}")
        data = file_path.read_bytes()
        if len(data) != entry["size"]:
            raise AgentWorkflowBundleError(f"bundle file size mismatch: {entry['path']}")
        if sha256_bytes(data) != entry["sha256"]:
            raise AgentWorkflowBundleError(f"bundle file checksum mismatch: {entry['path']}")
        data_map[entry["path"]] = data
    return manifest, data_map


def load_bundle_archive(bundle_path: Path) -> tuple[dict[str, Any], dict[str, bytes]]:
    with tarfile.open(bundle_path, "r:*") as archive:
        try:
            manifest_member = archive.getmember("manifest.json")
        except KeyError as exc:
            raise AgentWorkflowBundleError(f"bundle archive is missing manifest.json: {bundle_path}") from exc
        manifest_file = archive.extractfile(manifest_member)
        if manifest_file is None:
            raise AgentWorkflowBundleError(f"could not read manifest.json from archive: {bundle_path}")
        manifest = load_json_text(manifest_file.read().decode("utf-8"), source=f"{bundle_path}:manifest.json")
        validated_files = validate_manifest(manifest)
        data_map: dict[str, bytes] = {}
        for entry in validated_files:
            member_name = f"files/{entry['path']}"
            try:
                member = archive.getmember(member_name)
            except KeyError as exc:
                raise AgentWorkflowBundleError(f"bundle archive is missing file: {entry['path']}") from exc
            member_file = archive.extractfile(member)
            if member_file is None:
                raise AgentWorkflowBundleError(f"could not read archive file: {entry['path']}")
            data = member_file.read()
            if len(data) != entry["size"]:
                raise AgentWorkflowBundleError(f"bundle file size mismatch: {entry['path']}")
            if sha256_bytes(data) != entry["sha256"]:
                raise AgentWorkflowBundleError(f"bundle file checksum mismatch: {entry['path']}")
            data_map[entry["path"]] = data
        return manifest, data_map


def load_bundle(bundle_path: Path) -> tuple[dict[str, Any], dict[str, bytes]]:
    if bundle_path.is_dir():
        return load_bundle_directory(bundle_path)
    if bundle_path.is_file():
        return load_bundle_archive(bundle_path)
    raise AgentWorkflowBundleError(f"bundle path does not exist: {bundle_path}")


def resolve_destination_path(dest_root: Path, relative_path: str) -> Path:
    target = (dest_root / relative_path).resolve()
    resolved_root = dest_root.resolve()
    try:
        target.relative_to(resolved_root)
    except ValueError as exc:
        raise AgentWorkflowBundleError(f"bundle file escapes destination root: {relative_path}") from exc
    return target


def existing_file_sha256(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def import_plan(
    dest_root: Path,
    bundle_data: dict[str, bytes],
    manifest_files: list[dict[str, Any]],
    *,
    overwrite: bool,
) -> list[dict[str, str]]:
    operations: list[dict[str, str]] = []
    for entry in manifest_files:
        relative_path = entry["path"]
        target_path = resolve_destination_path(dest_root, relative_path)
        if target_path.exists():
            if not target_path.is_file():
                raise AgentWorkflowBundleError(f"destination path exists and is not a file: {relative_path}")
            if existing_file_sha256(target_path) == entry["sha256"]:
                status = "unchanged"
            elif overwrite:
                status = "overwrite"
            else:
                status = "conflict"
        else:
            status = "create"
        operations.append({"path": relative_path, "status": status})
    return operations


def apply_import(
    dest_root: Path,
    bundle_data: dict[str, bytes],
    operations: list[dict[str, str]],
) -> None:
    for operation in operations:
        if operation["status"] not in {"create", "overwrite"}:
            continue
        relative_path = operation["path"]
        target_path = resolve_destination_path(dest_root, relative_path)
        target_path.parent.mkdir(parents=True, exist_ok=True)
        target_path.write_bytes(bundle_data[relative_path])


def summarize_operations(operations: list[dict[str, str]]) -> dict[str, int]:
    counts = {"create": 0, "overwrite": 0, "unchanged": 0, "conflict": 0}
    for operation in operations:
        counts[operation["status"]] += 1
    return counts


def handle_export(args: argparse.Namespace) -> int:
    output_path = Path(args.output)
    categories = list(args.include or DEFAULT_EXPORT_CATEGORIES)
    bundle_files, missing_paths = collect_bundle_files(ROOT_DIR, categories)
    manifest = build_manifest(
        categories=categories,
        bundle_files=bundle_files,
        missing_paths=missing_paths,
    )
    bundle_format = bundle_format_for_output(output_path)
    ensure_output_path_is_writable(output_path, bundle_format=bundle_format)
    if bundle_format == "dir":
        write_directory_bundle(output_path, manifest, bundle_files)
    else:
        output_path.parent.mkdir(parents=True, exist_ok=True)
        write_tar_bundle(output_path, manifest, bundle_files)

    summary = {
        "status": "ok",
        "command": "export",
        "output": str(output_path),
        "bundle_format": bundle_format,
        "categories": categories,
        "file_count": len(bundle_files),
        "missing_paths": missing_paths,
    }
    if args.json:
        print(json.dumps(summary))
    else:
        print(
            f"exported {len(bundle_files)} files to {output_path} "
            f"({bundle_format}; categories: {', '.join(categories)})"
        )
        if missing_paths:
            print(f"missing optional paths: {', '.join(missing_paths)}")
    return 0


def handle_import(args: argparse.Namespace) -> int:
    bundle_path = Path(args.bundle_path)
    dest_root = Path(args.dest)
    if dest_root.exists() and not dest_root.is_dir():
        raise AgentWorkflowBundleError(f"destination root must be a directory: {dest_root}")

    manifest, bundle_data = load_bundle(bundle_path)
    manifest_files = validate_manifest(manifest)
    operations = import_plan(
        dest_root,
        bundle_data,
        manifest_files,
        overwrite=args.overwrite,
    )
    counts = summarize_operations(operations)

    if not args.dry_run and counts["conflict"]:
        conflict_paths = [operation["path"] for operation in operations if operation["status"] == "conflict"]
        raise AgentWorkflowBundleError(
            "import would overwrite existing files; rerun with --overwrite or preview first with --dry-run: "
            + ", ".join(conflict_paths)
        )

    if not args.dry_run:
        dest_root.mkdir(parents=True, exist_ok=True)
        apply_import(dest_root, bundle_data, operations)

    summary = {
        "status": "ok",
        "command": "import",
        "bundle_path": str(bundle_path),
        "dest": str(dest_root),
        "dry_run": bool(args.dry_run),
        "overwrite": bool(args.overwrite),
        "counts": counts,
        "operations": operations,
    }
    if args.json:
        print(json.dumps(summary))
    else:
        for operation in operations:
            prefix = "would " if args.dry_run else ""
            print(f"{prefix}{operation['status']}: {operation['path']}")
        print(
            f"summary: create={counts['create']} overwrite={counts['overwrite']} "
            f"unchanged={counts['unchanged']} conflict={counts['conflict']}"
        )
    return 0


def main() -> int:
    args = parse_args()
    try:
        if args.command == "export":
            return handle_export(args)
        if args.command == "import":
            return handle_import(args)
        raise AgentWorkflowBundleError(f"unsupported command: {args.command}")
    except AgentWorkflowBundleError as exc:
        print(str(exc), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
