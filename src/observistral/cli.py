from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path

from observistral.chatbot import ChatBot
from observistral.config import ProviderConfig
from observistral.factory import build_provider, supported_providers
from observistral.personas import supported_personas



def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Observistral chatbot with provider abstraction")
    parser.add_argument("prompt", nargs="?", default="", help="chat input or review request")
    parser.add_argument("--mode", choices=["実況", "壁打ち", "diff批評"], default="壁打ち")
    parser.add_argument("--persona", default=os.getenv("OBS_PERSONA", "default"))
    parser.add_argument("--provider", default=os.getenv("OBS_PROVIDER", "openai-compatible"))
    parser.add_argument("--model", default=os.getenv("OBS_MODEL", "gpt-4o-mini"))
    parser.add_argument("--api-key", default=os.getenv("OBS_API_KEY"))
    parser.add_argument("--base-url", default=os.getenv("OBS_BASE_URL"))
    parser.add_argument("--temperature", type=float, default=0.4)
    parser.add_argument("--max-tokens", type=int, default=1024)
    parser.add_argument("--timeout-seconds", type=int, default=int(os.getenv("OBS_TIMEOUT_SECONDS", "120")))
    parser.add_argument("--device", default=os.getenv("OBS_HF_DEVICE", "auto"), help="HF local device: auto|cpu|cuda")
    parser.add_argument("--diff-file", help="Path to git diff or patch file for diff批評")
    parser.add_argument("--stdin", action="store_true", help="Append stdin text to prompt")
    parser.add_argument("--list-providers", action="store_true", help="List supported provider aliases and exit")
    parser.add_argument("--list-personas", action="store_true", help="List supported personas and exit")
    return parser.parse_args(argv)



def _read_diff_file(path: str | None) -> str | None:
    if not path:
        return None
    diff_path = Path(path)
    if not diff_path.exists():
        raise FileNotFoundError(f"diff file not found: {path}")
    return diff_path.read_text(encoding="utf-8")



def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)

    if args.list_providers:
        print("\n".join(supported_providers()))
        return 0

    if args.list_personas:
        print("\n".join(supported_personas()))
        return 0

    stdin_text = sys.stdin.read().strip() if args.stdin else ""
    prompt = args.prompt.strip()
    if stdin_text:
        prompt = f"{prompt}\n{stdin_text}".strip()

    if not prompt:
        raise ValueError("prompt is required (argument or --stdin input)")

    config = ProviderConfig(
        provider=args.provider,
        model=args.model,
        api_key=args.api_key,
        base_url=args.base_url,
        timeout_seconds=args.timeout_seconds,
        device=args.device,
    )

    provider = build_provider(config)
    bot = ChatBot(provider=provider)
    resp = bot.run(
        user_input=prompt,
        mode=args.mode,
        persona=args.persona,
        temperature=args.temperature,
        max_tokens=args.max_tokens,
        diff_text=_read_diff_file(args.diff_file),
    )
    print(resp.content)
    return 0


if __name__ == "__main__":
    sys.exit(main())
