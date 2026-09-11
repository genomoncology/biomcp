//! Trial detail retrieval exposed through the stable trial facade.

use crate::error::BioMcpError;
use crate::sources::clinicaltrials::ClinicalTrialsClient;
use crate::sources::nci_cts::NciCtsClient;
use biodata::{
    ClinicalTrialArms, ClinicalTrialEligibility, ClinicalTrialIntervention,
    ClinicalTrialPlannedOutcome, ClinicalTrialSection, NciCtsV2DetailPlan, NciCtsV2DetailResponse,
};

use super::{
    TRIAL_SECTION_ALL, TRIAL_SECTION_ARMS, TRIAL_SECTION_CONTACTS, TRIAL_SECTION_ELIGIBILITY,
    TRIAL_SECTION_LOCATIONS, TRIAL_SECTION_NAMES, TRIAL_SECTION_OUTCOMES, TRIAL_SECTION_REFERENCES,
    Trial, TrialDesign, TrialIdentity, TrialResponse, TrialSectionState, TrialSectionStates,
    TrialSource,
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
    match section {
        ClinicalTrialSection::NotRequested => TrialSectionState::NotRequested,
        ClinicalTrialSection::Unavailable => TrialSectionState::Unavailable,
        ClinicalTrialSection::Absent => TrialSectionState::Absent,
        ClinicalTrialSection::Present(_) => TrialSectionState::Present,
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
    shared: &biodata::ClinicalTrial,
    include_arms: bool,
) -> Result<TrialDesign, BioMcpError> {
    let interventions = shared.interventions().unwrap_or_default().to_vec();
    let (arms, assignments) = if include_arms {
        (
            shared.arms().map(<[_]>::to_vec),
            shared.arm_intervention_assignments().map(<[_]>::to_vec),
        )
    } else {
        (None, None)
    };
    TrialDesign::new(interventions, arms, assignments).map_err(BioMcpError::TrialDesign)
}

fn product_from_core(
    core: &biodata::ClinicalTrialCore,
    source: &str,
    design: TrialDesign,
) -> Trial {
    let identities = core
        .identities()
        .iter()
        .map(|value| TrialIdentity {
            authority: value.authority().to_owned(),
            identifier: value.identifier().to_owned(),
        })
        .collect::<Vec<_>>();
    let nct_id = identities
        .iter()
        .find(|value| value.authority == "clinicaltrials.gov")
        .map(|value| value.identifier.clone())
        .unwrap_or_default();
    let phases = core
        .phases()
        .iter()
        .map(|value| value.code().to_owned())
        .collect::<Vec<_>>();
    let phase = (!phases.is_empty()).then(|| phases.join("/"));
    Trial {
        identities,
        nct_id,
        source: Some(source.to_owned()),
        title: core.brief_title().to_owned(),
        official_title: core.official_title().map(str::to_owned),
        status: core.overall_status().code().to_owned(),
        why_stopped: Some(core.stop_reason().map(str::to_owned)),
        phase,
        phases,
        study_type: Some(core.study_type().code().to_owned()),
        conditions: core.conditions().to_vec(),
        design,
        sponsor: Some(core.lead_sponsor_name().to_owned()),
        enrollment: core.enrollment_count(),
        summary: core.brief_summary().map(str::to_owned),
        start_date: core.start_date().map(str::to_owned),
        completion_date: core.completion_date().map(str::to_owned),
        eligibility: None,
        eligibility_provenance: None,
        site_directory: None,
        site_offset: 0,
        site_limit: None,
        outcomes: None,
        references: None,
    }
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

fn product_outcomes<T>(
    section: &ClinicalTrialSection<T>,
) -> Option<Vec<ClinicalTrialPlannedOutcome>>
where
    T: AsRef<[ClinicalTrialPlannedOutcome]>,
{
    match section {
        ClinicalTrialSection::Present(values) => Some(values.as_ref().to_vec()),
        ClinicalTrialSection::NotRequested
        | ClinicalTrialSection::Unavailable
        | ClinicalTrialSection::Absent => None,
    }
}

fn product_from_nci_response(
    response: &NciCtsV2DetailResponse,
    request_eligibility: bool,
    include_arms: bool,
) -> Result<TrialResponse, BioMcpError> {
    let shared = response.projection().trial();
    let eligibility = product_eligibility(response.eligibility(), request_eligibility)?;
    let arms_state = section_state(&response.arms_state());
    let mut trial = product_from_core(
        response.core(),
        "NCI CTS",
        product_nci_design(shared, include_arms)?,
    );
    trial.eligibility = eligibility;
    let outcomes = response.outcomes();
    trial.outcomes = product_outcomes(&outcomes);
    if let ClinicalTrialSection::Present(directory) = response.site_directory() {
        trial.set_site_directory(Some(directory.clone()));
    }
    Ok(TrialResponse {
        trial,
        section_states: TrialSectionStates {
            arms: arms_state,
            eligibility: section_state(&response.eligibility()),
            outcomes: section_state(&outcomes),
            references: TrialSectionState::NotRequested,
            contacts: section_state(&response.contacts_state()),
            locations: section_state(&response.locations_state()),
        },
    })
}

fn product_from_ctgov_response(
    response: biodata::ClinicalTrialsGovApiV2Response,
    section_flags: TrialSections,
) -> Result<TrialResponse, BioMcpError> {
    let mut trial = product_from_core(
        response.core(),
        "ClinicalTrials.gov",
        product_design(response.interventions(), response.arms())?,
    );
    if let ClinicalTrialSection::Present(directory) = response.site_directory() {
        trial.set_site_directory(Some(directory.clone()));
    }
    trial.outcomes = product_outcomes(response.outcomes());
    let reference_state = section_state(response.references());
    if section_flags.include_references
        && let ClinicalTrialSection::Present(values) = response.references()
    {
        trial.references = Some(values.clone());
    }
    trial.eligibility = match (section_flags.request_eligibility, response.eligibility()) {
        (true, ClinicalTrialSection::Present(value)) => Some(value.clone()),
        (true, ClinicalTrialSection::Absent) | (false, _) => None,
        (true, ClinicalTrialSection::NotRequested | ClinicalTrialSection::Unavailable) => {
            return Err(BioMcpError::InternalProcessing);
        }
    };
    if section_flags.include_eligibility_provenance
        && trial
            .eligibility
            .as_ref()
            .and_then(ClinicalTrialEligibility::registry_text)
            .is_some()
    {
        trial.eligibility_provenance = Some(super::documents::eligibility_provenance(&response));
    }
    Ok(TrialResponse {
        trial,
        section_states: TrialSectionStates {
            arms: section_state(response.arms()),
            eligibility: section_state(response.eligibility()),
            outcomes: section_state(response.outcomes()),
            references: reference_state,
            contacts: section_state(&response.contacts_state()),
            locations: section_state(&response.locations_state()),
        },
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
            let mut trial = product_from_nci_response(
                &response,
                section_flags.request_eligibility,
                section_flags.include_arms,
            )?;
            trial.section_states.references = if section_flags.include_references {
                TrialSectionState::Unavailable
            } else {
                TrialSectionState::NotRequested
            };

            Ok(trial)
        }
    }
}

#[cfg(test)]
mod tests;
