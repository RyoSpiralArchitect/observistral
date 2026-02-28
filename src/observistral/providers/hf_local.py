from __future__ import annotations

import os

from observistral.providers.base import ChatProvider
from observistral.types import ChatRequest, ChatResponse


class HuggingFaceLocalProvider(ChatProvider):
    """Offline/local Hugging Face provider using transformers text-generation pipeline."""

    def __init__(self, model: str, device: str = "auto") -> None:
        super().__init__(model)
        try:
            from transformers import pipeline
        except ImportError as exc:  # pragma: no cover
            raise RuntimeError(
                "transformers is required for HuggingFaceLocalProvider. Install with `pip install observistral[hf]`."
            ) from exc

        local_only = os.getenv("OBS_HF_LOCAL_ONLY", "0") == "1"
        device_arg = 0 if device == "cuda" else -1
        self._pipeline = pipeline(
            "text-generation",
            model=model,
            device=device_arg,
            local_files_only=local_only,
        )

    def chat(self, request: ChatRequest) -> ChatResponse:
        prompt = self._to_prompt(request)
        generated = self._pipeline(
            prompt,
            max_new_tokens=request.max_tokens or 256,
            temperature=request.temperature,
            do_sample=True,
            return_full_text=True,
        )
        text = generated[0]["generated_text"]
        answer = text[len(prompt) :].strip() if text.startswith(prompt) else text
        return ChatResponse(content=answer, model=self.model, raw={"generated": generated})

    @staticmethod
    def _to_prompt(request: ChatRequest) -> str:
        lines = [f"{m.role.upper()}: {m.content}" for m in request.messages]
        lines.append("ASSISTANT:")
        return "\n".join(lines)
