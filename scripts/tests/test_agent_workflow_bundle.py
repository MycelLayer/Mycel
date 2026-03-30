import json
import subprocess
import tarfile
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "agent_workflow_bundle.py"


class AgentWorkflowBundleCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        scripts_dir = self.root / "scripts"
        scripts_dir.mkdir(parents=True, exist_ok=True)
        script = scripts_dir / "agent_workflow_bundle.py"
        script.write_text(SOURCE_SCRIPT.read_text(encoding="utf-8"), encoding="utf-8")
        script.chmod(0o755)

        (self.root / "docs" / "ROLE-CHECKLISTS").mkdir(parents=True, exist_ok=True)
        (self.root / ".agent-local" / "mailboxes").mkdir(parents=True, exist_ok=True)

        self.write("AGENTS.md", "# Agents\n")
        self.write("AGENTS-LOCAL.md", "# Agents Local\n")
        self.write("docs/DEV-SETUP.md", "# Dev Setup\n")
        self.write("docs/AGENT-REGISTRY.md", "# Agent Registry\n")
        self.write("docs/ROLE-CHECKLISTS/README.md", "# Role Checklists\n")
        self.write("docs/ROLE-CHECKLISTS/coding.md", "# Coding\n")
        self.write("docs/ROLE-CHECKLISTS/delivery.md", "# Delivery\n")
        self.write("docs/ROLE-CHECKLISTS/doc.md", "# Doc\n")
        self.write(".agent-local/DEV-SETUP-STATUS.example.md", "# Dev Setup Status Example\n")
        self.write(".agent-local/mailboxes/EXAMPLE-work-continuation-handoff.md", "# Work Handoff\n")
        self.write(".agent-local/mailboxes/EXAMPLE-delivery-continuation-note.md", "# Delivery Note\n")
        self.write(".agent-local/mailboxes/EXAMPLE-doc-continuation-note.md", "# Doc Note\n")
        self.write(".agent-local/mailboxes/EXAMPLE-planning-sync-handoff.md", "# Planning Sync\n")
        self.write(".agent-local/mailboxes/EXAMPLE-planning-sync-resolution.md", "# Planning Resolution\n")
        self.write(".agent-local/dev-setup-status.md", "# Local Dev Setup Status\n")
        self.write(".agent-local/agents.json", "{\n  \"version\": 2\n}\n")

        self.write("scripts/agent_bootstrap.py", "print('bootstrap')\n")
        self.write("scripts/agent_work_cycle.py", "print('work_cycle')\n")
        self.write("scripts/agent_registry.py", "print('registry')\n")
        self.write("scripts/agent_registry_reconcile.py", "print('registry_reconcile')\n")
        self.write("scripts/agent_guard.py", "print('guard')\n")
        self.write("scripts/agent_timestamp.py", "print('timestamp')\n")
        self.write("scripts/agent_safe_commit.py", "print('safe_commit')\n")
        self.write("scripts/agent_push.py", "print('push')\n")
        self.write("scripts/check-runtime-preflight.py", "print('runtime_preflight')\n")
        self.write("scripts/check-dev-env.py", "print('check_dev_env')\n")
        self.write("scripts/update-dev-setup-status.py", "print('update_dev_setup_status')\n")
        self.write("scripts/item_id_checklist.py", "print('item_id_checklist')\n")
        self.write("scripts/item_id_checklist_mark.py", "print('item_id_checklist_mark')\n")
        self.write("scripts/check_checklist_refresh.py", "print('check_checklist_refresh')\n")
        self.write("scripts/mailbox_handoff.py", "print('mailbox_handoff')\n")
        self.write("scripts/mailbox_gc.py", "print('mailbox_gc')\n")
        self.write("scripts/render_next_work_items.py", "print('render_next_work_items')\n")
        self.write("scripts/render_files_changed_table.py", "print('render_files_changed_table')\n")
        self.write("scripts/render_files_changed_from_json.py", "print('render_files_changed_from_json')\n")
        self.write("scripts/codex_thread_metadata.py", "print('codex_thread_metadata')\n")
        self.write("scripts/codex_token_usage_summary.py", "print('codex_token_usage_summary')\n")

        subprocess.run(["git", "init"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "config", "user.name", "Test User"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "config", "user.email", "test@example.com"], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "add", "."], cwd=self.root, check=True, capture_output=True, text=True)
        subprocess.run(["git", "commit", "-m", "initial"], cwd=self.root, check=True, capture_output=True, text=True)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def write(self, relative_path: str, content: str) -> None:
        path = self.root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            ["python3", str(self.root / "scripts" / "agent_workflow_bundle.py"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
            check=False,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def test_export_writes_manifest_and_default_files_to_directory(self) -> None:
        output_dir = self.root / "bundle"

        proc = self.run_cli("export", str(output_dir), "--json")

        payload = json.loads(proc.stdout)
        self.assertEqual("ok", payload["status"])
        self.assertEqual("dir", payload["bundle_format"])
        manifest = json.loads((output_dir / "manifest.json").read_text(encoding="utf-8"))
        self.assertEqual(1, manifest["bundle_version"])
        self.assertIn("docs", manifest["categories"])
        self.assertIn("templates", manifest["categories"])
        self.assertIn("scripts", manifest["categories"])
        paths = {entry["path"] for entry in manifest["files"]}
        self.assertIn("AGENTS.md", paths)
        self.assertIn("scripts/agent_workflow_bundle.py", paths)
        self.assertNotIn(".agent-local/agents.json", paths)
        self.assertTrue((output_dir / "files" / "AGENTS.md").is_file())
        self.assertTrue((output_dir / "files" / "scripts" / "agent_workflow_bundle.py").is_file())

    def test_export_can_include_optional_local_state_in_tar_bundle(self) -> None:
        output_path = self.root / "bundle.tar.gz"

        proc = self.run_cli("export", str(output_path), "--include", "local-state", "--json")

        payload = json.loads(proc.stdout)
        self.assertEqual("tar", payload["bundle_format"])
        with tarfile.open(output_path, "r:gz") as archive:
            manifest = json.loads(
                archive.extractfile("manifest.json").read().decode("utf-8")  # type: ignore[union-attr]
            )
        paths = {entry["path"] for entry in manifest["files"]}
        self.assertEqual({".agent-local/agents.json", ".agent-local/dev-setup-status.md"}, paths)

    def test_import_dry_run_reports_create_operations(self) -> None:
        bundle_dir = self.root / "bundle"
        self.run_cli("export", str(bundle_dir))
        dest_root = self.root / "import-target"

        proc = self.run_cli("import", str(bundle_dir), "--dest", str(dest_root), "--dry-run", "--json")

        payload = json.loads(proc.stdout)
        self.assertTrue(payload["dry_run"])
        self.assertGreater(payload["counts"]["create"], 0)
        self.assertFalse(dest_root.exists())
        self.assertFalse((dest_root / "AGENTS.md").exists())

    def test_import_rejects_conflicts_without_overwrite(self) -> None:
        bundle_dir = self.root / "bundle"
        self.run_cli("export", str(bundle_dir))
        dest_root = self.root / "import-target"
        self.write("import-target/AGENTS.md", "conflicting content\n")

        proc = self.run_cli("import", str(bundle_dir), "--dest", str(dest_root), check=False)

        self.assertEqual(1, proc.returncode)
        self.assertIn("--overwrite", proc.stderr)

    def test_import_overwrites_when_requested(self) -> None:
        bundle_dir = self.root / "bundle"
        self.run_cli("export", str(bundle_dir))
        dest_root = self.root / "import-target"
        self.write("import-target/AGENTS.md", "conflicting content\n")

        self.run_cli("import", str(bundle_dir), "--dest", str(dest_root), "--overwrite")

        self.assertEqual("# Agents\n", (dest_root / "AGENTS.md").read_text(encoding="utf-8"))

    def test_import_rejects_manifest_path_traversal(self) -> None:
        bundle_dir = self.root / "malicious-bundle"
        (bundle_dir / "files").mkdir(parents=True, exist_ok=True)
        manifest = {
            "bundle_version": 1,
            "created_at": "2026-03-30T00:00:00Z",
            "source_repo_root": str(self.root),
            "source_git_head": None,
            "categories": ["docs"],
            "missing_paths": [],
            "files": [
                {
                    "path": "../escaped.txt",
                    "categories": ["docs"],
                    "sha256": "0" * 64,
                    "size": 0,
                }
            ],
        }
        (bundle_dir / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")

        proc = self.run_cli("import", str(bundle_dir), "--dest", str(self.root / "import-target"), check=False)

        self.assertEqual(1, proc.returncode)
        self.assertIn("unsafe segments", proc.stderr)


if __name__ == "__main__":
    unittest.main()
