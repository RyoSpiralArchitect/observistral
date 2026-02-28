from __future__ import annotations

from typing import Any

from observistral.http_client import HttpResponseError, post_json
from observistral.providers.base import ChatProvider
from observistral.types import ChatMessage, ChatRequest, ChatResponse


class AnthropicProvider(ChatProvider):
    def __init__(
        self,
        model: str,
        api_key: str,
        base_url: str = "https://api.anthropic.com/v1",
        timeout_seconds: int = 120,
    ) -> None:
        super().__init__(model)
        self.api_key = api_key
        self.base_url = base_url.rstrip("/")
        self.timeout_seconds = timeout_seconds

    def chat(self, request: ChatRequest) -> ChatResponse:
        url = f"{self.base_url}/messages"
        system_text = self._extract_system(request.messages)
        messages = [
            {"role": m.role, "content": m.content}
            for m in request.messages
            if m.role in {"user", "assistant"}
        ]

        payload: dict[str, Any] = {
            "model": self.model,
            "messages": messages,
            "temperature": request.temperature,
            "max_tokens": request.max_tokens or 1024,
        }
        if system_text:
            payload["system"] = system_text
        if request.metadata:
            payload.update(request.metadata)

        headers = {
            "x-api-key": self.api_key,
            "anthropic-version": "2023-06-01",
            "Content-Type": "application/json",
        }

        try:
            data = post_json(url, payload=payload, headers=headers, timeout_seconds=self.timeout_seconds)
        except HttpResponseError as exc:
            detail = f"HTTP {exc.status}"
            if exc.body:
                detail = f"{detail}: {exc.body}"
            raise RuntimeError(f"Anthropic API error ({detail})") from exc
        content_blocks = data.get("content", [])
        text = "".join(block.get("text", "") for block in content_blocks if block.get("type") == "text")
        return ChatResponse(content=text, model=self.model, raw=data)

    @staticmethod
    def _extract_system(messages: list[ChatMessage]) -> str | None:
        systems = [m.content for m in messages if m.role == "system"]
        if not systems:
            return None
        return "\n".join(systems)
