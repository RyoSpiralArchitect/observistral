#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeReviewDecision {
    Approved,
    Held,
    Pending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrReadyMergeStatus {
    PrReadyMergeApproved,
    HumanApprovalPending,
    Blocked,
}

impl PrReadyMergeStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::PrReadyMergeApproved => "PR-ready merge approved",
            Self::HumanApprovalPending => "human approval pending",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MergeGateReview<'a> {
    pub case_id: &'a str,
    pub case_status: &'a str,
    pub decision: MergeReviewDecision,
}

pub fn pr_ready_handoff_status(review: Option<&MergeGateReview<'_>>) -> PrReadyMergeStatus {
    let Some(review) = review else {
        return PrReadyMergeStatus::HumanApprovalPending;
    };
    if review.case_status != "passed" {
        return PrReadyMergeStatus::HumanApprovalPending;
    }
    match review.decision {
        MergeReviewDecision::Approved => PrReadyMergeStatus::PrReadyMergeApproved,
        MergeReviewDecision::Held | MergeReviewDecision::Pending => {
            PrReadyMergeStatus::HumanApprovalPending
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        pr_ready_handoff_status, MergeGateReview, MergeReviewDecision, PrReadyMergeStatus,
    };

    #[test]
    fn approved_passing_merge_gate_is_pr_ready_handoff() {
        let review = MergeGateReview {
            case_id: "self-fix-pr-ready-merge-approval",
            case_status: "passed",
            decision: MergeReviewDecision::Approved,
        };

        let status = pr_ready_handoff_status(Some(&review));

        assert_eq!(status, PrReadyMergeStatus::PrReadyMergeApproved);
        assert_eq!(status.label(), "PR-ready merge approved");
    }

    #[test]
    fn missing_approval_stays_human_pending() {
        assert_eq!(
            pr_ready_handoff_status(None),
            PrReadyMergeStatus::HumanApprovalPending
        );
    }

    #[test]
    fn held_passing_merge_gate_stays_human_pending() {
        let review = MergeGateReview {
            case_id: "self-fix-pr-ready-merge-approval",
            case_status: "passed",
            decision: MergeReviewDecision::Held,
        };

        assert_eq!(
            pr_ready_handoff_status(Some(&review)),
            PrReadyMergeStatus::HumanApprovalPending
        );
    }

    #[test]
    fn failed_case_is_blocked_even_if_approved() {
        let review = MergeGateReview {
            case_id: "self-fix-pr-ready-merge-approval",
            case_status: "failed",
            decision: MergeReviewDecision::Approved,
        };

        assert_eq!(
            pr_ready_handoff_status(Some(&review)),
            PrReadyMergeStatus::Blocked
        );
    }
}
