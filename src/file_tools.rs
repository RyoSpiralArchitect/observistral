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

// ── search_files ──────────────────────────────────────────────────────────────

const MAX_SEARCH_RESULTS: usize = 50;
const MAX_SEARCH_OUT_CHARS: usize = 6000;
const MAX_LINE_DISPLAY: usize = 200;

/// True for directory names that should never be searched.
fn skip_dir(name: &str) -> bool {
    matches!(
        name,
        "target" | "node_modules" | ".git" | "__pycache__" | "dist" | "build"
            | ".next" | ".nuxt" | "vendor" | ".venv" | "venv" | ".tox"
            | "coverage" | ".cache" | ".idea" | ".vscode" | "out" | ".svn"
    )
}

/// True for file extensions that are clearly binary — skip to avoid UTF-8 errors.
fn skip_extension(ext: &str) -> bool {
    matches!(
        ext,
        "exe" | "dll" | "so" | "dylib" | "bin" | "o" | "a" | "obj" | "wasm"
            | "zip" | "tar" | "gz" | "bz2" | "7z" | "rar" | "xz" | "zst"
            | "jpg" | "jpeg" | "png" | "gif" | "bmp" | "ico" | "webp" | "avif"
            | "mp3" | "mp4" | "wav" | "avi" | "mov" | "mkv" | "flac" | "ogg"
            | "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx"
            | "db" | "sqlite" | "sqlite3" | "parquet" | "arrow"
            | "lock" // Cargo.lock / package-lock.json can be enormous
    )
}

/// Recursive file search (literal, no regex dependency).
/// Returns "relative/path:line_no: content" for each matching line.
pub fn tool_search_files(
    pattern: &str,
    dir: &str,
    case_insensitive: bool,
    base: Option<&str>,
) -> (String, bool) {
    if pattern.is_empty() {
        return ("ERROR: search pattern cannot be empty".to_string(), true);
    }

    // Resolve search root.
    let search_root = if dir.trim().is_empty() {
        base.map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
    } else {
        match resolve_safe_path(dir, base) {
            Ok(p) => p,
            Err(e) => return (format!("ERROR: {e}"), true),
        }
    };

    if !search_root.is_dir() {
        return (
            format!("ERROR: '{}' is not a directory", search_root.display()),
            true,
        );
    }

    let needle = if case_insensitive {
        pattern.to_ascii_lowercase()
    } else {
        pattern.to_string()
    };

    let mut results: Vec<String> = Vec::new();
    let mut stack: Vec<PathBuf> = vec![search_root.clone()];
    let mut truncated = false;

    'outer: while let Some(dir_path) = stack.pop() {
        let rd = match std::fs::read_dir(&dir_path) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let mut subdirs: Vec<PathBuf> = Vec::new();
        let mut files: Vec<PathBuf> = Vec::new();

        for entry in rd.flatten() {
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if name.starts_with('.') && name != ".obstral.md" {
                continue; // skip hidden
            }
            if path.is_dir() {
                if !skip_dir(name) {
                    subdirs.push(path);
                }
            } else {
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if !skip_extension(&ext) {
                    files.push(path);
                }
            }
        }

        for file_path in &files {
            let content = match std::fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let rel = file_path
                .strip_prefix(&search_root)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| file_path.display().to_string());

            for (ln, line) in content.lines().enumerate() {
                let cmp = if case_insensitive {
                    line.to_ascii_lowercase()
                } else {
                    line.to_string()
                };
                if cmp.contains(&needle) {
                    let display: String =
                        line.trim_end().chars().take(MAX_LINE_DISPLAY).collect();
                    results.push(format!("{}:{}: {}", rel, ln + 1, display));
                    if results.len() >= MAX_SEARCH_RESULTS {
                        truncated = true;
                        break 'outer;
                    }
                }
            }
        }

        stack.extend(subdirs);
    }

    if results.is_empty() {
        return (
            format!(
                "[search_files] No matches for '{}' in '{}'",
                pattern,
                search_root.display()
            ),
            false,
        );
    }

    let count = results.len();
    let cap_note = if truncated {
        format!(" (first {MAX_SEARCH_RESULTS} shown — more may exist)")
    } else {
        String::new()
    };
    let header = format!(
        "[search_files: '{}' — {} match(es){}]\n",
        pattern, count, cap_note
    );
    let body = results.join("\n");

    let out = if body.chars().count() > MAX_SEARCH_OUT_CHARS {
        let trunc: String = body.chars().take(MAX_SEARCH_OUT_CHARS).collect();
        format!("{header}{trunc}\n[…output truncated]")
    } else {
        format!("{header}{body}")
    };

    (out, false)
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
