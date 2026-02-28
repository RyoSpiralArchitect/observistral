#!/usr/bin/env python3
"""Minimal OBSTRAL-compatible server that avoids running a custom EXE."""

from __future__ import annotations

import argparse
import json
import os
import shlex
import subprocess
import sys
import threading
import time
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any
from urllib import error as urlerror
from urllib import request as urlrequest


REPO_ROOT = Path(__file__).resolve().parents[1]
WEB_ROOT = REPO_ROOT / "web"
WORKSPACE_ROOT = REPO_ROOT.resolve()
MAX_BODY_BYTES = 2 * 1024 * 1024
ANTHROPIC_VERSION = "2023-06-01"
DIRECT_OPENER = urlrequest.build_opener(urlrequest.ProxyHandler({}))
LOCAL_TOOL_MAX_STEPS = 10
PENDING_LOCK = threading.Lock()
PENDING_EDITS: dict[str, dict[str, Any]] = {}

LOCAL_TOOL_PROMPT = (
    "You are a coding agent with local workspace tools.\n"
    "Workspace root: {workspace}\n\n"
    "When the user asks to create/edit/read files, use tool calls."
    " Do not claim lack of permission. Use tools first, then summarize results.\n"
    "If the user asks to create a repository/project/app/game, you must actually create "
    "folders/files (mkdir/write_file) and then report what was created."
)

LONGRUN_AUTONOMY_PROMPT = (
    "Execution policy: long-run autonomous coding.\n"
    "- Decompose work into modules and execute step-by-step.\n"
    "- Continue working without unnecessary confirmation until done.\n"
    "- Reuse tools repeatedly for inspect/edit/verify loops.\n"
    "- If blocked, clearly state blocker and best next action.\n"
)

CODER_IDENTITY_PROMPT = (
    "Identity: You are the Coder.\n"
    "- You are responsible for producing working code changes, not just advice.\n"
    "- Prefer concrete edits, commands, and verification over abstract discussion.\n"
    "- Keep explanations concise and implementation-focused.\n"
    "- For implementation requests, execute filesystem changes instead of only giving steps.\n"
)

OBSERVER_NOVELIST_PROMPT = (
    "Observer persona style (novelist):\n"
    "- Voice: cynical, literary, slightly dark humor.\n"
    "- Keep technical facts precise, but narrate them with metaphor and irony.\n"
    "- Do not be blandly polite; be sharp, memorable, and theatrical.\n"
    "- Always anchor tone to real evidence from coder_context.\n"
)


def _observer_persona_prompt(persona: str) -> str:
    p = str(persona or "").strip().lower()
    if p == "novelist":
        return OBSERVER_NOVELIST_PROMPT
    if p == "cynical":
        return (
            "Observer persona style (cynical): direct, critical, unsentimental. "
            "Prioritize risks and contradictions."
        )
    if p == "cheerful":
        return (
            "Observer persona style (cheerful): energetic but still evidence-driven. "
            "Do not hide risks."
        )
    if p == "thoughtful":
        return (
            "Observer persona style (thoughtful): calm, analytical, and reflective. "
            "Emphasize trade-offs."
        )
    return ""

LOCAL_TOOLS = [
    {
        "type": "function",
        "function": {
            "name": "list_files",
            "description": "List files/directories under the workspace root using a glob pattern.",
            "parameters": {
                "type": "object",
                "properties": {
                    "pattern": {"type": "string", "description": "Glob pattern (default: **/*)"},
                    "max_results": {"type": "integer", "minimum": 1, "maximum": 1000},
                },
                "additionalProperties": False,
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "read_file",
            "description": "Read UTF-8 text from a file path under workspace root.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "max_bytes": {"type": "integer", "minimum": 1, "maximum": 300000},
                },
                "required": ["path"],
                "additionalProperties": False,
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "write_file",
            "description": "Write UTF-8 text to a file path under workspace root. Creates parent dirs by default.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "content": {"type": "string"},
                    "overwrite": {"type": "boolean"},
                    "ensure_parent": {"type": "boolean"},
                },
                "required": ["path", "content"],
                "additionalProperties": False,
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "mkdir",
            "description": "Create a directory path under workspace root.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "parents": {"type": "boolean"},
                },
                "required": ["path"],
                "additionalProperties": False,
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "run_command",
            "description": "Run a terminal command in workspace root and return stdout/stderr.",
            "parameters": {
                "type": "object",
                "properties": {
                    "command": {"type": "string"},
                    "timeout_seconds": {"type": "integer", "minimum": 1, "maximum": 600},
                },
                "required": ["command"],
                "additionalProperties": False,
            },
        },
    },
]

DEFAULT_BASE_URL = {
    "mistral": "https://api.mistral.ai/v1",
    "codestral": "https://codestral.mistral.ai/v1",
    "openai-compatible": "https://api.openai.com/v1",
    "anthropic": "https://api.anthropic.com/v1",
    "mistral-cli": "",
}

DEFAULT_MODEL = {
    "mistral": "mistral-small-latest",
    "codestral": "codestral-latest",
    "openai-compatible": "gpt-4o-mini",
    "anthropic": "claude-3-5-sonnet-latest",
    "mistral-cli": "mistral-medium-latest",
}


def _env_present(name: str) -> bool:
    v = os.environ.get(name, "").strip()
    return bool(v)


def _provider_key_from_env(provider: str) -> str:
    if provider in ("mistral", "codestral"):
        return os.environ.get("MISTRAL_API_KEY", "").strip() or os.environ.get(
            "OBS_API_KEY", ""
        ).strip()
    if provider == "openai-compatible":
        return os.environ.get("OBS_API_KEY", "").strip() or os.environ.get(
            "OPENAI_API_KEY", ""
        ).strip()
    if provider == "anthropic":
        return os.environ.get("ANTHROPIC_API_KEY", "").strip()
    return ""


