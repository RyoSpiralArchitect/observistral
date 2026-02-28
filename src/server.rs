use anyhow::{Context, Result, anyhow};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::chatbot::ChatBot;
use crate::config::{PartialConfig, ProviderKind};
use crate::modes::Mode;
use crate::personas;
use crate::providers;
use crate::types::ChatMessage;

const INDEX_HTML: &str = include_str!("../web/index.html");
const APP_JS: &str = include_str!("../web/app.js");
const STYLES_CSS: &str = include_str!("../web/styles.css");
const REACT_JS: &str = include_str!("../web/vendor/react.production.min.js");
const REACT_DOM_JS: &str = include_str!("../web/vendor/react-dom.production.min.js");

#[derive(Parser, Clone, Debug)]
pub struct ServeArgs {
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    #[arg(long, default_value_t = 8080)]
    pub port: u16,
}

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
    defaults: PartialConfig,
}

pub async fn run(args: ServeArgs, defaults: PartialConfig) -> Result<()> {
    let addr: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .context("invalid --host/--port")?;

    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;

    let state = AppState {
        client: reqwest::Client::new(),
        defaults,
    };

    println!("OBSTRAL UI: http://{addr}/");

    loop {
        let (stream, _peer) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(err) = handle_connection(stream, state).await {
                eprintln!("serve: {err}");
            }
        });
    }
}

#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    body: Vec<u8>,
}

async fn handle_connection(mut stream: TcpStream, state: AppState) -> Result<()> {
    let req = read_http_request(&mut stream).await?;

    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/") => {
            write_response(
                &mut stream,
                200,
                "OK",
                "text/html; charset=utf-8",
                INDEX_HTML.as_bytes(),
            )
            .await
        }
        ("GET", "/assets/app.js") => {
            write_response(
                &mut stream,
                200,
                "OK",
                "text/javascript; charset=utf-8",
                APP_JS.as_bytes(),
            )
            .await
        }
        ("GET", "/assets/styles.css") => {
            write_response(
                &mut stream,
                200,
                "OK",
                "text/css; charset=utf-8",
                STYLES_CSS.as_bytes(),
            )
            .await
        }
        ("GET", "/assets/vendor/react.production.min.js") => {
            write_response(
                &mut stream,
                200,
                "OK",
                "text/javascript; charset=utf-8",
                REACT_JS.as_bytes(),
            )
            .await
        }
        ("GET", "/assets/vendor/react-dom.production.min.js") => {
            write_response(
                &mut stream,
                200,
                "OK",
                "text/javascript; charset=utf-8",
                REACT_DOM_JS.as_bytes(),
            )
            .await
        }
        ("GET", "/api/status") => api_status(&mut stream).await,
        ("POST", "/api/chat") => api_chat(&mut stream, state, &req.body).await,
        ("POST", "/api/chat_stream") => api_chat_stream(&mut stream, state, &req.body).await,
        _ => {
            write_text(
                &mut stream,
                404,
                "Not Found",
                "text/plain; charset=utf-8",
                "not found\n",
            )
            .await
        }
    }
}

async fn api_status(stream: &mut TcpStream) -> Result<()> {
    fn env_present(key: &str) -> bool {
        std::env::var(key)
            .ok()
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
    }

    let resp = ApiStatusResponse {
        ok: true,
        version: env!("CARGO_PKG_VERSION"),
        providers: ApiStatusProviders {
            mistral: ApiProviderStatus {
                api_key_present: env_present("MISTRAL_API_KEY"),
            },
            anthropic: ApiProviderStatus {
                api_key_present: env_present("ANTHROPIC_API_KEY"),
            },
            openai_compatible: ApiProviderStatus {
                api_key_present: env_present("OBS_API_KEY") || env_present("OPENAI_API_KEY"),
            },
        },
    };

    write_json(stream, 200, "OK", &resp).await
}

