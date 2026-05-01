use crate::observer::{Proposal, ProposalStatus, Risk, RiskAxis, Severity};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const RISK_RECURRING_UNRESOLVED_PROPOSAL: &str = "Recurring unresolved Observer proposal";

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

pub fn recurring_unresolved_risks(
    mem: &CritiqueMemory,
    candidate_proposals: &[Proposal],
) -> Vec<Risk> {
    let mut seen_keys: BTreeSet<String> = BTreeSet::new();
    let mut recurring: Vec<(String, u32, ProposalStatus)> = Vec::new();

    for proposal in candidate_proposals {
        let key = normalize_title_key(&proposal.title);
        if key.is_empty() || !seen_keys.insert(key.clone()) {
            continue;
        }
        let seen_prev = mem.proposal_counts.get(&key).copied().unwrap_or(0);
        if seen_prev == 0 {
            continue;
        }
        let seen_now = seen_prev.saturating_add(1);
        recurring.push((
            proposal.title.trim().to_string(),
            seen_prev,
            status_for_seen_count(seen_now),
        ));
    }

    if recurring.is_empty() {
        return Vec::new();
    }

    recurring.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let severity = if recurring
        .iter()
        .any(|(_, _, status)| *status == ProposalStatus::Escalated)
    {
        Severity::Crit
    } else {
        Severity::Warn
    };
    let evidence = recurring
        .iter()
        .take(3)
        .map(|(title, seen_prev, status)| {
            format!(
                "proposal: {title}\nseen_before: {seen_prev}\nprojected_status: {}",
                proposal_status_label(*status)
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    vec![Risk {
        axis: RiskAxis::Reliability,
        severity,
        description: RISK_RECURRING_UNRESOLVED_PROPOSAL.to_string(),
        evidence: Some(evidence),
    }]
}

pub fn is_recurring_unresolved_proposal(description: &str) -> bool {
    description == RISK_RECURRING_UNRESOLVED_PROPOSAL
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

fn proposal_status_label(status: ProposalStatus) -> &'static str {
    match status {
        ProposalStatus::New => "new",
        ProposalStatus::Unresolved => "unresolved",
        ProposalStatus::Escalated => "escalated",
        ProposalStatus::Addressed => "addressed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observer::{Cost, DevPhase};

    fn proposal(title: &str) -> Proposal {
        Proposal {
            title: title.to_string(),
            to_coder: format!("Fix {title}."),
            severity: Severity::Warn,
            score: 0,
            phase: DevPhase::Core,
            impact: String::new(),
            cost: Cost::Low,
            status: ProposalStatus::New,
            quote: "n/a".to_string(),
            axis: Some(RiskAxis::Reliability),
        }
    }

    #[test]
    fn recurring_unresolved_risks_surface_seen_proposal_memory() {
        let mut mem = CritiqueMemory::default();
        mem.proposal_counts
            .insert(normalize_title_key("Refresh runtime eval proof"), 1);

        let risks = recurring_unresolved_risks(
            &mem,
            &[
                proposal("Refresh runtime eval proof"),
                proposal("Run verification after edits"),
            ],
        );

        assert_eq!(risks.len(), 1);
        assert!(is_recurring_unresolved_proposal(
            risks[0].description.as_str()
        ));
        assert_eq!(risks[0].severity, Severity::Warn);
        assert!(risks[0]
            .evidence
            .as_deref()
            .unwrap_or("")
            .contains("projected_status: unresolved"));
    }

    #[test]
    fn recurring_unresolved_risks_escalate_after_multiple_prior_sightings() {
        let mut mem = CritiqueMemory::default();
        mem.proposal_counts
            .insert(normalize_title_key("Refresh runtime eval proof"), 2);

        let risks = recurring_unresolved_risks(&mem, &[proposal("Refresh runtime eval proof")]);

        assert_eq!(risks[0].severity, Severity::Crit);
        assert!(risks[0]
            .evidence
            .as_deref()
            .unwrap_or("")
            .contains("projected_status: escalated"));
    }
}