def _provider_from_req(req: dict[str, Any]) -> str:
    provider = str(req.get("provider") or "mistral").strip()
    if provider in (
        "mistral",
        "codestral",
        "openai-compatible",
        "anthropic",
        "hf",
        "mistral-cli",
    ):
        return provider
    return "mistral"


def _cot_level(req: dict[str, Any]) -> str:
    v = req.get("cot")
    if v is None:
        return "brief"
    if isinstance(v, bool):
        return "brief" if v else "off"
    s = str(v).strip().lower()
    if s in ("off", "false", "0", "none", "disable", "disabled"):
        return "off"
    if s in ("brief", "short", "on", "true", "1", "simple"):
        return "brief"
    return "brief"


def _autonomy_level(req: dict[str, Any]) -> str:
    v = req.get("autonomy")
    if v is None:
        return "longrun"
    s = str(v).strip().lower()
    if s in ("off", "none", "false", "0"):
        return "off"
    return "longrun"


def _as_bool(v: Any, default: bool) -> bool:
    if v is None:
        return default
    if isinstance(v, bool):
        return v
    if isinstance(v, (int, float)):
        return bool(v)
    s = str(v).strip().lower()
    if s in ("1", "true", "yes", "on"):
        return True
    if s in ("0", "false", "no", "off"):
        return False
    return default


def _force_tool_use(req: dict[str, Any]) -> bool:
    mode = str(req.get("mode") or "").strip().lower()
    default = mode != "observer"
    return _as_bool(req.get("force_tools"), default)


def _requires_edit_approval(req: dict[str, Any] | None) -> bool:
    if req is None:
        return False
    env_default = _as_bool(os.environ.get("OBS_REQUIRE_EDIT_APPROVAL"), True)
    return _as_bool(req.get("require_edit_approval"), env_default)


def _requires_command_approval(req: dict[str, Any] | None) -> bool:
    if req is None:
        return False
    env_default = _as_bool(os.environ.get("OBS_REQUIRE_COMMAND_APPROVAL"), True)
    if "require_command_approval" in req:
        return _as_bool(req.get("require_command_approval"), env_default)
    return _requires_edit_approval(req)


def _pending_summary(item: dict[str, Any]) -> dict[str, Any]:
    args = item.get("args") if isinstance(item.get("args"), dict) else {}
    path = str(args.get("path") or "")
    if not path and str(item.get("action") or "") == "run_command":
        path = "<command>"
    preview = str(args.get("content_preview") or args.get("command_preview") or "")
    out = {
        "id": item.get("id"),
        "action": item.get("action"),
        "status": item.get("status"),
        "path": path,
        "created_at": item.get("created_at"),
        "updated_at": item.get("updated_at"),
        "preview": preview,
        "error": item.get("error"),
    }
    if item.get("result") is not None:
        out["result"] = item.get("result")
    return out


def _queue_pending_edit(action: str, args: dict[str, Any]) -> dict[str, Any]:
    now = int(time.time())
    eid = "edit_" + uuid.uuid4().hex[:10]
    item = {
        "id": eid,
        "action": action,
        "status": "pending",
        "created_at": now,
        "updated_at": now,
        "args": args,
        "result": None,
        "error": None,
    }
    with PENDING_LOCK:
        PENDING_EDITS[eid] = item
    return item


def _list_pending_edits() -> list[dict[str, Any]]:
    with PENDING_LOCK:
        items = list(PENDING_EDITS.values())
    items.sort(key=lambda x: int(x.get("created_at") or 0), reverse=True)
    return [_pending_summary(i) for i in items]


def _approve_pending_edit(edit_id: str) -> dict[str, Any]:
    with PENDING_LOCK:
        item = PENDING_EDITS.get(edit_id)
    if item is None:
        raise RuntimeError("pending edit not found")
    if item.get("status") != "pending":
        return _pending_summary(item)

    args = item.get("args") if isinstance(item.get("args"), dict) else {}
    action = str(item.get("action") or "")
    now = int(time.time())
    try:
        if action == "write_file":
            path = _resolve_workspace_path(str(args.get("path") or ""))
            result = _apply_write_file(
                path=path,
                content=str(args.get("content") or ""),
                overwrite=bool(args.get("overwrite", True)),
                ensure_parent=bool(args.get("ensure_parent", True)),
            )
        elif action == "mkdir":
            path = _resolve_workspace_path(str(args.get("path") or ""))
            result = _apply_mkdir(path=path, parents=bool(args.get("parents", True)))
        elif action == "run_command":
            result = _apply_run_command(
                command=str(args.get("command") or ""),
                timeout_seconds=int(args.get("timeout_seconds") or 120),
            )
        else:
            raise RuntimeError(f"unknown pending action: {action}")

        with PENDING_LOCK:
            cur = PENDING_EDITS.get(edit_id)
            if cur:
                cur["status"] = "approved"
                cur["updated_at"] = now
                cur["result"] = result
                cur["error"] = None
                item = cur
    except Exception as e:
        with PENDING_LOCK:
            cur = PENDING_EDITS.get(edit_id)
            if cur:
                cur["status"] = "error"
                cur["updated_at"] = now
                cur["error"] = str(e)
                item = cur
        raise

    return _pending_summary(item)


def _reject_pending_edit(edit_id: str) -> dict[str, Any]:
    with PENDING_LOCK:
        item = PENDING_EDITS.get(edit_id)
        if item is None:
            raise RuntimeError("pending edit not found")
        if item.get("status") == "pending":
            item["status"] = "rejected"
            item["updated_at"] = int(time.time())
    return _pending_summary(item)


