use anyhow::{Context, Result, anyhow};
use clap::ValueEnum;
use reqwest::{Client, StatusCode, Url};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

use crate::prompts::{self, Mode, Persona};

#[derive(Clone, Debug, ValueEnum, Serialize, Deserialize)]
pub enum ProviderKind {
    #[serde(rename = "openai-compatible")]
    #[value(name = "openai-compatible", alias = "openai", alias = "openai_compat")]
    OpenAiCompatible,

    #[serde(rename = "mistral")]
    #[value(name = "mistral")]
    Mistral,

    #[serde(rename = "anthropic")]
    #[value(name = "anthropic")]
    Anthropic,
}

impl ProviderKind {
    pub fn key(&self) -> &'static str {
        match self {
            ProviderKind::OpenAiCompatible => "openai-compatible",
            ProviderKind::Mistral => "mistral",
            ProviderKind::Anthropic => "anthropic",
        }
    }
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.key())
    }
}

pub fn supported_providers() -> Vec<&'static str> {
    vec!["openai-compatible", "mistral", "anthropic"]
}

#[derive(Clone, Debug, Default)]
pub struct PartialConfig {
    pub vibe: bool,
    pub provider: Option<ProviderKind>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub mode: Option<Mode>,
    pub persona: Option<Persona>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct RunConfig {
    pub provider: ProviderKind,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: String,
    pub mode: Mode,
    pub persona: Persona,
    pub temperature: f64,
    pub max_tokens: u32,
    pub timeout_seconds: u64,
}

impl PartialConfig {
    pub fn resolve(self) -> Result<RunConfig> {
        let provider = if self.vibe {
            self.provider.unwrap_or(ProviderKind::Mistral)
        } else {
            self.provider.unwrap_or(ProviderKind::OpenAiCompatible)
        };

        let mode = if self.vibe {
            self.mode.unwrap_or(Mode::Vibe)
        } else {
            self.mode.unwrap_or(Mode::Kabeuchi)
        };

        let persona = self.persona.unwrap_or(Persona::Default);

        let model = match self.model {
            Some(m) if !m.trim().is_empty() => m,
            _ => default_model(&provider, self.vibe).to_string(),
        };

        let base_url = match self.base_url {
            Some(u) if !u.trim().is_empty() => u,
            _ => default_base_url(&provider).to_string(),
        };
        let base_url = base_url.trim().trim_end_matches('/').to_string();
        validate_base_url(&base_url).context("invalid --base-url")?;

        let temperature = self.temperature.unwrap_or(0.7);
        if !(0.0..=2.0).contains(&temperature) {
            return Err(anyhow!(
                "invalid temperature: {temperature} (expected 0.0..=2.0)"
            ));
        }

        let max_tokens = self.max_tokens.unwrap_or(1024);
        if max_tokens == 0 {
            return Err(anyhow!("invalid max_tokens: 0 (expected >= 1)"));
        }

        let timeout_seconds = self.timeout_seconds.unwrap_or(120);
        if timeout_seconds == 0 {
            return Err(anyhow!("invalid timeout_seconds: 0 (expected >= 1)"));
        }

        let api_key = match self.api_key.and_then(|k| {
            let k = k.trim().to_string();
            if k.is_empty() { None } else { Some(k) }
        }) {
            Some(k) => Some(k),
            None => resolve_api_key_from_env(&provider),
        };

        match provider {
            ProviderKind::Mistral if api_key.is_none() => {
                return Err(anyhow!(
                    "missing API key for mistral. Set MISTRAL_API_KEY (or pass --api-key)."
                ));
            }
            ProviderKind::Anthropic if api_key.is_none() => {
                return Err(anyhow!(
                    "missing API key for anthropic. Set ANTHROPIC_API_KEY (or pass --api-key)."
                ));
            }
            _ => {}
        }

        Ok(RunConfig {
            provider,
            model,
            api_key,
            base_url,
            mode,
            persona,
            temperature,
            max_tokens,
            timeout_seconds,
        })
    }
}

fn default_base_url(provider: &ProviderKind) -> &'static str {
    match provider {
        ProviderKind::OpenAiCompatible => "https://api.openai.com/v1",
        ProviderKind::Mistral => "https://api.mistral.ai/v1",
        ProviderKind::Anthropic => "https://api.anthropic.com/v1",
    }
}

fn default_model(provider: &ProviderKind, vibe: bool) -> &'static str {
    match provider {
        ProviderKind::OpenAiCompatible => "gpt-4o-mini",
        ProviderKind::Mistral => {
            if vibe {
                "devstral-2"
            } else {
                "devstral-2"
            }
        }
        ProviderKind::Anthropic => "claude-3-5-sonnet-latest",
    }
}

