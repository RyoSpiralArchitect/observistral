from __future__ import annotations

from observistral.config import ProviderConfig
from observistral.providers.anthropic import AnthropicProvider
from observistral.providers.base import ChatProvider
from observistral.providers.hf_local import HuggingFaceLocalProvider
from observistral.providers.openai_compat import OpenAICompatibleProvider

OPENAI_COMPAT_NAMES = {"openai", "openai-compatible", "openai_compat", "compatible"}
MISTRAL_NAMES = {"mistral", "mistral-ai", "mistralai"}
ANTHROPIC_NAMES = {"anthropic"}
HF_LOCAL_NAMES = {"hf", "huggingface", "huggingface-local", "local"}



def supported_providers() -> list[str]:
    return sorted(OPENAI_COMPAT_NAMES | MISTRAL_NAMES | ANTHROPIC_NAMES | HF_LOCAL_NAMES)



def build_provider(config: ProviderConfig) -> ChatProvider:
    provider = config.normalized_provider

    if provider in OPENAI_COMPAT_NAMES:
        return OpenAICompatibleProvider(
            model=config.model,
            api_key=config.api_key,
            base_url=config.base_url or "https://api.openai.com/v1",
            timeout_seconds=config.timeout_seconds,
        )

    if provider in MISTRAL_NAMES:
        if not config.api_key:
            raise ValueError("api_key is required for Mistral provider")
        return OpenAICompatibleProvider(
            model=config.model,
            api_key=config.api_key,
            base_url=config.base_url or "https://api.mistral.ai/v1",
            timeout_seconds=config.timeout_seconds,
        )

    if provider in ANTHROPIC_NAMES:
        if not config.api_key:
            raise ValueError("api_key is required for Anthropic provider")
        return AnthropicProvider(
            model=config.model,
            api_key=config.api_key,
            base_url=config.base_url or "https://api.anthropic.com/v1",
            timeout_seconds=config.timeout_seconds,
        )

    if provider in HF_LOCAL_NAMES:
        return HuggingFaceLocalProvider(model=config.model, device=config.device)

    raise ValueError(f"Unsupported provider: {config.provider}")
