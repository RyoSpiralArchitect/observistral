use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub version: u32,
    pub created_at_ms: u128,
    pub updated_at_ms: u128,

    pub tool_root: Option<String>,
    pub checkpoint: Option<String>,
    pub cur_cwd: Option<String>,

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
        messages: Vec<serde_json::Value>,
    ) -> Self {
        let now = now_ms();
        Self {
            version: Self::VERSION,
            created_at_ms: now,
            updated_at_ms: now,
            tool_root,
            checkpoint,
            cur_cwd,
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
}

fn save_text_atomic(path: &Path, text: &str) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
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
}

#[derive(Serialize)]
struct AgentSessionSnapshot<'a> {
    version: u32,
    created_at_ms: u128,
    updated_at_ms: u128,
    tool_root: Option<&'a str>,
    checkpoint: Option<&'a str>,
    cur_cwd: Option<&'a str>,
    messages: &'a [serde_json::Value],
}

/// Best-effort session auto-saver for long CLI runs.
///
/// Writes an OpenAI-compatible message array (including tool_calls + tool_call_id)
/// to a JSON file atomically, so the agent can resume after crashes or interruptions.
pub struct SessionAutoSaver {
    path: PathBuf,
    created_at_ms: u128,
    last_saved: Mutex<SaveKey>,
    warned: AtomicBool,
}

impl SessionAutoSaver {
    pub fn new(path: PathBuf, existing: Option<&AgentSession>) -> Self {
        let created_at_ms = existing.map(|s| s.created_at_ms).unwrap_or_else(now_ms);
        Self {
            path,
            created_at_ms,
            last_saved: Mutex::new(SaveKey::default()),
            warned: AtomicBool::new(false),
        }
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
        let key = SaveKey {
            messages_len: messages.len(),
            checkpoint: checkpoint.map(|s| s.to_string()),
            cur_cwd: cur_cwd.map(|s| s.to_string()),
        };
        {
            let last = self
                .last_saved
                .lock()
                .expect("SessionAutoSaver last_saved poisoned");
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
