from __future__ import annotations

from typing import Any

import requests

from observistral.providers.base import ChatProvider
from observistral.types import ChatRequest, ChatResponse


class OpenAICompatibleProvider(ChatProvider):
    """Works with OpenAI-compatible APIs (OpenAI, gateways, vLLM, LM Studio, etc.)."""

    def __init__(
        self,
        model: str,
        api_key: str | None,
        base_url: str = "https://api.openai.com/v1",
        timeout_seconds: int = 120,
    ) -> None:
        super().__init__(model)
        self.api_key = api_key
        self.base_url = base_url.rstrip("/")
        self.timeout_seconds = timeout_seconds

    def chat(self, request: ChatRequest) -> ChatResponse:
        url = f"{self.base_url}/chat/completions"
        payload: dict[str, Any] = {
            "model": self.model,
            "messages": [{"role": m.role, "content": m.content} for m in request.messages],
            "temperature": request.temperature,
        }
        if request.max_tokens is not None:
            payload["max_tokens"] = request.max_tokens
        if request.metadata:
            payload.update(request.metadata)

        headers = {"Content-Type": "application/json"}
        if self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"

        response = requests.post(
            url,
            json=payload,
            headers=headers,
            timeout=self.timeout_seconds,
        )
        response.raise_for_status()
        data = response.json()
        content = data["choices"][0]["message"]["content"]
        return ChatResponse(content=content, model=self.model, raw=data)
