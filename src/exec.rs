use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::process::Command;

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

pub struct ExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Best-effort cleanup for LLM-produced shell transcripts.
///
/// Common failure modes:
/// - Prompt markers accidentally included in the command: `$ ...`, `PS> ...`, `> ...`
/// - Stray trailing braces when a model mixes tool-call syntax into a shell line.
fn sanitize_shellish_command(cmd: &str) -> String {
    let raw = cmd.replace("\r\n", "\n").trim().to_string();
    if raw.is_empty() {
        return String::new();
    }

    let mut out: Vec<String> = Vec::new();
    for ln0 in raw.split('\n') {
        let mut ln = ln0.trim_end_matches('\r').to_string();
        let ltrim = ln.trim_start();

        // PowerShell prompt markers.
        if let Some(rest) = ltrim.strip_prefix("PS> ") {
            ln = rest.to_string();
        } else if let Some(rest) = ltrim.strip_prefix("PS>") {
            // Handle "PS>cmd" / "PS>cmd args"
            ln = rest.trim_start().to_string();
        } else if let Some(rest) = ltrim.strip_prefix("> ") {
            ln = rest.to_string();
        } else if let Some(rest) = ltrim.strip_prefix("$ ") {
            ln = rest.to_string();
        } else {
            // Strip "$" prompt with multiple spaces: "$   git status"
            let lt = ltrim;
            if lt.starts_with('$') && lt[1..].starts_with(char::is_whitespace) {
                ln = lt.trim_start_matches('$').trim_start().to_string();
            }
        }

        // Some models leak tool-call syntax into code fences (e.g. "assistant to=...").
        // Strip anything after the first known marker to keep the command runnable.
        let lower = ln.to_ascii_lowercase();
        let noise_tokens = [
            "assistant to=",
            "to=multi_tool_use.",
            "to=functions.",
            "to=web.run",
            "recipient_name",
            "parameters:",
        ];
        let mut cut_at: Option<usize> = None;
        for tok in &noise_tokens {
            if let Some(idx) = lower.find(tok) {
                cut_at = Some(cut_at.map_or(idx, |c| c.min(idx)));
            }
        }
        if let Some(idx) = cut_at {
            ln.truncate(idx);
            ln = ln.trim_end().to_string();
        }

        if ln.trim().is_empty() {
            continue;
        }
        out.push(ln);
    }

    let mut s = out.join("\n").trim().to_string();

    // Strip a leading "$" without a following space: "$git status"
    let t = s.trim_start();
    if let Some(rest) = t.strip_prefix('$') {
        if rest.chars().next().is_some_and(|c| !c.is_whitespace()) {
            s = rest.trim_start().to_string();
        }
    }

    // Strip trailing unmatched "}" / "]" (common artifacts).
    // Models sometimes emit "}}]}" or similar when mixing tool syntax into a shell line.
    loop {
        let t = s.trim_end();
        if t.is_empty() {
            s = t.to_string();
            break;
        }

        let mut changed = false;
        if t.ends_with('}') && t.matches('{').count() < t.matches('}').count() {
            s = t[..t.len() - 1].trim_end().to_string();
            changed = true;
        } else if t.ends_with(']') && t.matches('[').count() < t.matches(']').count() {
            s = t[..t.len() - 1].trim_end().to_string();
            changed = true;
        }

        if !changed {
            break;
        }
    }

    s
}

