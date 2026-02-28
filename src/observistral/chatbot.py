from __future__ import annotations

from dataclasses import dataclass

from observistral.personas import resolve_persona
from observistral.providers.base import ChatProvider
from observistral.types import ChatMessage, ChatRequest, ChatResponse


MODE_PROMPTS: dict[str, str] = {
    "実況": "あなたは実況アシスタントです。状況をテンポよく、臨場感を持って説明してください。",
    "壁打ち": "あなたは壁打ち相手です。アイデアを構造化し、次の具体的アクションを提案してください。",
    "diff批評": "あなたはコードレビュアーです。入力されたdiffを読み、リスク・改善案・テスト観点を批評してください。",
    "VIBE": "あなたは熟練のソフトウェアエンジニアです。ユーザーがVIBEコーディングできるように、最短で動く実装案を提示してください。",
}


def supported_modes() -> list[str]:
    return list(MODE_PROMPTS)


def build_system_text(mode: str, persona: str) -> str:
    system_prompt = MODE_PROMPTS.get(mode, MODE_PROMPTS["壁打ち"])
    persona_prompt = resolve_persona(persona).prompt
    return f"{system_prompt}\n\n[Persona]\n{persona_prompt}"


def compose_user_text(user_input: str, mode: str, diff_text: str | None) -> str:
    if mode != "diff批評" or not diff_text:
        return user_input
    return (
        f"{user_input}\n\n"
        "以下が差分です。重要箇所を優先してレビューしてください。\n"
        "```diff\n"
        f"{diff_text.strip()}\n"
        "```"
    )


@dataclass(slots=True)
class ChatBot:
    provider: ChatProvider

    def run(
        self,
        user_input: str,
        mode: str = "壁打ち",
        persona: str = "default",
        temperature: float = 0.4,
        max_tokens: int = 1024,
        diff_text: str | None = None,
    ) -> ChatResponse:
        system_text = build_system_text(mode=mode, persona=persona)
        request = ChatRequest(
            messages=[
                ChatMessage(role="system", content=system_text),
                ChatMessage(role="user", content=compose_user_text(user_input=user_input, mode=mode, diff_text=diff_text)),
            ],
            temperature=temperature,
            max_tokens=max_tokens,
        )
        return self.provider.chat(request)
