from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class Persona:
    key: str
    label: str
    prompt: str


PERSONAS: dict[str, Persona] = {
    "default": Persona(
        key="default",
        label="Balanced",
        prompt="バランス重視で、実用的かつ誠実に回答してください。",
    ),
    "novelist": Persona(
        key="novelist",
        label="Novelist",
        prompt="描写力の高い小説家の文体で、情景や比喩を交えつつ読みやすく回答してください。",
    ),
    "cynical": Persona(
        key="cynical",
        label="Cynical",
        prompt="皮肉とユーモアを少し効かせつつ、核心を鋭く指摘してください。攻撃的にはならないでください。",
    ),
    "cheerful": Persona(
        key="cheerful",
        label="Cheerful",
        prompt="明るく前向きで、相手を励ますトーンで回答してください。",
    ),
    "thoughtful": Persona(
        key="thoughtful",
        label="Thoughtful",
        prompt="思慮深く、前提を確認しながら段階的に丁寧に回答してください。",
    ),
}


def normalize_persona(persona: str) -> str:
    return persona.strip().lower()


def supported_personas() -> list[str]:
    return sorted(PERSONAS)


def resolve_persona(persona: str) -> Persona:
    key = normalize_persona(persona)
    if key not in PERSONAS:
        raise ValueError(f"Unsupported persona: {persona}. Available: {', '.join(supported_personas())}")
    return PERSONAS[key]
