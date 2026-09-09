//! Trial detail retrieval exposed through the stable trial facade.

use super::shared::{
    ClinicalTrialArms, ClinicalTrialEligibility, ClinicalTrialIntervention, ClinicalTrialReference,
    ClinicalTrialSection,
};
use crate::error::BioMcpError;
use crate::sources::clinicaltrials::ClinicalTrialsClient;
use crate::sources::nci_cts::{NciCtsClient, NciCtsV2DetailPlan, NciCtsV2DetailResponse};
use crate::transform;

use super::{
    TRIAL_SECTION_ALL, TRIAL_SECTION_ARMS, TRIAL_SECTION_CONTACTS, TRIAL_SECTION_ELIGIBILITY,
    TRIAL_SECTION_LOCATIONS, TRIAL_SECTION_NAMES, TRIAL_SECTION_OUTCOMES, TRIAL_SECTION_REFERENCES,
    Trial, TrialDesign, TrialSource,
};

#[derive(Debug, Clone, Copy, Default)]
struct TrialSections {
    request_eligibility: bool,
    include_eligibility_provenance: bool,
    include_contacts: bool,
    include_locations: bool,
    include_outcomes: bool,
    include_arms: bool,
    include_references: bool,
}

fn parse_sections(sections: &[String]) -> Result<TrialSections, BioMcpError> {
    let is_default = !sections.iter().any(|value| {
        let value = value.trim();
        !value.is_empty() && value != "--json" && value != "-j"
    });
    let mut out = TrialSections {
        request_eligibility: is_default,
        ..TrialSections::default()
    };
    let mut include_all = false;

    for raw in sections {
        let section = raw.trim().to_ascii_lowercase();
        if section.is_empty() {
            continue;
        }
        if section == "--json" || section == "-j" {
            continue;
        }
        match section.as_str() {
            TRIAL_SECTION_ELIGIBILITY => {
                out.request_eligibility = true;
                out.include_eligibility_provenance = true;
            }
            TRIAL_SECTION_CONTACTS => out.include_contacts = true,
            TRIAL_SECTION_LOCATIONS => out.include_locations = true,
            TRIAL_SECTION_OUTCOMES => out.include_outcomes = true,
            TRIAL_SECTION_ARMS => out.include_arms = true,
            TRIAL_SECTION_REFERENCES => out.include_references = true,
            TRIAL_SECTION_ALL => include_all = true,
            _ => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown section \"{section}\" for trial. Available: {}",
                    TRIAL_SECTION_NAMES.join(", ")
                )));
            }
        }
    }

    if include_all {
        out.request_eligibility = true;
        out.include_contacts = true;
        out.include_locations = true;
        out.include_outcomes = true;
        out.include_arms = true;
        out.include_references = true;
    }

    Ok(out)
}

fn product_references(
    section: ClinicalTrialSection<Vec<ClinicalTrialReference>>,
) -> Result<Vec<ClinicalTrialReference>, BioMcpError> {
    match section {
        ClinicalTrialSection::Present(values) => Ok(values),
        ClinicalTrialSection::Absent => Ok(Vec::new()),
        ClinicalTrialSection::NotRequested | ClinicalTrialSection::Unavailable => {
            Err(BioMcpError::InternalProcessing)
        }
    }
}

pub(crate) fn product_design(
    interventions: &ClinicalTrialSection<Vec<ClinicalTrialIntervention>>,
    arms: &ClinicalTrialSection<ClinicalTrialArms>,
) -> Result<TrialDesign, BioMcpError> {
    let interventions = match interventions {
        ClinicalTrialSection::Present(values) => values.clone(),
        ClinicalTrialSection::Absent => Vec::new(),
        ClinicalTrialSection::NotRequested | ClinicalTrialSection::Unavailable => {
            return Err(BioMcpError::InternalProcessing);
        }
    };
    let (arms, assignments) = match arms {
        ClinicalTrialSection::Present(value) => (
            Some(value.arms().to_vec()),
            Some(value.assignments().to_vec()),
        ),
        ClinicalTrialSection::Absent | ClinicalTrialSection::NotRequested => (None, None),
        ClinicalTrialSection::Unavailable => return Err(BioMcpError::InternalProcessing),
    };
    TrialDesign::new(interventions, arms, assignments).map_err(BioMcpError::TrialDesign)
}

fn product_nci_design(
    response: &NciCtsV2DetailResponse,
    include_arms: bool,
) -> Result<TrialDesign, BioMcpError> {
    let interventions = response.interventions.clone();
    let (arms, assignments) = if include_arms {
        (
            Some(response.arms.clone()),
            Some(response.assignments.clone()),
        )
    } else {
        (None, None)
    };
    TrialDesign::new(interventions, arms, assignments).map_err(BioMcpError::TrialDesign)
}

fn product_eligibility(
    section: ClinicalTrialSection<&ClinicalTrialEligibility>,
    requested: bool,
) -> Result<Option<ClinicalTrialEligibility>, BioMcpError> {
    match (requested, section) {
        (true, ClinicalTrialSection::Present(value)) => Ok(Some(value.clone())),
        (true, ClinicalTrialSection::Absent) | (false, _) => Ok(None),
        (true, ClinicalTrialSection::NotRequested | ClinicalTrialSection::Unavailable) => {
            Err(BioMcpError::InternalProcessing)
        }
    }
}

