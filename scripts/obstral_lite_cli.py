#!/usr/bin/env python3
"""CLI front-end for scripts/serve_lite.py chat runtime."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any
from urllib import error as urlerror
from urllib import request as urlrequest

try:
    from . import serve_lite  # type: ignore
except Exception:
    import serve_lite  # type: ignore


PROVIDERS = ["openai-compatible", "mistral", "codestral", "anthropic", "mistral-cli", "hf"]
CLI_DEFAULT_PROVIDER = "openai-compatible"
CLI_DEFAULT_MODEL = "gpt-4o-mini"
CLI_DEFAULT_BASE_URL = "https://api.openai.com/v1"


def _read_optional(path: str | None) -> str | None:
    if not path:
        return None
    p = Path(path)
    if not p.exists():
        raise RuntimeError(f"file not found: {p}")
    return p.read_text(encoding="utf-8", errors="replace")


def _server_join(base: str, path: str) -> str:
    b = str(base or "").strip().rstrip("/")
    p = str(path or "").strip()
    if not p.startswith("/"):
        p = "/" + p
    return b + p


def _http_json(method: str, url: str, body: dict[str, Any] | None = None, timeout: int = 30) -> Any:
    data = None
    headers: dict[str, str] = {"Accept": "application/json"}
    if body is not None:
        data = json.dumps(body).encode("utf-8")
        headers["Content-Type"] = "application/json"
    req = urlrequest.Request(url=url, data=data, headers=headers, method=method)
    try:
        with urlrequest.urlopen(req, timeout=timeout) as resp:
            raw = resp.read()
    except urlerror.HTTPError as e:
        raw = e.read()
        msg = raw.decode("utf-8", errors="replace")
        raise RuntimeError(f"HTTP {e.code}\n{msg}") from None
    except urlerror.URLError as e:
        raise RuntimeError(f"request failed: {e.reason}") from None

    if not raw:
        return {}
    try:
        return json.loads(raw.decode("utf-8"))
    except json.JSONDecodeError:
        return raw.decode("utf-8", errors="replace")


def _coerce_provider_defaults(
    provider: str, base_url: str | None, model: str | None
) -> tuple[str, str]:
    p = str(provider or "").strip() or CLI_DEFAULT_PROVIDER
    b = str(base_url or "").strip()
    m = str(model or "").strip()

    # If user switched provider but left CLI defaults, apply provider defaults.
    if not b or (p != CLI_DEFAULT_PROVIDER and b == CLI_DEFAULT_BASE_URL):
        b = str(serve_lite.DEFAULT_BASE_URL.get(p, CLI_DEFAULT_BASE_URL))
    if not m or (p != CLI_DEFAULT_PROVIDER and m == CLI_DEFAULT_MODEL):
        m = str(serve_lite.DEFAULT_MODEL.get(p, CLI_DEFAULT_MODEL))
    return b, m


def _provider_needs_api_key(provider: str) -> bool:
    return provider in ("openai-compatible", "mistral", "codestral", "anthropic")


def _effective_api_key(provider: str, inline_key: str | None) -> tuple[str, str]:
    inline = str(inline_key or "").strip()
    if inline:
        return inline, "inline"
    env_key = str(serve_lite._provider_key_from_env(provider) or "").strip()
    if env_key:
        return env_key, "env"
    return "", "none"


def _preflight(req: dict[str, Any]) -> None:
    provider = str(req.get("provider") or "").strip()
    base_url = str(req.get("base_url") or "").strip()
    api_key = str(req.get("api_key") or "").strip()
    if hasattr(serve_lite, "_preflight_provider_config"):
        serve_lite._preflight_provider_config(provider, base_url, api_key)


def _run_doctor_from_req(req: dict[str, Any], check_models: bool = False) -> int:
    provider = str(req.get("provider") or "")
    base_url = str(req.get("base_url") or "")
    model = str(req.get("model") or "")
    tool_root = str(req.get("tool_root") or "")
    api_key, key_source = _effective_api_key(provider, str(req.get("api_key") or ""))
    needs_key = _provider_needs_api_key(provider)

    print("OBSTRAL Doctor")
    print(f"  provider:   {provider}")
    print(f"  mode:       {req.get('mode')}")
    print(f"  base_url:   {base_url or '(empty)'}")
    print(f"  model:      {model or '(empty)'}")
    if tool_root.strip():
        print(f"  tool_root:  {tool_root.strip()}")
    print(f"  api_key:    {'present' if bool(api_key) else 'missing'} ({key_source})")
    print(f"  cot:        {req.get('cot')}")
    print(f"  autonomy:   {req.get('autonomy')}")
    print(f"  approval:   {req.get('require_edit_approval')}")
    print(f"  cmd apprv:  {req.get('require_command_approval')}")

    warnings: list[str] = []
    try:
        _preflight({**req, "api_key": api_key})
    except Exception as e:
        warnings.append(str(e))

    if needs_key and not api_key:
        if provider in ("mistral", "codestral"):
            warnings.append("API key missing. Set MISTRAL_API_KEY or OBS_API_KEY.")
        elif provider == "openai-compatible":
            warnings.append("API key missing. Set OPENAI_API_KEY or OBS_API_KEY.")
        elif provider == "anthropic":
            warnings.append("API key missing. Set ANTHROPIC_API_KEY.")

    if warnings:
        print("")
        print("Warnings:")
        for w in warnings:
            print(f"  - {w}")
    else:
        print("")
        print("Config check: OK")

    if check_models:
        print("")
        print("Model probe:")
        try:
            out = serve_lite._models_impl(
                {
                    "provider": provider,
                    "base_url": base_url,
                    "api_key": api_key,
                }
            )
            models = out.get("models") if isinstance(out, dict) else None
            count = len(models) if isinstance(models, list) else 0
            print(f"  success: {count} models")
            if isinstance(models, list) and models:
                for m in models[:8]:
                    print(f"  - {m}")
        except Exception as e:
            print(f"  failed: {e}")

    return 0


def _common_req(args: argparse.Namespace) -> dict[str, Any]:
    provider = str(args.provider or "").strip() or CLI_DEFAULT_PROVIDER
    base_url, model = _coerce_provider_defaults(provider, args.base_url, args.model)
    if provider == "mistral-cli":
        base_url = ""

    req: dict[str, Any] = {
        "provider": provider,
        "model": model,
        "chat_model": args.chat_model or model,
        "code_model": args.code_model or model,
        "base_url": base_url,
        "api_key": args.api_key,
        "tool_root": args.tool_root,
        "mode": args.mode,
        "persona": args.persona,
        "temperature": args.temperature,
        "max_tokens": args.max_tokens,
        "timeout_seconds": args.timeout_seconds,
        "cot": args.cot,
        "autonomy": args.autonomy,
        "require_edit_approval": args.require_edit_approval,
        "require_command_approval": args.require_command_approval,
    }
    if args.mistral_cli_agent:
        req["mistral_cli_agent"] = args.mistral_cli_agent
    if args.mistral_cli_max_turns:
        req["mistral_cli_max_turns"] = args.mistral_cli_max_turns
    return req


def _run_chat(args: argparse.Namespace) -> int:
    prompt = str(args.prompt or "").strip()
    if args.stdin:
        stdin_text = sys.stdin.read()
        if stdin_text.strip():
            prompt = (prompt + "\n\n[stdin]\n" + stdin_text).strip()
    if not prompt:
        raise RuntimeError("prompt is required")

    diff_text = _read_optional(args.diff_file)
    req = _common_req(args)
    req["input"] = prompt
    req["history"] = []
    req["diff"] = diff_text

    # Fail fast for obvious provider/base_url/api-key mismatches.
    _preflight({**req, "api_key": _effective_api_key(req["provider"], req.get("api_key"))[0]})

    if str(args.server or "").strip():
        url = _server_join(str(args.server), "/api/chat")
        out = _http_json("POST", url, req, timeout=int(req.get("timeout_seconds") or 120) + 10)
        if isinstance(out, dict) and out.get("error"):
            raise RuntimeError(str(out.get("error")))
        if isinstance(out, dict):
            print(str(out.get("content") or ""))
        else:
            print(str(out))
        return 0

    out2 = serve_lite._chat_impl(req)
    print(str(out2.get("content") or ""))
    return 0


def _run_repl(args: argparse.Namespace) -> int:
    history: list[dict[str, str]] = []
    diff_text = _read_optional(args.diff_file)
    base_req = _common_req(args)
    server = str(args.server or "").strip()
    use_server = bool(server)
    print("OBSTRAL Lite CLI REPL")
    print("  /help  show commands")
    print("  /exit  quit")
    print(
        "  provider={}  model={}  mode={}".format(
            base_req.get("provider"), base_req.get("model"), base_req.get("mode")
        )
    )
    if use_server:
        print(f"  server={server}")
    try:
        _preflight(
            {
                **base_req,
                "api_key": _effective_api_key(
                    str(base_req.get("provider")), str(base_req.get("api_key") or "")
                )[0],
            }
        )
    except Exception as e:
        print(f"  preflight warning: {e}")
    print("")

    while True:
        try:
            line = input("obstral-lite> ").strip()
        except EOFError:
            print("")
            break
        except KeyboardInterrupt:
            print("")
            continue

        if not line:
            continue
        if line in ("/exit", "/quit"):
            break
        if line == "/help":
            print("Commands:")
            print("  /help")
            print("  /exit | /quit")
            print("  /reset")
            print("  /config")
            print("  /doctor")
            continue
        if line == "/reset":
            history = []
            print("(history cleared)")
            continue
        if line == "/config":
            cfg = _common_req(args)
            safe = {
                "provider": cfg.get("provider"),
                "model": cfg.get("model"),
                "chat_model": cfg.get("chat_model"),
                "code_model": cfg.get("code_model"),
                "base_url": cfg.get("base_url"),
                "tool_root": cfg.get("tool_root"),
                "mode": cfg.get("mode"),
                "persona": cfg.get("persona"),
                "temperature": cfg.get("temperature"),
                "max_tokens": cfg.get("max_tokens"),
                "timeout_seconds": cfg.get("timeout_seconds"),
                "cot": cfg.get("cot"),
                "autonomy": cfg.get("autonomy"),
                "require_edit_approval": cfg.get("require_edit_approval"),
                "require_command_approval": cfg.get("require_command_approval"),
            }
            print(json.dumps(safe, ensure_ascii=False, indent=2))
            continue
        if line == "/doctor":
            _run_doctor_from_req(_common_req(args), check_models=False)
            continue

        req = _common_req(args)
        req["input"] = line
        req["history"] = history
        req["diff"] = diff_text

        try:
            if use_server:
                url = _server_join(server, "/api/chat")
                out = _http_json(
                    "POST",
                    url,
                    req,
                    timeout=int(req.get("timeout_seconds") or 120) + 10,
                )
                if isinstance(out, dict) and out.get("error"):
                    raise RuntimeError(str(out.get("error")))
            else:
                out = serve_lite._chat_impl(req)
        except Exception as e:
            print(f"Error: {e}", file=sys.stderr)
            continue

        if isinstance(out, dict):
            content = str(out.get("content") or "")
        else:
            content = str(out or "")
        print("")
        print(content)
        print("")
        history.append({"role": "user", "content": line})
        history.append({"role": "assistant", "content": content})

    return 0


def _run_doctor(args: argparse.Namespace) -> int:
    req = _common_req(args)
    api_key, _ = _effective_api_key(req["provider"], req.get("api_key"))
    return _run_doctor_from_req({**req, "api_key": api_key}, check_models=bool(args.check_models))


def _run_serve(args: argparse.Namespace) -> int:
    if getattr(args, "workspace", ""):
        serve_lite._set_workspace_root(str(args.workspace))
    host = str(args.host or "127.0.0.1").strip() or "127.0.0.1"
    port = int(args.port or 18080)
    server = serve_lite.ThreadingHTTPServer((host, port), serve_lite.LiteHandler)
    print(f"OBSTRAL Lite UI: http://{host}:{port}/")
    print(f"Workspace root: {serve_lite.WORKSPACE_ROOT.as_posix()}")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
    return 0


def _run_pending(args: argparse.Namespace) -> int:
    server = str(args.server or "http://127.0.0.1:18080").strip() or "http://127.0.0.1:18080"
    url = _server_join(server, "/api/pending_edits")
    out = _http_json("GET", url, None, timeout=10)
    if not isinstance(out, dict):
        raise RuntimeError("invalid pending response")
    items = out.get("pending")
    items = items if isinstance(items, list) else []
    if args.json:
        print(json.dumps(out, ensure_ascii=False, indent=2))
        return 0

    if not items:
        print("(no pending edits)")
        return 0

    for it in items:
        if not isinstance(it, dict):
            continue
        eid = str(it.get("id") or "")
        status = str(it.get("status") or "")
        action = str(it.get("action") or "")
        path = str(it.get("path") or "")
        preview = str(it.get("preview") or "").strip()
        if preview:
            preview = preview.replace("\r\n", "\n").replace("\r", "\n")
            preview = preview.split("\n", 1)[0]
            if len(preview) > 120:
                preview = preview[:120] + "..."
        line = f"{status}\t{eid}\t{action}\t{path}".rstrip()
        if preview:
            line += f"\tpreview={preview}"
        print(line)
    return 0


def _run_resolve_pending(args: argparse.Namespace, approve: bool) -> int:
    server = str(args.server or "http://127.0.0.1:18080").strip() or "http://127.0.0.1:18080"

    def resolve_one(edit_id: str) -> Any:
        path = "/api/approve_edit" if approve else "/api/reject_edit"
        url = _server_join(server, path)
        return _http_json("POST", url, {"id": edit_id}, timeout=30)

    if args.all:
        pending = _http_json("GET", _server_join(server, "/api/pending_edits"), None, timeout=10)
        items = pending.get("pending") if isinstance(pending, dict) else []
        items = items if isinstance(items, list) else []
        targets = [
            str(it.get("id") or "")
            for it in items
            if isinstance(it, dict) and str(it.get("status") or "") == "pending"
        ]
        if not targets:
            print("(no pending edits)")
            return 0
        for eid in targets:
            res = resolve_one(eid)
            if args.json:
                print(json.dumps(res, ensure_ascii=False))
            else:
                print(f"{'approved' if approve else 'rejected'}\t{eid}")
        return 0

    edit_id = str(args.id or "").strip()
    if not edit_id:
        raise RuntimeError("id is required (or use --all)")
    res2 = resolve_one(edit_id)
    if args.json:
        print(json.dumps(res2, ensure_ascii=False, indent=2))
    else:
        print(f"{'approved' if approve else 'rejected'}\t{edit_id}")
    return 0


def _add_common_flags(p: argparse.ArgumentParser) -> None:
    p.add_argument("--provider", choices=PROVIDERS, default=CLI_DEFAULT_PROVIDER)
    p.add_argument("--model", default=CLI_DEFAULT_MODEL)
    p.add_argument("--chat-model", default=None)
    p.add_argument("--code-model", default=None)
    p.add_argument("--base-url", default=CLI_DEFAULT_BASE_URL)
    p.add_argument("--api-key", default="")
    p.add_argument("--mode", default="VIBE")
    p.add_argument("--persona", default="default")
    p.add_argument("--temperature", type=float, default=0.4)
    p.add_argument("--max-tokens", type=int, default=1024)
    p.add_argument("--timeout-seconds", type=int, default=120)
    p.add_argument("--tool-root", default="", help="Restrict local tools to a subdir under workspace root")
    p.add_argument(
        "--server",
        default="",
        help="If set, call a running OBSTRAL Lite server (example: http://127.0.0.1:18080)",
    )
    p.add_argument("--diff-file", default=None)
    p.add_argument("--cot", choices=["off", "brief", "structured"], default="brief")
    p.add_argument("--autonomy", choices=["off", "longrun"], default="longrun")
    p.add_argument(
        "--require-edit-approval",
        action=argparse.BooleanOptionalAction,
        default=True,
    )
    p.add_argument(
        "--require-command-approval",
        action=argparse.BooleanOptionalAction,
        default=True,
    )
    p.add_argument("--mistral-cli-agent", default=None)
    p.add_argument("--mistral-cli-max-turns", type=int, default=None)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="OBSTRAL lite CLI")
    sub = parser.add_subparsers(dest="cmd", required=True)

    chat = sub.add_parser("chat", help="Run one-shot chat")
    _add_common_flags(chat)
    chat.add_argument("prompt")
    chat.add_argument("--stdin", action="store_true", help="Append stdin to prompt")

    repl = sub.add_parser("repl", help="Run interactive REPL")
    _add_common_flags(repl)

    sub.add_parser("list-providers", help="List providers")
    sub.add_parser("list-presets", help="Show provider default base_url/model")

    doctor = sub.add_parser("doctor", help="Validate CLI/provider configuration")
    _add_common_flags(doctor)
    doctor.add_argument("--check-models", action="store_true", help="Call provider /models endpoint")

    serve = sub.add_parser("serve", help="Run local OBSTRAL Lite web UI server")
    serve.add_argument("--host", default="127.0.0.1")
    serve.add_argument("--port", type=int, default=18080)
    serve.add_argument("--workspace", default="", help="Override workspace root for local tools")

    pending = sub.add_parser("pending", help="List pending edits from a running server")
    pending.add_argument("--server", default="http://127.0.0.1:18080")
    pending.add_argument("--json", action="store_true")

    approve = sub.add_parser("approve", help="Approve a pending edit")
    approve.add_argument("id", nargs="?", default="")
    approve.add_argument("--all", action="store_true", help="Approve all pending edits")
    approve.add_argument("--server", default="http://127.0.0.1:18080")
    approve.add_argument("--json", action="store_true")

    reject = sub.add_parser("reject", help="Reject a pending edit")
    reject.add_argument("id", nargs="?", default="")
    reject.add_argument("--all", action="store_true", help="Reject all pending edits")
    reject.add_argument("--server", default="http://127.0.0.1:18080")
    reject.add_argument("--json", action="store_true")

    args = parser.parse_args(argv)

    if args.cmd == "list-providers":
        for p in PROVIDERS:
            print(p)
        return 0
    if args.cmd == "list-presets":
        for p in PROVIDERS:
            base = str(serve_lite.DEFAULT_BASE_URL.get(p, ""))
            model = str(serve_lite.DEFAULT_MODEL.get(p, ""))
            print(f"{p}\tbase_url={base or '(none)'}\tmodel={model or '(none)'}")
        return 0
    if args.cmd == "chat":
        return _run_chat(args)
    if args.cmd == "repl":
        return _run_repl(args)
    if args.cmd == "doctor":
        return _run_doctor(args)
    if args.cmd == "serve":
        return _run_serve(args)
    if args.cmd == "pending":
        return _run_pending(args)
    if args.cmd == "approve":
        return _run_resolve_pending(args, approve=True)
    if args.cmd == "reject":
        return _run_resolve_pending(args, approve=False)
    raise RuntimeError(f"unknown command: {args.cmd}")


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        raise SystemExit(2)