fn split_args_simple(s: &str) -> Vec<String> {
    // Minimal tokeniser good enough for short command snippets.
    // Supports single and double quotes. In double quotes, \" and \\ are handled.
    let src = s.trim();
    if src.is_empty() {
        return Vec::new();
    }

    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut q: Option<char> = None;
    let mut chars = src.chars().peekable();

    while let Some(ch) = chars.next() {
        if let Some(qq) = q {
            if ch == qq {
                q = None;
                continue;
            }
            if qq == '"' && ch == '\\' {
                if let Some(n) = chars.next() {
                    cur.push(n);
                    continue;
                }
            }
            cur.push(ch);
            continue;
        }

        if ch == '\'' || ch == '"' {
            q = Some(ch);
            continue;
        }

        if ch.is_whitespace() {
            if !cur.is_empty() {
                out.push(cur.clone());
                cur.clear();
            }
            continue;
        }

        cur.push(ch);
    }

    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn ps_single_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

#[cfg(target_os = "windows")]
fn looks_bashish_windows_script(s: &str) -> bool {
    let x = s.to_ascii_lowercase();
    // Windows PowerShell 5.x doesn't support `&&`.
    if x.contains("&&") {
        return true;
    }
    // Common bash snippets produced by models.
    if x.contains("mkdir -p") || x.contains("mkdir --parents") {
        return true;
    }
    if x.contains("\ntouch ")
        || x.starts_with("touch ")
        || x.contains("; touch ")
        || x.contains("&& touch ")
    {
        return true;
    }
    if x.contains("rm -rf") || x.contains("rm -r ") || x.contains("rm -r\t") {
        return true;
    }
    if x.contains("\nexport ") || x.starts_with("export ") || x.contains("; export ") {
        return true;
    }
    false
}

#[cfg(target_os = "windows")]
fn bash_to_powershell_script(script: &str) -> String {
    // Port of the UI-side translator: convert common bash commands into a
    // Windows PowerShell script, so models can paste bash snippets and still run.
    let raw = script.replace("\r\n", "\n");
    if raw.trim().is_empty() {
        return String::new();
    }

    let mut out: Vec<String> = Vec::new();
    out.push("$ErrorActionPreference = 'Stop'".to_string());

    for line0 in raw.split('\n') {
        let mut line = line0.trim().to_string();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') {
            continue;
        }

        // Common model glitch: trailing brace (e.g. `New-Item ... }`).
        // Do NOT drop a standalone closing brace line (`}`): wrappers may use try/finally blocks.
        if line.len() > 1 && line.ends_with('}') && !line.contains('{') {
            line.pop();
            line = line.trim().to_string();
        }

        // Replace `&&` (unsupported in Windows PowerShell 5.x) with `;`
        // then split into segments (naive, but good enough for typical snippets).
        let line = line.replace("&&", ";");
        for seg0 in line.split(';') {
            let seg = seg0.trim();
            if seg.is_empty() {
                continue;
            }

            let toks = split_args_simple(seg);
            if toks.is_empty() {
                continue;
            }
            let head = toks[0].as_str();

            if head == "mkdir" {
                let mut parents = false;
                let mut paths: Vec<String> = Vec::new();
                for t in toks.iter().skip(1) {
                    if t == "-p" || t == "--parents" {
                        parents = true;
                        continue;
                    }
                    if t.starts_with('-') {
                        continue;
                    }
                    paths.push(t.to_string());
                }
                if parents {
                    for p in paths {
                        out.push(format!(
                            "[System.IO.Directory]::CreateDirectory({}) | Out-Null",
                            ps_single_quote(&p)
                        ));
                    }
                } else {
                    // `mkdir` is an alias in PowerShell; keep it if there's no -p.
                    out.push(seg.to_string());
                }
                continue;
            }

            if head == "touch" {
                let mut paths: Vec<String> = Vec::new();
                for t in toks.iter().skip(1) {
                    if t.starts_with('-') {
                        continue;
                    }
                    paths.push(t.to_string());
                }
                for p in paths {
                    let q = ps_single_quote(&p);
                    out.push(format!(
                        "if (-not (Test-Path -LiteralPath {q})) {{ New-Item -ItemType File -Force -Path {q} | Out-Null }} else {{ (Get-Item -LiteralPath {q}).LastWriteTime = Get-Date }}"
                    ));
                }
                continue;
            }

            if head == "rm" {
                let mut recurse = false;
                let mut paths: Vec<String> = Vec::new();
                for t in toks.iter().skip(1) {
                    if t.starts_with('-') {
                        if t.contains('r') || t.contains('R') {
                            recurse = true;
                        }
                        continue;
                    }
                    paths.push(t.to_string());
                }
                for p in paths {
                    let q = ps_single_quote(&p);
                    out.push(format!(
                        "Remove-Item -Force {}-ErrorAction SilentlyContinue -LiteralPath {}",
                        if recurse { "-Recurse " } else { "" },
                        q
                    ));
                }
                continue;
            }

            if head == "cp" && toks.len() >= 3 {
                out.push(format!(
                    "Copy-Item -Force -LiteralPath {} -Destination {}",
                    ps_single_quote(&toks[1]),
                    ps_single_quote(&toks[2])
                ));
                continue;
            }

            if head == "mv" && toks.len() >= 3 {
                out.push(format!(
                    "Move-Item -Force -LiteralPath {} -Destination {}",
                    ps_single_quote(&toks[1]),
                    ps_single_quote(&toks[2])
                ));
                continue;
            }

            if head == "export" && toks.len() >= 2 {
                let kv = &toks[1];
                if let Some(idx) = kv.find('=') {
                    let k = kv[..idx].to_string();
                    let v = kv[idx + 1..].to_string();
                    if !k.trim().is_empty() {
                        out.push(format!("$env:{k} = {}", ps_single_quote(&v)));
                        continue;
                    }
                }
            }

            out.push(seg.to_string());
        }
    }

    out.join("\n").trim().to_string()
}

