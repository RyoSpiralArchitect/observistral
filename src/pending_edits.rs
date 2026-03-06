use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use std::path::{Component, Path};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct PendingEditStore {
    items: Arc<Mutex<Vec<PendingEditItem>>>,
    seq: Arc<AtomicU64>,
}

#[derive(Clone)]
struct PendingEditItem {
    id: String,
    action: String,
    status: String, // "pending" | "approved" | "rejected"
    path: String,   // relative path (as provided by client)
    preview: String,
    diff: String,
    created_at_ms: u128,

    // Internal payload (not sent to UI).
    content: String,
    result: Option<serde_json::Value>,
}

#[derive(Serialize, Clone)]
pub struct PendingEditView {
    pub id: String,
    pub action: String,
    pub status: String,
    pub path: String,
    pub preview: String,
    pub diff: String,
    pub created_at: u128,
}

#[derive(Serialize)]
pub struct PendingEditResolveResponse {
    pub ok: bool,
    pub item: PendingEditResolvedItem,
}

#[derive(Serialize, Clone)]
pub struct PendingEditResolvedItem {
    pub id: String,
    pub action: String,
    pub status: String,
    pub path: String,
    pub result: Option<serde_json::Value>,
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn make_id(seq: u64) -> String {
    format!("pe_{seq}")
}

fn is_safe_rel_path(p: &Path) -> bool {
    if p.as_os_str().is_empty() {
        return false;
    }
    if p.is_absolute() {
        return false;
    }
    for c in p.components() {
        match c {
            Component::CurDir => {}
            Component::Normal(_) => {}
            // Reject ParentDir, RootDir, Prefix (e.g., C:\).
            _ => return false,
        }
    }
    true
}

fn truncate_preview(s: &str, max_chars: usize, max_lines: usize) -> String {
    let mut out = String::new();
    let mut lines = 0usize;
    for line in s.lines() {
        if lines >= max_lines {
            out.push_str("\n...truncated...\n");
            break;
        }
        if out.len() + line.len() + 1 > max_chars {
            out.push_str("\n...truncated...\n");
            break;
        }
        out.push_str(line);
        out.push('\n');
        lines += 1;
    }
    out.trim_end().to_string()
}

fn simple_diff(old: &str, new: &str) -> String {
    // Minimal, safe diff: show a small head/tail of both sides.
    // (We avoid introducing a diff dependency here.)
    if old == new {
        return String::new();
    }
    let old_p = truncate_preview(old, 1200, 40);
    let new_p = truncate_preview(new, 1200, 40);
    format!("--- before ---\n{old_p}\n\n--- after ---\n{new_p}\n")
        .trim_end()
        .to_string()
}

impl PendingEditStore {
    pub fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(Vec::new())),
            seq: Arc::new(AtomicU64::new(1)),
        }
    }

    pub async fn list(&self) -> Vec<PendingEditView> {
        let items = self.items.lock().await;
        items
            .iter()
            .map(|it| PendingEditView {
                id: it.id.clone(),
                action: it.action.clone(),
                status: it.status.clone(),
                path: it.path.clone(),
                preview: it.preview.clone(),
                diff: it.diff.clone(),
                created_at: it.created_at_ms,
            })
            .collect()
    }

    pub async fn queue_write_file(
        &self,
        workspace_root: &Path,
        rel_path: &str,
        content: &str,
        action: &str,
    ) -> Result<String> {
        let rel = Path::new(rel_path);
        if !is_safe_rel_path(rel) {
            return Err(anyhow!(
                "unsafe path (must be relative, no '..'): {rel_path}"
            ));
        }

        let abs = workspace_root.join(rel);
        let old = std::fs::read_to_string(&abs).unwrap_or_default();
        let preview = truncate_preview(content, 2200, 80);
        let diff = simple_diff(&old, content);

        let seq = self.seq.fetch_add(1, Ordering::Relaxed);
        let id = make_id(seq);

        let mut items = self.items.lock().await;
        items.push(PendingEditItem {
            id: id.clone(),
            action: action.to_string(),
            status: "pending".to_string(),
            path: rel_path.to_string(),
            preview,
            diff,
            created_at_ms: now_ms(),
            content: content.to_string(),
            result: None,
        });

        Ok(id)
    }

    pub async fn reject(&self, id: &str) -> Result<PendingEditResolvedItem> {
        let mut items = self.items.lock().await;
        let it = items
            .iter_mut()
            .find(|x| x.id == id)
            .ok_or_else(|| anyhow!("pending edit not found: {id}"))?;

        it.status = "rejected".to_string();
        Ok(PendingEditResolvedItem {
            id: it.id.clone(),
            action: it.action.clone(),
            status: it.status.clone(),
            path: it.path.clone(),
            result: it.result.clone(),
        })
    }

    pub async fn approve(
        &self,
        workspace_root: &Path,
        id: &str,
    ) -> Result<PendingEditResolvedItem> {
        let mut items = self.items.lock().await;
        let it = items
            .iter_mut()
            .find(|x| x.id == id)
            .ok_or_else(|| anyhow!("pending edit not found: {id}"))?;

        if it.status != "pending" {
            return Ok(PendingEditResolvedItem {
                id: it.id.clone(),
                action: it.action.clone(),
                status: it.status.clone(),
                path: it.path.clone(),
                result: it.result.clone(),
            });
        }

        let rel = Path::new(&it.path);
        if !is_safe_rel_path(rel) {
            return Err(anyhow!(
                "unsafe path (must be relative, no '..'): {}",
                it.path
            ));
        }
        let abs = workspace_root.join(rel);
        if let Some(parent) = abs.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create_dir_all failed: {}", parent.display()))?;
        }

        std::fs::write(&abs, it.content.as_bytes())
            .with_context(|| format!("write failed: {}", abs.display()))?;

        it.status = "approved".to_string();
        it.result = Some(serde_json::json!({
            "ok": true,
            "bytes_written": it.content.as_bytes().len(),
        }));

        Ok(PendingEditResolvedItem {
            id: it.id.clone(),
            action: it.action.clone(),
            status: it.status.clone(),
            path: it.path.clone(),
            result: it.result.clone(),
        })
    }
}
