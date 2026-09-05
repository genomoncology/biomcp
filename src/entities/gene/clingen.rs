use std::time::Duration;

use tracing::warn;

use super::{GENE_SECTION_CLINGEN, Gene, GeneTimingEntry};
use crate::entities::section_outcome::SectionOutcome;
use crate::error::BioMcpError;
use crate::sources::clingen::{ClinGenClient, ClinGenFamilyState, GeneClinGen};

pub(super) type ClinGenPrefetchOutput = ((GeneClinGen, SectionOutcome), GeneTimingEntry);

pub(super) struct ClinGenPrefetch {
    handle: Option<tokio::task::JoinHandle<ClinGenPrefetchOutput>>,
}

impl ClinGenPrefetch {
    pub(super) fn new(handle: tokio::task::JoinHandle<ClinGenPrefetchOutput>) -> Self {
        Self {
            handle: Some(handle),
        }
    }

    pub(super) async fn settle(mut self) -> Result<ClinGenPrefetchOutput, tokio::task::JoinError> {
        let result = self
            .handle
            .as_mut()
            .expect("ClinGen prefetch handle is owned until settlement")
            .await;
        self.handle.take();
        result
    }

    pub(super) async fn abort_and_wait(mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
            let _ = handle.await;
        }
    }
}

impl Drop for ClinGenPrefetch {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

pub(super) fn classify_section(section: &(GeneClinGen, SectionOutcome)) -> String {
    section.1.outcome().as_str().to_string()
}

pub(super) async fn fetch_section(
    symbol: &str,
    timeout: Duration,
) -> (GeneClinGen, SectionOutcome) {
    fetch_section_with_client(symbol, timeout, ClinGenClient::new()).await
}

async fn fetch_section_with_client(
    symbol: &str,
    timeout: Duration,
    client: Result<ClinGenClient, BioMcpError>,
) -> (GeneClinGen, SectionOutcome) {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        let clingen = GeneClinGen::downloads_failed();
        let outcome = section_outcome(&clingen);
        return (clingen, outcome);
    }

    let client = match client {
        Ok(client) => client,
        Err(err) => {
            warn!(symbol = %symbol, "ClinGen client initialization failed: {err}");
            let clingen = GeneClinGen::client_init_failed();
            let outcome = section_outcome(&clingen);
            return (clingen, outcome);
        }
    };

    match client.gene_context(symbol, timeout).await {
        Ok(clingen) => {
            let outcome = section_outcome(&clingen);
            (clingen, outcome)
        }
        Err(err) => {
            warn!(
                symbol = %symbol,
                "ClinGen unavailable for gene clingen section: {err}"
            );
            let clingen = GeneClinGen::downloads_failed();
            let outcome = section_outcome(&clingen);
            (clingen, outcome)
        }
    }
}

pub(super) fn section_outcome(clingen: &GeneClinGen) -> SectionOutcome {
    let statuses = [&clingen.validity_status, &clingen.dosage_status];
    let has_data = statuses
        .iter()
        .any(|status| status.status == ClinGenFamilyState::Data);
    let has_unavailable = statuses.iter().any(|status| {
        matches!(
            status.status,
            ClinGenFamilyState::Failed | ClinGenFamilyState::TimedOut
        )
    });

    match (has_data, has_unavailable) {
        (true, true) => SectionOutcome::degraded(
            ["ClinGen"],
            "ClinGen gene evidence is partial; one result family is unavailable.",
        ),
        (false, true) => SectionOutcome::unavailable(
            "ClinGen gene evidence is incomplete; no ClinGen absence can be concluded.",
        ),
        (true, false) => SectionOutcome::data("ClinGen"),
        (false, false) => SectionOutcome::empty("ClinGen"),
    }
}

