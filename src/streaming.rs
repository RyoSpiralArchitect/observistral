use anyhow::{Context, Result, anyhow};
use serde_json::json;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::config::{ProviderKind, RunConfig};
use crate::types::ChatMessage;

#[derive(Debug, Clone)]
pub enum StreamToken {
    Delta(String),
    ToolCall(ToolCallData),
    Done,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ToolCallData {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// Simple chat: converts ChatMessage slice to JSON and delegates.
pub async fn stream_openai_compat(
    client: &reqwest::Client,
    cfg: &RunConfig,
    messages: &[ChatMessage],
    tools: Option<&serde_json::Value>,
    tx: mpsc::Sender<StreamToken>,
) -> Result<()> {
    let msgs_json: Vec<serde_json::Value> = messages
        .iter()
        .map(|m| json!({"role": m.role, "content": m.content}))
        .collect();
    stream_openai_compat_json(client, cfg, &msgs_json, tools, tx).await
}

/// Agent use: accepts pre-built JSON messages so tool_call_id / tool_calls
/// fields are preserved intact across iterations.
pub async fn stream_openai_compat_json(
    client: &reqwest::Client,
    cfg: &RunConfig,
    messages: &[serde_json::Value],
    tools: Option<&serde_json::Value>,
    tx: mpsc::Sender<StreamToken>,
) -> Result<()> {
    let url = format!("{}/chat/completions", cfg.base_url);

    let mut payload = json!({
        "model": cfg.model,
        "messages": messages,
        "temperature": cfg.temperature,
        "max_tokens": cfg.max_tokens,
        "stream": true,
    });

    if let Some(t) = tools {
        payload["tools"] = t.clone();
        payload["tool_choice"] = json!("auto");
    }

    let mut req = client
        .post(&url)
        .header("Content-Type", "application/json")
        .timeout(Duration::from_secs(cfg.timeout_seconds))
        .json(&payload);
    if let Some(key) = &cfg.api_key {
        req = req.bearer_auth(key);
    }

    let mut resp = req.send().await.with_context(|| {
        format!(
            "failed to connect to {url}\n\
             If behind a proxy, set: $env:HTTPS_PROXY=\"http://host:port\""
        )
    })?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        let label = match cfg.provider {
            ProviderKind::Mistral => "Mistral",
            _ => "OpenAI-compatible",
        };
        return Err(anyhow!("{label} API error (HTTP {status})\n{body}"));
    }

    let mut tc_id = String::new();
    let mut tc_name = String::new();
    let mut tc_args = String::new();
    let mut in_tool_call = false;

    let mut buf: Vec<u8> = Vec::new();
    while let Some(chunk) = resp.chunk().await? {
        buf.extend_from_slice(&chunk);
        while let Some(frame) = take_next_sse_frame(&mut buf) {
            let frame_str = String::from_utf8_lossy(&frame);
            let mut data_lines: Vec<&str> = Vec::new();
            for line in frame_str.split('\n') {
                let line = line.trim_end_matches('\r');
                if line.is_empty() || line.starts_with(':') {
                    continue;
                }
                if let Some(rest) = line.strip_prefix("data:") {
                    data_lines.push(rest.trim_start());
                }
            }
            if data_lines.is_empty() {
                continue;
            }
            let data = data_lines.join("\n");
            if data.trim() == "[DONE]" {
                if in_tool_call && !tc_name.is_empty() {
                    let _ = tx.send(StreamToken::ToolCall(ToolCallData {
                        id: tc_id.clone(),
                        name: tc_name.clone(),
                        arguments: tc_args.clone(),
                    })).await;
                }
                let _ = tx.send(StreamToken::Done).await;
                return Ok(());
            }

            let v: serde_json::Value = match serde_json::from_str(&data) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if let Some(err) = v.get("error") {
                let _ = tx.send(StreamToken::Error(err.to_string())).await;
                return Err(anyhow!("provider error: {err}"));
            }

            let finish_reason = v
                .pointer("/choices/0/finish_reason")
                .and_then(|x| x.as_str())
                .unwrap_or("");

            let delta_text = v
                .pointer("/choices/0/delta/content")
                .and_then(|x| x.as_str())
                .or_else(|| v.pointer("/choices/0/delta/text").and_then(|x| x.as_str()))
                .unwrap_or("");
            if !delta_text.is_empty() {
                let _ = tx.send(StreamToken::Delta(delta_text.to_string())).await;
            }

            if let Some(calls) = v.pointer("/choices/0/delta/tool_calls") {
                if let Some(arr) = calls.as_array() {
                    for call in arr {
                        if let Some(id) = call.get("id").and_then(|x| x.as_str()) {
                            if !id.is_empty() {
                                tc_id = id.to_string();
                                in_tool_call = true;
                            }
                        }
                        if let Some(fn_name) = call.pointer("/function/name").and_then(|x| x.as_str()) {
                            if !fn_name.is_empty() {
                                tc_name = fn_name.to_string();
                            }
                        }
                        if let Some(args_chunk) = call.pointer("/function/arguments").and_then(|x| x.as_str()) {
                            tc_args.push_str(args_chunk);
                        }
                    }
                }
            }

            if finish_reason == "tool_calls" && in_tool_call && !tc_name.is_empty() {
                let _ = tx.send(StreamToken::ToolCall(ToolCallData {
                    id: tc_id.clone(),
                    name: tc_name.clone(),
                    arguments: tc_args.clone(),
                })).await;
                tc_id.clear();
                tc_name.clear();
                tc_args.clear();
                in_tool_call = false;
            }

            if finish_reason == "stop" {
                let _ = tx.send(StreamToken::Done).await;
                return Ok(());
            }
        }
    }

    let _ = tx.send(StreamToken::Done).await;
    Ok(())
}

/// Send streaming tokens from the Anthropic provider.
pub async fn stream_anthropic(
    client: &reqwest::Client,
    cfg: &RunConfig,
    messages: &[ChatMessage],
    tx: mpsc::Sender<StreamToken>,
) -> Result<()> {
    let api_key = cfg
        .api_key
        .as_ref()
        .ok_or_else(|| anyhow!("missing API key for anthropic"))?;

    let url = format!("{}/messages", cfg.base_url);

    let system_msg = messages
        .iter()
        .find(|m| m.role == "system")
        .map(|m| m.content.as_str())
        .unwrap_or("");

    let chat_msgs: Vec<serde_json::Value> = messages
        .iter()
        .filter(|m| m.role == "user" || m.role == "assistant")
        .map(|m| json!({"role": m.role, "content": m.content}))
        .collect();

    let payload = json!({
        "model": cfg.model,
        "messages": chat_msgs,
        "system": system_msg,
        "temperature": cfg.temperature,
        "max_tokens": cfg.max_tokens,
        "stream": true,
    });

    let resp = client
        .post(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", crate::providers::anthropic::ANTHROPIC_VERSION)
        .header("Accept", "text/event-stream")
        .header("Content-Type", "application/json")
        .timeout(Duration::from_secs(cfg.timeout_seconds))
        .json(&payload)
        .send()
        .await
        .context("request failed")?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("Anthropic API error (HTTP {status})\n{body}"));
    }

