//! Actual product actions refer to retained claim occurrences.
use super::MyChemHit;
use biodata::DrugClaimOrigin;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrugConversion {
    pub response_digest: String,
    pub ordinal: usize,
    pub claim_index: Option<usize>,
    pub origin: Option<DrugClaimOrigin>,
    pub stage: &'static str,
    pub action: &'static str,
    pub reason: &'static str,
    pub target: Option<String>,
}
impl MyChemHit {
    pub(crate) fn record(
        &self,
        claim_index: Option<usize>,
        stage: &'static str,
        action: &'static str,
        reason: &'static str,
        target: Option<String>,
    ) {
        let origin = claim_index
            .and_then(|index| self.row.identity().claims().get(index))
            .map(|claim| claim.origin().clone());
        crate::utils::sync::recover_poison(self.conversion.lock()).push(DrugConversion {
            response_digest: self.page.digest().to_owned(),
            ordinal: self.row.source().ordinal(),
            claim_index,
            origin,
            stage,
            action,
            reason,
            target,
        });
    }
    pub(crate) fn record_field(
        &self,
        section: &str,
        field: &str,
        stage: &'static str,
        action: &'static str,
        reason: &'static str,
        target: Option<String>,
    ) {
        let index = self.row.identity().claims().iter().position(|claim| {
            claim.origin().section() == section && claim.origin().field() == field
        });
        if let Some(index) = index {
            self.record(Some(index), stage, action, reason, target);
        }
    }
    pub(crate) fn record_row(
        &self,
        stage: &'static str,
        action: &'static str,
        reason: &'static str,
    ) {
        self.record(None, stage, action, reason, None);
    }
}