async fn api_chat(stream: &mut TcpStream, state: AppState, body: &[u8]) -> Result<()> {
    let req: ApiChatRequest =
        serde_json::from_slice(body).context("invalid JSON body (expected ApiChatRequest)")?;

    let mut partial = state.defaults.clone();

    if req.vibe.unwrap_or(false) {
        partial.vibe = true;
    }

    if let Some(provider) = req.provider.clone() {
        // If provider changes, avoid accidentally reusing model/base_url/api_key from defaults.
        partial.provider = Some(provider);
        if req.model.is_none() {
            partial.model = None;
        }
        if req.chat_model.is_none() {
            partial.chat_model = None;
        }
        if req.code_model.is_none() {
            partial.code_model = None;
        }
        if req.base_url.is_none() {
            partial.base_url = None;
        }
        if req.api_key.is_none() {
            partial.api_key = None;
        }
    }

    if let Some(model) = req.model {
        partial.model = Some(model);
    }
    if let Some(chat_model) = req.chat_model {
        partial.chat_model = Some(chat_model);
    }
    if let Some(code_model) = req.code_model {
        partial.code_model = Some(code_model);
    }
    if let Some(api_key) = req.api_key {
        partial.api_key = Some(api_key);
    }
    if let Some(base_url) = req.base_url {
        partial.base_url = Some(base_url);
    }
    if let Some(mode) = req.mode {
        partial.mode = Some(mode);
    }
    if let Some(persona) = req.persona {
        partial.persona = Some(persona);
    }
    if let Some(temperature) = req.temperature {
        partial.temperature = Some(temperature);
    }
    if let Some(max_tokens) = req.max_tokens {
        partial.max_tokens = Some(max_tokens);
    }
    if let Some(timeout_seconds) = req.timeout_seconds {
        partial.timeout_seconds = Some(timeout_seconds);
    }

    let cfg = match partial.resolve() {
        Ok(cfg) => cfg,
        Err(err) => {
            return write_json(
                stream,
                400,
                "Bad Request",
                &ApiError {
                    error: err.to_string(),
                },
            )
            .await;
        }
    };

    let history = req
        .history
        .unwrap_or_default()
        .into_iter()
        .filter_map(|m| {
            let role = m.role.trim().to_string();
            if role != "user" && role != "assistant" {
                return None;
            }
            Some(ChatMessage {
                role,
                content: m.content,
            })
        })
        .collect::<Vec<_>>();

    let provider = providers::build_provider(state.client.clone(), &cfg);
    let bot = ChatBot::new(provider);

    let resp = match bot
        .run(
            &req.input,
            &history,
            &cfg.mode,
            &cfg.persona,
            cfg.temperature,
            cfg.max_tokens,
            req.diff.as_deref(),
            None,
        )
        .await
    {
        Ok(resp) => resp,
        Err(err) => {
            let msg = anyhow!(err).to_string();
            return write_json(stream, 502, "Bad Gateway", &ApiError { error: msg }).await;
        }
    };

    write_json(
        stream,
        200,
        "OK",
        &ApiChatResponse {
            content: resp.content,
            model: resp.model,
        },
    )
    .await
}

async fn api_chat_stream(stream: &mut TcpStream, state: AppState, body: &[u8]) -> Result<()> {
    let req: ApiChatRequest =
        serde_json::from_slice(body).context("invalid JSON body (expected ApiChatRequest)")?;

    let (cfg, history, diff) = match build_chat_request(state.defaults.clone(), req) {
        Ok(v) => v,
        Err(err) => {
            // Even for stream endpoint, return JSON error with 400 for easier debugging.
            return write_json(
                stream,
                400,
                "Bad Request",
                &ApiError {
                    error: err.to_string(),
                },
            )
            .await;
        }
    };

    write_sse_header(stream, 200, "OK").await?;

    let persona_def = personas::resolve_persona(&cfg.persona)?;
    let system_text = format!(
        "{}\n\n[Persona]\n{}",
        crate::modes::mode_prompt(&cfg.mode),
        persona_def.prompt
    );
    let user_text =
        crate::modes::compose_user_text(&history.user_input, &cfg.mode, diff.as_deref(), None);

    // Stream the response as SSE deltas.
    let result = match cfg.provider {
        ProviderKind::OpenAiCompatible | ProviderKind::Mistral => {
            stream_openai_compat(
                stream,
                &state.client,
                &cfg,
                &system_text,
                &history.messages,
                &user_text,
            )
            .await
        }
        ProviderKind::Anthropic => {
            stream_anthropic(
                stream,
                &state.client,
                &cfg,
                &system_text,
                &history.messages,
                &user_text,
            )
            .await
        }
        ProviderKind::Hf => {
            // Fallback: non-streaming request, then emit deltas in chunks so the UI can render progressively.
            let provider = providers::build_provider(state.client.clone(), &cfg);
            let bot = ChatBot::new(provider);
            match bot
                .run(
                    &history.user_input,
                    &history.messages,
                    &cfg.mode,
                    &cfg.persona,
                    cfg.temperature,
                    cfg.max_tokens,
                    diff.as_deref(),
                    None,
                )
                .await
            {
                Ok(resp) => {
                    let text = resp.content;
                    stream_text_in_chunks(stream, &text, 96, Duration::from_millis(6)).await
                }
                Err(err) => Err(err),
            }
        }
    };

    if let Err(err) = result {
        let msg = anyhow!(err).to_string();
        let data = serde_json::to_string(&ApiError { error: msg })?;
        write_sse_event(stream, "error", &data).await?;
    }

    write_sse_event(stream, "done", "{}").await?;
    Ok(())
}

