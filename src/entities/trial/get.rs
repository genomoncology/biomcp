//! Trial detail retrieval exposed through the stable trial facade.

use crate::error::BioMcpError;
use crate::sources::clinicaltrials::ClinicalTrialsClient;
use crate::sources::nci_cts::NciCtsClient;
use crate::transform;
use biodata::{
    ClinicalTrialAgeBound, ClinicalTrialAgeBoundForm, ClinicalTrialAgeRange, ClinicalTrialArms,
    ClinicalTrialEligibilityClassification, ClinicalTrialIntervention, ClinicalTrialSection,
    NciCtsV2DetailPlan, NciCtsV2DetailResponse, NciCtsV2Eligibility,
};

use super::{
    TRIAL_SECTION_ALL, TRIAL_SECTION_ARMS, TRIAL_SECTION_CONTACTS, TRIAL_SECTION_ELIGIBILITY,
    TRIAL_SECTION_LOCATIONS, TRIAL_SECTION_NAMES, TRIAL_SECTION_OUTCOMES, TRIAL_SECTION_REFERENCES,
    Trial, TrialDesign, TrialSource,
};

const ELIGIBILITY_MAX_CHARS: usize = 12_000;

#[derive(Debug, Clone, Copy, Default)]
struct TrialSections {
    include_eligibility: bool,
    include_eligibility_provenance: bool,
    include_contacts: bool,
    include_locations: bool,
    include_outcomes: bool,
    include_arms: bool,
    include_references: bool,
}

fn parse_sections(sections: &[String]) -> Result<TrialSections, BioMcpError> {
    let mut out = TrialSections::default();
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
                out.include_eligibility = true;
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
        out.include_eligibility = true;
        out.include_contacts = true;
        out.include_locations = true;
        out.include_outcomes = true;
        out.include_arms = true;
        out.include_references = true;
    }

    Ok(out)
}

fn product_references(
    section: biodata::ClinicalTrialSection<Vec<biodata::ClinicalTrialReference>>,
) -> Result<Vec<biodata::ClinicalTrialReference>, BioMcpError> {
    match section {
        biodata::ClinicalTrialSection::Present(values) => Ok(values
            .into_iter()
            .filter(|value| {
                value
                    .citation()
                    .map(str::trim)
                    .is_some_and(|citation| !citation.is_empty())
            })
            .collect()),
        biodata::ClinicalTrialSection::Absent => Ok(Vec::new()),
        biodata::ClinicalTrialSection::NotRequested
        | biodata::ClinicalTrialSection::Unavailable => Err(BioMcpError::InternalProcessing),
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
    TrialDesign::new(interventions, arms, assignments).map_err(|()| BioMcpError::InternalProcessing)
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
    TrialDesign::new(interventions, arms, assignments).map_err(|()| BioMcpError::InternalProcessing)
}

fn truncate_inline_text(value: &str, max_chars: usize) -> String {
    let count = value.chars().count();
    if count <= max_chars {
        return value.to_string();
    }
    let truncated = value.chars().take(max_chars).collect::<String>();
    format!("{truncated}\n\n(truncated, {count} chars total)")
}

fn nci_eligibility_text(eligibility: NciCtsV2Eligibility<'_>) -> Option<String> {
    let criteria = eligibility.criteria()?;
    if criteria.is_empty() {
        return None;
    }
    let mut rendered = String::new();
    let mut prior_classification = None;
    for criterion in criteria {
        if prior_classification != Some(criterion.classification()) {
            if !rendered.is_empty() {
                rendered.push('\n');
            }
            rendered.push_str(match criterion.classification() {
                ClinicalTrialEligibilityClassification::Inclusion => "Inclusion Criteria:\n",
                ClinicalTrialEligibilityClassification::Exclusion => "Exclusion Criteria:\n",
            });
            prior_classification = Some(criterion.classification());
        }
        rendered.push_str("- ");
        rendered.push_str(criterion.description());
        rendered.push('\n');
    }
    rendered.pop();
    Some(truncate_inline_text(&rendered, ELIGIBILITY_MAX_CHARS))
}

fn product_age(bound: &ClinicalTrialAgeBound) -> Option<super::TrialAge> {
    match bound.form() {
        ClinicalTrialAgeBoundForm::Limited => {
            super::TrialAge::from_provider(bound.source().source())
        }
        ClinicalTrialAgeBoundForm::SourceStatedNoLimit => Some(super::TrialAge::unparsed(
            bound.source().source().trim().to_string(),
        )),
    }
}

fn product_age_range(range: Option<&ClinicalTrialAgeRange>) -> Option<String> {
    let range = range?;
    super::format_age_range(
        range.minimum().and_then(product_age).as_ref(),
        range.maximum().and_then(product_age).as_ref(),
    )
}

fn product_eligibility(eligibility: NciCtsV2Eligibility<'_>) -> Option<super::TrialEligibility> {
    let range = eligibility.age_range();
    let minimum_age = range
        .and_then(ClinicalTrialAgeRange::minimum)
        .and_then(product_age);
    let maximum_age = range
        .and_then(ClinicalTrialAgeRange::maximum)
        .and_then(product_age);
    let sex = eligibility
        .sex()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| match value.to_ascii_lowercase().as_str() {
            "female" | "f" => "Female".to_string(),
            "male" | "m" => "Male".to_string(),
            "all" => "All".to_string(),
            _ => value.to_string(),
        });
    (sex.is_some() || minimum_age.is_some() || maximum_age.is_some()).then_some(
        super::TrialEligibility {
            sex,
            minimum_age,
            maximum_age,
        },
    )
}