    let mut resp = resp;
    let mut buf: Vec<u8> = Vec::new();
    while let Some(chunk) = resp.chunk().await? {
        buf.extend_from_slice(&chunk);
        while let Some(frame) = take_next_sse_frame(&mut buf) {
            let frame_str = String::from_utf8_lossy(&frame);
            let mut event: Option<&str> = None;
            let mut data_lines: Vec<&str> = Vec::new();
            for line in frame_str.split('\n') {
                let line = line.trim_end_matches('\r');
                if line.is_empty() || line.starts_with(':') {
                    continue;
                }
                if let Some(rest) = line.strip_prefix("event:") {
                    event = Some(rest.trim());
                }
                if let Some(rest) = line.strip_prefix("data:") {
                    data_lines.push(rest.trim_start());
                }
            }
            if data_lines.is_empty() {
                continue;
            }
            let data = data_lines.join("\n");
            if data.trim().is_empty() || data.trim() == "[DONE]" {
                continue;
            }

            let v: serde_json::Value = match serde_json::from_str(&data) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let ty = v.get("type").and_then(|x| x.as_str()).or(event).unwrap_or("");

            if ty == "error" {
                let msg = v.to_string();
                let _ = tx.send(StreamToken::Error(msg.clone())).await;
                return Err(anyhow!("Anthropic stream error: {msg}"));
            }

            if ty == "content_block_delta" {
                let delta = v.pointer("/delta/text").and_then(|x| x.as_str()).unwrap_or("");
                if !delta.is_empty() {
                    let _ = tx.send(StreamToken::Delta(delta.to_string())).await;
                }
            }

            if ty == "message_stop" {
                let _ = tx.send(StreamToken::Done).await;
                return Ok(());
            }
        }
    }

    let _ = tx.send(StreamToken::Done).await;
    Ok(())
}

fn take_next_sse_frame(buf: &mut Vec<u8>) -> Option<Vec<u8>> {
    let pos_lf = find_subslice(buf, b"\n\n");
    let pos_crlf = find_subslice(buf, b"\r\n\r\n");
    let pos = match (pos_lf, pos_crlf) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }?;
    let sep_len = if buf[pos..].starts_with(b"\r\n\r\n") { 4 } else { 2 };
    let frame = buf[..pos].to_vec();
    buf.drain(..pos + sep_len);
    Some(frame)
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}
