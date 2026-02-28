from __future__ import annotations

import argparse
import os
import sys
from datetime import datetime
from pathlib import Path

from observistral.chatbot import ChatBot, build_system_text, compose_user_text, supported_modes
from observistral.config import ProviderConfig
from observistral.factory import MISTRAL_NAMES
from observistral.factory import build_provider, supported_providers
from observistral.personas import supported_personas
from observistral.types import ChatMessage, ChatRequest
from observistral.transcript import (
    default_transcript_filename,
    ensure_unique_path,
    format_history_text,
    last_message_content,
    render_transcript_markdown,
    strip_wrapping_quotes,
)

DEFAULT_HISTORY_LIMIT = 20


def _configure_utf8_stdio() -> None:
    for stream in (sys.stdout, sys.stderr):
        try:
            reconfigure = getattr(stream, "reconfigure", None)
            if reconfigure is not None:
                reconfigure(encoding="utf-8")
        except Exception:
            pass


def _print_effective_config(
    *,
    provider: str,
    model: str,
    base_url: str | None,
    mode: str,
    persona: str,
    temperature: float,
    max_tokens: int,
) -> None:
    print(
        "\n".join(
            [
                "[OBSTRAL]",
                f"provider: {provider}",
                f"model: {model}",
                f"base_url: {base_url or '(default)'}",
                f"mode: {mode}",
                f"persona: {persona}",
                f"temperature: {temperature}",
                f"max_tokens: {max_tokens}",
            ]
        ),
        file=sys.stderr,
    )


def _resolve_api_key(provider: str, api_key: str | None) -> str | None:
    if api_key:
        return api_key
    if provider.strip().lower() in MISTRAL_NAMES:
        return os.getenv("MISTRAL_API_KEY")
    if provider.strip().lower() == "anthropic":
        return os.getenv("ANTHROPIC_API_KEY")
    return None


