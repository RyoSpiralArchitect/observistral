from __future__ import annotations

import shutil
import unittest
from pathlib import Path
from uuid import uuid4

from observistral.cli import _read_diff_file, parse_args


class TestCli(unittest.TestCase):
    def _make_tmp_dir(self) -> Path:
        root = Path(__file__).resolve().parent / "_tmp"
        root.mkdir(exist_ok=True)
        path = root / uuid4().hex
        path.mkdir()
        self.addCleanup(lambda: shutil.rmtree(path, ignore_errors=True))
        return path

    def test_read_diff_file(self) -> None:
        tmp_dir = self._make_tmp_dir()
        diff = tmp_dir / "sample.diff"
        diff.write_text("+ hello", encoding="utf-8")

        text = _read_diff_file(str(diff))

        self.assertEqual(text, "+ hello")

    def test_read_diff_file_none(self) -> None:
        self.assertIsNone(_read_diff_file(None))

    def test_parse_args_persona_flags(self) -> None:
        args = parse_args(["hello", "--persona", "cynical", "--list-personas"])
        self.assertEqual(args.persona, "cynical")
        self.assertTrue(args.list_personas)

    def test_parse_args_list_modes_flag(self) -> None:
        args = parse_args(["hello", "--list-modes"])
        self.assertTrue(args.list_modes)

    def test_parse_args_vibe_flag(self) -> None:
        args = parse_args(["hello", "--vibe"])
        self.assertTrue(args.vibe)

    def test_parse_args_repl_flag(self) -> None:
        args = parse_args(["--repl"])
        self.assertTrue(args.repl)
