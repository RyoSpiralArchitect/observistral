from __future__ import annotations

from observistral.chatbot import ChatBot
from observistral.config import ProviderConfig
from observistral.factory import build_provider, supported_providers
from observistral.personas import resolve_persona, supported_personas
from observistral.providers.base import ChatProvider
from observistral.providers.openai_compat import OpenAICompatibleProvider
from observistral.types import ChatRequest, ChatResponse


class DummyProvider(ChatProvider):
    def __init__(self) -> None:
        super().__init__(model="dummy")
        self.last_request: ChatRequest | None = None

    def chat(self, request: ChatRequest) -> ChatResponse:
        self.last_request = request
        return ChatResponse(content="ok", model=self.model, raw={"seen": True})


def test_mode_prompt_injection() -> None:
    provider = DummyProvider()
    bot = ChatBot(provider)

    response = bot.run("変更差分を見て", mode="diff批評")

    assert response.content == "ok"
    assert provider.last_request is not None
    assert provider.last_request.messages[0].role == "system"
    assert "コードレビュアー" in provider.last_request.messages[0].content


def test_persona_prompt_injection() -> None:
    provider = DummyProvider()
    bot = ChatBot(provider)

    bot.run("雰囲気重視で", mode="実況", persona="novelist")

    assert provider.last_request is not None
    system_msg = provider.last_request.messages[0].content
    assert "Persona" in system_msg
    assert "小説家" in system_msg


def test_diff_text_is_appended_when_diff_mode() -> None:
    provider = DummyProvider()
    bot = ChatBot(provider)

    bot.run("レビューして", mode="diff批評", diff_text="+ added")

    assert provider.last_request is not None
    user_msg = provider.last_request.messages[1].content
    assert "```diff" in user_msg
    assert "+ added" in user_msg


def test_factory_openai_compatible_allows_missing_api_key() -> None:
    config = ProviderConfig(provider="openai-compatible", model="gpt-4o-mini", api_key=None)
    provider = build_provider(config)
    assert isinstance(provider, OpenAICompatibleProvider)


def test_factory_rejects_missing_api_key_for_anthropic() -> None:
    config = ProviderConfig(provider="anthropic", model="claude-3-5-sonnet-latest", api_key=None)
    try:
        build_provider(config)
        assert False, "expected ValueError"
    except ValueError as exc:
        assert "api_key" in str(exc)


def test_supported_providers_contains_expected_aliases() -> None:
    providers = supported_providers()
    assert "openai-compatible" in providers
    assert "anthropic" in providers
    assert "hf" in providers


def test_persona_catalog() -> None:
    personas = supported_personas()
    assert "novelist" in personas
    assert "cynical" in personas
    assert "cheerful" in personas
    assert "thoughtful" in personas
    assert resolve_persona("novelist").label == "Novelist"