def _run_repl(
    *,
    provider: str,
    model: str,
    api_key: str | None,
    base_url: str | None,
    timeout_seconds: int,
    device: str,
    mode: str,
    persona: str,
    temperature: float,
    max_tokens: int,
) -> int:
    history: list[ChatMessage] = []

    def rebuild_provider(cfg: ProviderConfig):
        nonlocal bot
        provider_client = build_provider(cfg)
        bot = ChatBot(provider=provider_client)

    def reset_history(reason: str) -> None:
        history.clear()
        print(f"[reset] {reason}", file=sys.stderr)

    config = ProviderConfig(
        provider=provider,
        model=model,
        api_key=_resolve_api_key(provider, api_key),
        base_url=base_url,
        timeout_seconds=timeout_seconds,
        device=device,
    )

    bot: ChatBot
    rebuild_provider(config)

    print("OBSTRAL REPL: /help, /exit", file=sys.stderr)
    _print_effective_config(
        provider=config.provider,
        model=config.model,
        base_url=config.base_url,
        mode=mode,
        persona=persona,
        temperature=temperature,
        max_tokens=max_tokens,
    )

    while True:
        try:
            line = input(f"obstral[{mode}|{persona}|{config.provider}]> ").strip()
        except EOFError:
            print("", file=sys.stderr)
            return 0
        except KeyboardInterrupt:
            print("", file=sys.stderr)
            return 130

        if not line:
            continue

        if line.startswith("/"):
            cmdline = line[1:].strip()
            if not cmdline:
                continue
            parts = cmdline.split(maxsplit=1)
            cmd = parts[0].lower()
            arg = parts[1].strip() if len(parts) == 2 else ""

            if cmd in {"exit", "quit"}:
                return 0

            if cmd == "help":
                print(
                    "\n".join(
                        [
                            "Commands:",
                            "  /help",
                            "  /exit",
                            "  /config",
                            "  /reset",
                            "  /history [n|all]",
                            "  /save [path]",
                            "  /copy [n]",
                            "  /modes",
                            "  /personas",
                            "  /providers",
                            "  /mode <name>",
                            "  /persona <name>",
                            "  /provider <name>",
                            "  /model <name>",
                            "  /base-url <url>",
                            "  /api-key <key>",
                            "  /temperature <float>",
                            "  /max-tokens <int>",
                            "  /vibe  (mode=VIBE, provider=mistral, model=devstral-2)",
                        ]
                    ),
                    file=sys.stderr,
                )
                continue

            if cmd == "config":
                _print_effective_config(
                    provider=config.provider,
                    model=config.model,
                    base_url=config.base_url,
                    mode=mode,
                    persona=persona,
                    temperature=temperature,
                    max_tokens=max_tokens,
                )
                continue

            if cmd == "reset":
                reset_history("history cleared")
                continue

            if cmd == "history":
                raw = arg.strip()
                if not raw:
                    limit: int | None = DEFAULT_HISTORY_LIMIT
                elif raw.lower() == "all":
                    limit = None
                else:
                    try:
                        limit = int(raw)
                    except ValueError:
                        print("Usage: /history [n|all]", file=sys.stderr)
                        continue

                count = len(history)
                if limit is None:
                    print(f"[history] all ({count} messages)", file=sys.stderr)
                else:
                    print(f"[history] last {min(limit, count)}/{count} messages", file=sys.stderr)

                text = format_history_text(history, limit=limit)
                print(text or "(empty)", file=sys.stderr)
                continue

            if cmd == "save":
                path_arg = strip_wrapping_quotes(arg) if arg else ""
                out_path = Path(path_arg).expanduser() if path_arg else (Path.cwd() / default_transcript_filename())
                out_path = ensure_unique_path(out_path)
                out_path.parent.mkdir(parents=True, exist_ok=True)

                meta = {
                    "saved_at": datetime.now().isoformat(timespec="seconds"),
                    "provider": config.provider,
                    "model": config.model,
                    "base_url": config.base_url or "(default)",
                    "mode": mode,
                    "persona": persona,
                    "temperature": str(temperature),
                    "max_tokens": str(max_tokens),
                }
                out_path.write_text(render_transcript_markdown(history, meta=meta), encoding="utf-8")
                print(f"[saved] {out_path}", file=sys.stderr)
                continue

            if cmd == "copy":
                raw = arg.strip()
                if not raw:
                    nth = 1
                else:
                    try:
                        nth = int(raw)
                    except ValueError:
                        print("Usage: /copy [n]", file=sys.stderr)
                        continue

                content = last_message_content(history, role="assistant", nth=nth)
                if content is None:
                    print("(no assistant message)", file=sys.stderr)
                else:
                    print(content)
                continue

            if cmd == "modes":
                print("\n".join(supported_modes()), file=sys.stderr)
                continue

            if cmd == "personas":
                print("\n".join(supported_personas()), file=sys.stderr)
                continue

            if cmd == "providers":
                print("\n".join(supported_providers()), file=sys.stderr)
                continue

            if cmd == "mode":
                if not arg:
                    print("Usage: /mode <name>", file=sys.stderr)
                    continue
                value = "VIBE" if arg.strip().lower() == "vibe" else arg.strip()
                if value not in supported_modes():
                    print(f"Unsupported mode: {value}", file=sys.stderr)
                    continue
                mode = value
                reset_history(f"mode => {mode}")
                continue

            if cmd == "persona":
                if not arg:
                    print("Usage: /persona <name>", file=sys.stderr)
                    continue
                key = arg.strip().lower()
                if key not in supported_personas():
                    print(f"Unsupported persona: {key}", file=sys.stderr)
                    continue
                persona = key
                reset_history(f"persona => {persona}")
                continue

            if cmd == "provider":
                if not arg:
                    print("Usage: /provider <name>", file=sys.stderr)
                    continue
                value = arg.strip().lower()
                if value not in supported_providers():
                    print(f"Unsupported provider: {value}", file=sys.stderr)
                    continue
                prev_config = config
                config = ProviderConfig(
                    provider=value,
                    model=prev_config.model,
                    api_key=_resolve_api_key(value, prev_config.api_key),
                    base_url=prev_config.base_url,
                    timeout_seconds=prev_config.timeout_seconds,
                    device=prev_config.device,
                )
                try:
                    rebuild_provider(config)
                except Exception as exc:
                    print(f"Error: {exc}", file=sys.stderr)
                    config = prev_config
                    rebuild_provider(prev_config)
                    continue
                reset_history(f"provider => {config.provider}")
                continue

            if cmd == "model":
                if not arg:
                    print("Usage: /model <name>", file=sys.stderr)
                    continue
                prev = config.model
                config = ProviderConfig(
                    provider=config.provider,
                    model=arg,
                    api_key=config.api_key,
                    base_url=config.base_url,
                    timeout_seconds=config.timeout_seconds,
                    device=config.device,
                )
                try:
                    rebuild_provider(config)
                except Exception as exc:
                    print(f"Error: {exc}", file=sys.stderr)
                    config = ProviderConfig(
                        provider=config.provider,
                        model=prev,
                        api_key=config.api_key,
                        base_url=config.base_url,
                        timeout_seconds=config.timeout_seconds,
                        device=config.device,
                    )
                    rebuild_provider(config)
                    continue
                reset_history(f"model => {config.model}")
                continue

            if cmd == "base-url":
                if not arg:
                    print("Usage: /base-url <url>", file=sys.stderr)
                    continue
                config = ProviderConfig(
                    provider=config.provider,
                    model=config.model,
                    api_key=config.api_key,
                    base_url=arg,
                    timeout_seconds=config.timeout_seconds,
                    device=config.device,
                )
                rebuild_provider(config)
                reset_history(f"base_url => {config.base_url}")
                continue

            if cmd == "api-key":
                if not arg:
                    print("Usage: /api-key <key>", file=sys.stderr)
                    continue
                config = ProviderConfig(
                    provider=config.provider,
                    model=config.model,
                    api_key=arg,
                    base_url=config.base_url,
                    timeout_seconds=config.timeout_seconds,
                    device=config.device,
                )
                rebuild_provider(config)
                reset_history("api_key updated")
                continue

            if cmd == "temperature":
                if not arg:
                    print("Usage: /temperature <float>", file=sys.stderr)
                    continue
                try:
                    temperature = float(arg)
                except ValueError:
                    print(f"Invalid temperature: {arg}", file=sys.stderr)
                    continue
                print(f"temperature => {temperature}", file=sys.stderr)
                continue

            if cmd == "max-tokens":
                if not arg:
                    print("Usage: /max-tokens <int>", file=sys.stderr)
                    continue
                try:
                    max_tokens = int(arg)
                except ValueError:
                    print(f"Invalid max-tokens: {arg}", file=sys.stderr)
                    continue
                print(f"max_tokens => {max_tokens}", file=sys.stderr)
                continue

            if cmd == "vibe":
                mode = "VIBE"
                config = ProviderConfig(
                    provider="mistral",
                    model="devstral-2",
                    api_key=_resolve_api_key("mistral", config.api_key),
                    base_url=config.base_url,
                    timeout_seconds=config.timeout_seconds,
                    device=config.device,
                )
                rebuild_provider(config)
                reset_history("vibe preset applied")
                continue

            print(f"Unknown command: /{cmd}", file=sys.stderr)
            continue

        # Send a multi-turn request to the provider with history.
        system_text = build_system_text(mode=mode, persona=persona)
        user_text = compose_user_text(user_input=line, mode=mode, diff_text=None)
        history.append(ChatMessage(role="user", content=user_text))

        request = ChatRequest(
            messages=[ChatMessage(role="system", content=system_text), *history],
            temperature=temperature,
            max_tokens=max_tokens,
        )
        try:
            resp = bot.provider.chat(request)
        except Exception as exc:
            history.pop()
            print(f"Error: {exc}", file=sys.stderr)
            continue
        history.append(ChatMessage(role="assistant", content=resp.content))
        print(resp.content)



