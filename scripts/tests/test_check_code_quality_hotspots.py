import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "check_code_quality_hotspots.py"


class CheckCodeQualityHotspotsCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        target = self.root / "scripts" / "check_code_quality_hotspots.py"
        target.write_text(SOURCE_SCRIPT.read_text(encoding="utf-8"), encoding="utf-8")
        target.chmod(0o755)
        (self.root / "apps").mkdir(parents=True, exist_ok=True)
        (self.root / "crates").mkdir(parents=True, exist_ok=True)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["python3", str(self.root / "scripts" / "check_code_quality_hotspots.py"), *args],
            cwd=self.root,
            text=True,
            capture_output=True,
        )

    def test_reports_large_file_large_function_and_repeated_literals(self) -> None:
        rust_file = self.root / "crates" / "big.rs"
        rust_file.write_text(
            "\n".join(
                [
                    "fn too_big() {",
                    *["    let _value = \"repeated literal value\";" for _ in range(3)],
                    *["    let _n = 1;" for _ in range(5)],
                    "}",
                ]
            )
            + "\n",
            encoding="utf-8",
        )

        proc = self.run_cli(
            "--file-lines",
            "4",
            "--function-lines",
            "4",
            "--literal-repeats",
            "3",
            "crates",
        )

        self.assertEqual(0, proc.returncode)
        self.assertIn("[file-size] crates/big.rs:1", proc.stdout)
        self.assertIn("[function-size] crates/big.rs:1", proc.stdout)
        self.assertIn("[literal-repeat] crates/big.rs:2", proc.stdout)

    def test_supports_github_warning_and_fail_on_findings(self) -> None:
        py_file = self.root / "apps" / "warn.py"
        py_file.write_text(
            "\n".join(
                [
                    "def very_long_function():",
                    *["    value = 'another repeated literal'" for _ in range(3)],
                    *["    step = 1" for _ in range(5)],
                ]
            )
            + "\n",
            encoding="utf-8",
        )

        proc = self.run_cli(
            "--file-lines",
            "4",
            "--function-lines",
            "4",
            "--literal-repeats",
            "3",
            "--github-warning",
            "--fail-on-findings",
            "apps",
        )

        self.assertEqual(1, proc.returncode)
        self.assertIn("::warning file=apps/warn.py,line=1::", proc.stdout)
        self.assertIn("Summary:", proc.stdout)

    def test_reports_clean_when_no_hotspots_found(self) -> None:
        rust_file = self.root / "crates" / "small.rs"
        rust_file.write_text("fn ok() {\n    let _value = \"short\";\n}\n", encoding="utf-8")

        proc = self.run_cli("crates")

        self.assertEqual(0, proc.returncode)
        self.assertIn("No code-quality hotspots found", proc.stdout)


if __name__ == "__main__":
    unittest.main()