fn product_from_nci_response(
    plan: &NciCtsV2DetailPlan,
    response: &NciCtsV2DetailResponse,
    request_eligibility: bool,
    include_arms: bool,
) -> Result<Trial, BioMcpError> {
    let eligibility = product_eligibility(response.eligibility(), request_eligibility)?;

    Ok(Trial {
        nct_id: plan.requested_identity().to_string(),
        source: Some("NCI CTS".to_string()),
        title: response.title.clone(),
        status: response.status.clone(),
        why_stopped: Some(
            response
                .stop_reason
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned),
        ),
        phase: response.phase.clone(),
        study_type: Some(response.study_type.clone()),
        conditions: response.conditions.clone(),
        design: product_nci_design(response, include_arms)?,
        sponsor: Some(response.sponsor.clone()),
        enrollment: response
            .enrollment
            .and_then(|value| i32::try_from(value).ok()),
        summary: response
            .summary
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned),
        start_date: response.start_date.clone(),
        completion_date: response.completion_date.clone(),
        eligibility,
        eligibility_provenance: None,
        contacts: None,
        locations: None,
        outcomes: None,
        references: None,
    })
}

fn looks_like_nct_id(value: &str) -> bool {
    let v = value.trim().as_bytes();
    if v.len() != 11 {
        return false;
    }
    if &v[0..3] != b"NCT" {
        return false;
    }
    v[3..].iter().all(|b| b.is_ascii_digit())
}

fn normalize_nct_id(value: &str) -> String {
    let trimmed = value.trim();
    if let Some(prefix) = trimmed.get(..3)
        && prefix.eq_ignore_ascii_case("NCT")
    {
        return format!("NCT{}", &trimmed[3..]);
    }
    trimmed.to_string()
}

pub(super) fn validated_nct_id(value: &str) -> Result<String, BioMcpError> {
    let nct_id = normalize_nct_id(value);
    let nct_id = nct_id.trim();
    if nct_id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "NCT ID is required. Example: biomcp get trial NCT02576665".into(),
        ));
    }
    if nct_id.len() > 64 {
        return Err(BioMcpError::InvalidArgument("NCT ID is too long.".into()));
    }
    if !looks_like_nct_id(nct_id) {
        return Err(BioMcpError::InvalidArgument(format!(
            "Expected an NCT ID like NCT02576665 (got '{nct_id}')"
        )));
    }
    Ok(nct_id.to_string())
}

pub async fn get(
    nct_id: &str,
    sections: &[String],
    source: TrialSource,
) -> Result<Trial, BioMcpError> {
    let nct_id = validated_nct_id(nct_id)?;
    let nct_id = nct_id.as_str();
    let section_flags = parse_sections(sections)?;

    match source {
        TrialSource::ClinicalTrialsGov => {
            let client = ClinicalTrialsClient::new()?;
            let response = client.get_detail(nct_id, sections).await?;
            let mut study = response.study;
            if let Some(protocol) = study.protocol_section.as_mut() {
                protocol.arms_interventions_module = None;
            }
            let mut trial = transform::trial::from_ctgov_study(&study)?;
            trial.design = product_design(response.shared.interventions(), response.shared.arms())?;
            if section_flags.include_references {
                trial.references = Some(product_references(response.shared.references().clone())?);
            }
            trial.source = Some("ClinicalTrials.gov".into());
            if !section_flags.include_contacts {
                trial.contacts = None;
            }
            trial.eligibility = match (
                section_flags.request_eligibility,
                response.shared.eligibility(),
            ) {
                (true, ClinicalTrialSection::Present(value)) => Some(value.clone()),
                (true, ClinicalTrialSection::Absent) | (false, _) => None,
                (true, ClinicalTrialSection::NotRequested | ClinicalTrialSection::Unavailable) => {
                    return Err(BioMcpError::InternalProcessing);
                }
            };
            if !section_flags.include_locations {
                trial.locations = None;
            }

            if section_flags.include_eligibility_provenance
                && trial
                    .eligibility
                    .as_ref()
                    .and_then(ClinicalTrialEligibility::registry_text)
                    .is_some()
            {
                trial.eligibility_provenance =
                    Some(super::documents::eligibility_provenance(nct_id, &study));
            }
            if section_flags.include_references && trial.references.is_none() {
                trial.references = Some(Vec::new());
            }

            Ok(trial)
        }
        TrialSource::NciCts => {
            let plan = NciCtsV2DetailPlan::new(nct_id, section_flags.request_eligibility)
                .map_err(|_| BioMcpError::InternalProcessing)?;
            let client = NciCtsClient::new()?;
            let response = client.get(&plan).await?;
            let mut trial = product_from_nci_response(
                &plan,
                &response,
                section_flags.request_eligibility,
                section_flags.include_arms,
            )?;
            if section_flags.include_references && trial.references.is_none() {
                trial.references = Some(Vec::new());
            }

            Ok(trial)
        }
    }
}

#[cfg(test)]
mod tests;
