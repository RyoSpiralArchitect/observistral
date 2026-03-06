/// Lightweight project context scanner.
///
/// Scans a directory root in <200 ms and returns a structured summary that is
/// injected into the Coder's system message so it understands the project
/// without having to ask.
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tokio::time::timeout;

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectScanResult {
    pub root: String,
    pub stack_label: String,
    pub stack: Vec<String>,
    pub git_branch: String,
    pub git_modified: u32,
    pub git_untracked: u32,
    pub context_text: String,
}

impl Default for ProjectScanResult {
    fn default() -> Self {
        Self {
            root: String::new(),
            stack_label: String::new(),
            stack: Vec::new(),
            git_branch: String::new(),
            git_modified: 0,
            git_untracked: 0,
            context_text: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectContext {
    pub root: String,
    pub stack: Vec<String>,
    pub git_branch: Option<String>,
    pub git_modified: u32,
    pub git_untracked: u32,
    pub git_recent: Vec<String>,
    pub tree: Vec<(String, usize)>, // (dir_name, file_count)
    pub key_files: Vec<String>,
    pub readme_excerpt: Option<String>,
    /// Content of .obstral.md / AGENTS.md / CLAUDE.md — project-specific instructions.
    pub agents_md: Option<String>,
    /// Auto-detected or .obstral.md-configured test command.
    pub test_cmd: Option<String>,
}

// ── Main scan function ─────────────────────────────────────────────────────────

impl ProjectContext {
    /// Scan a root directory.  Returns None only if the path doesn't exist.
    /// All sub-errors (git unavailable, no README, etc.) are silently swallowed.
    pub async fn scan(root: &str) -> Option<Self> {
        let root = root.trim();
        if root.is_empty() {
            return None;
        }
        let p = Path::new(root);
        if !p.is_dir() {
            return None;
        }
        let root = p.to_string_lossy().to_string();

        // Stack detection is sync and fast.
        let stack = detect_stack(&root);

        // Tree is sync and fast (top-level only).
        let tree = scan_tree(&root);

        // Key files list (existence check only).
        let key_files = detect_key_files(&root);

        // README excerpt (sync read, first 15 lines).
        let readme_excerpt = read_readme_excerpt(&root);

        // Git info: run async with tight timeouts.
        let (git_branch, git_modified, git_untracked, git_recent) = scan_git(&root).await;

        // Project-specific instruction file (.obstral.md > AGENTS.md > CLAUDE.md).
        let agents_md = try_read_agents_file(&root);

        // Test command: from .obstral.md `test_cmd:` line, then auto-detect from stack.
        let test_cmd = detect_test_cmd(&root, agents_md.as_deref());

        Some(ProjectContext {
            root,
            stack,
            git_branch,
            git_modified,
            git_untracked,
            git_recent,
            tree,
            key_files,
            readme_excerpt,
            agents_md,
            test_cmd,
        })
    }

    /// One-line label shown in TUI header and web badge.
    /// E.g. "Rust · React · git:main"
    pub fn stack_label(&self) -> String {
        let mut parts: Vec<String> = self.stack.clone();
        if let Some(ref branch) = self.git_branch {
            parts.push(format!("git:{branch}"));
        }
        if parts.is_empty() {
            return String::new();
        }
        parts.join(" · ")
    }

    /// Build the context text block (<300 tokens) for injection into the system message.
    pub fn to_context_text(&self) -> String {
        let mut out = String::with_capacity(512);
        out.push_str("[Project Context — auto-detected]\n");

        // Stack
        if !self.stack.is_empty() {
            out.push_str(&format!("stack: {}\n", self.stack.join(", ")));
        }

        // Test command (auto-detected or configured in .obstral.md)
        if let Some(ref cmd) = self.test_cmd {
            let cmd = cmd.trim();
            if !cmd.is_empty() {
                out.push_str(&format!("test_cmd: {cmd}\n"));
            }
        }

        // Git
        let git_line = match &self.git_branch {
            Some(branch) => format!(
                "git:   branch={}  modified={}  untracked={}\n",
                branch, self.git_modified, self.git_untracked
            ),
            None => "git:   (not a git repository)\n".to_string(),
        };
        out.push_str(&git_line);

        // Recent commits
        if !self.git_recent.is_empty() {
            let commits: Vec<String> = self
                .git_recent
                .iter()
                .map(|c| format!("\"{}\"", c))
                .collect();
            out.push_str(&format!("recent: {}\n", commits.join(" · ")));
        }

        // Tree
        if !self.tree.is_empty() {
            out.push_str("tree:\n");
            for (dir, count) in &self.tree {
                out.push_str(&format!("  {:<14} {} files\n", format!("{}/", dir), count));
            }
        }

        // Key files
        if !self.key_files.is_empty() {
            out.push_str(&format!("key:  {}\n", self.key_files.join(" · ")));
        }

        // README excerpt
        if let Some(ref excerpt) = self.readme_excerpt {
            if !excerpt.trim().is_empty() {
                out.push_str("readme:\n");
                for line in excerpt.lines().take(5) {
                    out.push_str(&format!("  {line}\n"));
                }
            }
        }

        out.trim_end().to_string()
    }

    /// Convert to the serialisable result type for the REST API.
    pub fn to_scan_result(&self) -> ProjectScanResult {
        ProjectScanResult {
            root: self.root.clone(),
            stack_label: self.stack_label(),
            stack: self.stack.clone(),
            git_branch: self.git_branch.clone().unwrap_or_default(),
            git_modified: self.git_modified,
            git_untracked: self.git_untracked,
            context_text: self.to_context_text(),
        }
    }
}

// ── Stack detection ───────────────────────────────────────────────────────────

fn detect_stack(root: &str) -> Vec<String> {
    let mut stack: Vec<String> = Vec::new();

    // Rust
    if Path::new(root).join("Cargo.toml").is_file() {
        stack.push("Rust".to_string());
    }

    // Node / React / TypeScript
    let pkg_path = Path::new(root).join("package.json");
    if pkg_path.is_file() {
        if let Ok(src) = std::fs::read_to_string(&pkg_path) {
            if src.contains("\"react\"") || src.contains("react-dom") {
                stack.push("React".to_string());
            } else if src.contains("\"typescript\"") || src.contains("tsconfig") {
                stack.push("TypeScript".to_string());
            } else {
                stack.push("Node".to_string());
            }
        } else {
            stack.push("Node".to_string());
        }
    }

    // Python
    if Path::new(root).join("pyproject.toml").is_file()
        || Path::new(root).join("requirements.txt").is_file()
        || Path::new(root).join("setup.py").is_file()
    {
        stack.push("Python".to_string());
    }

    // Go
    if Path::new(root).join("go.mod").is_file() {
        stack.push("Go".to_string());
    }

    // Java / Maven
    if Path::new(root).join("pom.xml").is_file() {
        stack.push("Java".to_string());
    }

    // tsconfig without package.json
    if stack.is_empty() && Path::new(root).join("tsconfig.json").is_file() {
        stack.push("TypeScript".to_string());
    }

    stack
}

// ── Git scanning ──────────────────────────────────────────────────────────────

async fn scan_git(root: &str) -> (Option<String>, u32, u32, Vec<String>) {
    let branch = run_git(root, &["rev-parse", "--abbrev-ref", "HEAD"]).await;
    if branch.is_none() {
        return (None, 0, 0, vec![]);
    }

    let status_out = run_git(root, &["status", "--short"])
        .await
        .unwrap_or_default();
    let mut modified = 0u32;
    let mut untracked = 0u32;
    for line in status_out.lines() {
        if line.starts_with("??") {
            untracked += 1;
        } else if !line.trim().is_empty() {
            modified += 1;
        }
    }

    let log_out = run_git(root, &["log", "--oneline", "-3", "--no-decorate"])
        .await
        .unwrap_or_default();
    let recent: Vec<String> = log_out
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            // Strip the leading hash (first word).
            if let Some(idx) = l.find(' ') {
                l[idx + 1..].trim().chars().take(60).collect()
            } else {
                l.to_string()
            }
        })
        .collect();

    (branch, modified, untracked, recent)
}

async fn run_git(root: &str, args: &[&str]) -> Option<String> {
    let fut = tokio::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output();

    let out = timeout(Duration::from_secs(2), fut).await.ok()?.ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

// ── Directory tree ────────────────────────────────────────────────────────────

const SKIP_DIRS: &[&str] = &[
    "target",
    "node_modules",
    "__pycache__",
    ".git",
    "dist",
    "build",
    ".next",
    ".nuxt",
    "vendor",
];

fn scan_tree(root: &str) -> Vec<(String, usize)> {
    let dir = match std::fs::read_dir(root) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    let mut entries: Vec<(String, usize)> = Vec::new();

    for entry in dir.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if SKIP_DIRS.contains(&name.as_str()) {
            continue;
        }
        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        if ft.is_dir() {
            let count = count_direct_files(&entry.path());
            entries.push((name, count));
        }
    }

    // Sort by name for deterministic output.
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries.truncate(8);
    entries
}

fn count_direct_files(dir: &std::path::PathBuf) -> usize {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
                .count()
        })
        .unwrap_or(0)
}

