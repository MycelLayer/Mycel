import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_CHECK = REPO_ROOT / "scripts" / "check_checklist_refresh.py"
SOURCE_ITEM_ID = REPO_ROOT / "scripts" / "item_id_checklist.py"


class CheckChecklistRefreshCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        (self.root / ".agent-local" / "agents" / "agt_doc" / "checklists").mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_CHECK, self.root / "scripts" / "check_checklist_refresh.py")
        shutil.copy2(SOURCE_ITEM_ID, self.root / "scripts" / "item_id_checklist.py")
        (self.root / "scripts" / "check_checklist_refresh.py").chmod(0o755)
        (self.root / "scripts" / "item_id_checklist.py").chmod(0o755)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def write_file(self, relative_path: str, content: str) -> Path:
        path = self.root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        return path

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            ["python3", str(self.root / "scripts" / "check_checklist_refresh.py"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def test_reports_up_to_date_for_matching_regular_checklist(self) -> None:
        self.write_file(
            "docs/source.md",
            """# Source

- Alpha <!-- item-id: item.alpha -->
""",
        )
        checklist = self.write_file(
            ".agent-local/agents/agt_doc/checklists/source-checklist.md",
            """# Agent Item-ID Checklist Copy

- Agent UID: `agt_doc`
- Display ID: `doc-1`
- Source: `docs/source.md`
- Generated at: `2026-03-16T09:00:00+0800`

# Source

- [X] Alpha <!-- item-id: item.alpha -->
""",
        )

        payload = json.loads(self.run_cli(str(checklist), "--json").stdout)

        self.assertEqual("up-to-date", payload["status"])
        self.assertEqual([], payload["reasons"])

    def test_reports_refresh_needed_for_new_item_ids(self) -> None:
        self.write_file(
            "docs/source.md",
            """# Source

- Alpha <!-- item-id: item.alpha -->
- Beta <!-- item-id: item.beta -->
""",
        )
        checklist = self.write_file(
            ".agent-local/agents/agt_doc/checklists/source-checklist.md",
            """# Agent Item-ID Checklist Copy

- Agent UID: `agt_doc`
- Display ID: `doc-1`
- Source: `docs/source.md`
- Generated at: `2026-03-16T09:00:00+0800`

# Source

- [ ] Alpha <!-- item-id: item.alpha -->
""",
        )

        payload = json.loads(self.run_cli(str(checklist), "--json").stdout)

        self.assertEqual("refresh-needed", payload["status"])
        self.assertEqual(["item.beta"], payload["missing_item_ids"])

    def test_reports_refresh_needed_for_agents_workcycle_wording_change(self) -> None:
        self.write_file(
            "AGENTS.md",
            """# Repo Working Agreements

## New chat bootstrap
- Bootstrap <!-- item-id: bootstrap.one -->

## Work Cycle Workflow
- Workflow updated wording <!-- item-id: workflow.one -->
""",
        )
        checklist = self.write_file(
            ".agent-local/agents/agt_doc/checklists/AGENTS-workcycle-checklist-3.md",
            """# Agent Item-ID Checklist Copy

- Agent UID: `agt_doc`
- Display ID: `doc-1`
- Source: `AGENTS.md`
- Generated at: `2026-03-16T09:00:00+0800`

# Repo Working Agreements

## Work Cycle Workflow

- [ ] Workflow old wording <!-- item-id: workflow.one -->
""",
        )

        payload = json.loads(self.run_cli(str(checklist), "--json").stdout)

        self.assertEqual("refresh-needed", payload["status"])
        self.assertIn("source checklist wording or structure changed", payload["reasons"])

    def test_ignores_problem_subitems_when_checking_refresh(self) -> None:
        self.write_file(
            "docs/source.md",
            """# Source

- Beta <!-- item-id: item.beta -->
""",
        )
        checklist = self.write_file(
            ".agent-local/agents/agt_doc/checklists/source-checklist.md",
            """# Agent Item-ID Checklist Copy

- Agent UID: `agt_doc`
- Display ID: `doc-1`
- Source: `docs/source.md`
- Generated at: `2026-03-16T09:00:00+0800`

# Source

- [!] Beta <!-- item-id: item.beta -->
  - Problem: local note only
""",
        )

        payload = json.loads(self.run_cli(str(checklist), "--json").stdout)

        self.assertEqual("up-to-date", payload["status"])


if __name__ == "__main__":
    unittest.main()
