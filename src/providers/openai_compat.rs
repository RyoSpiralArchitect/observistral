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

    async fn post_json(&self, url: &str, payload: &Value) -> Result<Result<Value, (StatusCode, String)>> {
        let mut req = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .timeout(self.timeout)
            .json(payload);
        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }

        let resp = req.send().await.context("request failed")?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Ok(Err((status, body)));
        }
        let data: Value = serde_json::from_str(&body).context("invalid JSON response")?;
        Ok(Ok(data))
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

        let data: Value = match self.post_json(&url, &payload).await? {
            Ok(v) => v,
            Err((status, body)) => {
                // Some OpenAI models reject `max_tokens` and require `max_completion_tokens` instead.
                if status == StatusCode::BAD_REQUEST
                    && body.contains("max_completion_tokens")
                    && body.contains("max_tokens")
                    && request.max_tokens.is_some()
                {
                    let mut payload2 = payload.clone();
                    if let Some(mt) = request.max_tokens {
                        if let Value::Object(obj) = &mut payload2 {
                            obj.remove("max_tokens");
                            obj.insert("max_completion_tokens".to_string(), json!(mt));
                        }
                    }
                    match self.post_json(&url, &payload2).await? {
                        Ok(v2) => v2,
                        Err((status2, body2)) => return Err(http_error(self.kind_label, status2, &body2)),
                    }
                } else {
                    return Err(http_error(self.kind_label, status, &body));
                }
            }
        };
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

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    fn make_request(msg: &str) -> ChatRequest {
        ChatRequest {
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: msg.to_string(),
            }],
            temperature: Some(0.4),
            max_tokens: Some(64),
            metadata: None,
        }
    }

    #[tokio::test]
    async fn returns_content_on_success() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST).path("/chat/completions");
            then.status(200).json_body(serde_json::json!({
                "choices": [{"message": {"role": "assistant", "content": "hello"}}]
            }));
        });

        let client = reqwest::Client::new();
        let provider = OpenAICompatibleProvider::new(
            client,
            "test",
            "gpt-4o-mini".to_string(),
            None,
            server.base_url(),
            Duration::from_secs(5),
        );

        let resp = provider.chat(&make_request("hi")).await.unwrap();
        assert_eq!(resp.content, "hello");
        assert_eq!(resp.model, "gpt-4o-mini");
        mock.assert();
    }

    #[tokio::test]
    async fn returns_error_on_401() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(POST).path("/chat/completions");
            then.status(401).body("Unauthorized");
        });

        let client = reqwest::Client::new();
        let provider = OpenAICompatibleProvider::new(
            client,
            "test",
            "gpt-4o-mini".to_string(),
            Some("bad-key".to_string()),
            server.base_url(),
            Duration::from_secs(5),
        );

        let err = provider.chat(&make_request("hi")).await.unwrap_err();
        assert!(err.to_string().contains("authentication failed"), "{err}");
    }

    #[tokio::test]
    async fn sends_bearer_auth_header() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/chat/completions")
                .header("Authorization", "Bearer my-key");
            then.status(200).json_body(serde_json::json!({
                "choices": [{"message": {"role": "assistant", "content": "ok"}}]
            }));
        });

        let client = reqwest::Client::new();
        let provider = OpenAICompatibleProvider::new(
            client,
            "test",
            "model".to_string(),
            Some("my-key".to_string()),
            server.base_url(),
            Duration::from_secs(5),
        );

        provider.chat(&make_request("hi")).await.unwrap();
        mock.assert();
    }
}
