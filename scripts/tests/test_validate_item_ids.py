import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "validate_item_ids.py"


class ValidateItemIdsCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE_SCRIPT, self.root / "scripts" / "validate_item_ids.py")
        (self.root / "scripts" / "validate_item_ids.py").chmod(0o755)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            [str(self.root / "scripts" / "validate_item_ids.py"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def write_file(self, relative_path: str, content: str) -> Path:
        path = self.root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        return path

    def test_valid_file_passes(self) -> None:
        self.write_file(
            "docs/checklist.md",
            """# Example

- [ ] First item <!-- item-id: bootstrap.git-status -->
- [X] Second item <!-- item-id: bootstrap.repo-layout -->
""",
        )

        proc = self.run_cli("docs/checklist.md")

        self.assertEqual(0, proc.returncode)
        self.assertIn("valid: True", proc.stdout)
        self.assertIn("item_ids: 2", proc.stdout)

    def test_duplicate_item_id_fails(self) -> None:
        self.write_file(
            "docs/checklist.md",
            """- [ ] First item <!-- item-id: duplicate.item -->
- [ ] Second item <!-- item-id: duplicate.item -->
""",
        )

        proc = self.run_cli("docs/checklist.md", check=False)

        self.assertEqual(1, proc.returncode)
        self.assertIn("duplicate item-id 'duplicate.item' first seen on line 1", proc.stdout)

    def test_invalid_item_id_value_fails(self) -> None:
        self.write_file(
            "docs/checklist.md",
            """- [ ] First item <!-- item-id: Invalid Item -->
""",
        )

        proc = self.run_cli("docs/checklist.md", check=False)

        self.assertEqual(1, proc.returncode)
        self.assertIn("invalid item-id 'Invalid Item'", proc.stdout)

    def test_item_id_must_be_on_list_item_line(self) -> None:
        self.write_file(
            "docs/checklist.md",
            """Paragraph only <!-- item-id: bootstrap.git-status -->
""",
        )

        proc = self.run_cli("docs/checklist.md", check=False)

        self.assertEqual(1, proc.returncode)
        self.assertIn("item-id must be attached to a Markdown list item line", proc.stdout)

    def test_missing_item_ids_fails(self) -> None:
        self.write_file(
            "docs/checklist.md",
            """# Example

No item ids here.
""",
        )

        proc = self.run_cli("docs/checklist.md", check=False)

        self.assertEqual(1, proc.returncode)
        self.assertIn("no item-id markers found", proc.stdout)

    def test_json_output_reports_issues(self) -> None:
        self.write_file(
            "docs/checklist.md",
            """- [ ] First item <!-- item-id: duplicate.item -->
- [ ] Second item <!-- item-id: duplicate.item -->
""",
        )

        proc = self.run_cli("docs/checklist.md", "--json", check=False)
        payload = json.loads(proc.stdout)

        self.assertEqual(1, proc.returncode)
        self.assertEqual("ok", payload["status"])
        self.assertEqual("docs/checklist.md", payload["results"][0]["path"])
        self.assertFalse(payload["results"][0]["valid"])
        self.assertEqual(2, payload["results"][0]["item_count"])


if __name__ == "__main__":
    unittest.main()
