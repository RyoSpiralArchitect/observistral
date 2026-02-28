use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use reqwest::StatusCode;
use serde_json::{Value, json};
use std::time::Duration;

use crate::types::{ChatMessage, ChatRequest, ChatResponse};

use super::ChatProvider;

pub struct OpenAICompatibleProvider {
    client: reqwest::Client,
    kind_label: &'static str,
    model: String,
    api_key: Option<String>,
    base_url: String,
    timeout: Duration,
}

impl OpenAICompatibleProvider {
    pub fn new(
        client: reqwest::Client,
        kind_label: &'static str,
        model: String,
        api_key: Option<String>,
        base_url: String,
        timeout: Duration,
    ) -> Self {
        Self {
            client,
            kind_label,
            model,
            api_key,
            base_url: base_url.trim_end_matches('/').to_string(),
            timeout,
        }
    }
}

fn http_error(kind: &str, status: StatusCode, body: &str) -> anyhow::Error {
    let hint = match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => "authentication failed (check API key)",
        StatusCode::TOO_MANY_REQUESTS => "rate limited (try again later)",
        s if s.is_server_error() => "server error",
        _ => "request failed",
    };
    if body.trim().is_empty() {
        anyhow!("{kind} API error (HTTP {status}): {hint}")
    } else {
        anyhow!("{kind} API error (HTTP {status}): {hint}\n{body}")
    }
}

fn to_openai_messages(messages: &[ChatMessage]) -> Vec<Value> {
    messages
        .iter()
        .map(|m| json!({ "role": m.role, "content": m.content }))
        .collect()
}

#[async_trait]
impl ChatProvider for OpenAICompatibleProvider {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        let mut payload = json!({
            "model": self.model,
            "messages": to_openai_messages(&request.messages),
            "temperature": request.temperature.unwrap_or(0.4),
        });
        if let Some(max_tokens) = request.max_tokens {
            payload["max_tokens"] = json!(max_tokens);
        }
        if let Some(Value::Object(extra)) = &request.metadata {
            if let Value::Object(obj) = &mut payload {
                for (k, v) in extra {
                    obj.insert(k.clone(), v.clone());
                }
            }
        }

        let mut req = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .timeout(self.timeout)
            .json(&payload);
        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }

        let resp = req.send().await.context("request failed")?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(http_error(self.kind_label, status, &body));
        }

        let data: Value = resp.json().await.context("invalid JSON response")?;
        let content = data
            .pointer("/choices/0/message/content")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();

        Ok(ChatResponse {
            content,
            model: self.model.clone(),
            raw: Some(data),
        })
    }
}

