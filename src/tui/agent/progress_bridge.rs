use super::*;
use crate::progress_state::{ProgressSaveContext, RepoProgressState};
use serde_json::Value;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ProgressBridgeView {
    progress: Option<RepoProgressState>,
    source: Option<&'static str>,
}

impl ProgressBridgeView {
    pub(super) fn resolve(
        persisted: Option<&RepoProgressState>,
        context: &ProgressSaveContext,
        messages: &[Value],
    ) -> Self {
        let current = RepoProgressState::derive(context, messages);
        let persisted = persisted
            .filter(|state| state.task_matches(context.task_summary.as_str()))
            .cloned();
        if current.has_details() {
            return Self {
                progress: Some(current),
                source: Some("current"),
            };
        }
        if let Some(state) = persisted.filter(|state| state.has_details()) {
            return Self {
                progress: Some(state),
                source: Some("persisted"),
            };
        }
        Self::default()
    }

    pub(super) fn telemetry_payload(&self) -> Option<Value> {
        let progress = self.progress.as_ref()?;
        Some(serde_json::json!({
            "source": self.source.unwrap_or("unknown"),
            "completed_artifacts": progress.completed_artifacts.len(),
            "verified_commands": progress.verified_commands.len(),
            "accepted_strategies": progress.accepted_strategies.len(),
            "repeated_dead_ends": progress.repeated_dead_ends.len(),
        }))
    }

    pub(super) fn prompt(&self) -> Option<String> {
        let progress = self.progress.as_ref()?;
        let mut out = String::from("[Progress Bridge]\n");
        out.push_str(&format!(
            "task: {}\n",
            compact_one_line(progress.task_summary.as_str(), 180)
        ));
        out.push_str(&format!(
            "objective: {}\n",
            compact_one_line(progress.current_objective.as_str(), 180)
        ));
        if !progress.lane.trim().is_empty() || !progress.artifact_mode.trim().is_empty() {
            out.push_str(&format!(
                "lane: {}  artifact_mode: {}\n",
                if progress.lane.trim().is_empty() {
                    "-"
                } else {
                    progress.lane.as_str()
                },
                if progress.artifact_mode.trim().is_empty() {
                    "-"
                } else {
                    progress.artifact_mode.as_str()
                }
            ));
        }
        for artifact in progress.completed_artifacts.iter().take(4) {
            out.push_str(&format!(
                "completed_artifact: {} ({})\n",
                compact_one_line(artifact.path.as_str(), 140),
                artifact.source
            ));
        }
        for verification in progress.verified_commands.iter().take(2) {
            out.push_str(&format!(
                "verified_command: {}\n",
                compact_one_line(verification.command.as_str(), 180)
            ));
        }
        for strategy in progress.accepted_strategies.iter().take(2) {
            out.push_str(&format!(
                "accepted_strategy: {} -> {}\n",
                compact_one_line(strategy.wrong_assumption.as_str(), 100),
                compact_one_line(strategy.next_minimal_action.as_str(), 140)
            ));
        }
        for dead_end in progress.repeated_dead_ends.iter().take(2) {
            out.push_str(&format!(
                "repeated_dead_end: {} [count={}]\n",
                compact_one_line(dead_end.command.as_str(), 140),
                dead_end.count
            ));
        }
        out.push_str(
            "This is repo-level progress memory.\n\
Prefer continuing from completed artifacts and known verification paths instead of restarting discovery.\n\
If current tool output contradicts this bridge, trust the current evidence.",
        );
        Some(out)
    }

    pub(super) fn compact_prompt(&self) -> Option<String> {
        let progress = self.progress.as_ref()?;
        let mut lines = vec![
            "[Progress Bridge cache]".to_string(),
            format!(
                "- objective: {}",
                compact_one_line(progress.current_objective.as_str(), 100)
            ),
            format!(
                "- completed_artifacts: {}",
                progress.completed_artifacts.len()
            ),
            format!("- verified_commands: {}", progress.verified_commands.len()),
        ];
        if let Some(artifact) = progress.completed_artifacts.first() {
            lines.push(format!(
                "- top_artifact: {}",
                compact_one_line(artifact.path.as_str(), 100)
            ));
        }
        if let Some(verification) = progress.verified_commands.first() {
            lines.push(format!(
                "- top_verify: {}",
                compact_one_line(verification.command.as_str(), 100)
            ));
        }
        Some(lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress_state::{ProgressArtifact, ProgressVerification};

    #[test]
    fn progress_bridge_prefers_current_progress_when_available() {
        let context = ProgressSaveContext::new(
            "Fix the failing Rust test.",
            "fix_existing_files",
            "modify_existing",
        );
        let persisted = RepoProgressState {
            version: RepoProgressState::VERSION,
            updated_at_ms: 1,
            task_summary: "Fix the failing Rust test.".to_string(),
            current_objective: "old objective".to_string(),
            lane: "fix_existing_files".to_string(),
            artifact_mode: "modify_existing".to_string(),
            completed_artifacts: vec![ProgressArtifact {
                path: "src/old.rs".to_string(),
                source: "patch_file".to_string(),
            }],
            verified_commands: vec![],
            accepted_strategies: vec![],
            repeated_dead_ends: vec![],
        };
        let messages = vec![
            serde_json::json!({"role":"assistant","content":"<plan>\ngoal: fix src/lib.rs\nsteps: 1) inspect 2) patch\nacceptance: 1) cargo test passes\nrisks: stale read\nassumptions: src/lib.rs is broken\n</plan>"}),
            serde_json::json!({"role":"assistant","tool_calls":[
                {"id":"call_patch","type":"function","function":{"name":"patch_file","arguments":"{\"path\":\"src/lib.rs\",\"search\":\"bug\",\"replace\":\"fix\"}"}}
            ]}),
            serde_json::json!({"role":"tool","tool_call_id":"call_patch","content":"OK: patched 'src/lib.rs'"}),
        ];

        let view = ProgressBridgeView::resolve(Some(&persisted), &context, &messages);
        let prompt = view.prompt().expect("prompt");

        assert!(prompt.contains("src/lib.rs"));
        assert!(!prompt.contains("src/old.rs"));
    }

    #[test]
    fn progress_bridge_uses_persisted_progress_when_current_is_empty() {
        let context = ProgressSaveContext::new(
            "Fix the failing Rust test.",
            "fix_existing_files",
            "modify_existing",
        );
        let persisted = RepoProgressState {
            version: RepoProgressState::VERSION,
            updated_at_ms: 1,
            task_summary: "Fix the failing Rust test.".to_string(),
            current_objective: "fix src/lib.rs".to_string(),
            lane: "fix_existing_files".to_string(),
            artifact_mode: "modify_existing".to_string(),
            completed_artifacts: vec![ProgressArtifact {
                path: "src/lib.rs".to_string(),
                source: "patch_file".to_string(),
            }],
            verified_commands: vec![ProgressVerification {
                command: "cargo test 2>&1".to_string(),
            }],
            accepted_strategies: vec![],
            repeated_dead_ends: vec![],
        };
        let view = ProgressBridgeView::resolve(Some(&persisted), &context, &[]);
        let telemetry = view.telemetry_payload().expect("telemetry");

        assert_eq!(telemetry["source"].as_str(), Some("persisted"));
        assert_eq!(telemetry["verified_commands"].as_u64(), Some(1));
    }
}
