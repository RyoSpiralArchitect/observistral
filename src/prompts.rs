use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, ValueEnum, Serialize, Deserialize)]
pub enum Mode {
    #[serde(rename = "実況")]
    #[value(name = "実況", alias = "jikkyo", alias = "live")]
    Jikkyo,

    #[serde(rename = "壁打ち")]
    #[value(name = "壁打ち", alias = "kabeuchi", alias = "ideation")]
    Kabeuchi,

    #[serde(rename = "diff批評")]
    #[value(name = "diff批評", alias = "diff", alias = "review")]
    DiffReview,

    #[serde(rename = "VIBE")]
    #[value(name = "VIBE", alias = "vibe")]
    Vibe,
}

impl Mode {
    pub fn label(&self) -> &'static str {
        match self {
            Mode::Jikkyo => "実況",
            Mode::Kabeuchi => "壁打ち",
            Mode::DiffReview => "diff批評",
            Mode::Vibe => "VIBE",
        }
    }
}

#[derive(Clone, Debug, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Persona {
    #[serde(rename = "default")]
    Default,
    Novelist,
    Cynical,
    Cheerful,
    Thoughtful,
}

impl Persona {
    pub fn key(&self) -> &'static str {
        match self {
            Persona::Default => "default",
            Persona::Novelist => "novelist",
            Persona::Cynical => "cynical",
            Persona::Cheerful => "cheerful",
            Persona::Thoughtful => "thoughtful",
        }
    }
}

pub fn supported_modes() -> Vec<&'static str> {
    vec!["実況", "壁打ち", "diff批評", "VIBE"]
}

pub fn supported_personas() -> Vec<&'static str> {
    vec!["default", "novelist", "cynical", "cheerful", "thoughtful"]
}

fn mode_prompt(mode: &Mode) -> &'static str {
    match mode {
        Mode::Jikkyo => {
            "You are a live narrator assistant. As you work, explain what you are doing and why."
        }
        Mode::Kabeuchi => {
            "You are an ideation partner. Ask clarifying questions, surface tradeoffs, and propose options."
        }
        Mode::DiffReview => {
            "You are a senior code reviewer. Review the diff with rigor: bugs, risks, tests, and concrete improvements."
        }
        Mode::Vibe => {
            "You are a vibe-coding co-pilot. Move fast and stay pragmatic: propose a design and implementation steps with working code where possible."
        }
    }
}

fn persona_prompt(persona: &Persona) -> &'static str {
    match persona {
        Persona::Default => "Balanced. Be concise, accurate, and practical.",
        Persona::Novelist => {
            "Novelist. Write with narrative clarity, vivid imagery, and strong pacing (without losing correctness)."
        }
        Persona::Cynical => {
            "Cynical. Be blunt and skeptical. Call out weak assumptions and hidden risks."
        }
        Persona::Cheerful => "Cheerful. Be upbeat and encouraging, but still specific and useful.",
        Persona::Thoughtful => {
            "Thoughtful. Be reflective, careful, and explicit about tradeoffs and uncertainty."
        }
    }
}

pub fn build_system_prompt(mode: &Mode, persona: &Persona) -> String {
    format!(
        "{}\n\n[Persona]\n{}",
        mode_prompt(mode),
        persona_prompt(persona)
    )
}

pub fn compose_user_text(user_input: &str, mode: &Mode, diff_text: Option<&str>) -> String {
    if !matches!(mode, Mode::DiffReview) {
        return user_input.to_string();
    }
    let Some(diff) = diff_text else {
        return user_input.to_string();
    };
    let diff = diff.trim();
    if diff.is_empty() {
        return user_input.to_string();
    }
    format!("{user_input}\n\nHere is the diff to review:\n```diff\n{diff}\n```")
}