fn is_poison_proxy(v: &str) -> bool {
    let s = v.trim().to_ascii_lowercase();
    // Known-bad proxy setting: forces all HTTPS traffic into a dead local proxy.
    // Example from Git: "port 443 via 127.0.0.1 ... Could not connect to server"
    let prefixes = [
        "http://127.0.0.1:9",
        "https://127.0.0.1:9",
        "http://localhost:9",
        "https://localhost:9",
    ];
    prefixes.iter().any(|p| s.starts_with(p))
}

fn scrub_poison_proxy_env(cmd: &mut Command) {
    for k in [
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "GIT_HTTP_PROXY",
        "GIT_HTTPS_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
    ] {
        if let Ok(v) = std::env::var(k) {
            if !v.trim().is_empty() && is_poison_proxy(&v) {
                cmd.env_remove(k);
            }
        }
    }
}

/// Check whether a command matches the dangerous-command blocklist.
/// Returns `Some(reason)` if blocked, `None` if safe to run.
pub fn check_dangerous_command(cmd: &str) -> Option<&'static str> {
    let s = cmd.trim().to_ascii_lowercase();

    // Git destructive patterns (cross-platform).
    let git_dangerous = [
        ("git reset --hard", "git reset --hard discards local changes"),
        ("git clean -fd", "git clean -fd removes untracked files/dirs"),
        ("git clean -xdf", "git clean -xdf removes ignored files too"),
    ];
    for (pat, reason) in &git_dangerous {
        if s.contains(pat) {
            return Some(reason);
        }
    }
    // Block "remove everything from index" variants.
    // Examples:
    // - git rm --cached -r .
    // - git rm -r --cached .
    // - git rm --cached -r ./
    if s.contains("git rm")
        && (s.contains("--cached") || s.contains("--cache"))
        && (s.contains(" -r") || s.contains("--recursive"))
        && (s.ends_with(" .") || s.ends_with(" ./") || s.contains(" . "))
    {
        return Some("git rm --cached -r . would remove the entire repo from the index");
    }

    // Unix destructive patterns
    let unix_dangerous = [
        ("rm -rf /", "rm -rf / would erase the entire filesystem"),
        ("rm -rf /*", "rm -rf /* would erase the entire filesystem"),
        ("rm -rf ~", "rm -rf ~ would erase the home directory"),
        ("> /dev/sda", "writing to raw disk device"),
        ("dd if=", "dd writes raw bytes to block devices"),
        ("mkfs.", "mkfs formats a filesystem partition"),
        (":(){ :|:& };:", "fork bomb"),
        ("shutdown", "shutdown/reboot command"),
        ("halt", "halt command"),
        ("reboot", "reboot command"),
        ("chmod -r 000 /", "removing all permissions from root"),
        ("chown -r root /", "changing ownership of root"),
    ];
    for (pat, reason) in &unix_dangerous {
        if s.contains(pat) {
            return Some(reason);
        }
    }

    // Windows destructive patterns
    // Note: OpenSSH / PowerShell variants can permute arg order; keep these checks flexible.
    if s.contains("remove-item") && s.contains("-recurse") {
        if s.contains("c:\\") || s.contains("c:/") {
            return Some("recursive remove of C: drive");
        }
    }

    let win_dangerous = [
        ("format ", "format command can erase drives"),
        ("del /s /q c:\\", "recursive delete of C: drive"),
        ("del /f /s /q c:\\", "recursive delete of C: drive"),
        ("rd /s /q c:\\", "recursive remove of C: drive"),
        ("remove-item -recurse -force c:\\", "recursive remove of C: drive"),
        ("remove-item -recurse c:\\", "recursive remove of C: drive"),
        ("stop-computer", "stop-computer shuts down the machine"),
        ("restart-computer", "restart-computer reboots the machine"),
        ("disable-computerrestore", "disabling system restore"),
        ("clear-disk", "clear-disk erases disk contents"),
        ("initialize-disk", "initialize-disk reformats a disk"),
    ];
    for (pat, reason) in &win_dangerous {
        if s.contains(pat) {
            return Some(reason);
        }
    }

    // Repo-destructive git patterns (avoid self-sabotage in agentic runs).
    let git_dangerous = [
        ("git reset --hard", "git reset --hard discards local changes"),
        ("git clean -fdx", "git clean -fdx deletes untracked files/directories"),
        ("git rm --cached -r .", "git rm --cached -r . can remove the entire repo from the index"),
        ("git rm -r --cached .", "git rm -r --cached . can remove the entire repo from the index"),
    ];
    for (pat, reason) in &git_dangerous {
        if s.contains(pat) {
            return Some(reason);
        }
    }

    None
}

