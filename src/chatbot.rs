use anyhow::Result;
use std::sync::Arc;

use crate::lang_detect;
use crate::loop_detect;
use crate::modes::{compose_user_text, mode_prompt, Mode};
use crate::personas;
use crate::providers::ChatProvider;
use crate::types::{ChatMessage, ChatRequest, ChatResponse};

pub struct ChatBot {
    provider: Arc<dyn ChatProvider>,
}

impl ChatBot {
    pub fn new(provider: Arc<dyn ChatProvider>) -> Self {
        Self { provider }
    }

    pub async fn run(
        &self,
        user_input: &str,
        history: &[ChatMessage],
        mode: &Mode,
        persona: &str,
        lang: Option<&str>,
        cot: &str,
        temperature: f64,
        max_tokens: u32,
        diff_text: Option<&str>,
        log_text: Option<&str>,
    ) -> Result<ChatResponse> {
        let persona_def = personas::resolve_persona(persona)?;
        let cot_instr = crate::modes::cot_instruction(cot, mode);
        let lang_instr = crate::modes::language_instruction(lang, mode);
        let system_text = format!(
            "[Language]\n{}\n\n{}{}\n\n[Persona]\n{}",
            lang_instr,
            mode_prompt(mode),
            cot_instr,
            persona_def.prompt
        );

        let user_text = compose_user_text(user_input, mode, diff_text, log_text);

        // Helper: build a request with the already-composed system prompt.
        let build_request = |hist: &[ChatMessage], user: &str| {
            let mut messages: Vec<ChatMessage> = Vec::with_capacity(1 + hist.len() + 1);
            messages.push(ChatMessage {
                role: "system".to_string(),
                content: system_text.clone(),
            });
            for m in hist {
                // Keep only chat history roles.
                if m.role == "user" || m.role == "assistant" {
                    messages.push(m.clone());
                }
            }
            messages.push(ChatMessage {
                role: "user".to_string(),
                content: user.to_string(),
            });
            ChatRequest {
                messages,
                temperature: Some(temperature),
                max_tokens: Some(max_tokens),
                metadata: None,
            }
        };

        let request1 = build_request(history, &user_text);
        let mut resp = self.provider.chat(&request1).await?;

        // Anti-loop (Observer): if the model repeats the same critique with no new signal,
        // retry once with an explicit diff-only instruction. If it still repeats, suppress it.
        if matches!(mode, Mode::Observer) {
            let prev_asst = history
                .iter()
                .rev()
                .find(|m| m.role == "assistant" && !m.content.trim().is_empty());
            if let Some(prev) = prev_asst {
                let prev_text = prev.content.trim();
                let cur_text = resp.content.trim();
                let sim = if !loop_detect::is_skippable_for_loop(cur_text) {
                    loop_detect::similarity(prev_text, cur_text)
                } else {
                    0.0
                };
                let loopish = prev_text.len() >= 120 && cur_text.len() >= 180 && sim >= 0.82;
                if loopish {
                    let loop_fix = if lang.unwrap_or("").eq_ignore_ascii_case("fr") {
                        "CORRECTION BOUCLE: Ton dernier message se répète. Fais une critique NOUVELLE basée UNIQUEMENT sur les informations NOUVELLES depuis le message précédent. Ne répète pas les mêmes proposals. S'il n'y a rien de nouveau, réponds exactement: [Observer] No new critique. Loop detected."
                    } else if lang.unwrap_or("").eq_ignore_ascii_case("en") {
                        "LOOP FIX: Your last message repeated the same critique. Write a NEW critique ONLY based on NEW information since your previous message. Do not restate the same proposals. If there is no new signal, reply exactly: [Observer] No new critique. Loop detected."
                    } else {
                        "LOOP FIX: 直前の批評と内容がほぼ同一です。前回から増えた情報に基づく「新しい」批評だけを書いてください。同じ提案の焼き直しは禁止。新しい指摘が無い場合は、次の1行だけを厳密に出力: [Observer] No new critique. Loop detected."
                    };

                    // Retry with an extended history that includes the repeated response.
                    let mut hist2: Vec<ChatMessage> = Vec::with_capacity(history.len() + 2);
                    for m in history {
                        if m.role == "user" || m.role == "assistant" {
                            hist2.push(m.clone());
                        }
                    }
                    hist2.push(ChatMessage {
                        role: "user".to_string(),
                        content: user_text.clone(),
                    });
                    hist2.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: resp.content.clone(),
                    });

                    let mut request2 = build_request(&hist2, loop_fix);
                    // Make the loop-break retry more deterministic.
                    request2.temperature = Some(0.2);
                    if let Ok(resp2) = self.provider.chat(&request2).await {
                        let cur2 = resp2.content.trim();
                        let sim2 = if !loop_detect::is_skippable_for_loop(cur2) {
                            loop_detect::similarity(prev_text, cur2)
                        } else {
                            0.0
                        };
                        let loopish2 = prev_text.len() >= 120 && cur2.len() >= 180 && sim2 >= 0.82;
                        if loopish2 {
                            resp.content = "[Observer] No new critique. Loop detected.".to_string();
                        } else {
                            resp = resp2;
                        }
                    } else {
                        // If retry fails, prefer not spamming the same template repeatedly.
                        resp.content = "[Observer] No new critique. Loop detected.".to_string();
                    }
                }
            }
        }

        // Language enforcement (Observer): if the model ignores the requested language (ja/fr),
        // retry once with an explicit rewrite instruction. This complements the UI-side retry and
        // makes non-stream calls more reliable.
        if matches!(mode, Mode::Observer) {
            let expected = if lang.unwrap_or("").eq_ignore_ascii_case("fr") {
                "fr"
            } else if lang.unwrap_or("").eq_ignore_ascii_case("en") {
                "en"
            } else {
                "ja"
            };

            if expected != "en" && lang_detect::needs_language_rewrite(expected, &resp.content) {
                // IMPORTANT: Do NOT reuse the full system prompt (persona/mode) for the rewrite.
                // Some personas strongly bias language/tone; use a minimal translator system prompt.
                let system_fix = if expected == "fr" {
                    "You are a strict translator.\n\
Rewrite the provided text into French ONLY.\n\
Do not add new content.\n\
Output ONLY the rewritten text.\n\
Keep proposals block keys in English (title/to_coder/severity/score/phase/impact/cost)."
                } else {
                    "You are a strict translator.\n\
Rewrite the provided text into Japanese ONLY.\n\
Do not add new content.\n\
Output ONLY the rewritten text.\n\
Keep proposals block keys in English (title/to_coder/severity/score/phase/impact/cost)."
                };

                let user_fix = format!(
                    "TEXT:\n```text\n{}\n```",
                    resp.content.trim_end()
                );

                let request2 = ChatRequest {
                    messages: vec![
                        ChatMessage {
                            role: "system".to_string(),
                            content: system_fix.to_string(),
                        },
                        ChatMessage {
                            role: "user".to_string(),
                            content: user_fix,
                        },
                    ],
                    temperature: Some(0.0),
                    max_tokens: Some(max_tokens),
                    metadata: None,
                };

                if let Ok(resp2) = self.provider.chat(&request2).await {
                    if !resp2.content.trim().is_empty()
                        && !lang_detect::needs_language_rewrite(expected, &resp2.content)
                    {
                        resp = resp2;
                    }
                }
            }
        }

        Ok(resp)
    }
}
