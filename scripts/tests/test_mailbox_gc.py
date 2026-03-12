import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "mailbox_gc.py"


class MailboxGcCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        (self.root / ".agent-local" / "mailboxes").mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_SCRIPT, self.root / "scripts" / "mailbox_gc.py")
        (self.root / "scripts" / "mailbox_gc.py").chmod(0o755)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            [str(self.root / "scripts" / "mailbox_gc.py"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def write_registry(self, payload: dict) -> None:
        registry_path = self.root / ".agent-local" / "agents.json"
        registry_path.parent.mkdir(parents=True, exist_ok=True)
        registry_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    def write_mailbox(self, relative_path: str, content: str = "# mailbox\n") -> None:
        path = self.root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def registry_entry(self, agent_uid: str) -> dict:
        return {
            "agent_uid": agent_uid,
            "role": "coding",
            "current_display_id": "coding-1",
            "display_history": [
                {
                    "display_id": "coding-1",
                    "assigned_at": "2026-03-12T12:00:00+0800",
                    "released_at": None,
                    "released_reason": None,
                }
            ],
            "assigned_by": "user",
            "assigned_at": "2026-03-12T12:00:00+0800",
            "confirmed_by_agent": True,
            "confirmed_at": "2026-03-12T12:00:00+0800",
            "last_touched_at": "2026-03-12T12:00:00+0800",
            "inactive_at": None,
            "status": "active",
            "scope": "mailbox-gc-test",
            "files": [],
            "mailbox": f".agent-local/mailboxes/{agent_uid}.md",
            "recovery_of": None,
            "superseded_by": None,
        }

    def test_scan_reports_referenced_missing_orphaned_and_archived_mailboxes(self) -> None:
        self.write_registry(
            {
                "version": 2,
                "updated_at": "2026-03-12T12:00:00+0800",
                "agent_count": 2,
                "agents": [
                    self.registry_entry("agt_live"),
                    {
                        **self.registry_entry("agt_missing"),
                        "mailbox": ".agent-local/mailboxes/agt_missing.md",
                    },
                ],
            }
        )
        self.write_mailbox(".agent-local/mailboxes/agt_live.md")
        self.write_mailbox(".agent-local/mailboxes/agt_orphan.md")
        self.write_mailbox(".agent-local/mailboxes/EXAMPLE-planning-sync-handoff.md")
        self.write_mailbox(".agent-local/mailboxes/archive/2026-03/agt_old.md")

        payload = json.loads(self.run_cli("scan", "--json").stdout)

        self.assertEqual(1, payload["referenced_count"])
        self.assertEqual(1, payload["missing_referenced_count"])
        self.assertEqual(1, payload["orphaned_count"])
        self.assertEqual(1, payload["archived_count"])
        self.assertEqual(".agent-local/mailboxes/agt_live.md", payload["referenced"][0]["path"])
        self.assertEqual(".agent-local/mailboxes/agt_missing.md", payload["missing_referenced"][0]["mailbox"])
        self.assertEqual(".agent-local/mailboxes/agt_orphan.md", payload["orphaned"][0]["path"])

    def test_archive_moves_only_orphaned_uid_mailboxes(self) -> None:
        self.write_registry(
            {
                "version": 2,
                "updated_at": "2026-03-12T12:00:00+0800",
                "agent_count": 1,
                "agents": [self.registry_entry("agt_live")],
            }
        )
        self.write_mailbox(".agent-local/mailboxes/agt_live.md", "# live\n")
        self.write_mailbox(".agent-local/mailboxes/agt_orphan.md", "# orphan\n")
        self.write_mailbox(".agent-local/mailboxes/EXAMPLE-planning-sync-handoff.md", "# example\n")

        payload = json.loads(self.run_cli("archive", "--json").stdout)

        self.assertEqual(1, payload["archived_count"])
        self.assertTrue((self.root / ".agent-local/mailboxes/agt_live.md").exists())
        self.assertFalse((self.root / ".agent-local/mailboxes/agt_orphan.md").exists())
        self.assertTrue((self.root / payload["archived"][0]["destination"]).exists())
        self.assertTrue((self.root / ".agent-local/mailboxes/EXAMPLE-planning-sync-handoff.md").exists())

    def test_archive_dry_run_reports_moves_without_changing_files(self) -> None:
        self.write_registry(
            {
                "version": 2,
                "updated_at": "2026-03-12T12:00:00+0800",
                "agent_count": 0,
                "agents": [],
            }
        )
        self.write_mailbox(".agent-local/mailboxes/agt_orphan.md", "# orphan\n")

        payload = json.loads(self.run_cli("archive", "--dry-run", "--json").stdout)

        self.assertTrue(payload["dry_run"])
        self.assertEqual(1, payload["archived_count"])
        self.assertTrue((self.root / ".agent-local/mailboxes/agt_orphan.md").exists())


if __name__ == "__main__":
    unittest.main()