#[derive(Deserialize)]
struct ApiChatRequest {
    input: String,

    vibe: Option<bool>,
    provider: Option<ProviderKind>,
    model: Option<String>,
    chat_model: Option<String>,
    code_model: Option<String>,
    api_key: Option<String>,
    base_url: Option<String>,
    mode: Option<Mode>,
    persona: Option<String>,
    temperature: Option<f64>,
    max_tokens: Option<u32>,
    timeout_seconds: Option<u64>,

    history: Option<Vec<ApiMessage>>,
    diff: Option<String>,
}

#[derive(Deserialize)]
struct ApiMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ApiChatResponse {
    content: String,
    model: String,
}

#[derive(Serialize)]
struct ApiStatusResponse {
    ok: bool,
    version: &'static str,
    providers: ApiStatusProviders,
}

#[derive(Serialize)]
struct ApiStatusProviders {
    mistral: ApiProviderStatus,
    anthropic: ApiProviderStatus,
    #[serde(rename = "openai-compatible")]
    openai_compatible: ApiProviderStatus,
}

#[derive(Serialize)]
struct ApiProviderStatus {
    api_key_present: bool,
}

#[derive(Serialize)]
struct ApiError {
    error: String,
}

#[derive(Clone)]
struct BuiltHistory {
    user_input: String,
    messages: Vec<ChatMessage>,
}

fn build_chat_request(
    defaults: PartialConfig,
    req: ApiChatRequest,
) -> Result<(crate::config::RunConfig, BuiltHistory, Option<String>)> {
    let mut partial = defaults;

    if req.vibe.unwrap_or(false) {
        partial.vibe = true;
    }

    if let Some(provider) = req.provider.clone() {
        partial.provider = Some(provider);
        if req.model.is_none() {
            partial.model = None;
        }
        if req.chat_model.is_none() {
            partial.chat_model = None;
        }
        if req.code_model.is_none() {
            partial.code_model = None;
        }
        if req.base_url.is_none() {
            partial.base_url = None;
        }
        if req.api_key.is_none() {
            partial.api_key = None;
        }
    }

    if let Some(model) = req.model {
        partial.model = Some(model);
    }
    if let Some(chat_model) = req.chat_model {
        partial.chat_model = Some(chat_model);
    }
    if let Some(code_model) = req.code_model {
        partial.code_model = Some(code_model);
    }
    if let Some(api_key) = req.api_key {
        partial.api_key = Some(api_key);
    }
    if let Some(base_url) = req.base_url {
        partial.base_url = Some(base_url);
    }
    if let Some(mode) = req.mode {
        partial.mode = Some(mode);
    }
    if let Some(persona) = req.persona {
        partial.persona = Some(persona);
    }
    if let Some(temperature) = req.temperature {
        partial.temperature = Some(temperature);
    }
    if let Some(max_tokens) = req.max_tokens {
        partial.max_tokens = Some(max_tokens);
    }
    if let Some(timeout_seconds) = req.timeout_seconds {
        partial.timeout_seconds = Some(timeout_seconds);
    }

    let cfg = partial.resolve()?;

    let history = req
        .history
        .unwrap_or_default()
        .into_iter()
        .filter_map(|m| {
            let role = m.role.trim().to_string();
            if role != "user" && role != "assistant" {
                return None;
            }
            Some(ChatMessage {
                role,
                content: m.content,
            })
        })
        .collect::<Vec<_>>();

    Ok((
        cfg,
        BuiltHistory {
            user_input: req.input,
            messages: history,
        },
        req.diff,
    ))
}