/// Run a local shell command and return its combined output.
///
/// On Windows, PowerShell is used. Here-strings (`@'...'@` / `@"..."@`) are
/// handled by writing a temp `.ps1` file and invoking with `-File`, which avoids
/// the column-0 terminator constraint when passing via `-Command`.
pub async fn run_command(command: &str, cwd: Option<&str>) -> Result<ExecResult> {
    let cleaned = sanitize_shellish_command(command);
    if cleaned.trim().is_empty() {
        return Ok(ExecResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
        });
    }

    if let Some(reason) = check_dangerous_command(&cleaned) {
        return Ok(ExecResult {
            stdout: String::new(),
            stderr: format!("[BLOCKED] dangerous command: {reason}"),
            exit_code: -1,
        });
    }

    // Windows: many models paste bash snippets. Translate common bash commands into PowerShell
    // so the tool works reliably in a PowerShell-only runtime.
    #[cfg(target_os = "windows")]
    let cleaned = if looks_bashish_windows_script(&cleaned) {
        bash_to_powershell_script(&cleaned)
    } else {
        cleaned
    };

    if let Some(reason) = check_dangerous_command(&cleaned) {
        return Ok(ExecResult {
            stdout: String::new(),
            stderr: format!("[BLOCKED] dangerous command: {reason}"),
            exit_code: -1,
        });
    }

    let cmd_str = cleaned.trim();

    let mut cmd = build_command(cmd_str).await?;
    scrub_poison_proxy_env(&mut cmd);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(cwd) = cwd.filter(|s| !s.trim().is_empty()) {
        cmd.current_dir(cwd);
    }

    let output = cmd.output().await.context("failed to spawn command")?;

    Ok(ExecResult {
        stdout: decode_output(&output.stdout),
        stderr: decode_output(&output.stderr),
        exit_code: output.status.code().unwrap_or(-1),
    })
}

