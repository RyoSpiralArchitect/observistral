use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use reqwest::StatusCode;
use serde_json::{Value, json};
use std::time::Duration;

use crate::types::{ChatMessage, ChatRequest, ChatResponse};

use super::ChatProvider;

pub const ANTHROPIC_VERSION: &str = "2023-06-01";

pub struct AnthropicProvider {
    client: reqwest::Client,
    model: String,
    api_key: Option<String>,
    base_url: String,
    timeout: Duration,
}

impl AnthropicProvider {
    pub fn new(
        client: reqwest::Client,
        model: String,
        api_key: Option<String>,
        base_url: String,
        timeout: Duration,
    ) -> Self {
        Self {
            client,
            model,
            api_key,
            base_url: base_url.trim_end_matches('/').to_string(),
            timeout,
        }
    }
}

fn http_error(status: StatusCode, body: &str) -> anyhow::Error {
    let hint = match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => "authentication failed (check API key)",
        StatusCode::TOO_MANY_REQUESTS => "rate limited (try again later)",
        s if s.is_server_error() => "server error",
        _ => "request failed",
    };
    if body.trim().is_empty() {
        anyhow!("Anthropic API error (HTTP {status}): {hint}")
    } else {
        anyhow!("Anthropic API error (HTTP {status}): {hint}\n{body}")
    }
}

fn extract_system(messages: &[ChatMessage]) -> Option<String> {
    let systems: Vec<&str> = messages
        .iter()
        .filter(|m| m.role == "system")
        .map(|m| m.content.as_str())
        .collect();
    if systems.is_empty() {
        None
    } else {
        Some(systems.join("\n"))
    }
}

fn to_anthropic_messages(messages: &[ChatMessage]) -> Vec<Value> {
    messages
        .iter()
        .filter(|m| m.role == "user" || m.role == "assistant")
        .map(|m| json!({ "role": m.role, "content": m.content }))
        .collect()
}

#[async_trait]
impl ChatProvider for AnthropicProvider {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let api_key = self.api_key.as_ref().ok_or_else(|| anyhow!("missing API key"))?;
        let url = format!("{}/messages", self.base_url);

        let mut payload = json!({
            "model": self.model,
            "messages": to_anthropic_messages(&request.messages),
            "temperature": request.temperature.unwrap_or(0.4),
            "max_tokens": request.max_tokens.unwrap_or(1024),
        });

        if let Some(system_text) = extract_system(&request.messages) {
            payload["system"] = json!(system_text);
        }
        if let Some(Value::Object(extra)) = &request.metadata {
            if let Value::Object(obj) = &mut payload {
                for (k, v) in extra {
                    obj.insert(k.clone(), v.clone());
                }
            }
        }

        let resp = self
            .client
            .post(url)
            .header("x-api-key", api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("Content-Type", "application/json")
            .timeout(self.timeout)
            .json(&payload)
            .send()
            .await
            .context("request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(http_error(status, &body));
        }

        let data: Value = resp.json().await.context("invalid JSON response")?;
        let blocks = data.get("content").and_then(|x| x.as_array()).cloned().unwrap_or_default();
        let mut text = String::new();
        for b in blocks {
            if b.get("type").and_then(|x| x.as_str()) == Some("text") {
                if let Some(t) = b.get("text").and_then(|x| x.as_str()) {
                    text.push_str(t);
                }
            }
        }

        Ok(ChatResponse {
            content: text,
            model: self.model.clone(),
            raw: Some(data),
        })
    }
}
