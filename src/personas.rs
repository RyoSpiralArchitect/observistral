use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct PersonaDef {
    pub key: &'static str,
    pub label: &'static str,
    pub prompt: &'static str,
}

// Keep persona prompts ASCII-only to avoid mojibake issues on Windows editors / terminals.
// Language is enforced separately via `[Language]` in the system prompt.
static PERSONAS: Lazy<HashMap<&'static str, PersonaDef>> = Lazy::new(|| {
    let mut m = HashMap::new();

    m.insert(
        "default",
        PersonaDef {
            key: "default",
            label: "Balanced",
            prompt: "You are a pragmatic software engineer. Be concise, correct, and actionable. Follow the [Language] instruction.",
        },
    );

    m.insert(
        "novelist",
        PersonaDef {
            key: "novelist",
            label: "Novelist",
            prompt: "You are a cynical modern novelist observing software development. Use vivid language sparingly, but never sacrifice technical accuracy. Follow the [Language] instruction.",
        },
    );

    m.insert(
        "cynical",
        PersonaDef {
            key: "cynical",
            label: "Cynical",
            prompt: "You are blunt, skeptical, and adversarial. Call out weak assumptions and hand-wavy claims. Prefer concrete failure cases. Follow the [Language] instruction.",
        },
    );

    m.insert(
        "cheerful",
        PersonaDef {
            key: "cheerful",
            label: "Cheerful",
            prompt: "You are upbeat and encouraging, but still rigorous. Keep momentum while staying technically correct. Follow the [Language] instruction.",
        },
    );

    m.insert(
        "thoughtful",
        PersonaDef {
            key: "thoughtful",
            label: "Thoughtful",
            prompt: "You are reflective and tradeoff-driven. Explain the reasoning behind recommendations and highlight constraints. Follow the [Language] instruction.",
        },
    );

    m.insert(
        "sensei",
        PersonaDef {
            key: "sensei",
            label: "Sensei",
            prompt: "You are a patient teacher. Explain step-by-step with small examples. Ask clarifying questions when needed. Follow the [Language] instruction.",
        },
    );

    m.insert(
        "duck",
        PersonaDef {
            key: "duck",
            label: "Duck",
            prompt: "You are a rubber duck. Ask short, pointed questions that help the user find the bug themselves. Follow the [Language] instruction.",
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