def _base_url(provider: str, req: dict[str, Any]) -> str:
    v = str(req.get("base_url") or "").strip()
    if v:
        return v.rstrip("/")
    return DEFAULT_BASE_URL.get(provider, "https://api.openai.com/v1")


def _pick_model(provider: str, req: dict[str, Any]) -> str:
    mode = str(req.get("mode") or "")
    use_code_model = mode == "VIBE" or mode.startswith("diff")

    candidates: list[str] = []
    if use_code_model:
        candidates.extend(
            [
                str(req.get("code_model") or "").strip(),
                str(req.get("model") or "").strip(),
                str(req.get("chat_model") or "").strip(),
            ]
        )
    else:
        candidates.extend(
            [
                str(req.get("chat_model") or "").strip(),
                str(req.get("model") or "").strip(),
                str(req.get("code_model") or "").strip(),
            ]
        )

    for model in candidates:
        if model:
            return model
    return DEFAULT_MODEL.get(provider, "gpt-4o-mini")


def _pick_api_key(provider: str, req: dict[str, Any]) -> str:
    inline = str(req.get("api_key") or "").strip()
    if inline:
        return inline
    return _provider_key_from_env(provider)


def _preflight_provider_config(provider: str, base_url: str, api_key: str) -> None:
    p = str(provider or "").strip().lower()
    u = str(base_url or "").strip().lower()
    k = str(api_key or "").strip()

    if p in ("mistral", "codestral"):
        if "api.openai.com" in u:
            raise RuntimeError(
                "Provider is mistral/codestral but Base URL is OpenAI. "
                "Use https://api.mistral.ai/v1 or https://codestral.mistral.ai/v1."
            )
        if k.startswith("sk-proj-"):
            raise RuntimeError(
                "This API key looks like an OpenAI key (sk-proj-*). "
                "Use MISTRAL_API_KEY or OBS_API_KEY for Mistral/Codestral."
            )

    if p == "openai-compatible" and "api.mistral.ai" in u:
        raise RuntimeError(
            "Provider is openai-compatible but Base URL points to Mistral. "
            "Switch provider to mistral/codestral or change Base URL."
        )


def _extract_text_from_anthropic_response(payload: dict[str, Any]) -> str:
    items = payload.get("content")
    if isinstance(items, list):
        chunks: list[str] = []
        for item in items:
            if not isinstance(item, dict):
                continue
            if item.get("type") == "text":
                chunks.append(str(item.get("text") or ""))
        return "".join(chunks)
    return ""


def _normalize_assistant_content(content: Any) -> str:
    if content is None:
        return ""
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        chunks: list[str] = []
        for item in content:
            if isinstance(item, str):
                chunks.append(item)
                continue
            if not isinstance(item, dict):
                continue
            if item.get("type") in ("output_text", "text"):
                txt = item.get("text")
                if isinstance(txt, str):
                    chunks.append(txt)
        return "".join(chunks)
    return str(content)


def _api_key_env_hint(provider: str) -> str:
    p = str(provider or "").strip().lower()
    if p == "openai-compatible":
        return "OBS_API_KEY or OPENAI_API_KEY"
    if p in ("mistral", "codestral"):
        return "MISTRAL_API_KEY or OBS_API_KEY"
    if p == "anthropic":
        return "ANTHROPIC_API_KEY"
    if p == "mistral-cli":
        return "run `vibe --setup` (Mistral CLI auth)"
    return "provider-specific API key env var"


def _model_suggestions(provider: str) -> list[str]:
    p = str(provider or "").strip().lower()
    if p == "openai-compatible":
        return ["gpt-4o-mini", "gpt-4.1-mini", "o4-mini"]
    if p == "codestral":
        return ["codestral-latest", "mistral-small-latest"]
    if p == "mistral":
        return ["mistral-small-latest", "codestral-latest", "mistral-large-latest"]
    if p == "anthropic":
        return ["claude-3-5-sonnet-latest", "claude-3-5-haiku-latest"]
    if p == "mistral-cli":
        return ["mistral-medium-latest", "mistral-small-latest"]
    return []


def _extract_provider_error_message(body: str) -> str:
    txt = str(body or "").strip()
    if not txt:
        return ""
    try:
        j = json.loads(txt)
    except Exception:
        return txt

    if isinstance(j, dict):
        err = j.get("error")
        if isinstance(err, dict):
            msg = err.get("message")
            if isinstance(msg, str) and msg.strip():
                return msg.strip()
            for k in ("type", "code"):
                v = err.get(k)
                if isinstance(v, str) and v.strip():
                    return v.strip()
        if isinstance(err, str) and err.strip():
            return err.strip()
        for k in ("message", "detail"):
            v = j.get(k)
            if isinstance(v, str) and v.strip():
                return v.strip()

    return txt


def _friendly_http_error(
    status: int, body: str, headers: Any, error_context: dict[str, Any] | None
) -> RuntimeError:
    ctx = error_context or {}
    provider = str(ctx.get("provider") or "").strip()
    model = str(ctx.get("model") or "").strip()

    msg = _extract_provider_error_message(body)
    msg_lower = msg.lower()
    lines: list[str] = []

    if status in (401, 403):
        lines.append(f"HTTP {status}: authentication failed.")
        lines.append(f"Set API key env var: {_api_key_env_hint(provider)}")
    elif status == 429:
        lines.append("HTTP 429: rate limited.")
        retry_after = ""
        if headers is not None:
            try:
                retry_after = str(headers.get("Retry-After") or "").strip()
            except Exception:
                retry_after = ""
        if retry_after:
            lines.append(f"retry_after: {retry_after}s")
        else:
            lines.append("retry_after: not provided")
    elif status == 400 and (
        "invalid model" in msg_lower
        or "invalid_model" in msg_lower
        or "model_not_found" in msg_lower
    ):
        lines.append("HTTP 400: invalid_model")
        if model:
            lines.append(f"requested_model: {model}")
        sugg = _model_suggestions(provider)
        if sugg:
            lines.append("fallback_candidates: " + ", ".join(sugg))
    else:
        lines.append(f"HTTP {status}")

    if msg:
        lines.append(msg if len(msg) <= 1200 else (msg[:1200] + "..."))

    return RuntimeError("\n".join(lines))


