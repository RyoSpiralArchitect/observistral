use anyhow::{Context, Result, anyhow};
use clap::ValueEnum;
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::modes::Mode;
use crate::personas;

#[derive(Clone, Debug, PartialEq, ValueEnum, Serialize, Deserialize)]
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

    #[serde(rename = "hf")]
    #[value(name = "hf", alias = "huggingface")]
    Hf,
}

impl ProviderKind {
    pub fn key(&self) -> &'static str {
        match self {
            ProviderKind::OpenAiCompatible => "openai-compatible",
            ProviderKind::Mistral => "mistral",
            ProviderKind::Anthropic => "anthropic",
            ProviderKind::Hf => "hf",
        }
    }
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.key())
    }
}

pub fn supported_providers() -> Vec<&'static str> {
    vec!["openai-compatible", "mistral", "anthropic", "hf"]
}

pub fn normalize_provider(s: &str) -> String {
    s.trim().to_ascii_lowercase()
}

fn parse_provider(s: &str) -> Option<ProviderKind> {
    match normalize_provider(s).as_str() {
        "openai-compatible" | "openai" | "openai_compat" => Some(ProviderKind::OpenAiCompatible),
        "mistral" => Some(ProviderKind::Mistral),
        "anthropic" => Some(ProviderKind::Anthropic),
        "hf" | "huggingface" => Some(ProviderKind::Hf),
        _ => None,
    }
}

#[derive(Clone, Debug, Default)]
pub struct PartialConfig {
    pub vibe: bool,
    pub provider: Option<ProviderKind>,
    pub model: Option<String>,
    pub chat_model: Option<String>,
    pub code_model: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub mode: Option<Mode>,
    pub persona: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub timeout_seconds: Option<u64>,
    pub hf_device: Option<String>,
    pub hf_local_only: Option<bool>,
}

#[derive(Clone, Debug)]
pub struct RunConfig {
    pub provider: ProviderKind,
    pub model: String,
    pub chat_model: String,
    pub code_model: String,
    pub api_key: Option<String>,
    pub base_url: String,
    pub mode: Mode,
    pub persona: String,
    pub temperature: f64,
    pub max_tokens: u32,
    pub timeout_seconds: u64,
    pub hf_device: String,
    pub hf_local_only: bool,
}

impl PartialConfig {
    pub fn resolve(mut self) -> Result<RunConfig> {
        if self.provider.is_none() {
            self.provider = env_trimmed("OBS_PROVIDER").and_then(|v| parse_provider(&v));
        }
        if self.model.is_none() {
            self.model = env_trimmed("OBS_MODEL");
        }
        if self.chat_model.is_none() {
            self.chat_model = env_trimmed("OBS_CHAT_MODEL");
        }
        if self.code_model.is_none() {
            self.code_model = env_trimmed("OBS_CODE_MODEL");
        }
        if self.base_url.is_none() {
            self.base_url = env_trimmed("OBS_BASE_URL");
        }
        if self.timeout_seconds.is_none() {
            self.timeout_seconds = env_trimmed("OBS_TIMEOUT_SECONDS").and_then(|v| v.parse().ok());
        }
        if self.persona.is_none() {
            self.persona = env_trimmed("OBS_PERSONA");
        }
        if self.hf_device.is_none() {
            self.hf_device = env_trimmed("OBS_HF_DEVICE");
        }
        if self.hf_local_only.is_none() {
            self.hf_local_only = env_trimmed("OBS_HF_LOCAL_ONLY").and_then(|v| parse_bool(&v));
        }

        // Provider defaulting:
        // - `--vibe` is a preset that implies Mistral/Codestral unless explicitly overridden.
        // - Otherwise, prefer an "it just works" default for local dev: infer from base_url/env keys,
        //   falling back to OpenAI-compatible (so users with only OPENAI_API_KEY don't error out).
        let provider = if let Some(p) = self.provider.clone() {
            p
        } else if self.vibe {
            ProviderKind::Mistral
        } else if let Some(u) = self
            .base_url
            .as_deref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            // Infer from explicit base_url when present.
            let low = u.to_ascii_lowercase();
            if low.contains("anthropic") {
                ProviderKind::Anthropic
            } else if low.contains("mistral.ai") {
                ProviderKind::Mistral
            } else {
                ProviderKind::OpenAiCompatible
            }
        } else if env_trimmed("MISTRAL_API_KEY").is_some() {
            ProviderKind::Mistral
        } else if env_trimmed("ANTHROPIC_API_KEY").is_some() {
            ProviderKind::Anthropic
        } else if env_trimmed("OPENAI_API_KEY").is_some() || env_trimmed("OBS_API_KEY").is_some() {
            ProviderKind::OpenAiCompatible
        } else {
            ProviderKind::OpenAiCompatible
        };

