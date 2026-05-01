#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeGateReviewDecision {
    Approved,
    Held,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeGateBoardStatus {
    NeedsReview,
    Approved,
    Held,
    RollbackAvailable,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MergeGateReview {
    pub decision: MergeGateReviewDecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MergeGateCase<'a> {
    pub id: &'a str,
    pub status: &'a str,
    pub rollback_command: Option<&'a str>,
    pub review: Option<MergeGateReview>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeGateBoardEntry<'a> {
    pub id: &'a str,
    pub review_status: MergeGateBoardStatus,
    pub can_approve: bool,
    pub can_preview_rollback: bool,
}

pub fn board_entry_for_case<'a>(case: &'a MergeGateCase<'a>) -> MergeGateBoardEntry<'a> {
    let review_status = status_for_case(case);
    MergeGateBoardEntry {
        id: case.id,
        review_status,
        can_approve: case.status == "passed"
            && !matches!(review_status, MergeGateBoardStatus::Approved),
        can_preview_rollback: case.rollback_command.is_some(),
    }
}

fn status_for_case(case: &MergeGateCase<'_>) -> MergeGateBoardStatus {
    if let Some(review) = case.review {
        return match review.decision {
            MergeGateReviewDecision::Approved => MergeGateBoardStatus::Approved,
            MergeGateReviewDecision::Held => MergeGateBoardStatus::Held,
        };
    }
    if case.status == "passed" {
        MergeGateBoardStatus::NeedsReview
    } else if case.rollback_command.is_some() {
        MergeGateBoardStatus::RollbackAvailable
    } else {
        MergeGateBoardStatus::Blocked
    }
}

#[cfg(test)]
mod tests {
    use super::{
        board_entry_for_case, MergeGateBoardStatus, MergeGateCase, MergeGateReview,
        MergeGateReviewDecision,
    };

    #[test]
    fn approved_passing_case_stays_approved() {
        let entry = board_entry_for_case(&MergeGateCase {
            id: "green-case",
            status: "passed",
            rollback_command: None,
            review: Some(MergeGateReview {
                decision: MergeGateReviewDecision::Approved,
            }),
        });

        assert_eq!(entry.review_status, MergeGateBoardStatus::Approved);
        assert!(!entry.can_approve);
        assert!(!entry.can_preview_rollback);
    }

    #[test]
    fn held_passing_case_stays_held() {
        let entry = board_entry_for_case(&MergeGateCase {
            id: "held-case",
            status: "passed",
            rollback_command: None,
            review: Some(MergeGateReview {
                decision: MergeGateReviewDecision::Held,
            }),
        });

        assert_eq!(entry.review_status, MergeGateBoardStatus::Held);
        assert!(entry.can_approve);
    }

    #[test]
    fn stale_approval_cannot_approve_failed_case_with_rollback() {
        let entry = board_entry_for_case(&MergeGateCase {
            id: "failed-with-rollback",
            status: "failed",
            rollback_command: Some("git -C repo reset --hard abc123"),
            review: Some(MergeGateReview {
                decision: MergeGateReviewDecision::Approved,
            }),
        });

        assert_eq!(entry.review_status, MergeGateBoardStatus::RollbackAvailable);
        assert!(!entry.can_approve);
        assert!(entry.can_preview_rollback);
    }

    #[test]
    fn stale_approval_cannot_approve_failed_case_without_rollback() {
        let entry = board_entry_for_case(&MergeGateCase {
            id: "failed-without-rollback",
            status: "failed",
            rollback_command: None,
            review: Some(MergeGateReview {
                decision: MergeGateReviewDecision::Approved,
            }),
        });

        assert_eq!(entry.review_status, MergeGateBoardStatus::Blocked);
        assert!(!entry.can_approve);
        assert!(!entry.can_preview_rollback);
    }
}
