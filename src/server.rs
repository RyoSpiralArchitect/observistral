use anyhow::{Context, Result, anyhow};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::process::Command;

use crate::chatbot::ChatBot;
use crate::config::{PartialConfig, ProviderKind};
use crate::modes::Mode;
use crate::personas;
use crate::providers;
use crate::types::ChatMessage;

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
extern "system" {
    fn MultiByteToWideChar(
        CodePage: u32,
        dwFlags: u32,
        lpMultiByteStr: *const i8,
        cbMultiByte: i32,
        lpWideCharStr: *mut u16,
        cchWideChar: i32,
    ) -> i32;
}

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
        ("POST", "/api/models") => api_models(&mut stream, state, &req.body).await,
        ("POST", "/api/chat") => api_chat(&mut stream, state, &req.body).await,
        ("POST", "/api/chat_stream") => api_chat_stream(&mut stream, state, &req.body).await,
        ("POST", "/api/exec") => api_exec(&mut stream, &req.body).await,
        ("POST", "/api/chat_tools") => api_chat_tools(&mut stream, state, &req.body).await,
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

async fn api_chat_tools(stream: &mut TcpStream, state: AppState, body: &[u8]) -> Result<()> {
    use serde_json::json;

    #[derive(Deserialize)]
    struct Req {
        messages: Vec<serde_json::Value>,
        tools: Option<Vec<serde_json::Value>>,
        model: String,
        base_url: String,
        api_key: Option<String>,
        temperature: Option<f64>,
        max_tokens: Option<u32>,
        timeout_seconds: Option<u64>,
    }

    let req: Req = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(e) => {
            #[derive(Serialize)]
            struct E { error: String }
            return write_json(stream, 400, "Bad Request", &E { error: e.to_string() }).await;
        }
    };

    let url = format!("{}/chat/completions", req.base_url.trim_end_matches('/'));

    let mut payload = json!({
        "model": req.model,
        "messages": req.messages,
        "temperature": req.temperature.unwrap_or(0.7),
        "max_tokens": req.max_tokens.unwrap_or(4096),
    });

    if let Some(tools) = &req.tools {
        if !tools.is_empty() {
            payload["tools"] = json!(tools);
        }
    }

    let timeout = Duration::from_secs(req.timeout_seconds.unwrap_or(120));
    let mut http_req = state.client
        .post(&url)
        .header("Content-Type", "application/json")
        .timeout(timeout)
        .json(&payload);

    if let Some(key) = req.api_key.as_deref().filter(|k| !k.is_empty()) {
        http_req = http_req.bearer_auth(key);
    }

    let resp = match http_req.send().await {
        Ok(r) => r,
        Err(e) => {
            #[derive(Serialize)]
            struct E { error: String }
            return write_json(stream, 502, "Bad Gateway", &E { error: e.to_string() }).await;
        }
    };

    let status = resp.status();
    let resp_text = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        #[derive(Serialize)]
        struct E { error: String }
        return write_json(stream, 502, "Bad Gateway",
            &E { error: format!("API error (HTTP {status}): {resp_text}") }).await;
    }

    let v: serde_json::Value = match serde_json::from_str(&resp_text) {
        Ok(v) => v,
        Err(e) => {
            #[derive(Serialize)]
            struct E { error: String }
            return write_json(stream, 502, "Bad Gateway",
                &E { error: format!("invalid JSON from API: {e}") }).await;
        }
    };

    write_json(stream, 200, "OK", &v).await
}

