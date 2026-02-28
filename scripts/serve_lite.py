#!/usr/bin/env python3
"""Minimal OBSTRAL-compatible server that avoids running a custom EXE."""

from __future__ import annotations

import argparse
import json
import os
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any
from urllib import error as urlerror
from urllib import request as urlrequest


REPO_ROOT = Path(__file__).resolve().parents[1]
WEB_ROOT = REPO_ROOT / "web"
MAX_BODY_BYTES = 2 * 1024 * 1024
ANTHROPIC_VERSION = "2023-06-01"
DIRECT_OPENER = urlrequest.build_opener(urlrequest.ProxyHandler({}))

DEFAULT_BASE_URL = {
    "mistral": "https://api.mistral.ai/v1",
    "openai-compatible": "https://api.openai.com/v1",
    "anthropic": "https://api.anthropic.com/v1",
}

DEFAULT_MODEL = {
    "mistral": "mistral-small-latest",
    "openai-compatible": "gpt-4o-mini",
    "anthropic": "claude-3-5-sonnet-latest",
}


def _env_present(name: str) -> bool:
    v = os.environ.get(name, "").strip()
    return bool(v)


def _provider_key_from_env(provider: str) -> str:
    if provider == "mistral":
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
    if provider in ("mistral", "openai-compatible", "anthropic", "hf"):
        return provider
    return "mistral"


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


def _http_json(
    method: str,
    url: str,
    *,
    headers: dict[str, str] | None = None,
    body: dict[str, Any] | None = None,
    timeout_seconds: int = 120,
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
        raise RuntimeError(f"HTTP {e.code}\n{text}") from None
    except urlerror.URLError as e:
        raise RuntimeError(f"request failed: {e.reason}") from None

    if not raw:
        return {}
    try:
        return json.loads(raw.decode("utf-8"))
    except json.JSONDecodeError:
        raise RuntimeError("invalid JSON response") from None


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
    if mode or persona or diff:
        system_lines: list[str] = []
        if mode:
            system_lines.append(f"Mode: {mode}")
        if persona:
            system_lines.append(f"Persona: {persona}")
        if diff:
            system_lines.append("Diff:\n" + diff)
        messages.insert(0, {"role": "system", "content": "\n\n".join(system_lines)})

    messages.append({"role": "user", "content": str(req.get("input") or "")})
    return messages


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

    if provider in ("mistral", "openai-compatible"):
        payload: dict[str, Any] = {
            "model": model,
            "messages": messages,
            "stream": False,
        }
        if temperature is not None:
            payload["temperature"] = temperature
        if max_tokens is not None:
            payload["max_tokens"] = max_tokens

        headers = {"Accept": "application/json"}
        if api_key:
            headers["Authorization"] = f"Bearer {api_key}"

        res = _http_json(
            "POST",
            f"{base_url}/chat/completions",
            headers=headers,
            body=payload,
            timeout_seconds=timeout,
        )
        content = (
            (((res.get("choices") or [{}])[0]).get("message") or {}).get("content") or ""
        )
        return {"content": str(content), "model": model}

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
        )
        content = _extract_text_from_anthropic_response(res)
        return {"content": content, "model": model}

    raise RuntimeError(f"unsupported provider: {provider}")


def _models_impl(req: dict[str, Any]) -> dict[str, Any]:
    provider = _provider_from_req(req)
    if provider == "hf":
        return {"models": []}

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
                    "providers": {
                        "mistral": {
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
    parser.add_argument("--port", type=int, default=8080)
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
