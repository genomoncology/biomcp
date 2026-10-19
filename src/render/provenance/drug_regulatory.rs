//! Regulatory status reflects evidence, empty answers, and failed components.

use super::{SectionOutcomeState, SectionSource, normalize_sources};
use crate::entities::drug::Drug;
use crate::sources::fda_orphan::{FDA_ORPHAN_SOURCE, FdaOrphanOutcome};

pub(super) fn section_source(drug: &Drug) -> Option<SectionSource> {
    let mut contributors = Vec::new();
    let mut successful = Vec::new();
    let mut failed = false;
    let mut requested = false;
    if let Some(approvals) = drug.section_outcomes.get("approvals") {
        match approvals.outcome() {
            SectionOutcomeState::Data | SectionOutcomeState::Empty => {
                requested = true;
                successful.extend_from_slice(approvals.sources());
                if approvals.outcome() == SectionOutcomeState::Data {
                    contributors.extend_from_slice(approvals.sources());
                }
            }
            SectionOutcomeState::NotRequested => {}
            _ => {
                requested = true;
                failed = true;
            }
        }
    }
    if let Some(orphan) = &drug.fda_orphan_designations {
        requested = true;
        failed |= matches!(
            orphan.outcome,
            FdaOrphanOutcome::Unavailable | FdaOrphanOutcome::Degraded
        );
        if orphan.outcome != FdaOrphanOutcome::Unavailable {
            successful.push(FDA_ORPHAN_SOURCE.to_string());
            if !orphan.records.is_empty() {
                contributors.push(FDA_ORPHAN_SOURCE.to_string());
            }
        }
    }
    for (source, populated) in [
        (
            "EMA",
            drug.ema_regulatory.as_ref().map(|rows| !rows.is_empty()),
        ),
        (
            "WHO Prequalification",
            drug.who_prequalification
                .as_ref()
                .map(|rows| !rows.is_empty()),
        ),
    ] {
        if let Some(populated) = populated {
            requested = true;
            successful.push(source.to_string());
            if populated {
                contributors.push(source.to_string());
            }
        }
    }
    if drug.who_note.is_some() {
        requested = true;
        failed = true;
    }
    if !requested {
        return None;
    }
    let (outcome, sources) = if failed && contributors.is_empty() {
        (SectionOutcomeState::Unavailable, Vec::new())
    } else if failed {
        (SectionOutcomeState::Degraded, contributors)
    } else if contributors.is_empty() {
        (SectionOutcomeState::Empty, successful)
    } else {
        (SectionOutcomeState::Data, contributors)
    };
    SectionSource {
        key: "regulatory".to_string(),
        label: "Regulatory".to_string(),
        outcome,
        sources: normalize_sources(sources),
    }
    .normalized()
}