// ── Key files ─────────────────────────────────────────────────────────────────

const KEY_CANDIDATES: &[&str] = &[
    "Cargo.toml",
    "package.json",
    "pyproject.toml",
    "go.mod",
    "pom.xml",
    "README.md",
    "readme.md",
    "Makefile",
    "docker-compose.yml",
    "Dockerfile",
    ".env.example",
];

fn detect_key_files(root: &str) -> Vec<String> {
    KEY_CANDIDATES
        .iter()
        .filter(|&&f| Path::new(root).join(f).is_file())
        .map(|&f| f.to_string())
        .collect()
}

// ── README excerpt ─────────────────────────────────────────────────────────────

fn read_readme_excerpt(root: &str) -> Option<String> {
    let path = ["README.md", "readme.md", "README.txt", "README"]
        .iter()
        .map(|n| Path::new(root).join(n))
        .find(|p| p.is_file())?;

    let content = std::fs::read_to_string(path).ok()?;
    let useful: Vec<&str> = content
        .lines()
        .filter(|l| !l.trim_start().starts_with("[!["))
        .take(15)
        .collect();

    let non_empty = useful.iter().filter(|l| !l.trim().is_empty()).count();
    if non_empty < 2 {
        return None;
    }

    Some(useful.join("\n"))
}

