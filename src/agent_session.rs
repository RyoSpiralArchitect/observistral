use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LastReflectionSummary {
    pub last_outcome: Option<String>,
    pub goal_delta: Option<String>,
    pub wrong_assumption: Option<String>,
    pub strategy_change: Option<String>,
    pub next_minimal_action: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct ObservationReadCache {
    pub command: String,
    pub path: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct ObservationSearchCache {
    pub command: String,
    pub pattern: String,
    pub hit_count: usize,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct ObservationCache {
    pub reads: Vec<ObservationReadCache>,
    pub searches: Vec<ObservationSearchCache>,
}

fn extract_tag_block<'a>(text: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = text.find(&open)?;
    let rest = &text[start + open.len()..];
    let end = rest.find(&close)?;
    Some(rest[..end].trim())
}

fn parse_reflection_summary(text: &str) -> Option<LastReflectionSummary> {
    let body = extract_tag_block(text, "reflect")?;
    let mut s = LastReflectionSummary::default();

    for line in body.lines() {
        let (k, v) = match line.split_once(':') {
            Some((k, v)) => (k.trim().to_ascii_lowercase(), v.trim().to_string()),
            None => continue,
        };
        if v.is_empty() {
            continue;
        }
        match k.as_str() {
            "last_outcome" => s.last_outcome = Some(v),
            "goal_delta" => s.goal_delta = Some(v.to_ascii_lowercase()),
            "wrong_assumption" => s.wrong_assumption = Some(v),
            "strategy_change" => s.strategy_change = Some(v.to_ascii_lowercase()),
            "next_minimal_action" => s.next_minimal_action = Some(v),
            _ => {}
        }
    }

    if s.last_outcome.is_none()
        && s.goal_delta.is_none()
        && s.wrong_assumption.is_none()
        && s.strategy_change.is_none()
        && s.next_minimal_action.is_none()
    {
        None
    } else {
        Some(s)
    }
}

pub fn last_reflection_summary_from_messages(
    messages: &[serde_json::Value],
) -> Option<LastReflectionSummary> {
    for msg in messages.iter().rev() {
        if msg.get("role").and_then(|r| r.as_str()) != Some("assistant") {
            continue;
        }
        let content = msg.get("content").and_then(|c| c.as_str()).unwrap_or("");
        if let Some(s) = parse_reflection_summary(content) {
            return Some(s);
        }
    }
    None
}

pub fn recent_reflection_summaries_from_messages(
    messages: &[serde_json::Value],
    max: usize,
) -> Vec<LastReflectionSummary> {
    let max = max.max(1).min(12);
    let mut out: Vec<LastReflectionSummary> = Vec::new();
    for msg in messages.iter().rev() {
        if out.len() >= max {
            break;
        }
        if msg.get("role").and_then(|r| r.as_str()) != Some("assistant") {
            continue;
        }
        let content = msg.get("content").and_then(|c| c.as_str()).unwrap_or("");
        if let Some(s) = parse_reflection_summary(content) {
            out.push(s);
        }
    }
    out.reverse(); // chronological
    out
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub version: u32,
    pub created_at_ms: u128,
    pub updated_at_ms: u128,

    pub tool_root: Option<String>,
    pub checkpoint: Option<String>,
    pub cur_cwd: Option<String>,

    #[serde(default)]
    pub last_reflection: Option<LastReflectionSummary>,

    #[serde(default)]
    pub recent_reflections: Vec<LastReflectionSummary>,

    #[serde(default)]
    pub observation_cache: Option<ObservationCache>,

    /// OpenAI-compatible message array (includes tool_calls + tool_call_id).
    pub messages: Vec<serde_json::Value>,
}

impl AgentSession {
    pub const VERSION: u32 = 1;

    #[allow(dead_code)]
    pub fn new(
        tool_root: Option<String>,
        checkpoint: Option<String>,
        cur_cwd: Option<String>,
        observation_cache: Option<ObservationCache>,
        messages: Vec<serde_json::Value>,
    ) -> Self {
        let now = now_ms();
        let last_reflection = last_reflection_summary_from_messages(&messages);
        let recent_reflections = recent_reflection_summaries_from_messages(&messages, 3);
        Self {
            version: Self::VERSION,
            created_at_ms: now,
            updated_at_ms: now,
            tool_root,
            checkpoint,
            cur_cwd,
            last_reflection,
            recent_reflections,
            observation_cache,
            messages,
        }
    }

    #[allow(dead_code)]
    pub fn touch(&mut self) {
        self.updated_at_ms = now_ms();
    }

    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read session file: {}", path.display()))?;
        let sess: AgentSession = serde_json::from_str(&text)
            .with_context(|| format!("failed to parse session JSON: {}", path.display()))?;
        if sess.version != Self::VERSION {
            anyhow::bail!(
                "unsupported session version {} (expected {})",
                sess.version,
                Self::VERSION
            );
        }
        Ok(sess)
    }

    #[allow(dead_code)]
    pub fn save_atomic(path: &Path, sess: &AgentSession) -> Result<()> {
        let json = serde_json::to_string_pretty(sess).context("failed to serialize session")?;
        save_text_atomic(path, &json)
    }

    /// Repairs common session corruption patterns so the agent can resume.
    /// Returns a short warning string if the message list was modified.
    pub fn repair_for_resume(&mut self) -> Option<String> {
        let mut pending_ids: Vec<String> = Vec::new();
        let mut pending_started_at: Option<usize> = None;
        let mut trim_from: Option<usize> = None;
        let mut reason: Option<String> = None;

        for (idx, msg) in self.messages.iter().enumerate() {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");
            match role {
                "assistant" => {
                    let tool_calls = msg.get("tool_calls");
                    let has_tool_calls = tool_calls
                        .and_then(|tc| tc.as_array())
                        .map(|a| !a.is_empty())
                        .unwrap_or(false);
                    if has_tool_calls {
                        if !pending_ids.is_empty() {
                            trim_from = pending_started_at.or(Some(idx));
                            reason = Some(
                                "found a new assistant tool_call before the previous tool_call completed"
                                    .to_string(),
                            );
                            break;
                        }
                        let ids: Vec<String> = tool_calls
                            .and_then(|tc| tc.as_array())
                            .into_iter()
                            .flatten()
                            .filter_map(|tc| {
                                tc.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())
                            })
                            .collect();
                        if !ids.is_empty() {
                            pending_ids = ids;
                            pending_started_at = Some(idx);
                        }
                    } else if !pending_ids.is_empty() {
                        trim_from = pending_started_at;
                        reason = Some(
                            "found a non-tool assistant message while tool results were still pending"
                                .to_string(),
                        );
                        break;
                    }
                }
                "tool" => {
                    let Some(id) = msg.get("tool_call_id").and_then(|v| v.as_str()) else {
                        trim_from = Some(idx);
                        reason = Some("tool message missing tool_call_id".to_string());
                        break;
                    };
                    if pending_ids.is_empty() {
                        trim_from = Some(idx);
                        reason = Some(
                            "tool result appeared without a preceding assistant tool_call"
                                .to_string(),
                        );
                        break;
                    }
                    if let Some(pos) = pending_ids.iter().position(|p| p == id) {
                        pending_ids.remove(pos);
                        if pending_ids.is_empty() {
                            pending_started_at = None;
                        }
                    } else {
                        trim_from = Some(idx);
                        reason = Some(
                            "tool result tool_call_id did not match the pending tool_call"
                                .to_string(),
                        );
                        break;
                    }
                }
                _ => {
                    if !pending_ids.is_empty() {
                        trim_from = pending_started_at;
                        reason = Some(format!(
                            "found a '{role}' message while tool results were still pending"
                        ));
                        break;
                    }
                }
            }
        }

        if trim_from.is_none() && !pending_ids.is_empty() {
            trim_from = pending_started_at;
            reason = Some("session ended mid tool_call (missing tool results)".to_string());
        }

        let Some(from) = trim_from else {
            return None;
        };

        if from >= self.messages.len() {
            return None;
        }

        let old_len = self.messages.len();
        self.messages.truncate(from);
        let trimmed = old_len - from;
        Some(format!(
            "repaired session: truncated {trimmed} message(s) from index {from} ({})",
            reason.unwrap_or_else(|| "unknown reason".to_string())
        ))
    }
}