def _http_json(
    method: str,
    url: str,
    *,
    headers: dict[str, str] | None = None,
    body: dict[str, Any] | None = None,
    timeout_seconds: int = 120,
    error_context: dict[str, Any] | None = None,
) -> dict[str, Any]:
    req_headers = dict(headers or {})
    data: bytes | None = None
    if body is not None:
        data = json.dumps(body, ensure_ascii=False).encode("utf-8")
        req_headers.setdefault("Content-Type", "application/json")

    req = urlrequest.Request(url=url, data=data, headers=req_headers, method=method)
    try:
        # Force direct egress and ignore process/global proxy variables.
        with DIRECT_OPENER.open(req, timeout=timeout_seconds) as resp:
            raw = resp.read()
    except urlerror.HTTPError as e:
        raw = e.read() if e.fp is not None else b""
        text = raw.decode("utf-8", errors="replace")
        raise _friendly_http_error(int(e.code), text, e.headers, error_context) from None
    except urlerror.URLError as e:
        raise RuntimeError(f"request failed: {e.reason}") from None

    if not raw:
        return {}
    try:
        return json.loads(raw.decode("utf-8"))
    except json.JSONDecodeError:
        raise RuntimeError("invalid JSON response") from None


def _resolve_workspace_path(raw_path: str) -> Path:
    s = str(raw_path or "").strip()
    if not s:
        raise RuntimeError("path is required")

    p = Path(s)
    if not p.is_absolute():
        p = WORKSPACE_ROOT / p
    p = p.resolve()

    try:
        p.relative_to(WORKSPACE_ROOT)
    except ValueError:
        raise RuntimeError("path escapes workspace root") from None

    return p


def _rel_path(p: Path) -> str:
    return p.resolve().relative_to(WORKSPACE_ROOT).as_posix()


def _tool_list_files(args: dict[str, Any]) -> dict[str, Any]:
    pattern = str(args.get("pattern") or "**/*").strip() or "**/*"
    max_results = int(args.get("max_results") or 200)
    max_results = min(1000, max(1, max_results))

    items: list[dict[str, Any]] = []
    for p in WORKSPACE_ROOT.glob(pattern):
        if len(items) >= max_results:
            break
        try:
            rel = p.resolve().relative_to(WORKSPACE_ROOT).as_posix()
        except ValueError:
            continue
        if rel.startswith(".git/") or "/.git/" in rel:
            continue
        if p.is_dir():
            items.append({"path": rel, "type": "dir"})
        else:
            size = p.stat().st_size if p.exists() else 0
            items.append({"path": rel, "type": "file", "size": size})

    return {
        "workspace_root": WORKSPACE_ROOT.as_posix(),
        "pattern": pattern,
        "items": items,
        "truncated": len(items) >= max_results,
    }


def _tool_read_file(args: dict[str, Any]) -> dict[str, Any]:
    path = _resolve_workspace_path(str(args.get("path") or ""))
    if not path.exists():
        raise RuntimeError("file does not exist")
    if not path.is_file():
        raise RuntimeError("path is not a file")

    max_bytes = int(args.get("max_bytes") or 120000)
    max_bytes = min(300000, max(1, max_bytes))

    raw = path.read_bytes()
    data = raw[:max_bytes].decode("utf-8", errors="replace")
    return {
        "path": _rel_path(path),
        "bytes": len(raw),
        "truncated": len(raw) > max_bytes,
        "content": data,
    }


def _apply_write_file(
    path: Path, content: str, overwrite: bool, ensure_parent: bool
) -> dict[str, Any]:
    if path.exists() and path.is_dir():
        raise RuntimeError("path is a directory")
    if path.exists() and not overwrite:
        raise RuntimeError("file exists and overwrite=false")
    if ensure_parent:
        path.parent.mkdir(parents=True, exist_ok=True)

    existed = path.exists()
    written = path.write_bytes(content.encode("utf-8"))
    return {
        "path": _rel_path(path),
        "bytes_written": written,
        "created": not existed,
        "workspace_root": WORKSPACE_ROOT.as_posix(),
    }


def _apply_mkdir(path: Path, parents: bool) -> dict[str, Any]:
    path.mkdir(parents=parents, exist_ok=True)
    return {"path": _rel_path(path), "workspace_root": WORKSPACE_ROOT.as_posix()}


def _apply_run_command(command: str, timeout_seconds: int) -> dict[str, Any]:
    cmd = str(command or "").strip()
    if not cmd:
        raise RuntimeError("command is required")

    timeout = min(600, max(1, int(timeout_seconds or 120)))
    try:
        cp = subprocess.run(
            cmd,
            cwd=str(WORKSPACE_ROOT),
            shell=True,
            capture_output=True,
            text=True,
            timeout=timeout,
            encoding="utf-8",
            errors="replace",
        )
    except subprocess.TimeoutExpired:
        raise RuntimeError(f"command timed out after {timeout}s") from None

    stdout = str(cp.stdout or "")
    stderr = str(cp.stderr or "")
    if len(stdout) > 120000:
        stdout = stdout[:120000] + "\n...truncated..."
    if len(stderr) > 120000:
        stderr = stderr[:120000] + "\n...truncated..."

    return {
        "command": cmd,
        "cwd": WORKSPACE_ROOT.as_posix(),
        "exit_code": int(cp.returncode),
        "ok": cp.returncode == 0,
        "stdout": stdout,
        "stderr": stderr,
    }


