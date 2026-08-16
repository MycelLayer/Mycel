import json
import shutil
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CHECK_SCRIPT = REPO_ROOT / "scripts" / "check-dev-env.py"


class CheckDevEnvCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "scripts").mkdir(parents=True, exist_ok=True)
        shutil.copy2(CHECK_SCRIPT, self.root / "scripts" / "check-dev-env.py")
        (self.root / "scripts" / "check-dev-env.py").chmod(0o755)
        (self.root / "Cargo.toml").write_text(
            '[workspace]\n[workspace.package]\nrust-version = "1.94"\n',
            encoding="utf-8",
        )
        (self.root / "rust-toolchain.toml").write_text(
            '[toolchain]\nchannel = "stable"\n',
            encoding="utf-8",
        )
        smoke_script = self.root / "sim" / "negative-validation" / "smoke.sh"
        smoke_script.parent.mkdir(parents=True)
        smoke_script.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
        smoke_script.chmod(0o755)
        (self.root / "sgconfig.yml").write_text("ruleDirs: []\n", encoding="utf-8")
        self.bin_dir = self.root / "bin"
        self.bin_dir.mkdir()
        self.write_fake_tools()

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def write_fake_tools(self) -> None:
        scripts = {
            "cargo": textwrap.dedent(
                """\
                #!/usr/bin/env python3
                import sys
                args = sys.argv[1:]
                if args == ["--version"]:
                    print("cargo 1.94.0")
                else:
                    print("ok")
                """
            ),
            "rustup": textwrap.dedent(
                """\
                #!/usr/bin/env python3
                import sys
                args = sys.argv[1:]
                if args == ["--version"]:
                    print("rustup 1.29.0")
                elif args[:3] == ["component", "list", "--toolchain"]:
                    print("rustfmt-x86_64-unknown-linux-gnu (installed)")
                    print("clippy-x86_64-unknown-linux-gnu (installed)")
                else:
                    raise SystemExit(1)
                """
            ),
            "rustc": '#!/usr/bin/env bash\necho "rustc 1.94.0"\n',
            "gh": '#!/usr/bin/env bash\necho "gh 2.83.1"\n',
            "rg": '#!/usr/bin/env bash\necho "ripgrep 14.1.0"\n',
            "cargo-nextest": '#!/usr/bin/env bash\necho "cargo-nextest 0.9.143"\n',
            "ast-grep": textwrap.dedent(
                """\
                #!/usr/bin/env python3
                import sys
                if sys.argv[1:] == ["--version"]:
                    print("ast-grep 0.45.1")
                else:
                    print("ok")
                """
            ),
        }
        for name, body in scripts.items():
            path = self.bin_dir / name
            path.write_text(body, encoding="utf-8")
            path.chmod(0o755)

    def run_cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            [str(self.root / "scripts" / "check-dev-env.py"), *args],
            cwd=self.root,
            env={"PATH": f"{self.bin_dir}:/usr/bin:/bin"},
            text=True,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"check-dev-env failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def test_quick_json_reports_found_tools_and_components(self) -> None:
        proc = self.run_cli("--json")
        payload = json.loads(proc.stdout)

        self.assertEqual(0, proc.returncode)
        self.assertEqual("passed", payload["status"])
        self.assertEqual("quick", payload["mode"])
        self.assertEqual("stable", payload["required_toolchain_channel"])
        checks = {(entry["kind"], entry["name"]): entry["status"] for entry in payload["checks"]}
        self.assertEqual("found", checks[("command", "cargo")])
        self.assertEqual("found", checks[("component", "rustfmt")])

    def test_full_json_requires_ci_tools_and_runs_ci_parity_checks(self) -> None:
        proc = self.run_cli("--full", "--json")
        payload = json.loads(proc.stdout)

        checks = {(entry["kind"], entry["name"]): entry for entry in payload["checks"]}
        self.assertEqual("found", checks[("command", "cargo-nextest")]["status"])
        self.assertEqual("found", checks[("command", "ast-grep")]["status"])
        for validation in (
            "fmt",
            "clippy",
            "check",
            "workspace-tests",
            "doctests",
            "sim-smoke",
            "ast-grep-quality",
        ):
            self.assertEqual("passed", checks[("validation", validation)]["status"])
        self.assertEqual(
            "ast-grep scan --config sgconfig.yml --report-style short --format github",
            checks[("validation", "ast-grep-quality")]["detail"],
        )

    def test_full_json_fails_when_cargo_nextest_is_missing(self) -> None:
        (self.bin_dir / "cargo-nextest").unlink()

        proc = self.run_cli("--full", "--json", check=False)
        payload = json.loads(proc.stdout)

        self.assertEqual(1, proc.returncode)
        self.assertEqual("failed", payload["status"])
        self.assertEqual("missing required command: cargo-nextest", payload["error"])

    def test_full_json_fails_when_ast_grep_version_check_fails(self) -> None:
        (self.bin_dir / "ast-grep").write_text(
            "#!/usr/bin/env bash\nexit 2\n", encoding="utf-8"
        )

        proc = self.run_cli("--full", "--json", check=False)
        payload = json.loads(proc.stdout)

        self.assertEqual(1, proc.returncode)
        self.assertEqual("failed", payload["status"])
        self.assertEqual(
            "required command failed its version check: ast-grep", payload["error"]
        )


if __name__ == "__main__":
    unittest.main()