fn save_text_atomic(path: &Path, text: &str) -> Result<()> {
    let parent0 = path.parent().unwrap_or_else(|| Path::new("."));
    // For "session.json", `parent()` can be empty ("") which should behave like ".".
    let parent = if parent0.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent0
    };
    std::fs::create_dir_all(parent)
        .with_context(|| format!("failed to create session directory: {}", parent.display()))?;

    // Best-effort atomic write: write to a temp file in the same directory, then rename.
    let mut tmp = tempfile::NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temp file under {}", parent.display()))?;
    use std::io::Write;
    tmp.write_all(text.as_bytes())
        .context("failed to write session temp file")?;
    tmp.flush().ok();

    let tmp_path: PathBuf = tmp.path().to_path_buf();
    match tmp.persist(path) {
        Ok(_) => Ok(()),
        Err(e) => {
            // If persist failed, ensure we don't leave an orphan.
            let _ = std::fs::remove_file(&tmp_path);
            Err(anyhow!("failed to persist session file: {}", e.error))
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct SaveKey {
    messages_len: usize,
    checkpoint: Option<String>,
    cur_cwd: Option<String>,
    observation_cache_hash: u64,
}

#[derive(Serialize)]
struct AgentSessionSnapshot<'a> {
    version: u32,
    created_at_ms: u128,
    updated_at_ms: u128,
    tool_root: Option<&'a str>,
    checkpoint: Option<&'a str>,
    cur_cwd: Option<&'a str>,
    last_reflection: Option<LastReflectionSummary>,
    recent_reflections: Vec<LastReflectionSummary>,
    observation_cache: Option<&'a ObservationCache>,
    messages: &'a [serde_json::Value],
}

/// Best-effort session auto-saver for long CLI runs.
///
/// Writes an OpenAI-compatible message array (including tool_calls + tool_call_id)
/// to a JSON file atomically, so the agent can resume after crashes or interruptions.
pub struct SessionAutoSaver {
    path: PathBuf,
    created_at_ms: u128,
    observation_cache: Mutex<Option<ObservationCache>>,
    last_saved: Mutex<SaveKey>,
    warned: AtomicBool,
}

impl SessionAutoSaver {
    pub fn new(path: PathBuf, existing: Option<&AgentSession>) -> Self {
        let created_at_ms = existing.map(|s| s.created_at_ms).unwrap_or_else(now_ms);
        Self {
            path,
            created_at_ms,
            observation_cache: Mutex::new(existing.and_then(|s| s.observation_cache.clone())),
            last_saved: Mutex::new(SaveKey::default()),
            warned: AtomicBool::new(false),
        }
    }

    pub fn set_observation_cache(&self, cache: Option<ObservationCache>) {
        let mut slot = self
            .observation_cache
            .lock()
            .expect("SessionAutoSaver observation_cache poisoned");
        *slot = cache;
    }

    pub fn save_or_error(
        &self,
        tool_root: Option<&str>,
        checkpoint: Option<&str>,
        cur_cwd: Option<&str>,
        messages: &[serde_json::Value],
    ) -> Result<()> {
        let _ = self.save_inner(tool_root, checkpoint, cur_cwd, messages, false)?;
        Ok(())
    }

    /// Returns a warning message only once (for UI display) when autosave fails.
    pub fn save_best_effort(
        &self,
        tool_root: Option<&str>,
        checkpoint: Option<&str>,
        cur_cwd: Option<&str>,
        messages: &[serde_json::Value],
    ) -> Option<String> {
        match self.save_inner(tool_root, checkpoint, cur_cwd, messages, true) {
            Ok(_) => None,
            Err(e) => {
                if !self.warned.swap(true, Ordering::Relaxed) {
                    Some(format!("{e:#}"))
                } else {
                    None
                }
            }
        }
    }

    fn save_inner(
        &self,
        tool_root: Option<&str>,
        checkpoint: Option<&str>,
        cur_cwd: Option<&str>,
        messages: &[serde_json::Value],
        skip_if_unchanged: bool,
    ) -> Result<bool> {
        let observation_cache = self
            .observation_cache
            .lock()
            .expect("SessionAutoSaver observation_cache poisoned")
            .clone();
        let mut observation_hasher = std::collections::hash_map::DefaultHasher::new();
        observation_cache.hash(&mut observation_hasher);
        let key = SaveKey {
            messages_len: messages.len(),
            checkpoint: checkpoint.map(|s| s.to_string()),
            cur_cwd: cur_cwd.map(|s| s.to_string()),
            observation_cache_hash: observation_hasher.finish(),
        };
        {
            let last = self
                .last_saved
                .lock()
                .expect("SessionAutoSaver last_saved poisoned");
            // Never overwrite a newer save with an older snapshot (e.g., Ctrl+C in the CLI main loop
            // while the agent task has already autosaved progress).
            if key.messages_len < last.messages_len {
                return Ok(false);
            }
            if skip_if_unchanged && *last == key {
                return Ok(false);
            }
        }

        let snap = AgentSessionSnapshot {
            version: AgentSession::VERSION,
            created_at_ms: self.created_at_ms,
            updated_at_ms: now_ms(),
            tool_root,
            checkpoint,
            cur_cwd,
            last_reflection: last_reflection_summary_from_messages(messages),
            recent_reflections: recent_reflection_summaries_from_messages(messages, 3),
            observation_cache: observation_cache.as_ref(),
            messages,
        };
        let json = serde_json::to_string_pretty(&snap).context("failed to serialize session")?;
        save_text_atomic(&self.path, &json)?;

        let mut last = self
            .last_saved
            .lock()
            .expect("SessionAutoSaver last_saved poisoned");
        *last = key;

        Ok(true)
    }
}

fn now_ms() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_path(prefix: &str, ext: &str) -> PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        PathBuf::from(format!("{prefix}-{n}.{ext}"))
    }

    #[test]
    fn save_atomic_supports_parentless_paths() {
        // This must work for paths like "session.json" where Path::parent() is empty ("").
        let path = unique_path("obstral-session-test", "json");
        let sess = AgentSession::new(
            None,
            None,
            None,
            None,
            vec![json!({"role":"user","content":"hi"})],
        );
        AgentSession::save_atomic(&path, &sess).expect("save_atomic");
        let text = std::fs::read_to_string(&path).expect("read");
        assert!(text.contains("\"messages\""));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn session_roundtrip_preserves_observation_cache() {
        let path = unique_path("obstral-session-obs", "json");
        let sess = AgentSession::new(
            Some("/tmp/demo".to_string()),
            Some("abc123".to_string()),
            Some("src".to_string()),
            Some(ObservationCache {
                reads: vec![ObservationReadCache {
                    command: "read_file(path=src/main.rs)".to_string(),
                    path: "src/main.rs".to_string(),
                }],
                searches: vec![ObservationSearchCache {
                    command: "search_files(pattern=reflect, dir=src)".to_string(),
                    pattern: "reflect".to_string(),
                    hit_count: 3,
                    paths: vec!["src/tui/agent.rs".to_string()],
                }],
            }),
            vec![json!({"role":"user","content":"hi"})],
        );
        AgentSession::save_atomic(&path, &sess).expect("save_atomic");
        let loaded = AgentSession::load(&path).expect("load");
        assert_eq!(
            loaded
                .observation_cache
                .as_ref()
                .and_then(|cache| cache.reads.first())
                .map(|read| read.path.as_str()),
            Some("src/main.rs")
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn autosaver_rewrites_when_observation_cache_changes() {
        let path = unique_path("obstral-session-autosave", "json");
        let existing = AgentSession::new(
            None,
            None,
            None,
            Some(ObservationCache::default()),
            vec![json!({"role":"user","content":"hi"})],
        );
        let saver = SessionAutoSaver::new(path.clone(), Some(&existing));
        saver
            .save_or_error(None, None, None, &existing.messages)
            .expect("save first");
        saver.set_observation_cache(Some(ObservationCache {
            reads: vec![ObservationReadCache {
                command: "read_file(path=README.md)".to_string(),
                path: "README.md".to_string(),
            }],
            searches: Vec::new(),
        }));
        assert!(saver
            .save_best_effort(None, None, None, &existing.messages)
            .is_none());
        let loaded = AgentSession::load(&path).expect("load");
        assert_eq!(
            loaded
                .observation_cache
                .as_ref()
                .map(|cache| cache.reads.len()),
            Some(1)
        );
        let _ = std::fs::remove_file(&path);
    }
}