def _tool_write_file(args: dict[str, Any], req: dict[str, Any] | None = None) -> dict[str, Any]:
    path = _resolve_workspace_path(str(args.get("path") or ""))
    content = str(args.get("content") or "")
    overwrite = bool(args.get("overwrite", True))
    ensure_parent = bool(args.get("ensure_parent", True))

    if _requires_edit_approval(req):
        pending = _queue_pending_edit(
            "write_file",
            {
                "path": _rel_path(path),
                "content": content,
                "content_preview": content[:2000],
                "overwrite": overwrite,
                "ensure_parent": ensure_parent,
            },
        )
        return {
            "needs_approval": True,
            "approval_id": pending["id"],
            "action": "write_file",
            "path": _rel_path(path),
            "message": "Awaiting approval via /api/approve_edit",
        }

    return _apply_write_file(path, content, overwrite, ensure_parent)


def _tool_mkdir(args: dict[str, Any], req: dict[str, Any] | None = None) -> dict[str, Any]:
    path = _resolve_workspace_path(str(args.get("path") or ""))
    parents = bool(args.get("parents", True))
    if _requires_edit_approval(req):
        pending = _queue_pending_edit(
            "mkdir",
            {
                "path": _rel_path(path),
                "parents": parents,
            },
        )
        return {
            "needs_approval": True,
            "approval_id": pending["id"],
            "action": "mkdir",
            "path": _rel_path(path),
            "message": "Awaiting approval via /api/approve_edit",
        }
    return _apply_mkdir(path, parents)


def _tool_run_command(args: dict[str, Any], req: dict[str, Any] | None = None) -> dict[str, Any]:
    command = str(args.get("command") or "").strip()
    if not command:
        raise RuntimeError("command is required")
    timeout_seconds = int(args.get("timeout_seconds") or 120)
    if _requires_command_approval(req):
        pending = _queue_pending_edit(
            "run_command",
            {
                "command": command,
                "command_preview": command[:2000],
                "timeout_seconds": timeout_seconds,
            },
        )
        return {
            "needs_approval": True,
            "approval_id": pending["id"],
            "action": "run_command",
            "path": "",
            "message": "Awaiting approval via /api/approve_edit",
        }
    return _apply_run_command(command=command, timeout_seconds=timeout_seconds)


def _tool_dispatch(
    name: str, args_json: Any, req: dict[str, Any] | None = None
) -> dict[str, Any]:
    args: dict[str, Any] = {}
    if isinstance(args_json, str) and args_json.strip():
        try:
            parsed = json.loads(args_json)
        except json.JSONDecodeError:
            raise RuntimeError("tool arguments are not valid JSON") from None
        if not isinstance(parsed, dict):
            raise RuntimeError("tool arguments must be a JSON object")
        args = parsed
    elif isinstance(args_json, dict):
        args = args_json

    if name == "list_files":
        return _tool_list_files(args)
    if name == "read_file":
        return _tool_read_file(args)
    if name == "write_file":
        return _tool_write_file(args, req=req)
    if name == "mkdir":
        return _tool_mkdir(args, req=req)
    if name == "run_command":
        return _tool_run_command(args, req=req)
    raise RuntimeError(f"unknown tool: {name}")


def _local_tools_enabled(req: dict[str, Any]) -> bool:
    flag = str(os.environ.get("OBS_ENABLE_LOCAL_TOOLS", "1")).strip().lower()
    if flag in ("0", "false", "off", "no"):
        return False

    if "local_tools" in req and isinstance(req["local_tools"], bool):
        return bool(req["local_tools"])

    mode = str(req.get("mode") or "").strip().lower()
    if mode == "observer":
        return False
    return True


def _chat_paths_for_provider(provider: str, base_url: str) -> list[str]:
    p = str(provider or "").strip().lower()
    base = str(base_url or "").strip().lower()
    if p == "codestral" or "codestral.mistral.ai" in base:
        return ["/chat/completion", "/chat/completions"]
    return ["/chat/completions"]


def _chat_openai_compat(
    *,
    req: dict[str, Any],
    provider: str,
    base_url: str,
    model: str,
    api_key: str,
    timeout: int,
    temperature: Any,
    max_tokens: Any,
    messages: list[dict[str, Any]],
    tools_enabled: bool,
    max_tool_steps: int,
) -> dict[str, Any]:
    headers = {"Accept": "application/json"}
    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"

    msg_list = [dict(m) for m in messages]
    if tools_enabled:
        msg_list.insert(
            0,
            {
                "role": "system",
                "content": LOCAL_TOOL_PROMPT.format(workspace=WORKSPACE_ROOT.as_posix()),
            },
        )

    force_tools = tools_enabled and _force_tool_use(req)
    made_any_tool_call = False

    for _ in range(max_tool_steps):
        payload: dict[str, Any] = {
            "model": model,
            "messages": msg_list,
            "stream": False,
        }
        if temperature is not None:
            payload["temperature"] = temperature
        if max_tokens is not None:
            payload["max_tokens"] = max_tokens
        if tools_enabled:
            payload["tools"] = LOCAL_TOOLS
            payload["tool_choice"] = "required" if (force_tools and not made_any_tool_call) else "auto"

        paths = _chat_paths_for_provider(provider, base_url)
        res = None
        last_err: RuntimeError | None = None
        for idx, path in enumerate(paths):
            try:
                res = _http_json(
                    "POST",
                    f"{base_url}{path}",
                    headers=headers,
                    body=payload,
                    timeout_seconds=timeout,
                    error_context={"provider": provider, "model": model},
                )
                last_err = None
                break
            except RuntimeError as e:
                last_err = e
                # Endpoint variants differ across providers.
                if ("HTTP 404" in str(e) or "HTTP 405" in str(e)) and idx < (len(paths) - 1):
                    continue
                raise
        if res is None:
            if last_err is not None:
                raise last_err
            raise RuntimeError("chat request failed")

        choice = (res.get("choices") or [{}])[0]
        message = choice.get("message") if isinstance(choice, dict) else {}
        message = message if isinstance(message, dict) else {}
        content = _normalize_assistant_content(message.get("content"))
        tool_calls = message.get("tool_calls")

        if not tools_enabled or not isinstance(tool_calls, list) or not tool_calls:
            return {"content": str(content), "model": model}
        made_any_tool_call = True

        msg_list.append(
            {
                "role": "assistant",
                "content": content or "",
                "tool_calls": tool_calls,
            }
        )

        for tc in tool_calls:
            if not isinstance(tc, dict):
                continue
            tcid = str(tc.get("id") or "")
            fn = tc.get("function") if isinstance(tc.get("function"), dict) else {}
            name = str(fn.get("name") or "")
            args_json = fn.get("arguments")
            try:
                result = {"ok": True, "result": _tool_dispatch(name, args_json, req=req)}
            except Exception as e:
                result = {"ok": False, "error": str(e)}
            out = json.dumps(result, ensure_ascii=False)
            if len(out) > 150000:
                out = out[:150000] + "\n...truncated..."
            msg_list.append(
                {
                    "role": "tool",
                    "tool_call_id": tcid,
                    "content": out,
                }
            )

    raise RuntimeError("tool loop exceeded max steps")