/// Build a `Command` appropriate for the OS, handling Windows here-strings.
async fn build_command(cmd_str: &str) -> Result<Command> {
    if cfg!(target_os = "windows") {
        // Detect here-strings or multi-line scripts that need a temp file.
        let needs_tempfile = cmd_str.contains("@'")
            || cmd_str.contains("@\"")
            || cmd_str.contains('\n');

        if needs_tempfile {
            let mut tmp = tempfile::Builder::new()
                .prefix("obstral_exec_")
                .suffix(".ps1")
                .tempfile()
                .context("failed to create temp ps1 file")?;
            use std::io::Write;
            writeln!(tmp, "[Console]::OutputEncoding=[System.Text.Encoding]::UTF8")?;
            writeln!(tmp, "[Console]::InputEncoding=[System.Text.Encoding]::UTF8")?;
            writeln!(tmp, "$OutputEncoding=[System.Text.Encoding]::UTF8")?;
            writeln!(tmp)?;
            write!(tmp, "{}", cmd_str)?;
            let path = tmp.into_temp_path();
            let path_str = path.to_string_lossy().into_owned();
            // Keep temp path alive by leaking — the file is cleaned up at process exit.
            // (tokio::process::Command needs it to exist until `output()` returns.)
            let _ = path.keep();
            let mut c = Command::new("powershell");
            c.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File", &path_str]);
            return Ok(c);
        }

        let wrapped = format!(
            "[Console]::OutputEncoding=[System.Text.Encoding]::UTF8; \
             [Console]::InputEncoding=[System.Text.Encoding]::UTF8; \
             $OutputEncoding=[System.Text.Encoding]::UTF8; {}",
            cmd_str
        );
        let mut c = Command::new("powershell");
        c.args(["-NoProfile", "-NonInteractive", "-Command", &wrapped]);
        Ok(c)
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", cmd_str]);
        Ok(c)
    }
}

pub fn decode_output(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(s) = std::str::from_utf8(bytes) {
            return s.to_string();
        }
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
            String::from_utf16_lossy(&buf)
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_strips_prompt_markers() {
        let s = sanitize_shellish_command("$ mkdir -p foo\nPS> cd foo\n> touch a.txt");
        assert!(!s.contains("$ mkdir"));
        assert!(s.contains("mkdir -p foo"));
        assert!(s.contains("cd foo"));
        assert!(s.contains("touch a.txt"));
    }

    #[test]
    fn sanitize_strips_leaked_tool_noise() {
        let s = "New-Item -ItemType Directory -Force -Path 'src/'}}]} assistant to=multi_tool_use.parallel 0";
        let cleaned = sanitize_shellish_command(s);
        assert_eq!(cleaned, "New-Item -ItemType Directory -Force -Path 'src/'");
    }

    #[test]
    fn split_args_simple_handles_quotes() {
        let t = split_args_simple("cp 'a b.txt' \"c d.txt\"");
        assert_eq!(t, vec!["cp", "a b.txt", "c d.txt"]);
    }

    #[test]
    fn ps_single_quote_escapes() {
        assert_eq!(ps_single_quote("a'b"), "'a''b'");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn bash_translation_basic() {
        let s = "$ mkdir -p repo && cd repo && touch README.md";
        let cleaned = sanitize_shellish_command(s);
        assert!(looks_bashish_windows_script(&cleaned));
        let ps = bash_to_powershell_script(&cleaned);
        assert!(ps.contains("$ErrorActionPreference = 'Stop'"));
        assert!(ps.contains("[System.IO.Directory]::CreateDirectory('repo')"));
        assert!(ps.contains("cd repo") || ps.contains("Set-Location 'repo'"));
        assert!(ps.contains("New-Item -ItemType File"));
    }
}
