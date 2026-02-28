from __future__ import annotations

import json
import os
import sys


def _to_prompt(messages: list[dict[str, object]]) -> str:
    lines: list[str] = []
    for m in messages:
        role = str(m.get("role", "")).upper()
        content = str(m.get("content", ""))
        lines.append(f"{role}: {content}")
    lines.append("ASSISTANT:")
    return "\n".join(lines)


def main() -> int:
    try:
        req = json.load(sys.stdin)
    except Exception as exc:  # noqa: BLE001
        print(f"invalid JSON input: {exc}", file=sys.stderr)
        return 2

    model = str(req.get("model", "")).strip()
    if not model:
        print("missing model", file=sys.stderr)
        return 2

    messages = req.get("messages") or []
    if not isinstance(messages, list):
        print("messages must be a list", file=sys.stderr)
        return 2

    temperature = float(req.get("temperature", 0.4))
    max_new_tokens = int(req.get("max_new_tokens", 256))
    device = str(req.get("device", "auto")).strip().lower()
    local_only = bool(req.get("local_only", False)) or os.getenv("OBS_HF_LOCAL_ONLY", "0") == "1"

    device_arg = 0 if device == "cuda" else -1

    try:
        from transformers import pipeline  # type: ignore
    except Exception as exc:  # noqa: BLE001
        print(f"transformers import failed: {exc}", file=sys.stderr)
        return 3

    gen = pipeline(
        "text-generation",
        model=model,
        device=device_arg,
        local_files_only=local_only,
    )
    prompt = _to_prompt(messages)
    out = gen(
        prompt,
        max_new_tokens=max_new_tokens,
        temperature=temperature,
        do_sample=True,
        return_full_text=True,
    )
    text = out[0].get("generated_text", "") if out else ""
    answer = text[len(prompt) :].strip() if isinstance(text, str) and text.startswith(prompt) else str(text).strip()

    json.dump({"content": answer, "model": model}, sys.stdout, ensure_ascii=False)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

