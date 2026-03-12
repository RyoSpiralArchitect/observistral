use crate::observer::{Cost, Proposal, RiskAxis, Severity};

pub fn score_proposal(p: &mut Proposal) {
    let mut score: i32 = 50;

    score += match p.severity {
        Severity::Crit => 30,
        Severity::Warn => 10,
        Severity::Info => 0,
    };

    score += match p.cost {
        Cost::Low => 10,
        Cost::Medium => -5,
        Cost::High => -15,
    };

    if p.axis == Some(RiskAxis::Security) && p.severity != Severity::Info {
        score += 10;
    }

    // Nudge: core fixes are usually earlier/better ROI.
    if matches!(p.phase, crate::observer::DevPhase::Core) && p.severity != Severity::Info {
        score += 5;
    }

    p.score = score.clamp(0, 100) as u32;
}
