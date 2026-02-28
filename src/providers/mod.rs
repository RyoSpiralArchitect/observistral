pub mod anthropic;
pub mod hf_subprocess;
pub mod openai_compat;

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

use crate::config::{ProviderKind, RunConfig};
use crate::types::{ChatRequest, ChatResponse};

#[async_trait]
pub trait ChatProvider: Send + Sync {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse>;
}

pub fn build_provider(client: reqwest::Client, cfg: &RunConfig) -> Arc<dyn ChatProvider> {
    let timeout = Duration::from_secs(cfg.timeout_seconds);
    match cfg.provider {
        ProviderKind::Anthropic => Arc::new(anthropic::AnthropicProvider::new(
            client,
            cfg.model.clone(),
            cfg.api_key.clone(),
            cfg.base_url.clone(),
            timeout,
        )),
        ProviderKind::Mistral => Arc::new(openai_compat::OpenAICompatibleProvider::new(
            client,
            "Mistral",
            cfg.model.clone(),
            cfg.api_key.clone(),
            cfg.base_url.clone(),
            timeout,
        )),
        ProviderKind::OpenAiCompatible => Arc::new(openai_compat::OpenAICompatibleProvider::new(
            client,
            "OpenAI-compatible",
            cfg.model.clone(),
            cfg.api_key.clone(),
            cfg.base_url.clone(),
            timeout,
        )),
        ProviderKind::Hf => Arc::new(hf_subprocess::HuggingFaceSubprocessProvider::new(
            cfg.model.clone(),
            cfg.hf_device.clone(),
            cfg.hf_local_only,
            timeout,
        )),
    }
}

