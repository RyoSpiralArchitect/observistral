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
    Tool,   // exec result injected into display
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
    pub scroll: usize,
    pub streaming: bool,
}

impl Pane {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            textarea: TextArea::default(),
            scroll: 0,
            streaming: false,
        }
    }

    /// Append a delta to the current streaming message (last message must have complete=false).
    pub fn push_delta(&mut self, delta: &str) {
        if let Some(last) = self.messages.last_mut() {
            if !last.complete {
                last.content.push_str(delta);
                return;
            }
        }
        // No streaming message in flight — start one.
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

    /// Collect messages as ChatMessage vec for the API (system prepended externally).
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
    pub last_auto_obs_idx: Option<usize>, // index into coder.messages of last observed msg
    pub coder_cfg: RunConfig,
    pub observer_cfg: RunConfig,
    pub tool_root: Option<String>,
    pub quit: bool,
}

impl App {
    pub fn new(coder_cfg: RunConfig, observer_cfg: RunConfig, tool_root: Option<String>, auto_observe: bool) -> Self {
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

    /// Check whether auto-observe should fire.
    /// Returns the content of the last completed Coder assistant message if it should.
    pub fn auto_observe_trigger(&self) -> Option<String> {
        if !self.auto_observe {
            return None;
        }
        if self.observer.streaming || self.coder.streaming {
            return None;
        }
        // Find the index of the last complete assistant message in coder pane.
        let last_idx = self.coder.messages
            .iter()
            .enumerate()
            .rev()
            .find(|(_, m)| {
                matches!(m.role, Role::Assistant) && m.complete && m.content.trim().len() > 40
            })
            .map(|(i, _)| i);

        match last_idx {
            Some(idx) if Some(idx) != self.last_auto_obs_idx => {
                Some(self.coder.messages[idx].content.clone())
            }
            _ => None,
        }
    }
}