        let mode = if self.vibe {
            self.mode.unwrap_or(Mode::Vibe)
        } else {
            self.mode.unwrap_or(Mode::Kabeuchi)
        };

        let persona_in = self.persona.unwrap_or_else(|| "default".to_string());
        let persona_def = personas::resolve_persona(&persona_in).context("invalid persona")?;
        let persona = persona_def.key.to_string();

        let base_model = match self.model {
            Some(m) if !m.trim().is_empty() => m,
            _ => default_model(&provider, self.vibe).to_string(),
        };

        let chat_model = match self.chat_model {
            Some(m) if !m.trim().is_empty() => m,
            _ => base_model.clone(),
        };

        let code_model = match self.code_model {
            Some(m) if !m.trim().is_empty() => m,
            _ => base_model.clone(),
        };

        let model = if mode.uses_code_model() {
            code_model.clone()
        } else {
            chat_model.clone()
        };

        let base_url = match self.base_url {
            Some(u) if !u.trim().is_empty() => u,
            _ => default_base_url(&provider).to_string(),
        };
        let base_url = base_url.trim().trim_end_matches('/').to_string();
        validate_base_url(&base_url).context("invalid --base-url")?;

        if provider == ProviderKind::OpenAiCompatible {
            if let Ok(url) = Url::parse(&base_url) {
                if url.host_str() == Some("api.openai.com") && model.contains("devstral") {
                    return Err(anyhow!(
                        "invalid model for OpenAI base URL: {model}. If you want Devstral, use --provider mistral (base_url=https://api.mistral.ai/v1)."
                    ));
                }
            }
        }

        let temperature = self.temperature.unwrap_or(0.4);
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

        let api_key = self
            .api_key
            .and_then(|k| {
                let k = k.trim().to_string();
                if k.is_empty() { None } else { Some(k) }
            })
            .or_else(|| resolve_api_key_from_env(&provider));

        match provider {
            ProviderKind::Mistral if api_key.is_none() => {
                return Err(anyhow!(
                    "missing API key for mistral. Set MISTRAL_API_KEY (or OBS_API_KEY), or pass --api-key."
                ));
            }
            ProviderKind::Anthropic if api_key.is_none() => {
                return Err(anyhow!(
                    "missing API key for anthropic. Set ANTHROPIC_API_KEY (or pass --api-key)."
                ));
            }
            _ => {}
        }

        let hf_device = self.hf_device.unwrap_or_else(|| "auto".to_string());
        let hf_local_only = self.hf_local_only.unwrap_or(false);

        Ok(RunConfig {
            provider,
            model,
            chat_model,
            code_model,
            api_key,
            base_url,
            mode,
            persona,
            temperature,
            max_tokens,
            timeout_seconds,
            hf_device,
            hf_local_only,
        })
    }
}

fn default_base_url(provider: &ProviderKind) -> &'static str {
    match provider {
        ProviderKind::OpenAiCompatible => "https://api.openai.com/v1",
        ProviderKind::Mistral => "https://api.mistral.ai/v1",
        ProviderKind::Anthropic => "https://api.anthropic.com/v1",
        ProviderKind::Hf => "http://localhost",
    }
}

fn default_model(provider: &ProviderKind, vibe: bool) -> &'static str {
    match provider {
        ProviderKind::OpenAiCompatible => "gpt-4o-mini",
        ProviderKind::Mistral => {
            if vibe {
                "codestral-latest"
            } else {
                "mistral-small-latest"
            }
        }
        ProviderKind::Anthropic => "claude-3-5-sonnet-latest",
        ProviderKind::Hf => "local",
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

fn env_trimmed(key: &str) -> Option<String> {
    std::env::var(key).ok().and_then(|v| {
        let v = v.trim().to_string();
        if v.is_empty() { None } else { Some(v) }
    })
}

fn parse_bool(s: &str) -> Option<bool> {
    match s.trim() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn resolve_api_key_from_env(provider: &ProviderKind) -> Option<String> {
    let get = |k: &str| env_trimmed(k);
    match provider {
        ProviderKind::OpenAiCompatible => get("OBS_API_KEY").or_else(|| get("OPENAI_API_KEY")),
        ProviderKind::Mistral => get("MISTRAL_API_KEY").or_else(|| get("OBS_API_KEY")),
        ProviderKind::Anthropic => get("ANTHROPIC_API_KEY"),
        ProviderKind::Hf => None,
    }
}