async fn write_sse_header(stream: &mut TcpStream, code: u16, reason: &str) -> Result<()> {
    let header = format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Type: text/event-stream; charset=utf-8\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(header.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

async fn write_sse_event(stream: &mut TcpStream, event: &str, data: &str) -> Result<()> {
    // Single-line JSON payload is recommended for easy client parsing.
    let msg = format!("event: {event}\ndata: {data}\n\n");
    stream.write_all(msg.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

async fn stream_text_in_chunks(
    stream: &mut TcpStream,
    text: &str,
    chunk_chars: usize,
    delay: Duration,
) -> Result<()> {
    #[derive(Serialize)]
    struct Delta<'a> {
        delta: &'a str,
    }

    let mut start = 0;
    let chars: Vec<char> = text.chars().collect();
    while start < chars.len() {
        let end = (start + chunk_chars).min(chars.len());
        let chunk: String = chars[start..end].iter().collect();
        let data = serde_json::to_string(&Delta { delta: &chunk })?;
        write_sse_event(stream, "delta", &data).await?;
        start = end;
        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }
    }
    Ok(())
}

async fn stream_openai_compat(
    stream: &mut TcpStream,
    client: &reqwest::Client,
    cfg: &crate::config::RunConfig,
    system_text: &str,
    history: &[ChatMessage],
    user_text: &str,
) -> Result<()> {
    use serde_json::json;

    #[derive(Serialize)]
    struct Delta<'a> {
        delta: &'a str,
    }

    let url = format!("{}/chat/completions", cfg.base_url);

    let mut messages: Vec<serde_json::Value> = Vec::with_capacity(1 + history.len() + 1);
    messages.push(json!({"role":"system","content":system_text}));
    for m in history {
        messages.push(json!({"role": m.role, "content": m.content}));
    }
    messages.push(json!({"role":"user","content":user_text}));

    let payload = json!({
        "model": cfg.model,
        "messages": messages,
        "temperature": cfg.temperature,
        "max_tokens": cfg.max_tokens,
        "stream": true,
    });

    let mut req = client
        .post(url)
        .header("Content-Type", "application/json")
        .timeout(Duration::from_secs(cfg.timeout_seconds))
        .json(&payload);
    if let Some(key) = &cfg.api_key {
        req = req.bearer_auth(key);
    }

    let mut resp = req.send().await.context("request failed")?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!(
            "OpenAI-compatible API error (HTTP {status})\n{body}"
        ));
    }

    let mut buf: Vec<u8> = Vec::new();
    while let Some(chunk) = resp.chunk().await? {
        buf.extend_from_slice(&chunk);

        while let Some(frame) = take_next_sse_frame(&mut buf) {
            // Parse "data:" lines.
            let frame_str = String::from_utf8_lossy(&frame);
            let mut data_lines: Vec<&str> = Vec::new();
            for line in frame_str.split('\n') {
                let line = line.trim_end_matches('\r');
                if line.is_empty() {
                    continue;
                }
                if line.starts_with(':') {
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
                return Ok(());
            }

            let v: serde_json::Value = match serde_json::from_str(&data) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if let Some(err) = v.get("error") {
                return Err(anyhow!("provider error: {err}"));
            }

            let delta = v
                .pointer("/choices/0/delta/content")
                .and_then(|x| x.as_str())
                .or_else(|| v.pointer("/choices/0/delta/text").and_then(|x| x.as_str()))
                .unwrap_or("");

            if !delta.is_empty() {
                let data = serde_json::to_string(&Delta { delta })?;
                write_sse_event(stream, "delta", &data).await?;
            }
        }
    }

    Ok(())
}

async fn stream_anthropic(
    stream: &mut TcpStream,
    client: &reqwest::Client,
    cfg: &crate::config::RunConfig,
    system_text: &str,
    history: &[ChatMessage],
    user_text: &str,
) -> Result<()> {
    use serde_json::json;

    #[derive(Serialize)]
    struct Delta<'a> {
        delta: &'a str,
    }

    let url = format!("{}/messages", cfg.base_url);

    let api_key = cfg
        .api_key
        .as_ref()
        .ok_or_else(|| anyhow!("missing API key for Anthropic"))?;

    let mut messages: Vec<serde_json::Value> = Vec::with_capacity(history.len() + 1);
    for m in history {
        messages.push(json!({"role": m.role, "content": m.content}));
    }
    messages.push(json!({"role": "user", "content": user_text}));

    let payload = json!({
        "model": cfg.model,
        "system": system_text,
        "messages": messages,
        "temperature": cfg.temperature,
        "max_tokens": cfg.max_tokens,
        "stream": true,
    });

    let mut resp = client
        .post(url)
        .header("x-api-key", api_key)
        .header("anthropic-version", crate::providers::anthropic::ANTHROPIC_VERSION)
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

    let mut buf: Vec<u8> = Vec::new();
    while let Some(chunk) = resp.chunk().await? {
        buf.extend_from_slice(&chunk);

        while let Some(frame) = take_next_sse_frame(&mut buf) {
            let frame_str = String::from_utf8_lossy(&frame);
            let mut event_type = "";
            let mut data_str = "";

            for line in frame_str.split('\n') {
                let line = line.trim_end_matches('\r');
                if let Some(rest) = line.strip_prefix("event:") {
                    event_type = rest.trim();
                } else if let Some(rest) = line.strip_prefix("data:") {
                    data_str = rest.trim();
                }
            }

            if event_type == "message_stop" || data_str == "[DONE]" {
                return Ok(());
            }

            if event_type != "content_block_delta" {
                continue;
            }

            let v: serde_json::Value = match serde_json::from_str(data_str) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let text = v
                .pointer("/delta/text")
                .and_then(|x| x.as_str())
                .unwrap_or("");

            if !text.is_empty() {
                let data = serde_json::to_string(&Delta { delta: text })?;
                write_sse_event(stream, "delta", &data).await?;
            }
        }
    }

    Ok(())
}

