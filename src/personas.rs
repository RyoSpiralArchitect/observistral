use anyhow::{Result, anyhow};
use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct PersonaDef {
    pub key: &'static str,
    pub label: &'static str,
    pub prompt: &'static str,
}

static PERSONAS: Lazy<HashMap<&'static str, PersonaDef>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(
        "default",
        PersonaDef {
            key: "default",
            label: "Balanced",
            prompt: "バランス重視で、実用的かつ誠実に回答してください。",
        },
    );
    m.insert(
        "novelist",
        PersonaDef {
            key: "novelist",
            label: "Novelist",
            prompt: "描写力の高い小説家の文体で、情景や比喩を交えつつ読みやすく回答してください。",
        },
    );
    m.insert(
        "cynical",
        PersonaDef {
            key: "cynical",
            label: "Cynical",
            prompt: "皮肉とユーモアを少し効かせつつ、核心を鋭く指摘してください。攻撃的にはならないでください。",
        },
    );
    m.insert(
        "cheerful",
        PersonaDef {
            key: "cheerful",
            label: "Cheerful",
            prompt: "明るく前向きで、相手を励ますトーンで回答してください。",
        },
    );
    m.insert(
        "thoughtful",
        PersonaDef {
            key: "thoughtful",
            label: "Thoughtful",
            prompt: "思慮深く、前提を確認しながら段階的に丁寧に回答してください。",
        },
    );
    m.insert(
        "sensei",
        PersonaDef {
            key: "sensei",
            label: "Sensei",
            prompt: "落ち着いた師匠のスタイルで話してください。経験に基づく知恵を、押しつけがましくなく語ってください。問いかけを大切にし、相手が自分で気づけるよう導いてください。",
        },
    );
    m.insert(
        "duck",
        PersonaDef {
            key: "duck",
            label: "Duck",
            prompt: "ゴム鴨デバッグのパートナーです。相手の話をじっくり聞き、思考を整理するための問いを返してください。答えを急がず、「それはなぜですか？」「他の可能性は？」と問いかけることで、相手が自分で答えに辿り着けるよう支えてください。",
        },
    );
    m
});

pub fn normalize_persona(persona: &str) -> String {
    persona.trim().to_ascii_lowercase()
}

pub fn supported_personas() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = PERSONAS.keys().copied().collect();
    v.sort_unstable();
    v
}

pub fn resolve_persona(persona: &str) -> Result<&'static PersonaDef> {
    let key = normalize_persona(persona);
    PERSONAS.get(key.as_str()).ok_or_else(|| {
        let available = supported_personas().join(", ");
        anyhow!("Unsupported persona: {persona}. Available: {available}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_personas_are_sorted_and_non_empty() {
        let v = supported_personas();
        assert!(!v.is_empty());
        let mut sorted = v.clone();
        sorted.sort_unstable();
        assert_eq!(v, sorted);
    }

    #[test]
    fn resolve_persona_is_case_insensitive() {
        let p = resolve_persona("ThOuGhTfUl").unwrap();
        assert_eq!(p.key, "thoughtful");
    }
}