def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="OBSTRAL (observistral): provider-abstracted chatbot runner")
    parser.add_argument("prompt", nargs="?", default="", help="chat input or review request")
    parser.add_argument("--mode", choices=supported_modes(), default=None)
    parser.add_argument("--persona", default=None)
    parser.add_argument("--provider", default=None)
    parser.add_argument("--model", default=None)
    parser.add_argument("--api-key", default=None)
    parser.add_argument("--base-url", default=None)
    parser.add_argument("--temperature", type=float, default=0.4)
    parser.add_argument("--max-tokens", type=int, default=1024)
    parser.add_argument("--timeout-seconds", type=int, default=int(os.getenv("OBS_TIMEOUT_SECONDS", "120")))
    parser.add_argument("--device", default=os.getenv("OBS_HF_DEVICE", "auto"), help="HF local device: auto|cpu|cuda")
    parser.add_argument("--diff-file", help="Path to git diff or patch file for diff批評")
    parser.add_argument("--stdin", action="store_true", help="Append stdin text to prompt")
    parser.add_argument("--vibe", action="store_true", help="Shortcut for coding: --mode VIBE, --provider mistral, --model devstral-2")
    parser.add_argument("--repl", action="store_true", help="Start an interactive REPL session (auto when no prompt)")
    parser.add_argument("--show-config", action="store_true", help="Print effective config to stderr")
    parser.add_argument("--traceback", action="store_true", help="Show traceback on errors")
    parser.add_argument("--list-providers", action="store_true", help="List supported provider aliases and exit")
    parser.add_argument("--list-personas", action="store_true", help="List supported personas and exit")
    parser.add_argument("--list-modes", action="store_true", help="List supported modes and exit")
    return parser.parse_args(argv)