fn validate_base_url(base_url: &str) -> Result<()> {
    let url = Url::parse(base_url)?;
    match url.scheme() {
        "http" | "https" => {}
        other => {
            return Err(anyhow!(
                "unsupported URL scheme: {other} (expected http or https)"
            ));
        }
    }
    if url.host_str().is_none() {
        return Err(anyhow!("base_url missing host"));
    }
    Ok(())
}

fn resolve_api_key_from_env(provider: &ProviderKind) -> Option<String> {
    let get = |k: &str| {
        std::env::var(k).ok().and_then(|v| {
            let v = v.trim().to_string();
            if v.is_empty() { None } else { Some(v) }
        })
    };

    match provider {
        ProviderKind::OpenAiCompatible => get("OBS_API_KEY").or_else(|| get("OPENAI_API_KEY")),
        ProviderKind::Mistral => get("MISTRAL_API_KEY").or_else(|| get("OBS_API_KEY")),
        ProviderKind::Anthropic => get("ANTHROPIC_API_KEY"),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

pub async fn chat(
    client: &Client,
    cfg: &RunConfig,
    history: &[ChatMessage],
    user_input: &str,
    diff_text: Option<&str>,
) -> Result<String> {
    let system_text = prompts::build_system_prompt(&cfg.mode, &cfg.persona);
    let user_text = prompts::compose_user_text(user_input, &cfg.mode, diff_text);

    match cfg.provider {
        ProviderKind::Anthropic => {
            chat_anthropic(client, cfg, &system_text, history, &user_text).await
        }
        ProviderKind::OpenAiCompatible | ProviderKind::Mistral => {
            chat_openai_compat(client, cfg, &system_text, history, &user_text).await
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

async fn chat_openai_compat(
    client: &Client,
    cfg: &RunConfig,
    system_text: &str,
    history: &[ChatMessage],
    user_text: &str,
) -> Result<String> {
    #[derive(Deserialize)]
    struct Resp {
        choices: Vec<Choice>,
    }
    #[derive(Deserialize)]
    struct Choice {
        message: RespMessage,
    }
    #[derive(Deserialize)]
    struct RespMessage {
        content: String,
    }

    let url = format!("{}/chat/completions", cfg.base_url);
    let mut messages: Vec<ChatMessage> = Vec::with_capacity(1 + history.len() + 1);
    messages.push(ChatMessage {
        role: "system".to_string(),
        content: system_text.to_string(),
    });
    messages.extend(history.iter().cloned());
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_text.to_string(),
    });

    let payload = json!({
        "model": cfg.model,
        "messages": messages,
        "temperature": cfg.temperature,
        "max_tokens": cfg.max_tokens,
    });

    let mut req = client
        .post(url)
        .header("Content-Type", "application/json")
        .timeout(Duration::from_secs(cfg.timeout_seconds))
        .json(&payload);

    if let Some(key) = &cfg.api_key {
        req = req.bearer_auth(key);
    }

    let resp = req.send().await.context("request failed")?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(http_error("OpenAI-compatible", status, &body));
    }

    let data = resp.json::<Resp>().await.context("invalid JSON response")?;
    let content = data
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .unwrap_or_default();
    Ok(content)
}

async fn chat_anthropic(
    client: &Client,
    cfg: &RunConfig,
    system_text: &str,
    history: &[ChatMessage],
    user_text: &str,
) -> Result<String> {
    #[derive(Deserialize)]
    struct Resp {
        content: Vec<Block>,
    }
    #[derive(Deserialize)]
    struct Block {
        #[serde(rename = "type")]
        kind: String,
        text: Option<String>,
    }

    let Some(key) = &cfg.api_key else {
        return Err(anyhow!(
            "missing API key for anthropic. Set ANTHROPIC_API_KEY (or pass --api-key)."
        ));
    };

    let url = format!("{}/messages", cfg.base_url);

    // Anthropic: system is a top-level field; messages are user/assistant only.
    let mut messages: Vec<serde_json::Value> = Vec::with_capacity(history.len() + 1);
    for m in history {
        if m.role == "user" || m.role == "assistant" {
            messages.push(json!({"role": m.role, "content": m.content}));
        }
    }
    messages.push(json!({"role": "user", "content": user_text}));

    let payload = json!({
        "model": cfg.model,
        "system": system_text,
        "messages": messages,
        "temperature": cfg.temperature,
        "max_tokens": cfg.max_tokens,
    });

    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .timeout(Duration::from_secs(cfg.timeout_seconds))
        .json(&payload)
        .send()
        .await
        .context("request failed")?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(http_error("Anthropic", status, &body));
    }

    let data = resp.json::<Resp>().await.context("invalid JSON response")?;
    let text = data
        .content
        .into_iter()
        .filter(|b| b.kind == "text")
        .filter_map(|b| b.text)
        .collect::<String>();
    Ok(text)
}
