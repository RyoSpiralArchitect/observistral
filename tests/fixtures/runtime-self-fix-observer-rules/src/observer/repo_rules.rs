const TUI_REPLAY_PATHS: &[&str] = &[
    "src/tui/events.rs",
    "src/tui/app.rs",
    "src/tui/prefs.rs",
    "src/tui/ui.rs",
    "src/tui/suggestion.rs",
];

pub fn requires_tui_replay(path: &str) -> bool {
    TUI_REPLAY_PATHS.contains(&path)
}

#[cfg(test)]
mod tests {
    use super::requires_tui_replay;

    #[test]
    fn existing_tui_paths_still_require_replay() {
        assert!(requires_tui_replay("src/tui/events.rs"));
        assert!(requires_tui_replay("src/tui/prefs.rs"));
    }

    #[test]
    fn review_panel_is_replay_sensitive() {
        assert!(requires_tui_replay("src/tui/review_panel.rs"));
    }
}
