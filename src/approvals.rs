use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::io::{BufRead, BufReader, IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Debug)]
pub enum ApprovalRequest {
    Command {
        command: String,
        cwd: Option<String>,
    },
    Edit {
        action: String,
        path: String,
        preview: String,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ApprovalOutcome {
    Approved,
    Rejected,
}

#[async_trait]
pub trait Approver: Send + Sync {
    async fn approve(&self, req: ApprovalRequest) -> Result<ApprovalOutcome>;
}

pub struct AutoApprover;

#[async_trait]
impl Approver for AutoApprover {
    async fn approve(&self, _req: ApprovalRequest) -> Result<ApprovalOutcome> {
        Ok(ApprovalOutcome::Approved)
    }
}

pub struct CliApprover {
    pub command_approval: bool,
    pub edit_approval: bool,
    pub assume_yes: bool,
    allow_all_commands: AtomicBool,
    allow_all_edits: AtomicBool,
}

impl CliApprover {
    pub fn new(command_approval: bool, edit_approval: bool, assume_yes: bool) -> Self {
        Self {
            command_approval,
            edit_approval,
            assume_yes,
            allow_all_commands: AtomicBool::new(false),
            allow_all_edits: AtomicBool::new(false),
        }
    }
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

fn open_tty_reader() -> Option<Box<dyn BufRead>> {
    #[cfg(unix)]
    {
        if let Ok(f) = std::fs::File::open("/dev/tty") {
            return Some(Box::new(BufReader::new(f)) as Box<dyn BufRead>);
        }
    }
    #[cfg(windows)]
    {
        if let Ok(f) = std::fs::File::open("CONIN$") {
            return Some(Box::new(BufReader::new(f)) as Box<dyn BufRead>);
        }
    }
    if std::io::stdin().is_terminal() {
        return Some(Box::new(BufReader::new(std::io::stdin())));
    }
    #[allow(unreachable_code)]
    None
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum PromptDecision {
    ApproveOnce,
    ApproveAll,
    Reject,
}

fn prompt_approval(header: &str, body: &str, kind_hint: &str) -> Result<PromptDecision> {
    let mut reader = open_tty_reader().ok_or_else(|| {
        anyhow!(
            "cannot prompt for approval (no TTY). Pass --yes or disable approvals."
        )
    })?;

    let mut stderr = std::io::stderr();
    writeln!(stderr, "\n[APPROVAL REQUIRED] {header}")?;
    if !body.trim().is_empty() {
        writeln!(stderr, "{body}")?;
    }

    loop {
        write!(
            stderr,
            "Approve {kind_hint}? [y]es / [n]o / [a]ll / [q]uit > "
        )?;
        stderr.flush()?;

        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            return Err(anyhow!(
                "approval prompt got EOF (non-interactive). Pass --yes or disable approvals."
            ));
        }

        let t = line.trim().to_ascii_lowercase();
        match t.as_str() {
            "y" | "yes" => return Ok(PromptDecision::ApproveOnce),
            "" | "n" | "no" => return Ok(PromptDecision::Reject),
            "a" | "all" => return Ok(PromptDecision::ApproveAll),
            "q" | "quit" | "exit" => return Err(anyhow!("aborted by user")),
            "?" | "h" | "help" => {
                writeln!(
                    stderr,
                    "Choices: y=yes, n=no (default), a=approve all (this kind), q=abort"
                )?;
                continue;
            }
            _ => {
                writeln!(stderr, "Please enter y/n/a/q (or ? for help).")?;
                continue;
            }
        }
    }
}

#[async_trait]
impl Approver for CliApprover {
    async fn approve(&self, req: ApprovalRequest) -> Result<ApprovalOutcome> {
        if self.assume_yes {
            return Ok(ApprovalOutcome::Approved);
        }

        match req {
            ApprovalRequest::Command { command, cwd } => {
                if !self.command_approval {
                    return Ok(ApprovalOutcome::Approved);
                }
                if self.allow_all_commands.load(Ordering::Relaxed) {
                    return Ok(ApprovalOutcome::Approved);
                }

                let cwd_line = cwd.unwrap_or_else(|| "(workspace root)".to_string());
                let cmd_preview = truncate_preview(&command, 1200, 24);
                let body = format!("cwd: {cwd_line}\n\n{cmd_preview}");
                match prompt_approval("exec", &body, "command")? {
                    PromptDecision::ApproveOnce => Ok(ApprovalOutcome::Approved),
                    PromptDecision::ApproveAll => {
                        self.allow_all_commands.store(true, Ordering::Relaxed);
                        Ok(ApprovalOutcome::Approved)
                    }
                    PromptDecision::Reject => Ok(ApprovalOutcome::Rejected),
                }
            }
            ApprovalRequest::Edit { action, path, preview } => {
                if !self.edit_approval {
                    return Ok(ApprovalOutcome::Approved);
                }
                if self.allow_all_edits.load(Ordering::Relaxed) {
                    return Ok(ApprovalOutcome::Approved);
                }

                let p = path;
                let pr = truncate_preview(&preview, 2800, 120);
                let body = if pr.trim().is_empty() {
                    format!("path: {p}")
                } else {
                    format!("path: {p}\n\n{pr}")
                };
                let header = format!("{action}");
                match prompt_approval(&header, &body, "edit")? {
                    PromptDecision::ApproveOnce => Ok(ApprovalOutcome::Approved),
                    PromptDecision::ApproveAll => {
                        self.allow_all_edits.store(true, Ordering::Relaxed);
                        Ok(ApprovalOutcome::Approved)
                    }
                    PromptDecision::Reject => Ok(ApprovalOutcome::Rejected),
                }
            }
        }
    }
}