/// Detect the test command for this project.
/// Priority: `test_cmd:` line in .obstral.md > auto-detect from stack markers.
fn detect_test_cmd(root: &str, agents_md: Option<&str>) -> Option<String> {
    // 1. Explicit override in .obstral.md: `test_cmd: cargo test --workspace`
    if let Some(content) = agents_md {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("test_cmd:") {
                let cmd = rest.trim().to_string();
                if !cmd.is_empty() {
                    return Some(cmd);
                }
            }
        }
    }

    // 2. Auto-detect from stack.
    let p = Path::new(root);
    if p.join("Cargo.toml").is_file() {
        return Some("cargo test 2>&1".to_string());
    }
    if p.join("package.json").is_file() {
        // Check for a "test" script in package.json
        if let Ok(src) = std::fs::read_to_string(p.join("package.json")) {
            if src.contains("\"test\"") {
                let mgr = if p.join("pnpm-lock.yaml").is_file() {
                    "pnpm"
                } else if p.join("yarn.lock").is_file() {
                    "yarn"
                } else {
                    "npm"
                };
                return Some(format!("{mgr} test --passWithNoTests 2>&1"));
            }
        }
    }
    if p.join("pyproject.toml").is_file()
        || p.join("pytest.ini").is_file()
        || p.join("setup.cfg").is_file()
    {
        return Some("pytest -q 2>&1".to_string());
    }
    if p.join("go.mod").is_file() {
        return Some("go test ./... 2>&1".to_string());
    }

    None
}

/// Read the first project instruction file found in `root`.
/// Priority: .obstral.md > AGENTS.md > CLAUDE.md
/// Truncated to 200 lines to keep token cost bounded.
fn try_read_agents_file(root: &str) -> Option<String> {
    for name in &[".obstral.md", "AGENTS.md", "CLAUDE.md"] {
        let path = Path::new(root).join(name);
        if let Ok(content) = std::fs::read_to_string(&path) {
            let trimmed = content.trim();
            if trimmed.is_empty() {
                continue;
            }
            // Cap at 200 lines to avoid blowing the context window.
            let capped: String = trimmed.lines().take(200).collect::<Vec<_>>().join("\n");
            return Some(capped);
        }
    }
    None
}
