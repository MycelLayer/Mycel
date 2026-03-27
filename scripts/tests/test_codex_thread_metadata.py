import json
import sqlite3
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "codex_thread_metadata.py"


class CodexThreadMetadataCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        scripts_dir = self.root / "scripts"
        scripts_dir.mkdir(parents=True, exist_ok=True)
        target = scripts_dir / "codex_thread_metadata.py"
        target.write_text(SOURCE_SCRIPT.read_text(encoding="utf-8"), encoding="utf-8")
        target.chmod(0o755)

        self.codex_home = self.root / ".codex"
        self.codex_home.mkdir(parents=True, exist_ok=True)
        self.cwd = "/workspaces/Mycel"

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def write_state_db(self, *, thread_id: str, model: str = "gpt-5.4", effort: str = "medium") -> Path:
        state_db = self.codex_home / "state_7.sqlite"
        with sqlite3.connect(state_db) as conn:
            conn.execute(
                """
                CREATE TABLE threads (
                    id TEXT PRIMARY KEY,
                    cwd TEXT,
                    model TEXT,
                    reasoning_effort TEXT,
                    updated_at TEXT
                )
                """
            )
            conn.execute(
                """
                INSERT INTO threads (id, cwd, model, reasoning_effort, updated_at)
                VALUES (?, ?, ?, ?, ?)
                """,
                (thread_id, self.cwd, model, effort, "2026-03-27T15:19:21Z"),
            )
            conn.commit()
        return state_db

    def write_rollout(self, *, thread_id: str, model: str = "gpt-5.4", effort: str = "medium") -> Path:
        session_dir = self.codex_home / "sessions" / "2026" / "03" / "27"
        session_dir.mkdir(parents=True, exist_ok=True)
        rollout = session_dir / f"rollout-2026-03-27T15-17-33-{thread_id}.jsonl"
        lines = [
            {
                "timestamp": "2026-03-27T15:19:21.000Z",
                "type": "turn_context",
                "payload": {
                    "cwd": self.cwd,
                    "turn_id": "turn-123",
                    "model": model,
                    "effort": effort,
                },
            }
        ]
        rollout.write_text(
            "\n".join(json.dumps(line) for line in lines) + "\n",
            encoding="utf-8",
        )
        return rollout

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            [
                "python3",
                str(self.root / "scripts" / "codex_thread_metadata.py"),
                "--codex-home",
                str(self.codex_home),
                "--cwd",
                self.cwd,
                *args,
            ],
            cwd=self.root,
            text=True,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def test_shell_uses_state_db_without_sessions_tree(self) -> None:
        self.write_state_db(thread_id="019d2fdf-4245-7bb2-948f-76cd4ba9308f")

        proc = self.run_cli("--shell")

        self.assertIn('MODEL="gpt-5.4"', proc.stdout)
        self.assertIn('EFFORT="medium"', proc.stdout)
        self.assertIn('THREAD_ID="019d2fdf-4245-7bb2-948f-76cd4ba9308f"', proc.stdout)
        self.assertIn('STATE_DB=', proc.stdout)

    def test_json_uses_thread_lookup_then_targets_matching_rollout(self) -> None:
        thread_id = "019d2fdf-4245-7bb2-948f-76cd4ba9308f"
        self.write_state_db(thread_id=thread_id)
        rollout = self.write_rollout(thread_id=thread_id)

        proc = self.run_cli("--json")
        payload = json.loads(proc.stdout)

        self.assertEqual(thread_id, payload["thread_id"])
        self.assertEqual("gpt-5.4", payload["thread_model"])
        self.assertEqual("medium", payload["thread_reasoning_effort"])
        self.assertEqual("turn-123", payload["turn_id"])
        self.assertEqual(str(rollout), payload["session_path"])


if __name__ == "__main__":
    unittest.main()
