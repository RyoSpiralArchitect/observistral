/// Structured file-editing tools for the Coder agent.
///
/// These complement the `exec` tool for file I/O:
///   read_file  — read file content with truncation (safe, no shell)
///   write_file — atomic create/overwrite (handles encoding correctly)
///   patch_file — exact SEARCH/REPLACE (no quoting issues)
///
/// All paths are validated to stay within `base` (tool_root).
use anyhow::{Result, anyhow};
use std::path::{Component, Path, PathBuf};

// ── Path safety ───────────────────────────────────────────────────────────────

/// Resolve a file path safely within `base` (tool_root).
///
/// - Relative paths: joined with base; `..` components are rejected.
/// - Absolute paths: allowed only if they start with base (when base is set).
pub fn resolve_safe_path(rel: &str, base: Option<&str>) -> Result<PathBuf> {
    let rel = rel.trim();
    if rel.is_empty() {
        return Err(anyhow!("path cannot be empty"));
    }

    let p = Path::new(rel);

    if p.is_absolute() {
        if let Some(root) = base {
            let canon_p = p.to_string_lossy().replace('\\', "/");
            let canon_r = Path::new(root).to_string_lossy().replace('\\', "/");
            let canon_r = canon_r.trim_end_matches('/');
            if !canon_p.starts_with(canon_r) {
                return Err(anyhow!(
                    "absolute path '{}' is outside tool_root '{}'", rel, root
                ));
            }
        }
        return Ok(p.to_path_buf());
    }

    // Relative: reject any `..` component.
    for comp in p.components() {
        if matches!(comp, Component::ParentDir) {
            return Err(anyhow!("path traversal ('..') not allowed: {}", rel));
        }
    }

    let abs = match base {
        Some(root) => PathBuf::from(root).join(p),
        None => std::env::current_dir().unwrap_or_default().join(p),
    };

    Ok(abs)
}

// ── read_file ─────────────────────────────────────────────────────────────────

/// Maximum characters returned from a single read_file call.
/// ~8000 chars ≈ 2000 tokens; enough for most source files.
const MAX_READ_CHARS: usize = 8000;

/// Read a file's content. Returns `(result_text, is_error)`.
///
/// On success: header line + file content (truncated if large).
/// On error:   human-readable error message.
pub fn tool_read_file(path: &str, base: Option<&str>) -> (String, bool) {
    let abs_path = match resolve_safe_path(path, base) {
        Ok(p) => p,
        Err(e) => return (format!("ERROR: {e}"), true),
    };

    match std::fs::read_to_string(&abs_path) {
        Ok(content) => {
            let line_count = content.lines().count();
            let byte_count = content.len();
            let char_count = content.chars().count();

            let (shown, truncated) = if char_count > MAX_READ_CHARS {
                let s: String = content.chars().take(MAX_READ_CHARS).collect();
                (s, true)
            } else {
                (content, false)
            };

            let header = format!("[{path}] ({line_count} lines, {byte_count} bytes)\n");
            let trunc_note = if truncated {
                let hidden = line_count.saturating_sub(shown.lines().count());
                format!("\n[…truncated — {hidden} more lines not shown; use exec to view specific ranges]")
            } else {
                String::new()
            };

            (format!("{header}{shown}{trunc_note}"), false)
        }
        Err(e) => (format!("ERROR reading '{path}': {e}"), true),
    }
}

// ── write_file ────────────────────────────────────────────────────────────────

/// Atomically write `content` to a file (temp → rename).
/// Creates parent directories as needed. Returns `(result_text, is_error)`.
pub fn tool_write_file(path: &str, content: &str, base: Option<&str>) -> (String, bool) {
    let abs_path = match resolve_safe_path(path, base) {
        Ok(p) => p,
        Err(e) => return (format!("ERROR: {e}"), true),
    };

    // Create parent directories.
    if let Some(parent) = abs_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return (format!("ERROR creating parent directories: {e}"), true);
        }
    }

    // Atomic write: temp file → rename.
    let mut tmp_os = abs_path.as_os_str().to_owned();
    tmp_os.push(".__obstral_tmp");
    let tmp_path = PathBuf::from(tmp_os);

    if let Err(e) = std::fs::write(&tmp_path, content) {
        return (format!("ERROR writing temp file: {e}"), true);
    }

    if let Err(e) = std::fs::rename(&tmp_path, &abs_path) {
        let _ = std::fs::remove_file(&tmp_path);
        return (format!("ERROR finalizing write to '{path}': {e}"), true);
    }

    let line_count = content.lines().count();
    let byte_count = content.len();
    (
        format!("OK: wrote '{path}' ({line_count} lines, {byte_count} bytes)"),
        false,
    )
}

