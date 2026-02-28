from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Any
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen


@dataclass(frozen=True, slots=True)
class HttpResponseError(RuntimeError):
    url: str
    status: int
    body: str | None


def post_json(
    url: str,
    payload: dict[str, Any],
    headers: dict[str, str] | None = None,
    timeout_seconds: int = 120,
) -> dict[str, Any]:
    data = json.dumps(payload).encode("utf-8")
    request = Request(
        url,
        data=data,
        headers={"Content-Type": "application/json", **(headers or {})},
        method="POST",
    )

    try:
        with urlopen(request, timeout=timeout_seconds) as response:
            body = response.read()
            encoding = getattr(response.headers, "get_content_charset", lambda: None)() or "utf-8"
            text = body.decode(encoding, errors="replace")
            return json.loads(text)
    except HTTPError as exc:
        body_text: str | None = None
        try:
            body_text = exc.read().decode("utf-8", errors="replace")
        except Exception:
            body_text = None
        raise HttpResponseError(url=url, status=getattr(exc, "code", 0), body=body_text) from exc
    except URLError as exc:
        raise RuntimeError(f"Request failed: {exc}") from exc
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"Invalid JSON response from {url}: {exc}") from exc
