from __future__ import annotations

import unittest

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


class TestChatbot(unittest.TestCase):
    def test_mode_prompt_injection(self) -> None:
        provider = DummyProvider()
        bot = ChatBot(provider)

        response = bot.run("変更差分を見て", mode="diff批評")

        self.assertEqual(response.content, "ok")
        self.assertIsNotNone(provider.last_request)
        assert provider.last_request is not None
        self.assertEqual(provider.last_request.messages[0].role, "system")
        self.assertIn("コードレビュアー", provider.last_request.messages[0].content)

    def test_persona_prompt_injection(self) -> None:
        provider = DummyProvider()
        bot = ChatBot(provider)

        bot.run("雰囲気重視で", mode="実況", persona="novelist")

        self.assertIsNotNone(provider.last_request)
        assert provider.last_request is not None
        system_msg = provider.last_request.messages[0].content
        self.assertIn("Persona", system_msg)
        self.assertIn("小説家", system_msg)

    def test_diff_text_is_appended_when_diff_mode(self) -> None:
        provider = DummyProvider()
        bot = ChatBot(provider)

        bot.run("レビューして", mode="diff批評", diff_text="+ added")

        self.assertIsNotNone(provider.last_request)
        assert provider.last_request is not None
        user_msg = provider.last_request.messages[1].content
        self.assertIn("```diff", user_msg)
        self.assertIn("+ added", user_msg)

    def test_factory_openai_compatible_allows_missing_api_key(self) -> None:
        config = ProviderConfig(provider="openai-compatible", model="gpt-4o-mini", api_key=None)
        provider = build_provider(config)
        self.assertIsInstance(provider, OpenAICompatibleProvider)

    def test_factory_rejects_missing_api_key_for_anthropic(self) -> None:
        config = ProviderConfig(provider="anthropic", model="claude-3-5-sonnet-latest", api_key=None)
        with self.assertRaises(ValueError) as ctx:
            build_provider(config)
        self.assertIn("api_key", str(ctx.exception))

    def test_factory_rejects_missing_api_key_for_mistral(self) -> None:
        config = ProviderConfig(provider="mistral", model="devstral-2", api_key=None)
        with self.assertRaises(ValueError) as ctx:
            build_provider(config)
        self.assertIn("api_key", str(ctx.exception))

    def test_supported_providers_contains_expected_aliases(self) -> None:
        providers = supported_providers()
        self.assertIn("openai-compatible", providers)
        self.assertIn("mistral", providers)
        self.assertIn("anthropic", providers)
        self.assertIn("hf", providers)

    def test_persona_catalog(self) -> None:
        personas = supported_personas()
        self.assertIn("novelist", personas)
        self.assertIn("cynical", personas)
        self.assertIn("cheerful", personas)
        self.assertIn("thoughtful", personas)
        self.assertEqual(resolve_persona("novelist").label, "Novelist")