async fn api_exec(stream: &mut TcpStream, body: &[u8]) -> Result<()> {
    #[derive(Deserialize)]
    struct Req { command: String, cwd: Option<String> }
    #[derive(Serialize)]
    struct Res { stdout: String, stderr: String, exit_code: i32 }

    fn decode_output(bytes: &[u8]) -> String {
        if bytes.is_empty() {
            return String::new();
        }
        // On Windows, stdout/stderr are often not UTF-8 (e.g., CP932). Decode them
        // so the UI doesn't show mojibake. Fall back to UTF-8 on other platforms.
        #[cfg(target_os = "windows")]
        {
            if let Ok(s) = std::str::from_utf8(bytes) {
                return s.to_string();
            }
            // CP932 (Windows-31J / Shift-JIS) via Win32 API.
            const CP_932: u32 = 932;
            const MB_ERR_INVALID_CHARS: u32 = 0x0000_0008;

            unsafe {
                let src = bytes.as_ptr() as *const i8;
                let src_len = if bytes.len() > i32::MAX as usize {
                    i32::MAX
                } else {
                    bytes.len() as i32
                };

                let needed = MultiByteToWideChar(CP_932, MB_ERR_INVALID_CHARS, src, src_len, std::ptr::null_mut(), 0);
                if needed <= 0 {
                    // Fallback: allow invalid bytes.
                    let needed2 = MultiByteToWideChar(CP_932, 0, src, src_len, std::ptr::null_mut(), 0);
                    if needed2 <= 0 {
                        return String::from_utf8_lossy(bytes).into_owned();
                    }
                    let mut buf = vec![0u16; needed2 as usize];
                    let written = MultiByteToWideChar(CP_932, 0, src, src_len, buf.as_mut_ptr(), needed2);
                    if written <= 0 {
                        return String::from_utf8_lossy(bytes).into_owned();
                    }
                    buf.truncate(written as usize);
                    return String::from_utf16_lossy(&buf);
                }

                let mut buf = vec![0u16; needed as usize];
                let written = MultiByteToWideChar(CP_932, MB_ERR_INVALID_CHARS, src, src_len, buf.as_mut_ptr(), needed);
                if written <= 0 {
                    return String::from_utf8_lossy(bytes).into_owned();
                }
                buf.truncate(written as usize);
                return String::from_utf16_lossy(&buf);
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            String::from_utf8_lossy(bytes).into_owned()
        }
    }

    let req: Req = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(_) => {
            #[derive(Serialize)]
            struct E { error: String }
            return write_json(stream, 400, "Bad Request",
                &E { error: "invalid JSON".into() }).await;
        }
    };

    let cmd_str = req.command.trim();
    if cmd_str.is_empty() {
        #[derive(Serialize)]
        struct E { error: String }
        return write_json(stream, 400, "Bad Request",
            &E { error: "command is empty".into() }).await;
    }

    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("powershell");
        // Prepend UTF-8 encoding setup to avoid garbled Japanese output.
        let wrapped = format!(
            "[Console]::OutputEncoding=[System.Text.Encoding]::UTF8; \
             [Console]::InputEncoding=[System.Text.Encoding]::UTF8; \
             $OutputEncoding=[System.Text.Encoding]::UTF8; {}",
            cmd_str
        );
        c.args(["-NoProfile", "-NonInteractive", "-Command", &wrapped]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", cmd_str]);
        c
    };
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(cwd) = req.cwd.as_deref().filter(|s| !s.trim().is_empty()) {
        cmd.current_dir(cwd);
    }

    let output = match cmd.output().await {
        Ok(o) => o,
        Err(err) => {
            #[derive(Serialize)]
            struct E { error: String }
            return write_json(stream, 500, "Internal Server Error",
                &E { error: format!("spawn failed: {err}") }).await;
        }
    };

    write_json(stream, 200, "OK", &Res {
        stdout: decode_output(&output.stdout),
        stderr: decode_output(&output.stderr),
        exit_code: output.status.code().unwrap_or(-1),
    }).await
}

async fn api_status(stream: &mut TcpStream) -> Result<()> {
    fn env_present(key: &str) -> bool {
        std::env::var(key)
            .ok()
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
    }

    let workspace_root = std::env::current_dir()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let resp = ApiStatusResponse {
        ok: true,
        version: env!("CARGO_PKG_VERSION"),
        providers: ApiStatusProviders {
            mistral: ApiProviderStatus {
                api_key_present: env_present("MISTRAL_API_KEY") || env_present("OBS_API_KEY"),
            },
            anthropic: ApiProviderStatus {
                api_key_present: env_present("ANTHROPIC_API_KEY"),
            },
            openai_compatible: ApiProviderStatus {
                api_key_present: env_present("OBS_API_KEY") || env_present("OPENAI_API_KEY"),
            },
        },
        features: ApiFeatures { exec: true, chat_tools: true },
        workspace_root,
    };

    write_json(stream, 200, "OK", &resp).await
}

