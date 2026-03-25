import json
import os
import shutil
import sqlite3
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "agent_registry_reconcile.py"


class AgentRegistryReconcileTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        (self.root / ".agent-local" / "agents" / "agt_old" / "workcycles").mkdir(parents=True, exist_ok=True)
        (self.root / ".agent-local").mkdir(parents=True, exist_ok=True)
        (self.root / ".codex").mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_SCRIPT, self.root / "scripts" / "agent_registry_reconcile.py")
        (self.root / "scripts" / "agent_registry_reconcile.py").chmod(0o755)

        registry = {
            "version": 2,
            "updated_at": "2026-03-25T22:00:00+0800",
            "agent_count": 1,
            "agents": [
                {
                    "agent_uid": "agt_old",
                    "role": "coding",
                    "current_display_id": "coding-7",
                    "display_history": [
                        {
                            "display_id": "coding-7",
                            "assigned_at": "2026-03-25T21:00:00+0800",
                            "released_at": None,
                            "released_reason": None,
                        }
                    ],
                    "assigned_by": "user",
                    "assigned_at": "2026-03-25T21:00:00+0800",
                    "confirmed_by_agent": True,
                    "confirmed_at": "2026-03-25T21:00:00+0800",
                    "last_touched_at": "2026-03-25T21:05:00+0800",
                    "inactive_at": None,
                    "paused_at": None,
                    "status": "active",
                    "scope": "old-scope",
                    "files": [],
                    "mailbox": ".agent-local/mailboxes/agt_old.md",
                    "recovery_of": None,
                    "superseded_by": None,
                }
            ],
        }
        (self.root / ".agent-local" / "agents.json").write_text(
            json.dumps(registry, indent=2) + "\n",
            encoding="utf-8",
        )

        (self.root / ".agent-local" / "agents" / "agt_old" / "workcycles" / "token-usage-1.json").write_text(
            json.dumps(
                {
                    "thread_id": "thread-old",
                    "rollout_path": str(self.root / ".codex" / "sessions" / "rollout-old.jsonl"),
                    "timestamp": "2026-03-25T13:00:00Z",
                    "input_tokens": 1000,
                },
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )

        state_db = self.root / ".codex" / "state_1.sqlite"
        with sqlite3.connect(state_db) as conn:
            conn.execute(
                "CREATE TABLE threads (id TEXT PRIMARY KEY, cwd TEXT, model TEXT, reasoning_effort TEXT, updated_at TEXT)"
            )
            conn.execute(
                "INSERT INTO threads (id, cwd, model, reasoning_effort, updated_at) VALUES (?, ?, ?, ?, ?)",
                ("thread-old", str(self.root), "gpt-5.4", "medium", "2026-03-25T13:00:00Z"),
            )
            conn.commit()
        self.state_db = state_db

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        env = dict(os.environ)
        proc = subprocess.run(
            [str(self.root / "scripts" / "agent_registry_reconcile.py"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
            env=env,
        )
        if check and proc.returncode != 0:
            self.fail(proc.stderr or proc.stdout)
        return proc

    def test_scan_reports_stale_active_agent(self) -> None:
        proc = self.run_cli(
            "scan",
            "--registry",
            str(self.root / ".agent-local" / "agents.json"),
            "--state-db",
            str(self.state_db),
            "--stale-after-minutes",
            "15",
            "--json",
        )
        payload = json.loads(proc.stdout)
        self.assertEqual(1, len(payload["agents"]))
        entry = payload["agents"][0]
        self.assertTrue(entry["stale_active"])
        self.assertEqual("thread-old", entry["evidence"]["thread_id"])
        self.assertIsNotNone(entry["evidence"]["sqlite_thread_updated_at"])

    def test_reconcile_downgrades_stale_active_agent_to_inactive(self) -> None:
        proc = self.run_cli(
            "reconcile",
            "--registry",
            str(self.root / ".agent-local" / "agents.json"),
            "--state-db",
            str(self.state_db),
            "--stale-after-minutes",
            "15",
            "--downgrade-to",
            "inactive",
            "--json",
        )
        payload = json.loads(proc.stdout)
        self.assertEqual(1, payload["updated"])

        registry = json.loads((self.root / ".agent-local" / "agents.json").read_text(encoding="utf-8"))
        self.assertEqual("inactive", registry["agents"][0]["status"])
        self.assertIsNotNone(registry["agents"][0]["inactive_at"])


if __name__ == "__main__":
    unittest.main()
