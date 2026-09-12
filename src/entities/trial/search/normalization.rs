//! Product status ranking. BioData owns provider filter normalization.
use super::super::TrialSearchHit;

fn status_priority(value: &str) -> u8 {
    match value
        .trim()
        .to_ascii_uppercase()
        .replace([' ', '-'], "_")
        .as_str()
    {
        "RECRUITING" => 0,
        "ACTIVE_NOT_RECRUITING" => 1,
        "ENROLLING_BY_INVITATION" => 2,
        "NOT_YET_RECRUITING" => 3,
        "COMPLETED" => 4,
        "UNKNOWN" => 5,
        "WITHDRAWN" => 6,
        "TERMINATED" => 7,
        "SUSPENDED" => 8,
        _ => 9,
    }
}

pub(super) fn sort_trials_by_status_priority(rows: &mut [TrialSearchHit]) {
    rows.sort_by(|a, b| {
        status_priority(a.status())
            .cmp(&status_priority(b.status()))
            .then_with(|| a.nct_id().cmp(b.nct_id()))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn status_priority_remains_product_policy() {
        assert!(status_priority("recruiting") < status_priority("completed"));
        assert!(status_priority("completed") < status_priority("terminated"));
    }
}