async fn api_models(stream: &mut TcpStream, state: AppState, body: &[u8]) -> Result<()> {
    #[derive(Deserialize)]
    struct ApiModelsRequest {
        provider: Option<ProviderKind>,
        api_key: Option<String>,
        base_url: Option<String>,
    }

    #[derive(Serialize)]
    struct ApiModelsResponse {
        models: Vec<String>,
    }

    let req: ApiModelsRequest =
        serde_json::from_slice(body).context("invalid JSON body (expected ApiModelsRequest)")?;

    let mut partial = state.defaults;
    if let Some(provider) = req.provider {
        partial.provider = Some(provider);
    }
    if let Some(api_key) = req.api_key {
        partial.api_key = Some(api_key);
    }
    if let Some(base_url) = req.base_url {
        partial.base_url = Some(base_url);
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

    let models_res: Result<Vec<String>> = match cfg.provider {
        ProviderKind::OpenAiCompatible | ProviderKind::Mistral => {
            let url = format!("{}/models", cfg.base_url);
            let mut req = state
                .client
                .get(url)
                .header("Accept", "application/json")
                .timeout(Duration::from_secs(cfg.timeout_seconds));
            if let Some(key) = &cfg.api_key {
                req = req.bearer_auth(key);
            }
            let resp = req.send().await.context("request failed")?;
            let status = resp.status();
            if !status.is_success() {
                let body = resp.text().await.unwrap_or_default();
                return Err(anyhow!("models API error (HTTP {status})\n{body}"));
            }

            let v: serde_json::Value = resp.json().await.context("invalid JSON response")?;
            let mut out: Vec<String> = Vec::new();
            if let Some(arr) = v.get("data").and_then(|x| x.as_array()) {
                for item in arr {
                    if let Some(id) = item.get("id").and_then(|x| x.as_str()) {
                        out.push(id.to_string());
                    }
                }
            } else if let Some(arr) = v.as_array() {
                for item in arr {
                    if let Some(id) = item.as_str() {
                        out.push(id.to_string());
                    }
                }
            }
            Ok(out)
        }
        ProviderKind::Anthropic => {
            let api_key = cfg
                .api_key
                .as_ref()
                .ok_or_else(|| anyhow!("missing API key for anthropic"))?;
            let url = format!("{}/models", cfg.base_url);
            let resp = state
                .client
                .get(url)
                .header("x-api-key", api_key)
                .header("anthropic-version", crate::providers::anthropic::ANTHROPIC_VERSION)
                .header("Accept", "application/json")
                .timeout(Duration::from_secs(cfg.timeout_seconds))
                .send()
                .await
                .context("request failed")?;
            let status = resp.status();
            if !status.is_success() {
                let body = resp.text().await.unwrap_or_default();
                return Err(anyhow!("Anthropic models API error (HTTP {status})\n{body}"));
            }

            let v: serde_json::Value = resp.json().await.context("invalid JSON response")?;
            let mut out: Vec<String> = Vec::new();
            if let Some(arr) = v.get("data").and_then(|x| x.as_array()) {
                for item in arr {
                    if let Some(id) = item.get("id").and_then(|x| x.as_str()) {
                        out.push(id.to_string());
                    }
                }
            }
            Ok(out)
        }
        ProviderKind::Hf => Ok(Vec::new()),
    };

    match models_res {
        Ok(mut models) => {
            models.sort();
            models.dedup();
            write_json(stream, 200, "OK", &ApiModelsResponse { models }).await
        }
        Err(err) => write_json(
            stream,
            502,
            "Bad Gateway",
            &ApiError {
                error: err.to_string(),
            },
        )
        .await,
    }
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

    let cot_str = req.cot.as_deref().unwrap_or("brief");
    let resp = match bot
        .run(
            &req.input,
            &history,
            &cfg.mode,
            &cfg.persona,
            cot_str,
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

    let (cfg, history, diff, cot) = match build_chat_request(state.defaults.clone(), req) {
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
    let cot_str = cot.as_deref().unwrap_or("brief");
    let cot_instr = crate::modes::cot_instruction(cot_str, &cfg.mode);
    let system_text = format!(
        "{}{}\n\n[Persona]\n{}",
        crate::modes::mode_prompt(&cfg.mode),
        cot_instr,
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
            stream_hf_subprocess(stream, &cfg, &system_text, &history.messages, &user_text).await
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
    cot: Option<String>,
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
    features: ApiFeatures,
    workspace_root: String,
}

#[derive(Serialize)]
struct ApiFeatures {
    exec: bool,
    chat_tools: bool,
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
) -> Result<(crate::config::RunConfig, BuiltHistory, Option<String>, Option<String>)> {
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
        req.cot,
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
        let provider_label = match cfg.provider {
            ProviderKind::Mistral => "Mistral",
            _ => "OpenAI-compatible",
        };
        return Err(anyhow!(
            "{provider_label} API error (HTTP {status})\n{body}"
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

    let api_key = cfg
        .api_key
        .as_ref()
        .ok_or_else(|| anyhow!("missing API key for anthropic"))?;

    let url = format!("{}/messages", cfg.base_url);

    let mut messages: Vec<serde_json::Value> = Vec::with_capacity(history.len() + 1);
    for m in history {
        if m.role != "user" && m.role != "assistant" {
            continue;
        }
        messages.push(json!({"role": m.role, "content": m.content}));
    }
    messages.push(json!({"role":"user","content":user_text}));

    let payload = json!({
        "model": cfg.model,
        "messages": messages,
        "system": system_text,
        "temperature": cfg.temperature,
        "max_tokens": cfg.max_tokens,
        "stream": true,
    });

    let resp = client
        .post(url)
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
                if line.is_empty() {
                    continue;
                }
                if line.starts_with(':') {
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
            if data.trim().is_empty() {
                continue;
            }
            if data.trim() == "[DONE]" {
                return Ok(());
            }

            let v: serde_json::Value = match serde_json::from_str(&data) {
                Ok(v) => v,
                Err(_) => continue,
            };

            // Prefer the JSON "type" field; fall back to the SSE "event" field.
            let ty = v
                .get("type")
                .and_then(|x| x.as_str())
                .or(event)
                .unwrap_or("");

            if ty == "error" {
                return Err(anyhow!("Anthropic stream error: {v}"));
            }

            if ty == "content_block_delta" {
                let delta = v
                    .pointer("/delta/text")
                    .and_then(|x| x.as_str())
                    .unwrap_or("");
                if !delta.is_empty() {
                    let data = serde_json::to_string(&Delta { delta })?;
                    write_sse_event(stream, "delta", &data).await?;
                }
            }

            if ty == "message_stop" {
                return Ok(());
            }
        }
    }

    Ok(())
}

async fn stream_hf_subprocess(
    stream: &mut TcpStream,
    cfg: &crate::config::RunConfig,
    system_text: &str,
    history: &[ChatMessage],
    user_text: &str,
) -> Result<()> {
    use serde_json::json;

    let python = std::env::var("OBS_HF_PYTHON").unwrap_or_else(|_| "python".to_string());
    let script_path = std::path::PathBuf::from("scripts").join("hf_infer.py");

    let mut messages: Vec<serde_json::Value> = Vec::with_capacity(1 + history.len() + 1);
    messages.push(json!({"role":"system","content":system_text}));
    for m in history {
        messages.push(json!({"role": m.role, "content": m.content}));
    }
    messages.push(json!({"role":"user","content":user_text}));

    let payload = json!({
        "model": cfg.model,
        "messages": messages,
        "max_new_tokens": cfg.max_tokens,
        "temperature": cfg.temperature,
        "device": cfg.hf_device,
        "local_only": cfg.hf_local_only,
        "stream": true,
    });
    let input = serde_json::to_vec(&payload).context("failed to serialize hf request")?;

    let mut child = Command::new(&python)
        .arg(&script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn hf subprocess: {python} {script_path:?}"))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("failed to open stdin for hf subprocess"))?;
    stdin
        .write_all(&input)
        .await
        .context("failed to write request to hf subprocess")?;
    stdin
        .write_all(b"\n")
        .await
        .context("failed to write newline to hf subprocess")?;
    drop(stdin);

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("failed to open stdout for hf subprocess"))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("failed to open stderr for hf subprocess"))?;

    let stderr_task = tokio::spawn(async move {
        let mut buf: Vec<u8> = Vec::new();
        let _ = stderr.read_to_end(&mut buf).await;
        buf
    });

    let timeout = Duration::from_secs(cfg.timeout_seconds);
    let mut buf = [0u8; 8192];

    let status = match tokio::time::timeout(timeout, async {
        loop {
            let n = stdout.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            stream.write_all(&buf[..n]).await?;
            stream.flush().await?;
        }
        child.wait().await
    })
    .await
    {
        Ok(res) => res.context("hf subprocess wait failed")?,
        Err(_) => {
            // Best-effort: terminate runaway subprocess on timeout.
            let _ = child.kill().await;
            return Err(anyhow!("hf subprocess timed out after {}s", cfg.timeout_seconds));
        }
    };

    let stderr_bytes = stderr_task.await.unwrap_or_default();
    if !status.success() {
        let stderr_text = String::from_utf8_lossy(&stderr_bytes);
        let msg = stderr_text.trim();
        if msg.is_empty() {
            return Err(anyhow!("hf subprocess failed (exit code {status})"));
        }
        return Err(anyhow!("hf subprocess failed: {msg}"));
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
