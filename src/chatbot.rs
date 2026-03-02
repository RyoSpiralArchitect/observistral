use anyhow::Result;
use std::collections::HashSet;
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
            fn loop_threshold_for_len(max_len: usize) -> f64 {
                // Longer critiques tend to share more boilerplate (section headers, repeated terms).
                // Lower the threshold slightly to detect "same template again" for long outputs.
                if max_len >= 1200 {
                    0.75
                } else if max_len >= 800 {
                    0.78
                } else if max_len >= 500 {
                    0.80
                } else if max_len >= 300 {
                    0.82
                } else {
                    0.85
                }
            }

            fn collect_prev_asst_texts<'a>(history: &'a [ChatMessage], max_n: usize) -> Vec<&'a str> {
                let mut prev: Vec<&'a str> = Vec::new();
                for m in history.iter().rev() {
                    if m.role != "assistant" {
                        continue;
                    }
                    let t = m.content.trim();
                    if t.is_empty() {
                        continue;
                    }
                    if loop_detect::is_skippable_for_loop(t) {
                        continue;
                    }
                    prev.push(t);
                    if prev.len() >= max_n {
                        break;
                    }
                }
                prev
            }

            fn max_similarity(prev_texts: &[&str], cur_text: &str) -> (f64, usize) {
                let mut max_sim = 0.0f64;
                let mut max_prev_len = 0usize;
                for prev in prev_texts.iter().copied() {
                    let sim = loop_detect::similarity(prev, cur_text);
                    if sim > max_sim {
                        max_sim = sim;
                        max_prev_len = prev.len();
                    }
                }
                (max_sim, max_prev_len)
            }

            fn extract_proposal_titles(s: &str) -> Vec<String> {
                let mut titles: Vec<String> = Vec::new();
                let mut in_props = false;
                for line in s.lines() {
                    let t = line.trim();
                    if t == "--- proposals ---" {
                        in_props = true;
                        continue;
                    }
                    if t.starts_with("--- ") && t.ends_with(" ---") && t != "--- proposals ---" {
                        in_props = false;
                    }
                    if !in_props {
                        continue;
                    }

                    let low = t.to_ascii_lowercase();
                    if low.starts_with("title:") {
                        let v = t["title:".len()..].trim();
                        if !v.is_empty() {
                            titles.push(v.to_string());
                        }
                        continue;
                    }
                    // Accept "1) title: ..." as well.
                    if let Some(idx) = low.find("title:") {
                        if idx <= 6 {
                            let v = t[idx + "title:".len()..].trim();
                            if !v.is_empty() {
                                titles.push(v.to_string());
                            }
                        }
                    }
                }
                titles
            }

            fn loop_suppressed_message(history: &[ChatMessage], lang: Option<&str>) -> String {
                let mut titles: Vec<String> = Vec::new();
                let mut seen: HashSet<String> = HashSet::new();

                for m in history.iter().rev() {
                    if m.role != "assistant" {
                        continue;
                    }
                    for title in extract_proposal_titles(&m.content) {
                        let key = title.trim().to_ascii_lowercase();
                        if key.is_empty() {
                            continue;
                        }
                        if seen.insert(key) {
                            titles.push(title.trim().to_string());
                            if titles.len() >= 6 {
                                break;
                            }
                        }
                    }
                    if titles.len() >= 6 {
                        break;
                    }
                }

                if titles.is_empty() {
                    return "[Observer] No new critique. Loop detected.".to_string();
                }

                let header = if lang.unwrap_or("").eq_ignore_ascii_case("fr") {
                    "Points ouverts:"
                } else if lang.unwrap_or("").eq_ignore_ascii_case("en") {
                    "Open issues:"
                } else {
                    "未解決:"
                };

                let mut out = String::new();
                out.push_str("[Observer] No new critique. Loop detected.\n");
                out.push_str(header);
                out.push('\n');
                for t in titles.iter().take(5) {
                    out.push_str("- ");
                    out.push_str(t);
                    out.push('\n');
                }
                out.trim_end().to_string()
            }

            let prev_texts = collect_prev_asst_texts(history, 4);
            let cur_text = resp.content.trim();
            if !prev_texts.is_empty() && !loop_detect::is_skippable_for_loop(cur_text) {
                let (max_sim, max_prev_len) = max_similarity(&prev_texts, cur_text);
                let threshold = loop_threshold_for_len(max_prev_len.max(cur_text.len()));
                let loopish =
                    max_prev_len >= 120 && cur_text.len() >= 180 && max_sim >= threshold;
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
                        let (max_sim2, max_prev_len2) = if !loop_detect::is_skippable_for_loop(cur2)
                        {
                            max_similarity(&prev_texts, cur2)
                        } else {
                            (0.0, 0)
                        };
                        let threshold2 = loop_threshold_for_len(max_prev_len2.max(cur2.len()));
                        let loopish2 =
                            max_prev_len2 >= 120 && cur2.len() >= 180 && max_sim2 >= threshold2;
                        if loopish2 {
                            resp.content = loop_suppressed_message(history, lang);
                        } else {
                            resp = resp2;
                        }
                    } else {
                        // If retry fails, prefer not spamming the same template repeatedly.
                        resp.content = loop_suppressed_message(history, lang);
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