def _build_messages(req: dict[str, Any]) -> list[dict[str, str]]:
    history = req.get("history") if isinstance(req.get("history"), list) else []
    messages: list[dict[str, str]] = []
    for item in history:
        if not isinstance(item, dict):
            continue
        role = str(item.get("role") or "").strip()
        if role not in ("user", "assistant"):
            continue
        messages.append({"role": role, "content": str(item.get("content") or "")})

    mode = str(req.get("mode") or "").strip()
    persona = str(req.get("persona") or "").strip()
    diff = str(req.get("diff") or "").strip()
    cot = _cot_level(req)
    autonomy = _autonomy_level(req)
    mode_key = mode.lower()
    if mode or persona or diff:
        system_lines: list[str] = []
        if mode:
            system_lines.append(f"Mode: {mode}")
        if persona:
            system_lines.append(f"Persona: {persona}")
        if diff:
            system_lines.append("Diff:\n" + diff)
        if cot != "off":
            system_lines.append(
                "Reasoning format: include a short 'Reasoning (brief)' section "
                "with at most 3 bullets, then provide the final answer."
            )
        if autonomy == "longrun":
            system_lines.append(LONGRUN_AUTONOMY_PROMPT)
        if mode_key != "observer":
            system_lines.append(CODER_IDENTITY_PROMPT)
        if mode_key == "observer":
            system_lines.append(
                "Observer role: review coder outputs and code snippets from coder_context.\n"
                "- Focus on bugs, regressions, missing tests, and unsafe assumptions.\n"
                "- Cite concrete evidence from coder_context when possible.\n"
                "- If coder_context contains concrete errors (for example 429, 401, 403, 10061), "
                "you must explicitly mention them.\n"
                "- If you have actionable guidance, append:\n"
                "--- proposals ---\n"
                "1) title: <short title>\n"
                "   to_coder: <message to send>\n"
                "   severity: info|warn|crit"
            )
            persona_style = _observer_persona_prompt(persona)
            if persona_style:
                system_lines.append(persona_style)
        messages.insert(0, {"role": "system", "content": "\n\n".join(system_lines)})

    messages.append({"role": "user", "content": str(req.get("input") or "")})
    return messages


def _extract_cli_message_text(msg: dict[str, Any]) -> str:
    content = msg.get("content")
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        chunks: list[str] = []
        for part in content:
            if isinstance(part, str):
                chunks.append(part)
                continue
            if not isinstance(part, dict):
                continue
            text = part.get("text")
            if isinstance(text, str):
                chunks.append(text)
        return "".join(chunks)
    return ""


def _extract_mistral_cli_output(stdout: str) -> str:
    raw = str(stdout or "").strip()
    if not raw:
        return ""

    try:
        payload = json.loads(raw)
    except json.JSONDecodeError:
        return raw

    if isinstance(payload, dict):
        if isinstance(payload.get("content"), str):
            return payload["content"]
        messages = payload.get("messages")
        if isinstance(messages, list):
            for m in reversed(messages):
                if isinstance(m, dict) and str(m.get("role", "")).lower() == "assistant":
                    txt = _extract_cli_message_text(m)
                    if txt.strip():
                        return txt
        text = payload.get("text")
        if isinstance(text, str) and text.strip():
            return text

    if isinstance(payload, list):
        for m in reversed(payload):
            if isinstance(m, dict) and str(m.get("role", "")).lower() == "assistant":
                txt = _extract_cli_message_text(m)
                if txt.strip():
                    return txt

    return raw


