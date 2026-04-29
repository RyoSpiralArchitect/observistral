const PR_READY_RUNTIME_PATHS: &[&str] = &[
    "src/tui/agent/task_harness.rs",
    "src/tui/agent/done_gate.rs",
];

pub fn requires_pr_ready_handoff(path: &str) -> bool {
    PR_READY_RUNTIME_PATHS.contains(&path)
}

#[cfg(test)]
mod tests {
    use super::requires_pr_ready_handoff;

    #[test]
    fn existing_runtime_paths_still_require_pr_ready_handoff() {
        assert!(requires_pr_ready_handoff("src/tui/agent/task_harness.rs"));
        assert!(requires_pr_ready_handoff("src/tui/agent/done_gate.rs"));
    }

    #[test]
    fn session_bridge_requires_pr_ready_handoff() {
        assert!(requires_pr_ready_handoff("src/tui/agent/session_bridge.rs"));
    }
}
