//! Trial detail retrieval exposed through the stable trial facade.

use crate::error::BioMcpError;
use crate::sources::clinicaltrials::ClinicalTrialsClient;
use crate::sources::nci_cts::NciCtsClient;
use biodata::{ClinicalTrialSection, NciCtsV2DetailPlan, NciCtsV2DetailResponse};

use super::{
    TRIAL_SECTION_ALL, TRIAL_SECTION_ARMS, TRIAL_SECTION_CONTACTS, TRIAL_SECTION_ELIGIBILITY,
    TRIAL_SECTION_LOCATIONS, TRIAL_SECTION_NAMES, TRIAL_SECTION_OUTCOMES, TRIAL_SECTION_REFERENCES,
    TrialResponse, TrialSectionState, TrialSectionStates, TrialSource,
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

fn section_state<T>(section: &ClinicalTrialSection<T>) -> TrialSectionState {
    super::section_state(match section {
        ClinicalTrialSection::NotRequested => ClinicalTrialSection::NotRequested,
        ClinicalTrialSection::Unavailable => ClinicalTrialSection::Unavailable,
        ClinicalTrialSection::Absent => ClinicalTrialSection::Absent,
        ClinicalTrialSection::Present(_) => ClinicalTrialSection::Present(()),
    })
}

fn product_from_nci_response(
    response: NciCtsV2DetailResponse,
    include_arms: bool,
) -> TrialResponse {
    let arms_state = if include_arms {
        section_state(&response.arms_state())
    } else {
        TrialSectionState::NotRequested
    };
    let outcomes = response.outcomes();
    let states = TrialSectionStates {
        arms: arms_state,
        eligibility: section_state(&response.eligibility()),
        outcomes: section_state(&outcomes),
        references: TrialSectionState::NotRequested,
        contacts: section_state(&response.contacts_state()),
        locations: section_state(&response.locations_state()),
    };
    TrialResponse::new(response.into_projection(), "NCI CTS", None, states)
}

fn product_from_ctgov_response(
    response: biodata::ClinicalTrialsGovApiV2Response,
    section_flags: TrialSections,
) -> Result<TrialResponse, BioMcpError> {
    let reference_state = section_state(response.references());
    let provenance = (section_flags.include_eligibility_provenance
        && matches!(response.eligibility(), ClinicalTrialSection::Present(value) if value.registry_text().is_some()))
        .then(|| super::documents::eligibility_provenance(&response));
    let states = TrialSectionStates {
        arms: section_state(response.arms()),
        eligibility: section_state(response.eligibility()),
        outcomes: section_state(response.outcomes()),
        references: reference_state,
        contacts: section_state(&response.contacts_state()),
        locations: section_state(&response.locations_state()),
    };
    let projection = response
        .into_projection()
        .map_err(|_| BioMcpError::InternalProcessing)?;
    Ok(TrialResponse::new(
        projection,
        "ClinicalTrials.gov",
        provenance,
        states,
    ))
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
) -> Result<TrialResponse, BioMcpError> {
    let nct_id = validated_nct_id(nct_id)?;
    let nct_id = nct_id.as_str();
    let section_flags = parse_sections(sections)?;

    match source {
        TrialSource::ClinicalTrialsGov => {
            let client = ClinicalTrialsClient::new()?;
            let response = client.get_biodata_detail(nct_id, sections).await?;
            product_from_ctgov_response(response, section_flags)
        }
        TrialSource::NciCts => {
            let mut plan = NciCtsV2DetailPlan::new(nct_id, section_flags.request_eligibility)
                .map_err(|_| BioMcpError::InternalProcessing)?;
            if section_flags.include_outcomes {
                plan = plan.with_outcomes();
            }
            if section_flags.include_contacts {
                plan = plan.with_contacts();
            }
            if section_flags.include_locations {
                plan = plan.with_locations();
            }
            let client = NciCtsClient::new()?;
            let response = client.get(&plan).await?;
            let mut trial = product_from_nci_response(response, section_flags.include_arms);
            trial.section_states.references = if section_flags.include_references {
                TrialSectionState::Unavailable
            } else {
                TrialSectionState::NotRequested
            };

            Ok(trial)
        }
    }
}