def _chat_mistral_cli(req: dict[str, Any], timeout: int) -> dict[str, Any]:
    cmd_raw = os.environ.get("OBS_MISTRAL_CLI_CMD", "vibe").strip() or "vibe"
    base_cmd = shlex.split(cmd_raw)
    if not base_cmd:
        base_cmd = ["vibe"]

    mode = str(req.get("mode") or "").strip().lower()
    default_agent = "accept-edits" if mode != "observer" else "plan"
    agent = str(
        req.get("mistral_cli_agent")
        or os.environ.get("OBS_MISTRAL_CLI_AGENT", default_agent)
    ).strip()
    autonomy = _autonomy_level(req)
    default_turns = "8" if autonomy == "longrun" else "4"
    max_turns = int(
        req.get("mistral_cli_max_turns")
        or os.environ.get("OBS_MISTRAL_CLI_MAX_TURNS", default_turns)
    )
    max_turns = min(12, max(1, max_turns))

    messages = _build_messages(req)
    prompt_parts: list[str] = []
    for m in messages:
        role = str(m.get("role") or "").upper() or "USER"
        txt = str(m.get("content") or "").strip()
        if txt:
            prompt_parts.append(f"{role}:\n{txt}")
    prompt_parts.append("ASSISTANT:\n")
    prompt_text = "\n\n".join(prompt_parts)

    cmd = list(base_cmd)
    cmd.extend(["--output", "json", "--max-turns", str(max_turns)])
    if agent:
        cmd.extend(["--agent", agent])
    cmd.append(prompt_text)

    try:
        cp = subprocess.run(
            cmd,
            cwd=str(WORKSPACE_ROOT),
            capture_output=True,
            text=True,
            timeout=timeout,
            encoding="utf-8",
            errors="replace",
            shell=False,
        )
    except FileNotFoundError:
        raise RuntimeError(
            "Mistral CLI not found. Install with: uv tool install mistral-vibe "
            "or set OBS_MISTRAL_CLI_CMD."
        ) from None
    except subprocess.TimeoutExpired:
        raise RuntimeError(f"Mistral CLI timed out after {timeout}s") from None

    if cp.returncode != 0:
        detail = (cp.stderr or cp.stdout or "").strip()
        if len(detail) > 3000:
            detail = detail[:3000] + "\n...truncated..."
        raise RuntimeError(f"Mistral CLI failed (exit {cp.returncode})\n{detail}")

    content = _extract_mistral_cli_output(cp.stdout)
    return {"content": content, "model": "mistral-cli"}


def _chat_impl(req: dict[str, Any]) -> dict[str, Any]:
    provider = _provider_from_req(req)
    if provider == "hf":
        raise RuntimeError("HF provider is not supported in lite mode")

    base_url = _base_url(provider, req)
    model = _pick_model(provider, req)
    api_key = _pick_api_key(provider, req)
    timeout = int(req.get("timeout_seconds") or 120)
    temperature = req.get("temperature")
    max_tokens = req.get("max_tokens")
    messages = _build_messages(req)

    if provider in ("mistral", "codestral", "openai-compatible"):
        tools_enabled = provider in ("openai-compatible", "codestral", "mistral") and _local_tools_enabled(req)
        autonomy = _autonomy_level(req)
        max_tool_steps = 30 if autonomy == "longrun" else LOCAL_TOOL_MAX_STEPS
        return _chat_openai_compat(
            req=req,
            provider=provider,
            base_url=base_url,
            model=model,
            api_key=api_key,
            timeout=timeout,
            temperature=temperature,
            max_tokens=max_tokens,
            messages=messages,
            tools_enabled=tools_enabled,
            max_tool_steps=max_tool_steps,
        )

    if provider == "mistral-cli":
        return _chat_mistral_cli(req, timeout=timeout)

    if provider == "anthropic":
        # Anthropic ignores "system" role in messages; send it as top-level field.
        system_text = ""
        anthropic_messages: list[dict[str, Any]] = []
        for m in messages:
            if m["role"] == "system":
                system_text = m["content"]
                continue
            anthropic_messages.append({"role": m["role"], "content": m["content"]})

        payload = {
            "model": model,
            "messages": anthropic_messages,
            "max_tokens": int(max_tokens or 1024),
        }
        if system_text:
            payload["system"] = system_text
        if temperature is not None:
            payload["temperature"] = temperature

        headers = {
            "Accept": "application/json",
            "anthropic-version": ANTHROPIC_VERSION,
        }
        if api_key:
            headers["x-api-key"] = api_key

        res = _http_json(
            "POST",
            f"{base_url}/messages",
            headers=headers,
            body=payload,
            timeout_seconds=timeout,
            error_context={"provider": provider, "model": model},
        )
        content = _extract_text_from_anthropic_response(res)
        return {"content": content, "model": model}

    raise RuntimeError(f"unsupported provider: {provider}")


def _models_impl(req: dict[str, Any]) -> dict[str, Any]:
    provider = _provider_from_req(req)
    if provider == "hf":
        return {"models": []}
    if provider == "mistral-cli":
        return {"models": ["mistral-medium-latest", "mistral-small-latest"]}
    if provider == "codestral":
        return {"models": ["codestral-latest", "mistral-small-latest"]}

    base_url = _base_url(provider, req)
    api_key = _pick_api_key(provider, req)
    timeout = 60

    headers: dict[str, str] = {"Accept": "application/json"}
    if provider == "anthropic":
        if api_key:
            headers["x-api-key"] = api_key
        headers["anthropic-version"] = ANTHROPIC_VERSION
    elif api_key:
        headers["Authorization"] = f"Bearer {api_key}"

    res = _http_json(
        "GET",
        f"{base_url}/models",
        headers=headers,
        timeout_seconds=timeout,
        error_context={"provider": provider},
    )

    models: list[str] = []
    if isinstance(res, dict) and isinstance(res.get("data"), list):
        for item in res["data"]:
            if isinstance(item, dict) and item.get("id"):
                models.append(str(item["id"]))
    elif isinstance(res, list):
        for item in res:
            if isinstance(item, str):
                models.append(item)

    models = sorted(set(m for m in models if m.strip()))
    return {"models": models}


