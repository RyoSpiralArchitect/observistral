use tokio::task::JoinHandle;
use tui_textarea::TextArea;

use crate::config::RunConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Coder,
    Observer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
    /// False while streaming, true once the full message is received.
    pub complete: bool,
}

impl Message {
    pub fn new_streaming(role: Role) -> Self {
        Self { role, content: String::new(), complete: false }
    }
    pub fn new_complete(role: Role, content: String) -> Self {
        Self { role, content, complete: true }
    }
}

pub struct Pane {
    pub messages: Vec<Message>,
    pub textarea: TextArea<'static>,
    /// Lines from the bottom.  0 = pinned to bottom (auto-scroll).
    /// N = N lines above the bottom (user has scrolled up).
    pub scroll: usize,
    pub streaming: bool,
    /// Optional local filter for history readability (slash command: /find).
    pub find_query: String,
}

impl Pane {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            textarea: TextArea::default(),
            scroll: 0,
            streaming: false,
            find_query: String::new(),
        }
    }

    /// Append a delta to the current in-flight streaming message.
    pub fn push_delta(&mut self, delta: &str) {
        if let Some(last) = self.messages.last_mut() {
            if !last.complete {
                last.content.push_str(delta);
                return;
            }
        }
        // No open streaming message — start one.
        let mut msg = Message::new_streaming(Role::Assistant);
        msg.content.push_str(delta);
        self.messages.push(msg);
    }

    pub fn finish_stream(&mut self) {
        if let Some(last) = self.messages.last_mut() {
            last.complete = true;
        }
        self.streaming = false;
    }

    pub fn push_user(&mut self, text: String) {
        self.messages.push(Message::new_complete(Role::User, text));
    }

    pub fn push_tool(&mut self, text: String) {
        self.messages.push(Message::new_complete(Role::Tool, text));
    }

    /// Collect chat history for the API (User + Assistant only, complete only).
    pub fn chat_history(&self) -> Vec<crate::types::ChatMessage> {
        self.messages
            .iter()
            .filter(|m| matches!(m.role, Role::User | Role::Assistant) && m.complete)
            .map(|m| crate::types::ChatMessage {
                role: match m.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Tool => unreachable!(),
                },
                content: m.content.clone(),
            })
            .collect()
    }
}

pub struct App {
    pub coder: Pane,
    pub observer: Pane,
    pub focus: Focus,
    pub auto_observe: bool,
    pub last_auto_obs_idx: Option<usize>,
    pub coder_cfg: RunConfig,
    pub observer_cfg: RunConfig,
    pub tool_root: Option<String>,
    pub quit: bool,

    /// 100ms tick counter — drives the streaming spinner animation.
    pub tick_count: u64,

    /// How many agentic tool-call iterations have been completed so far
    /// for the current Coder task (reset to 0 on each new send).
    pub coder_iter: u32,

    /// Running task handles used for Ctrl+K cancellation.
    pub coder_task: Option<JoinHandle<()>>,
    pub observer_task: Option<JoinHandle<()>>,

    /// When true, incoming stream tokens for that pane are silently discarded
    /// (set after Ctrl+K, cleared on the next send).
    pub ignore_coder_tokens: bool,
    pub ignore_observer_tokens: bool,
}

impl App {
    pub fn new(
        coder_cfg: RunConfig,
        observer_cfg: RunConfig,
        tool_root: Option<String>,
        auto_observe: bool,
    ) -> Self {
        Self {
            coder: Pane::new(),
            observer: Pane::new(),
            focus: Focus::Coder,
            auto_observe,
            last_auto_obs_idx: None,
            coder_cfg,
            observer_cfg,
            tool_root,
            quit: false,
            tick_count: 0,
            coder_iter: 0,
            coder_task: None,
            observer_task: None,
            ignore_coder_tokens: false,
            ignore_observer_tokens: false,
        }
    }

    pub fn focused_pane_mut(&mut self) -> &mut Pane {
        match self.focus {
            Focus::Coder => &mut self.coder,
            Focus::Observer => &mut self.observer,
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Coder => Focus::Observer,
            Focus::Observer => Focus::Coder,
        };
    }

    /// Check whether the auto-observe timer should fire.
    ///
    /// Returns `(message_index, content)` if a new completed Coder message
    /// hasn't been observed yet.
    pub fn auto_observe_trigger(&self) -> Option<(usize, String)> {
        if !self.auto_observe { return None; }
        if self.observer.streaming || self.coder.streaming { return None; }

        let (idx, msg) = self.coder.messages
            .iter()
            .enumerate()
            .rev()
            .find(|(_, m)| {
                matches!(m.role, Role::Assistant)
                    && m.complete
                    && m.content.trim().len() > 40
            })?;

        if Some(idx) == self.last_auto_obs_idx { return None; }
        Some((idx, msg.content.clone()))
    }
}