fn product_from_nci_response(
    plan: &NciCtsV2DetailPlan,
    response: &NciCtsV2DetailResponse,
    include_eligibility: bool,
    include_arms: bool,
) -> Result<Trial, BioMcpError> {
    let shared = response.projection().trial();
    let phase = shared
        .phases()
        .first()
        .map(|value| value.code().to_string());
    let age_range = product_age_range(shared.age_range());
    let (eligibility, eligibility_text) = if include_eligibility {
        match response.eligibility() {
            ClinicalTrialSection::Present(value) => {
                (product_eligibility(value), nci_eligibility_text(value))
            }
            ClinicalTrialSection::Absent => (None, None),
            ClinicalTrialSection::NotRequested | ClinicalTrialSection::Unavailable => {
                return Err(BioMcpError::InternalProcessing);
            }
        }
    } else {
        (None, None)
    };

    Ok(Trial {
        nct_id: plan.requested_identity().to_string(),
        source: Some("NCI CTS".to_string()),
        title: shared.brief_title().to_string(),
        status: shared.overall_status().code().to_string(),
        why_stopped: Some(
            shared
                .stop_reason()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned),
        ),
        phase,
        study_type: Some(shared.study_type().code().to_string()),
        age_range,
        conditions: shared.conditions().to_vec(),
        design: product_nci_design(shared, include_arms)?,
        sponsor: Some(shared.lead_sponsor_name().to_string()),
        enrollment: shared
            .enrollment_count()
            .and_then(|value| i32::try_from(value).ok()),
        summary: response
            .brief_summary()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned),
        start_date: shared.start_date().map(str::to_owned),
        completion_date: shared.completion_date().map(str::to_owned),
        eligibility_text,
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
            let response = client.get_biodata_detail(nct_id, sections).await?;
            let mut study = response.study;
            if let Some(protocol) = study.protocol_section.as_mut() {
                protocol.arms_interventions_module = None;
                protocol.references_module = None;
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
            if !section_flags.include_eligibility {
                trial.eligibility = None;
            }
            if !section_flags.include_locations {
                trial.locations = None;
            }

            if section_flags.include_eligibility {
                if section_flags.include_eligibility_provenance {
                    trial.eligibility_provenance =
                        Some(super::documents::eligibility_provenance(nct_id, &study));
                }
                let criteria = study
                    .protocol_section
                    .as_ref()
                    .and_then(|p| p.eligibility_module.as_ref())
                    .and_then(|m| m.eligibility_criteria.as_deref())
                    .map(str::trim)
                    .filter(|s| !s.is_empty());

                if let Some(criteria) = criteria {
                    trial.eligibility_text =
                        Some(truncate_inline_text(criteria, ELIGIBILITY_MAX_CHARS));
                }
            }
            if section_flags.include_references && trial.references.is_none() {
                trial.references = Some(Vec::new());
            }

            Ok(trial)
        }
        TrialSource::NciCts => {
            let plan = NciCtsV2DetailPlan::new(nct_id, true)
                .map_err(|_| BioMcpError::InternalProcessing)?;
            let client = NciCtsClient::new()?;
            let response = client.get(&plan).await?;
            let mut trial = product_from_nci_response(
                &plan,
                &response,
                section_flags.include_eligibility,
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
