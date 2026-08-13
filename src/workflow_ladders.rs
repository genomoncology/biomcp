use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// dead-code reason: some workflow variants are sidecar-only outside test inventory validation
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum Workflow {
    TreatmentLookup,
    ArticleFollowUp,
    VariantPathogenicity,
    TrialRecruitment,
    MechanismPathway,
    PharmacogeneCumulative,
    MutationCatalog,
}

impl Workflow {
    #[cfg(test)]
    pub(crate) const ALL: [Self; 7] = [
        Self::TreatmentLookup,
        Self::ArticleFollowUp,
        Self::VariantPathogenicity,
        Self::TrialRecruitment,
        Self::MechanismPathway,
        Self::PharmacogeneCumulative,
        Self::MutationCatalog,
    ];

    pub(crate) fn slug(self) -> &'static str {
        match self {
            Self::TreatmentLookup => "treatment-lookup",
            Self::ArticleFollowUp => "article-follow-up",
            Self::VariantPathogenicity => "variant-pathogenicity",
            Self::TrialRecruitment => "trial-recruitment",
            Self::MechanismPathway => "mechanism-pathway",
            Self::PharmacogeneCumulative => "pharmacogene-cumulative",
            Self::MutationCatalog => "mutation-catalog",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WorkflowLadder {
    pub(crate) workflow: String,
    pub(crate) rationale: String,
    pub(crate) playbook: String,
    pub(crate) ladder: Vec<WorkflowLadderStep>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct WorkflowLadderStep {
    pub(crate) step: u32,
    pub(crate) command: String,
    pub(crate) what_it_gives: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct WorkflowMeta {
    pub(crate) workflow: String,
    pub(crate) rationale: String,
    pub(crate) playbook: String,
}

pub(crate) fn load(workflow: Workflow) -> Result<WorkflowLadder, BioMcpError> {
    let slug = workflow.slug();
    let path = format!("use-cases/{slug}.ladder.json");
    let ladder: WorkflowLadder = serde_json::from_str(&crate::skill_assets::text(&path)?)?;
    validate_ladder(slug, &ladder)?;
    Ok(ladder)
}

pub(crate) fn meta_for(workflow: Workflow) -> Result<WorkflowMeta, BioMcpError> {
    let ladder = load(workflow)?;
    Ok(WorkflowMeta {
        workflow: ladder.workflow,
        rationale: ladder.rationale,
        playbook: ladder.playbook,
    })
}

fn workflow_asset_error(slug: &str, message: impl Into<String>) -> BioMcpError {
    BioMcpError::InvalidArgument(format!(
        "Invalid workflow ladder sidecar for {slug}: {}",
        message.into()
    ))
}

fn validate_ladder(slug: &str, ladder: &WorkflowLadder) -> Result<(), BioMcpError> {
    if ladder.workflow != slug {
        return Err(workflow_asset_error(
            slug,
            format!("workflow field must be {slug}"),
        ));
    }
    if ladder.playbook != format!("biomcp skill {slug}") {
        return Err(workflow_asset_error(
            slug,
            format!("playbook field must be biomcp skill {slug}"),
        ));
    }
    if ladder.rationale.trim().is_empty() {
        return Err(workflow_asset_error(slug, "rationale must not be empty"));
    }
    if ladder.ladder.is_empty() {
        return Err(workflow_asset_error(slug, "ladder must not be empty"));
    }

    for (index, step) in ladder.ladder.iter().enumerate() {
        let expected = (index + 1) as u32;
        if step.step != expected {
            return Err(workflow_asset_error(
                slug,
                format!("step numbers must be contiguous from 1; expected {expected}"),
            ));
        }
        if step.command.trim().is_empty() {
            return Err(workflow_asset_error(slug, "command must not be empty"));
        }
        if step.what_it_gives.trim().is_empty() {
            return Err(workflow_asset_error(
                slug,
                "what_it_gives must not be empty",
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Workflow, load, meta_for};

    #[test]
    fn every_workflow_ladder_loads_and_validates() {
        for workflow in Workflow::ALL {
            let ladder = load(workflow).expect("workflow ladder should load");
            assert_eq!(ladder.workflow, workflow.slug());
            assert_eq!(ladder.playbook, format!("biomcp skill {}", workflow.slug()));
            assert!(!ladder.ladder.is_empty());
        }
    }

    #[test]
    fn workflow_meta_discards_sidecar_only_fields() {
        let meta = meta_for(Workflow::PharmacogeneCumulative).expect("workflow metadata");
        assert_eq!(meta.workflow, "pharmacogene-cumulative");
        assert!(!meta.rationale.is_empty());
        assert_eq!(meta.playbook, "biomcp skill pharmacogene-cumulative");
    }
}