def _read_diff_file(path: str | None) -> str | None:
    if not path:
        return None
    diff_path = Path(path)
    if not diff_path.exists():
        raise FileNotFoundError(f"diff file not found: {path}")
    return diff_path.read_text(encoding="utf-8")



def main(argv: list[str] | None = None) -> int:
    _configure_utf8_stdio()
    args = parse_args(argv)

    try:
        provider = args.provider or os.getenv("OBS_PROVIDER") or ("mistral" if args.vibe else "openai-compatible")
        persona = args.persona or os.getenv("OBS_PERSONA", "default")
        mode = args.mode or ("VIBE" if args.vibe else "壁打ち")
        base_url = args.base_url or os.getenv("OBS_BASE_URL")

        model = args.model or os.getenv("OBS_MODEL")
        if not model:
            model = "devstral-2" if provider.strip().lower() in MISTRAL_NAMES else "gpt-4o-mini"

        api_key = _resolve_api_key(provider, args.api_key or os.getenv("OBS_API_KEY"))

        if args.list_providers:
            print("\n".join(supported_providers()))
            return 0

        if args.list_personas:
            print("\n".join(supported_personas()))
            return 0

        if args.list_modes:
            print("\n".join(supported_modes()))
            return 0

        stdin_text = sys.stdin.read().strip() if args.stdin else ""
        prompt = args.prompt.strip()
        if stdin_text:
            prompt = f"{prompt}\n{stdin_text}".strip()

        if args.show_config:
            _print_effective_config(
                provider=provider,
                model=model,
                base_url=base_url,
                mode=mode,
                persona=persona,
                temperature=args.temperature,
                max_tokens=args.max_tokens,
            )

        should_start_repl = args.repl or (
            not prompt and not stdin_text and sys.stdin.isatty() and sys.stdout.isatty()
        )
        if should_start_repl:
            return _run_repl(
                provider=provider,
                model=model,
                api_key=api_key,
                base_url=base_url,
                timeout_seconds=args.timeout_seconds,
                device=args.device,
                mode=mode,
                persona=persona,
                temperature=args.temperature,
                max_tokens=args.max_tokens,
            )

        if not prompt:
            raise ValueError("prompt is required (argument or --stdin input), or use --repl")

        config = ProviderConfig(
            provider=provider,
            model=model,
            api_key=api_key,
            base_url=base_url,
            timeout_seconds=args.timeout_seconds,
            device=args.device,
        )

        provider_client = build_provider(config)
        bot = ChatBot(provider=provider_client)
        resp = bot.run(
            user_input=prompt,
            mode=mode,
            persona=persona,
            temperature=args.temperature,
            max_tokens=args.max_tokens,
            diff_text=_read_diff_file(args.diff_file),
        )
        print(resp.content)
        return 0
    except KeyboardInterrupt:
        print("\nInterrupted.", file=sys.stderr)
        return 130
    except Exception as exc:
        if args.traceback:
            raise
        print(f"Error: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
