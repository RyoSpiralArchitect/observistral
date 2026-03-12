use crate::observer::analyzer;
use crate::observer::detector;
use crate::observer::memory::{self, CritiqueMemory};
use crate::observer::scorer;
use crate::observer::{Critique, DevPhase, HealthScore, Proposal, ProposalStatus, RiskAxis};
use serde_json::Value;

fn status_rank(s: ProposalStatus) -> u8 {
    match s {
        ProposalStatus::Escalated => 0,
        ProposalStatus::Unresolved => 1,
        ProposalStatus::New => 2,
        ProposalStatus::Addressed => 3,
    }
}

fn phase_from_proposals(ps: &[Proposal]) -> DevPhase {
    if ps.iter().any(|p| p.phase == DevPhase::Core) {
        return DevPhase::Core;
    }
    if ps.iter().any(|p| p.phase == DevPhase::Feature) {
        return DevPhase::Feature;
    }
    if ps.is_empty() {
        DevPhase::Polish
    } else {
        DevPhase::Any
    }
}

fn summarize_from_counts(tool_results: usize, failures: usize, files_written: usize) -> String {
    if tool_results == 0 && failures == 0 && files_written == 0 {
        return "No tool activity detected.".to_string();
    }
    format!(
        "Observed {tool_results} tool results, {failures} failures, {files_written} file edits."
    )
}

fn slow_commands_summary(events: &[detector::Event]) -> Option<String> {
    let mut timings: Vec<(String, u64)> = events
        .iter()
        .filter_map(|e| match e {
            detector::Event::CommandTiming {
                command,
                duration_ms,
            } => {
                let cmd = command.trim();
                if cmd.is_empty() {
                    return None;
                }
                let cmd1 = cmd.lines().next().unwrap_or(cmd).trim();
                if cmd1.is_empty() {
                    return None;
                }
                Some((cmd1.to_string(), *duration_ms))
            }
            _ => None,
        })
        .collect();

    timings.sort_by(|a, b| b.1.cmp(&a.1));
    let max_ms = timings.first().map(|x| x.1).unwrap_or(0);
    if max_ms < 2000 {
        return None;
    }

    let mut out = String::new();
    out.push_str("Slow commands (top 3):\n");
    for (cmd, ms) in timings.iter().take(3) {
        let clipped = if cmd.len() > 120 {
            format!("{}…", &cmd[..120])
        } else {
            cmd.to_string()
        };
        out.push_str(&format!("- {clipped} ({ms}ms)\n"));
    }
    Some(out.trim_end().to_string())
}

fn edited_files_summary(events: &[detector::Event]) -> Option<String> {
    let mut uniq: Vec<String> = Vec::new();
    for e in events {
        if let detector::Event::FileWritten { path } = e {
            let p = path.trim();
            if p.is_empty() {
                continue;
            }
            if !uniq.iter().any(|x| x == p) {
                uniq.push(p.to_string());
            }
            if uniq.len() >= 3 {
                break;
            }
        }
    }
    if uniq.is_empty() {
        return None;
    }
    Some(format!("Edited files (up to 3):\n- {}", uniq.join("\n- ")))
}

fn summarize(
    events: &[detector::Event],
    tool_results: usize,
    failures: usize,
    files_written: usize,
) -> String {
    let mut out = summarize_from_counts(tool_results, failures, files_written);
    if let Some(extra) = slow_commands_summary(events) {
        out.push('\n');
        out.push_str(&extra);
    }
    if let Some(extra) = edited_files_summary(events) {
        out.push('\n');
        out.push_str(&extra);
    }
    out
}

fn critical_path_from_proposals(ps: &[Proposal]) -> String {
    let Some(top) = ps.first() else {
        return "none".to_string();
    };
    if top.impact.trim().is_empty() {
        top.title.trim().to_string()
    } else {
        format!("{} — {}", top.title.trim(), top.impact.trim())
    }
}

fn health_from_proposals(
    ps: &[Proposal],
    failure_count: usize,
    top_risk_axis: Option<RiskAxis>,
) -> HealthScore {
    let max_score = ps.iter().map(|p| p.score).max().unwrap_or(0);

    let mut health: i32 = 100;
    health -= (max_score as i32 * 75) / 100; // 0..75
    health -= (failure_count as i32 * 3).min(20);

    let score = health.clamp(0, 100) as u32;

    let rationale = if let Some(p) = ps.first() {
        if !p.impact.trim().is_empty() {
            p.impact.trim().to_string()
        } else {
            format!("Top issue: {}", p.title.trim())
        }
    } else if let Some(axis) = top_risk_axis {
        format!("Primary risk axis: {:?}", axis)
    } else {
        "No significant issues detected.".to_string()
    };

    HealthScore { score, rationale }
}

