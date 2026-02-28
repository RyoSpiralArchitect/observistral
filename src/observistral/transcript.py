from __future__ import annotations

import re
from datetime import datetime
from pathlib import Path

from observistral.types import ChatMessage


def strip_wrapping_quotes(text: str) -> str:
    value = text.strip()
    if len(value) >= 2 and value[0] == value[-1] and value[0] in {"'", '"'}:
        return value[1:-1]
    return value


def default_transcript_filename(now: datetime | None = None) -> str:
    ts = (now or datetime.now()).strftime("%Y%m%d-%H%M%S")
    return f"obstral-transcript-{ts}.md"


def ensure_unique_path(path: Path) -> Path:
    if not path.exists():
        return path
    for i in range(1, 1000):
        candidate = path.with_name(f"{path.stem}-{i}{path.suffix}")
        if not candidate.exists():
            return candidate
    raise RuntimeError(f"Unable to find a free filename for: {path}")


def format_history_text(messages: list[ChatMessage], *, limit: int | None) -> str:
    if limit is not None and limit < 0:
        raise ValueError("limit must be >= 0 or None")

    slice_messages = messages if limit is None else messages[-limit:]
    if not slice_messages:
        return ""

    base_index = len(messages) - len(slice_messages) + 1
    lines: list[str] = []
    for idx, msg in enumerate(slice_messages, start=base_index):
        lines.append(f"[{idx:03d}] {msg.role}")
        lines.append(msg.content)
        lines.append("")
    return "\n".join(lines).rstrip()


def last_message_content(messages: list[ChatMessage], *, role: str, nth: int = 1) -> str | None:
    if nth <= 0:
        raise ValueError("nth must be >= 1")
    seen = 0
    for msg in reversed(messages):
        if msg.role != role:
            continue
        seen += 1
        if seen == nth:
            return msg.content
    return None


def _fence_for_text(text: str) -> str:
    runs = re.findall(r"`+", text)
    max_run = max((len(r) for r in runs), default=0)
    return "`" * max(3, max_run + 1)


def render_transcript_markdown(messages: list[ChatMessage], *, meta: dict[str, str]) -> str:
    lines: list[str] = ["# OBSTRAL transcript", ""]

    for key, value in meta.items():
        lines.append(f"- {key}: {value}")

    lines.extend(["", "---", ""])

    for msg in messages:
        lines.append(f"## {msg.role}")
        lines.append("")
        fence = _fence_for_text(msg.content)
        lines.append(f"{fence}text")
        lines.append(msg.content.rstrip("\n"))
        lines.append(fence)
        lines.append("")

    return "\n".join(lines).rstrip() + "\n"