// ── patch_file ────────────────────────────────────────────────────────────────

/// Apply a SEARCH/REPLACE edit. `search` must appear **exactly once**.
/// Returns `(result_text, is_error)`.
pub fn tool_patch_file(
    path: &str,
    search: &str,
    replace: &str,
    base: Option<&str>,
) -> (String, bool) {
    if search.is_empty() {
        return ("ERROR: search string cannot be empty".to_string(), true);
    }

    let abs_path = match resolve_safe_path(path, base) {
        Ok(p) => p,
        Err(e) => return (format!("ERROR: {e}"), true),
    };

    let content = match std::fs::read_to_string(&abs_path) {
        Ok(c) => c,
        Err(e) => return (format!("ERROR reading '{path}': {e}"), true),
    };

    let count = content.matches(search).count();
    if count == 0 {
        // Show a short preview so the model can self-correct.
        let preview: String = content.lines().take(8).collect::<Vec<_>>().join("\n");
        return (
            format!(
                "ERROR: search string not found in '{path}'.\n\
                 File starts with:\n{preview}\n\n\
                 Tip: call read_file first to inspect exact content, then retry with the exact text."
            ),
            true,
        );
    }
    if count > 1 {
        return (
            format!(
                "ERROR: search string found {count} times in '{path}' — must be unique.\n\
                 Tip: include more surrounding lines to make the match unique."
            ),
            true,
        );
    }

    let new_content = content.replacen(search, replace, 1);

    // Atomic write.
    let mut tmp_os = abs_path.as_os_str().to_owned();
    tmp_os.push(".__obstral_tmp");
    let tmp_path = PathBuf::from(tmp_os);

    if let Err(e) = std::fs::write(&tmp_path, &new_content) {
        return (format!("ERROR writing temp file: {e}"), true);
    }
    if let Err(e) = std::fs::rename(&tmp_path, &abs_path) {
        let _ = std::fs::remove_file(&tmp_path);
        return (format!("ERROR finalizing patch to '{path}': {e}"), true);
    }

    let old_lines = content.lines().count();
    let new_lines = new_content.lines().count();
    let delta = new_lines as i64 - old_lines as i64;
    let delta_str = if delta >= 0 {
        format!("+{delta}")
    } else {
        format!("{delta}")
    };

    (
        format!("OK: patched '{path}' ({delta_str} lines, {new_lines} total)"),
        false,
    )
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_path_rejects_dotdot() {
        assert!(resolve_safe_path("../etc/passwd", Some("/tmp/root")).is_err());
        assert!(resolve_safe_path("a/../../etc", Some("/tmp/root")).is_err());
    }

    #[test]
    fn safe_path_relative_ok() {
        let p = resolve_safe_path("src/main.rs", Some("/tmp/root")).unwrap();
        assert!(p.to_string_lossy().contains("src"));
    }

    #[test]
    fn write_then_read() {
        let dir = std::env::temp_dir().join("obstral_test_wr");
        let _ = std::fs::create_dir_all(&dir);
        let base = dir.to_string_lossy().into_owned();

        let (r, err) = tool_write_file("hello.txt", "hello world\n", Some(&base));
        assert!(!err, "{r}");

        let (r2, err2) = tool_read_file("hello.txt", Some(&base));
        assert!(!err2, "{r2}");
        assert!(r2.contains("hello world"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn patch_basic() {
        let dir = std::env::temp_dir().join("obstral_test_patch");
        let _ = std::fs::create_dir_all(&dir);
        let base = dir.to_string_lossy().into_owned();

        tool_write_file("f.txt", "line1\nfoo bar\nline3\n", Some(&base));
        let (r, err) = tool_patch_file("f.txt", "foo bar", "baz qux", Some(&base));
        assert!(!err, "{r}");

        let (content, _) = tool_read_file("f.txt", Some(&base));
        assert!(content.contains("baz qux"));
        assert!(!content.contains("foo bar"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn patch_not_found_shows_preview() {
        let dir = std::env::temp_dir().join("obstral_test_notfound");
        let _ = std::fs::create_dir_all(&dir);
        let base = dir.to_string_lossy().into_owned();

        tool_write_file("f.txt", "hello world\n", Some(&base));
        let (r, err) = tool_patch_file("f.txt", "nonexistent", "x", Some(&base));
        assert!(err);
        assert!(r.contains("not found"));
        assert!(r.contains("hello world"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