fn run_from_events(
    events: Vec<detector::Event>,
    tool_results_count: usize,
    memory_opt: Option<&mut CritiqueMemory>,
) -> Critique {
    let risks = analyzer::analyze(&events);
    let mut proposals = analyzer::generate_proposals(&risks);

    for p in &mut proposals {
        scorer::score_proposal(p);
    }

    if let Some(mem) = memory_opt {
        memory::apply_memory(mem, &mut proposals);
    }

    proposals.sort_by(|a, b| {
        let ra = status_rank(a.status);
        let rb = status_rank(b.status);
        match ra.cmp(&rb) {
            std::cmp::Ordering::Equal => b.score.cmp(&a.score),
            other => other,
        }
    });

    // Keep the output small and UI-friendly by default.
    if proposals.len() > 5 {
        proposals.truncate(5);
    }

    let failures = events
        .iter()
        .filter(|e| matches!(e, detector::Event::CommandFailure { .. }))
        .count();
    let files_written = events
        .iter()
        .filter(|e| matches!(e, detector::Event::FileWritten { .. }))
        .count();

    let phase = phase_from_proposals(&proposals);
    let critical_path = critical_path_from_proposals(&proposals);
    let top_axis = risks.first().map(|r| r.axis);
    let health = health_from_proposals(&proposals, failures, top_axis);

    Critique {
        summary: summarize(&events, tool_results_count, failures, files_written),
        risks,
        proposals,
        phase,
        critical_path,
        health,
    }
}

pub fn run_observer(messages: &[Value], memory: Option<&mut CritiqueMemory>) -> Critique {
    let events = detector::detect_events(messages);
    let tool_results_count = messages
        .iter()
        .filter(|m| m.get("role").and_then(|r| r.as_str()) == Some("tool"))
        .count();
    run_from_events(events, tool_results_count, memory)
}

pub fn run_observer_from_transcript(
    transcript: &str,
    memory: Option<&mut CritiqueMemory>,
) -> Critique {
    let events = detector::detect_events_from_transcript(transcript);
    // In transcript mode, we don't have exact tool-result count; approximate with #execs.
    let tool_results_count = events
        .iter()
        .filter(|e| matches!(e, detector::Event::CommandExecuted { .. }))
        .count();
    run_from_events(events, tool_results_count, memory)
}

pub fn format_critique_as_observer_blocks(c: &Critique) -> String {
    let mut out = String::new();
    if !c.summary.trim().is_empty() {
        out.push_str(c.summary.trim_end());
        out.push_str("\n\n");
    }
    out.push_str("--- phase ---\n");
    out.push_str(match c.phase {
        crate::observer::DevPhase::Core => "core",
        crate::observer::DevPhase::Feature => "feature",
        crate::observer::DevPhase::Polish => "polish",
        crate::observer::DevPhase::Any => "any",
    });
    out.push_str("\n\n--- proposals ---\n");
    if c.proposals.is_empty() {
        out.push_str("(none)\n");
    } else {
        for (i, p) in c.proposals.iter().enumerate() {
            out.push_str(&format!("{}) title: {}\n", i + 1, p.title));
            out.push_str(&format!("   to_coder: {}\n", p.to_coder));
            out.push_str(&format!(
                "   severity: {}\n",
                match p.severity {
                    crate::observer::Severity::Info => "info",
                    crate::observer::Severity::Warn => "warn",
                    crate::observer::Severity::Crit => "crit",
                }
            ));
            out.push_str(&format!("   score: {}\n", p.score));
            out.push_str(&format!(
                "   phase: {}\n",
                match p.phase {
                    crate::observer::DevPhase::Core => "core",
                    crate::observer::DevPhase::Feature => "feature",
                    crate::observer::DevPhase::Polish => "polish",
                    crate::observer::DevPhase::Any => "any",
                }
            ));
            out.push_str(&format!("   impact: {}\n", p.impact));
            out.push_str(&format!(
                "   cost: {}\n",
                match p.cost {
                    crate::observer::Cost::Low => "low",
                    crate::observer::Cost::Medium => "medium",
                    crate::observer::Cost::High => "high",
                }
            ));
            out.push_str(&format!(
                "   status: {}\n",
                match p.status {
                    crate::observer::ProposalStatus::New => "new",
                    crate::observer::ProposalStatus::Unresolved => "[UNRESOLVED]",
                    crate::observer::ProposalStatus::Escalated => "[ESCALATED]",
                    crate::observer::ProposalStatus::Addressed => "addressed",
                }
            ));
            out.push_str(&format!("   quote: {}\n\n", p.quote));
        }
    }

    out.push_str("--- critical_path ---\n");
    let cp = c.critical_path.trim();
    out.push_str(if cp.is_empty() { "none" } else { cp });
    out.push_str("\n\n--- health ---\n");
    out.push_str(&format!("score: {}\n", c.health.score));
    if !c.health.rationale.trim().is_empty() {
        out.push_str(&format!("rationale: {}\n", c.health.rationale.trim()));
    }
    out
}