class LiteHandler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def _send_bytes(self, code: int, content_type: str, payload: bytes) -> None:
        self.send_response(code)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(payload)))
        self.send_header("Connection", "close")
        self.end_headers()
        self.wfile.write(payload)

    def _send_json(self, code: int, payload: dict[str, Any]) -> None:
        data = json.dumps(payload, ensure_ascii=False).encode("utf-8")
        self._send_bytes(code, "application/json; charset=utf-8", data)

    def _send_text(self, code: int, text: str) -> None:
        self._send_bytes(code, "text/plain; charset=utf-8", text.encode("utf-8"))

    def _read_json_body(self) -> dict[str, Any]:
        raw_len = self.headers.get("Content-Length", "0")
        try:
            length = int(raw_len)
        except ValueError:
            raise ValueError("invalid content-length")
        if length < 0 or length > MAX_BODY_BYTES:
            raise ValueError("request body too large")
        raw = self.rfile.read(length) if length else b"{}"
        try:
            payload = json.loads(raw.decode("utf-8"))
        except json.JSONDecodeError:
            raise ValueError("invalid JSON body")
        if not isinstance(payload, dict):
            raise ValueError("JSON body must be an object")
        return payload

    def _serve_file(self, path: Path, content_type: str) -> None:
        if not path.exists():
            self._send_text(404, "not found\n")
            return
        self._send_bytes(200, content_type, path.read_bytes())

    def do_GET(self) -> None:
        if self.path == "/":
            self._serve_file(WEB_ROOT / "index.html", "text/html; charset=utf-8")
            return
        if self.path == "/assets/app.js":
            self._serve_file(WEB_ROOT / "app.js", "text/javascript; charset=utf-8")
            return
        if self.path == "/assets/styles.css":
            self._serve_file(WEB_ROOT / "styles.css", "text/css; charset=utf-8")
            return
        if self.path == "/assets/vendor/react.production.min.js":
            self._serve_file(
                WEB_ROOT / "vendor" / "react.production.min.js",
                "text/javascript; charset=utf-8",
            )
            return
        if self.path == "/assets/vendor/react-dom.production.min.js":
            self._serve_file(
                WEB_ROOT / "vendor" / "react-dom.production.min.js",
                "text/javascript; charset=utf-8",
            )
            return
        if self.path == "/api/status":
            self._send_json(
                200,
                {
                    "ok": True,
                    "version": "lite",
                    "features": {
                        "edit_approval_default": _as_bool(os.environ.get("OBS_REQUIRE_EDIT_APPROVAL"), True),
                        "command_approval_default": _as_bool(os.environ.get("OBS_REQUIRE_COMMAND_APPROVAL"), True),
                    },
                    "providers": {
                        "mistral": {
                            "api_key_present": _env_present("MISTRAL_API_KEY")
                            or _env_present("OBS_API_KEY")
                        },
                        "codestral": {
                            "api_key_present": _env_present("MISTRAL_API_KEY")
                            or _env_present("OBS_API_KEY")
                        },
                        "anthropic": {
                            "api_key_present": _env_present("ANTHROPIC_API_KEY")
                        },
                        "openai-compatible": {
                            "api_key_present": _env_present("OBS_API_KEY")
                            or _env_present("OPENAI_API_KEY")
                        },
                    },
                },
            )
            return
        if self.path == "/api/pending_edits":
            self._send_json(200, {"pending": _list_pending_edits()})
            return

        self._send_text(404, "not found\n")

    def do_POST(self) -> None:
        try:
            payload = self._read_json_body()
        except ValueError as e:
            self._send_json(400, {"error": str(e)})
            return

        if self.path == "/api/models":
            try:
                self._send_json(200, _models_impl(payload))
            except Exception as e:
                self._send_json(502, {"error": str(e)})
            return

        if self.path == "/api/chat":
            try:
                self._send_json(200, _chat_impl(payload))
            except Exception as e:
                self._send_json(502, {"error": str(e)})
            return

        if self.path == "/api/chat_stream":
            self._serve_chat_stream(payload)
            return

        if self.path == "/api/approve_edit":
            try:
                edit_id = str(payload.get("id") or "").strip()
                if not edit_id:
                    raise RuntimeError("id is required")
                item = _approve_pending_edit(edit_id)
                self._send_json(200, {"ok": True, "item": item})
            except Exception as e:
                self._send_json(400, {"error": str(e)})
            return

        if self.path == "/api/reject_edit":
            try:
                edit_id = str(payload.get("id") or "").strip()
                if not edit_id:
                    raise RuntimeError("id is required")
                item = _reject_pending_edit(edit_id)
                self._send_json(200, {"ok": True, "item": item})
            except Exception as e:
                self._send_json(400, {"error": str(e)})
            return

        self._send_text(404, "not found\n")

    def _serve_chat_stream(self, payload: dict[str, Any]) -> None:
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream; charset=utf-8")
        self.send_header("Cache-Control", "no-cache")
        self.send_header("Connection", "close")
        self.end_headers()

        try:
            result = _chat_impl(payload)
            content = str(result.get("content") or "")
            if not content:
                self._write_sse("done", {})
                return
            chunk_size = 24
            for i in range(0, len(content), chunk_size):
                self._write_sse("delta", {"delta": content[i : i + chunk_size]})
            self._write_sse("done", {})
        except Exception as e:
            self._write_sse("error", {"error": str(e)})
            self._write_sse("done", {})
        except BrokenPipeError:
            return

    def _write_sse(self, event: str, data: dict[str, Any]) -> None:
        payload = json.dumps(data, ensure_ascii=False)
        msg = f"event: {event}\ndata: {payload}\n\n".encode("utf-8")
        self.wfile.write(msg)
        self.wfile.flush()

    def log_message(self, fmt: str, *args: Any) -> None:
        sys.stderr.write("%s - - [%s] %s\n" % (self.client_address[0], self.log_date_time_string(), fmt % args))


def main() -> int:
    parser = argparse.ArgumentParser(description="Run OBSTRAL lite server")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=18080)
    args = parser.parse_args()

    server = ThreadingHTTPServer((args.host, args.port), LiteHandler)
    print(f"OBSTRAL Lite UI: http://{args.host}:{args.port}/")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
