import json
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_SCRIPT = REPO_ROOT / "scripts" / "render_next_work_items.py"


class RenderNextWorkItemsCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        scripts_dir = self.root / "scripts"
        scripts_dir.mkdir(parents=True, exist_ok=True)
        script = scripts_dir / "render_next_work_items.py"
        script.write_text(SOURCE_SCRIPT.read_text(encoding="utf-8"), encoding="utf-8")
        script.chmod(0o755)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_cli(
        self, *args: str, stdin_text: str = "", check: bool = True
    ) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            ["python3", str(self.root / "scripts" / "render_next_work_items.py"), *args],
            cwd=self.root,
            text=True,
            input=stdin_text,
            capture_output=True,
        )
        if check and proc.returncode != 0:
            self.fail(f"command failed {args}: {proc.stderr or proc.stdout}")
        return proc

    def test_renders_numbered_items_and_marks_first_as_highest_value(self) -> None:
        spec = {
            "items": [
                {
                    "text": "continue the M3 governance tooling slice",
                    "tradeoff": "best roadmap alignment, but it keeps us in the current surface area",
                    "roadmap": "M3 / Phase 2",
                },
                {
                    "text": "review the latest CQH issue",
                    "tradeoff": "usually cheaper to land, but less directly tied to the roadmap",
                },
            ]
        }

        proc = self.run_cli("-", stdin_text=json.dumps(spec))

        self.assertEqual(
            "1. (最有價值) continue the M3 governance tooling slice Tradeoff: best roadmap alignment, "
            "but it keeps us in the current surface area Roadmap: M3 / Phase 2\n"
            "2. review the latest CQH issue Tradeoff: usually cheaper to land, but less directly tied to the roadmap\n",
            proc.stdout,
        )

    def test_uses_role_defaults_when_only_role_is_provided(self) -> None:
        proc = self.run_cli("-", stdin_text=json.dumps({"role": "coding"}))

        self.assertEqual(
            "1. (最有價值) review ROADMAP.md and identify the highest-value next coding work Tradeoff: "
            "best roadmap alignment, but it spends a little time on prioritization before implementation "
            "Roadmap: ROADMAP.md / next coding slice\n"
            "2. review the latest CQH issue and identify high-value work items Tradeoff: usually cheaper "
            "to land quickly, but it may be less directly tied to the main roadmap lane\n",
            proc.stdout,
        )

    def test_prepends_compaction_item_when_compaction_is_detected(self) -> None:
        spec = {
            "compaction_detected": True,
            "items": [
                {
                    "text": "continue with the next coding slice",
                    "tradeoff": "fastest way back into implementation, but only after context is safe again",
                }
            ],
        }

        proc = self.run_cli("-", stdin_text=json.dumps(spec))

        self.assertEqual(
            "1. (最有價值) compaction detected, we better open a new chat. Tradeoff: safest follow-up after compaction, "
            "but it pauses immediate work until a fresh chat is open.\n"
            "2. continue with the next coding slice Tradeoff: fastest way back into implementation, but only after context is safe again\n",
            proc.stdout,
        )

    def test_prepends_compaction_item_ahead_of_role_defaults(self) -> None:
        spec = {"compaction_detected": True, "role": "coding"}

        proc = self.run_cli("-", stdin_text=json.dumps(spec))

        self.assertIn("1. (最有價值) compaction detected, we better open a new chat.", proc.stdout)
        self.assertIn(
            "2. review ROADMAP.md and identify the highest-value next coding work",
            proc.stdout,
        )

    def test_can_append_role_defaults_after_explicit_items(self) -> None:
        spec = {
            "role": "coding",
            "append_role_defaults": True,
            "items": [
                {
                    "text": "tighten the hotspot-scan checklist wording to touched files only",
                    "tradeoff": "matches the confirmed tool behavior, but it still leaves broader UX improvements for later",
                }
            ],
        }

        proc = self.run_cli("-", stdin_text=json.dumps(spec))

        self.assertEqual(
            "1. (最有價值) tighten the hotspot-scan checklist wording to touched files only Tradeoff: "
            "matches the confirmed tool behavior, but it still leaves broader UX improvements for later\n"
            "2. review ROADMAP.md and identify the highest-value next coding work Tradeoff: best roadmap alignment, "
            "but it spends a little time on prioritization before implementation Roadmap: ROADMAP.md / next coding slice\n"
            "3. review the latest CQH issue and identify high-value work items Tradeoff: usually cheaper to land quickly, "
            "but it may be less directly tied to the main roadmap lane\n",
            proc.stdout,
        )

    def test_uses_traditional_chinese_role_defaults_when_locale_is_zh_tw(self) -> None:
        proc = self.run_cli("-", stdin_text=json.dumps({"role": "coding", "locale": "zh-TW"}))

        self.assertEqual(
            "1. (最有價值) 檢查 ROADMAP.md，找出最高價值的下一個 coding 工作 取捨: "
            "和 roadmap 對齊最好，但在開始實作前需要先花一點時間做優先順序判斷 路線圖: ROADMAP.md / next coding slice\n"
            "2. 檢查最新的 CQH issue，整理高價值工作項目 取捨: 通常比較快能落地，但可能沒有那麼直接貼近主要 roadmap 軌道\n",
            proc.stdout,
        )

    def test_localizes_compaction_defaults_when_locale_is_zh_tw(self) -> None:
        proc = self.run_cli("-", stdin_text=json.dumps({"compaction_detected": True, "locale": "zh-TW"}))

        self.assertEqual(
            "1. (最有價值) 偵測到 compact context，我們最好開一個新聊天再繼續。 取捨: "
            "這是 compact context 後最安全的下一步，但會先暫停眼前工作，直到新聊天開好為止。\n",
            proc.stdout,
        )

    def test_accepts_compaction_only_output_without_other_items(self) -> None:
        spec = {"compaction_detected": True}

        proc = self.run_cli("-", stdin_text=json.dumps(spec))

        self.assertIn("1. (最有價值) compaction detected, we better open a new chat.", proc.stdout)

    def test_rejects_empty_spec_without_items_or_compaction(self) -> None:
        proc = self.run_cli("-", stdin_text=json.dumps({}), check=False)

        self.assertEqual(1, proc.returncode)
        self.assertIn("at least one item or set compaction_detected=true", proc.stderr)


if __name__ == "__main__":
    unittest.main()
