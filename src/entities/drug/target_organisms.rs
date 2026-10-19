//! Source-supplied organisms for drug targets.

use serde::{Deserialize, Serialize};

use super::Drug;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DrugTargetOrganism {
    pub target: String,
    pub organism: Option<String>,
    pub source: String,
}

impl DrugTargetOrganism {
    pub(crate) fn sort_targets(targets: &mut [String], organisms: &mut [Self]) {
        organisms.sort_by_key(|row| (!row.is_human(), row.target.clone(), row.source.clone()));
        targets.sort_by_key(|target| {
            (
                !organisms
                    .iter()
                    .any(|row| row.target.eq_ignore_ascii_case(target) && row.is_human()),
                target.clone(),
            )
        });
    }

    pub(crate) fn is_human(&self) -> bool {
        self.organism.as_deref().is_some_and(|organism| {
            organism.trim().eq_ignore_ascii_case("Human")
                || organism.trim().eq_ignore_ascii_case("Homo sapiens")
        })
    }
}

impl Drug {
    pub(crate) fn target_is_human(&self, target: &str) -> bool {
        self.target_organisms
            .iter()
            .any(|row| row.target.eq_ignore_ascii_case(target) && row.is_human())
    }

    pub(super) fn record_target_organisms(&mut self, targets: &[String], source: &str) {
        self.target_organisms
            .extend(targets.iter().map(|target| DrugTargetOrganism {
                target: target.clone(),
                organism: None,
                source: source.to_string(),
            }));
    }

    pub(super) fn sort_targets_by_organism(&mut self) {
        DrugTargetOrganism::sort_targets(&mut self.targets, &mut self.target_organisms);
    }

    pub(crate) fn target_display_names(&self) -> Vec<String> {
        self.targets
            .iter()
            .map(|target| {
                let labels = self
                    .target_organisms
                    .iter()
                    .filter(|row| row.target.eq_ignore_ascii_case(target))
                    .map(|row| {
                        format!(
                            "{}; {}",
                            row.organism.as_deref().unwrap_or("organism not supplied"),
                            row.source
                        )
                    })
                    .collect::<Vec<_>>();
                let label = if labels.is_empty() {
                    "organism not supplied".to_string()
                } else {
                    labels.join("; ")
                };
                format!("{target} ({label})")
            })
            .collect()
    }
}
