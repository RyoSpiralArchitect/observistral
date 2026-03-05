use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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

    pub fn save_atomic(path: &Path, sess: &AgentSession) -> Result<()> {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent).with_context(|| {
            format!("failed to create session directory: {}", parent.display())
        })?;

        let json = serde_json::to_string_pretty(sess).context("failed to serialize session")?;

        // Best-effort atomic write: write to a temp file in the same directory, then rename.
        let mut tmp = tempfile::NamedTempFile::new_in(parent)
            .with_context(|| format!("failed to create temp file under {}", parent.display()))?;
        use std::io::Write;
        tmp.write_all(json.as_bytes())
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
}

fn now_ms() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

