//! Temporary product conversion actions with exact admitted or source-only custody.
use super::MyChemHit;
use biodata::{DrugClaimOrigin, DrugClaimValue};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConversionOrigin {
    pub section: String,
    pub field: String,
    pub section_index: Option<usize>,
    pub value_index: Option<usize>,
}
impl From<&DrugClaimOrigin> for ConversionOrigin {
    fn from(origin: &DrugClaimOrigin) -> Self {
        Self {
            section: origin.section().into(),
            field: origin.field().into(),
            section_index: origin.section_index(),
            value_index: origin.value_index(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConversionTermType {
    pub namespace: String,
    pub label: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DrugConversion {
    pub response_digest: String,
    pub ordinal: usize,
    pub claim_index: Option<usize>,
    pub origin: Option<ConversionOrigin>,
    pub lexical_text: Option<String>,
    pub namespace: Option<String>,
    pub source_term_type: Option<ConversionTermType>,
    pub source_only: bool,
    pub source_pointer: Option<String>,
    pub stage: &'static str,
    pub action: &'static str,
    pub reason: &'static str,
    pub target: Option<serde_json::Value>,
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
        let claim = claim_index.and_then(|index| self.row.identity().claims().get(index));
        let (lexical_text, namespace, source_term_type) = match claim.map(|claim| claim.value()) {
            Some(DrugClaimValue::Code(code)) => (
                Some(code.value().into()),
                Some(code.namespace().as_str().into()),
                None,
            ),
            Some(DrugClaimValue::Term(term)) => (
                Some(term.text().into()),
                None,
                term.source_type().map(|source| ConversionTermType {
                    namespace: source.namespace().as_str().into(),
                    label: source.label().as_str().into(),
                }),
            ),
            None => (None, None, None),
        };
        crate::utils::sync::recover_poison(self.conversion.lock()).push(DrugConversion {
            response_digest: self.page.digest().to_owned(),
            ordinal: self.row.source().ordinal(),
            claim_index,
            origin: claim.map(|claim| claim.origin().into()),
            lexical_text,
            namespace,
            source_term_type,
            source_only: false,
            source_pointer: None,
            stage,
            action,
            reason,
            target: target.map(serde_json::Value::String),
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
    pub(crate) fn record_claims(
        &self,
        stage: &'static str,
        action: &'static str,
        reason: &'static str,
    ) {
        for index in 0..self.row.identity().claims().len() {
            self.record(Some(index), stage, action, reason, None);
        }
    }
    /// Record an enrichment occurrence. This never admits or supplies an identity value.
    pub(crate) fn record_source(
        &self,
        pointer: &str,
        stage: &'static str,
        action: &'static str,
        reason: &'static str,
        target: Option<String>,
    ) {
        self.record_source_value(
            pointer,
            stage,
            action,
            reason,
            target.map(serde_json::Value::String),
        );
    }
    pub(crate) fn record_source_value(
        &self,
        pointer: &str,
        stage: &'static str,
        action: &'static str,
        reason: &'static str,
        target: Option<serde_json::Value>,
    ) {
        let Ok(raw) = serde_json::from_str::<serde_json::Value>(self.row.source().raw()) else {
            return;
        };
        let Some(value) = raw.pointer(pointer) else {
            return;
        };
        let parts = pointer
            .trim_start_matches('/')
            .split('/')
            .collect::<Vec<_>>();
        let Some(section) = parts.first() else {
            return;
        };
        let section_index = if matches!(*section, "ndc" | "unii" | "chebi") {
            parts.get(1).and_then(|index| index.parse::<usize>().ok())
        } else {
            None
        };
        let start = if section_index.is_some() { 2 } else { 1 };
        let value_index = parts[start..]
            .iter()
            .find_map(|index| index.parse::<usize>().ok());
        let field = parts[start..]
            .iter()
            .filter(|part| part.parse::<usize>().is_err())
            .copied()
            .collect::<Vec<_>>()
            .join(".");
        let source_term_type = Some(ConversionTermType {
            namespace: "MyChem".into(),
            label: format!("{section}.{field}"),
        });
        crate::utils::sync::recover_poison(self.conversion.lock()).push(DrugConversion {
            response_digest: self.page.digest().into(),
            ordinal: self.row.source().ordinal(),
            claim_index: None,
            origin: Some(ConversionOrigin {
                section: (*section).into(),
                field,
                section_index,
                value_index,
            }),
            lexical_text: Some(
                value
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| value.to_string()),
            ),
            namespace: None,
            source_term_type,
            source_only: true,
            source_pointer: Some(format!("/hits/{}{}", self.row.source().ordinal(), pointer)),
            stage,
            action,
            reason,
            target,
        });
    }
}