pub(super) async fn add_section(gene: &mut Gene, timeout: Duration) {
    let (clingen, outcome) = fetch_section(&gene.symbol, timeout).await;
    gene.clingen = Some(clingen);
    gene.section_outcomes
        .complete(GENE_SECTION_CLINGEN, outcome);
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::*;
    use crate::entities::section_outcome::SectionOutcomeState;
    use crate::sources::clingen::{ClinGenFamilyStatus, ClinGenOperation, DOSAGE_FAILED_MESSAGE};

    #[test]
    fn family_statuses_define_the_exact_aggregate_truth_table() {
        let cases = [
            (
                ClinGenFamilyStatus::data(ClinGenOperation::GeneValidityDownload),
                ClinGenFamilyStatus::empty(ClinGenOperation::GeneDosageDownload),
                SectionOutcomeState::Data,
                vec!["ClinGen"],
                None,
            ),
            (
                ClinGenFamilyStatus::empty(ClinGenOperation::GeneValidityDownload),
                ClinGenFamilyStatus::empty(ClinGenOperation::GeneDosageDownload),
                SectionOutcomeState::Empty,
                vec!["ClinGen"],
                None,
            ),
            (
                ClinGenFamilyStatus::data(ClinGenOperation::GeneValidityDownload),
                ClinGenFamilyStatus::failed(
                    ClinGenOperation::GeneDosageDownload,
                    DOSAGE_FAILED_MESSAGE,
                ),
                SectionOutcomeState::Degraded,
                vec!["ClinGen"],
                Some("ClinGen gene evidence is partial; one result family is unavailable."),
            ),
            (
                ClinGenFamilyStatus::empty(ClinGenOperation::GeneValidityDownload),
                ClinGenFamilyStatus::failed(
                    ClinGenOperation::GeneDosageDownload,
                    DOSAGE_FAILED_MESSAGE,
                ),
                SectionOutcomeState::Unavailable,
                Vec::new(),
                Some("ClinGen gene evidence is incomplete; no ClinGen absence can be concluded."),
            ),
        ];

        for (validity_status, dosage_status, state, sources, message) in cases {
            let clingen = GeneClinGen {
                validity: Vec::new(),
                haploinsufficiency: None,
                triplosensitivity: None,
                validity_status,
                dosage_status,
            };
            let outcome = section_outcome(&clingen);
            assert_eq!(outcome.outcome(), state);
            assert_eq!(outcome.sources(), sources);
            assert_eq!(outcome.message(), message);
        }
    }

    #[tokio::test]
    async fn client_construction_failure_marks_both_families_before_requests() {
        let (clingen, outcome) = fetch_section_with_client(
            "TP53",
            Duration::from_millis(10),
            Err(BioMcpError::InvalidArgument(
                "synthetic client construction failure".to_string(),
            )),
        )
        .await;

        assert_eq!(
            serde_json::to_value(clingen).unwrap(),
            serde_json::json!({
                "validity_status": {
                    "status": "failed",
                    "op": "client_init",
                    "message": "ClinGen client initialization failed."
                },
                "dosage_status": {
                    "status": "failed",
                    "op": "client_init",
                    "message": "ClinGen client initialization failed."
                }
            })
        );
        assert_eq!(outcome.outcome(), SectionOutcomeState::Unavailable);
        assert!(outcome.sources().is_empty());
    }

    struct RunningGuard(Arc<AtomicBool>);

    impl Drop for RunningGuard {
        fn drop(&mut self) {
            self.0.store(false, Ordering::SeqCst);
        }
    }

    fn pending_prefetch(running: Arc<AtomicBool>) -> ClinGenPrefetch {
        let handle = tokio::spawn(async move {
            running.store(true, Ordering::SeqCst);
            let _guard = RunningGuard(running);
            std::future::pending::<ClinGenPrefetchOutput>().await
        });
        ClinGenPrefetch::new(handle)
    }

    async fn assert_parent_cancel_stops_child<T>(
        running: Arc<AtomicBool>,
        parent: tokio::task::JoinHandle<T>,
    ) {
        for _ in 0..20 {
            if running.load(Ordering::SeqCst) {
                break;
            }
            tokio::task::yield_now().await;
        }
        assert!(running.load(Ordering::SeqCst));
        parent.abort();
        let _ = parent.await;
        for _ in 0..20 {
            if !running.load(Ordering::SeqCst) {
                break;
            }
            tokio::task::yield_now().await;
        }
        assert!(!running.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn canceling_parent_aborts_its_clingen_prefetch_task() {
        let running = Arc::new(AtomicBool::new(false));
        let child_running = Arc::clone(&running);
        let parent = tokio::spawn(async move {
            let _prefetch = pending_prefetch(child_running);
            std::future::pending::<()>().await;
        });

        assert_parent_cancel_stops_child(running, parent).await;
    }

    #[tokio::test]
    async fn canceling_during_settle_aborts_its_clingen_prefetch_task() {
        let running = Arc::new(AtomicBool::new(false));
        let child_running = Arc::clone(&running);
        let parent = tokio::spawn(async move { pending_prefetch(child_running).settle().await });

        assert_parent_cancel_stops_child(running, parent).await;
    }
}
