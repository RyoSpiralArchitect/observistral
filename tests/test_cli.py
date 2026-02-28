from __future__ import annotations

from pathlib import Path

from observistral.cli import _read_diff_file


def test_read_diff_file(tmp_path: Path) -> None:
    diff = tmp_path / "sample.diff"
    diff.write_text("+ hello", encoding="utf-8")

    text = _read_diff_file(str(diff))

    assert text == "+ hello"


def test_read_diff_file_none() -> None:
    assert _read_diff_file(None) is None
