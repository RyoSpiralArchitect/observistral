use crate::observer::{Proposal, ProposalStatus};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CritiqueMemory {
    /// Title-key -> times seen.
    #[serde(default)]
    pub proposal_counts: BTreeMap<String, u32>,
}

pub fn apply_memory(mem: &mut CritiqueMemory, proposals: &mut [Proposal]) {
    for p in proposals {
        let key = normalize_title_key(&p.title);
        if key.is_empty() {
            continue;
        }

        let seen_prev = mem.proposal_counts.get(&key).copied().unwrap_or(0);
        let seen_now = seen_prev.saturating_add(1);
        mem.proposal_counts.insert(key, seen_now);

        p.status = status_for_seen_count(seen_now);
        let bump: u32 = match p.status {
            ProposalStatus::New | ProposalStatus::Addressed => 0,
            ProposalStatus::Unresolved => 10,
            ProposalStatus::Escalated => 20,
        };
        p.score = p.score.saturating_add(bump).min(100);
    }
}

pub fn normalize_title_key(title: &str) -> String {
    title
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn status_for_seen_count(seen: u32) -> ProposalStatus {
    match seen {
        0 | 1 => ProposalStatus::New,
        2 => ProposalStatus::Unresolved,
        _ => ProposalStatus::Escalated,
    }
}
