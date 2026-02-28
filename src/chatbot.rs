use anyhow::Result;
use std::sync::Arc;

use crate::modes::{Mode, compose_user_text, mode_prompt};
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
        temperature: f64,
        max_tokens: u32,
        diff_text: Option<&str>,
        log_text: Option<&str>,
    ) -> Result<ChatResponse> {
        let persona_def = personas::resolve_persona(persona)?;
        let system_text = format!("{}\n\n[Persona]\n{}", mode_prompt(mode), persona_def.prompt);

        let user_text = compose_user_text(user_input, mode, diff_text, log_text);

        let mut messages: Vec<ChatMessage> = Vec::with_capacity(1 + history.len() + 1);
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: system_text,
        });
        for m in history {
            // Keep only chat history roles.
            if m.role == "user" || m.role == "assistant" {
                messages.push(m.clone());
            }
        }
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: user_text,
        });

        let request = ChatRequest {
            messages,
            temperature: Some(temperature),
            max_tokens: Some(max_tokens),
            metadata: None,
        };

        self.provider.chat(&request).await
    }
}