fn take_next_sse_frame(buf: &mut Vec<u8>) -> Option<Vec<u8>> {
    // SSE frames are separated by a blank line. Accept both LF and CRLF forms.
    let pos_lf = find_subslice(buf, b"\n\n");
    let pos_crlf = find_subslice(buf, b"\r\n\r\n");
    let pos = match (pos_lf, pos_crlf) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }?;

    let sep_len = if buf[pos..].starts_with(b"\r\n\r\n") {
        4
    } else {
        2
    };

    let frame = buf[..pos].to_vec();
    buf.drain(..pos + sep_len);
    Some(frame)
}

async fn write_json<T: Serialize>(
    stream: &mut TcpStream,
    code: u16,
    reason: &str,
    value: &T,
) -> Result<()> {
    let body = serde_json::to_vec(value).context("failed to serialize JSON")?;
    write_response(
        stream,
        code,
        reason,
        "application/json; charset=utf-8",
        &body,
    )
    .await
}

async fn write_text(
    stream: &mut TcpStream,
    code: u16,
    reason: &str,
    content_type: &str,
    text: &str,
) -> Result<()> {
    write_response(stream, code, reason, content_type, text.as_bytes()).await
}

async fn write_response(
    stream: &mut TcpStream,
    code: u16,
    reason: &str,
    content_type: &str,
    body: &[u8],
) -> Result<()> {
    let header = format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes()).await?;
    stream.write_all(body).await?;
    Ok(())
}

async fn read_http_request(stream: &mut TcpStream) -> Result<HttpRequest> {
    const MAX_HEADER_BYTES: usize = 64 * 1024;
    const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;

    let mut buf: Vec<u8> = Vec::with_capacity(4096);

    let header_end = loop {
        if let Some(pos) = find_subslice(&buf, b"\r\n\r\n") {
            break pos;
        }
        if buf.len() >= MAX_HEADER_BYTES {
            return Err(anyhow!("request headers too large"));
        }
        let mut tmp = [0u8; 2048];
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            return Err(anyhow!("unexpected EOF while reading request"));
        }
        buf.extend_from_slice(&tmp[..n]);
    };

    let (head, rest) = buf.split_at(header_end + 4);
    let head_str = String::from_utf8_lossy(head);
    let mut lines = head_str.split("\r\n").filter(|l| !l.is_empty());

    let start = lines
        .next()
        .ok_or_else(|| anyhow!("missing request line"))?;
    let mut start_parts = start.split_whitespace();
    let method = start_parts
        .next()
        .ok_or_else(|| anyhow!("invalid request line"))?
        .to_string();
    let path = start_parts
        .next()
        .ok_or_else(|| anyhow!("invalid request line"))?
        .to_string();

    let mut content_length: usize = 0;
    for line in lines {
        let Some((k, v)) = line.split_once(':') else {
            continue;
        };
        if k.trim().eq_ignore_ascii_case("content-length") {
            content_length = v.trim().parse::<usize>().unwrap_or(0);
            break;
        }
    }

    if content_length > MAX_BODY_BYTES {
        return Err(anyhow!("request body too large"));
    }

    let mut body: Vec<u8> = Vec::with_capacity(content_length.min(4096));
    body.extend_from_slice(rest);

    while body.len() < content_length {
        let mut tmp = vec![0u8; (content_length - body.len()).min(8192)];
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            return Err(anyhow!("unexpected EOF while reading body"));
        }
        body.extend_from_slice(&tmp[..n]);
    }

    if body.len() > content_length {
        body.truncate(content_length);
    }

    Ok(HttpRequest { method, path, body })
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}
