//! One-way adapters from shared clinical trial projections to product output shapes.

use biodata::{ClinicalTrialContact, ClinicalTrialSite};
use serde::Serialize;

pub(crate) fn capture(value: &biodata::Capture) -> serde_json::Value {
    serde_json::json!({
        "source_authority": value.source_authority(),
        "provider_record_identity": value.provider_record_identity(),
        "digest": value.digest(),
    })
}

fn direction(value: biodata::ConversionDirection) -> &'static str {
    match value {
        biodata::ConversionDirection::SourceToHub => "source_to_hub",
        biodata::ConversionDirection::HubToTarget => "hub_to_target",
    }
}

fn disposition(value: biodata::ConversionDisposition) -> &'static str {
    match value {
        biodata::ConversionDisposition::ExactSemanticPreservation => "exact_semantic_preservation",
        biodata::ConversionDisposition::ExactLexicalPreservation => "exact_lexical_preservation",
        biodata::ConversionDisposition::Absence => "absence",
        biodata::ConversionDisposition::UnsupportedMeaning => "unsupported_meaning",
        biodata::ConversionDisposition::SourceOnlyMeaning => "source_only_meaning",
        biodata::ConversionDisposition::ContextRequired => "context_required",
        biodata::ConversionDisposition::LossyTransformation => "lossy_transformation",
        biodata::ConversionDisposition::RejectedSourceShape => "rejected_source_shape",
    }
}

pub(crate) fn report(values: &[biodata::ConversionReportEntry]) -> serde_json::Value {
    values
        .iter()
        .map(|value| {
            serde_json::json!({
                "source_contract": value.source_contract(),
                "source_version": value.source_version(),
                "target_contract": value.target_contract(),
                "target_version": value.target_version(),
                "direction": direction(value.direction()),
                "source_locator": value.source_locator(),
                "target_locator": value.target_locator(),
                "occurrence": value.occurrence(),
                "adapter_rule": value.adapter_rule(),
                "adapter_rule_version": value.adapter_rule_version(),
                "disposition": disposition(value.disposition()),
                "reason_code": value.reason_code(),
                "required_context": value.required_context(),
                "capture": {
                    "source_authority": value.capture().source_authority(),
                    "provider_record_identity": value.capture().provider_record_identity(),
                    "digest": value.capture().digest(),
                },
            })
        })
        .collect()
}

pub(crate) fn identities(values: &[biodata::AuthorityScopedReference]) -> serde_json::Value {
    values
        .iter()
        .map(|value| {
            serde_json::json!({
                "authority": value.authority(), "identifier": value.identifier()
            })
        })
        .collect()
}

pub(crate) fn interventions(
    values: Option<&[biodata::ClinicalTrialIntervention]>,
) -> serde_json::Value {
    values
        .unwrap_or_default()
        .iter()
        .map(|value| {
            serde_json::json!({
                "id": value.id().get(),
                "name": value.name(),
                "type": value.source_type().map(biodata::ExtensibleCode::code),
                "description": value.description(),
                "other_names": value.other_names().unwrap_or_default(),
            })
        })
        .collect()
}

pub(crate) fn arms(values: Option<&[biodata::ClinicalTrialArm]>) -> serde_json::Value {
    values
        .unwrap_or_default()
        .iter()
        .map(|value| {
            serde_json::json!({
                "id": value.id().get(), "name": value.name(),
                "type": value.source_type().map(biodata::ExtensibleCode::code),
                "description": value.description(),
            })
        })
        .collect()
}

pub(crate) fn assignments(
    values: Option<&[biodata::ClinicalTrialArmInterventionAssignment]>,
) -> serde_json::Value {
    values
        .unwrap_or_default()
        .iter()
        .map(|value| {
            serde_json::json!({
                "arm_id": value.arm_id().get(), "intervention_id": value.intervention_id().get()
            })
        })
        .collect()
}

fn age_bound(value: &biodata::ClinicalTrialAgeBound) -> serde_json::Value {
    let source = value.source();
    let mut output = serde_json::json!({
        "kind": match value.form() { biodata::ClinicalTrialAgeBoundForm::Limited => "limited", biodata::ClinicalTrialAgeBoundForm::SourceStatedNoLimit => "source_stated_no_limit" },
        "source": source.source(),
        "source_quantity": source.source_quantity(),
        "source_unit": match source.source_unit() {
            biodata::DurationUnit::Years => "years", biodata::DurationUnit::Months => "months",
            biodata::DurationUnit::Weeks => "weeks", biodata::DurationUnit::Days => "days",
        },
        "bound": match source.bound() { biodata::Bound::Minimum => "minimum", biodata::Bound::Maximum => "maximum" },
    });
    if let Some(rule) = value.rule() {
        output["rule"] = serde_json::json!({"name": rule.name(), "version": rule.version()});
    }
    output
}

fn code(value: &biodata::ExtensibleCode) -> serde_json::Value {
    serde_json::json!({
        "authority": value.authority(), "code": value.code(), "display": value.display(),
        "vocabulary_version": value.vocabulary_version(),
        "recognized_meaning": value.recognized_meaning(),
    })
}

pub(crate) fn eligibility(value: &biodata::ClinicalTrialEligibility) -> serde_json::Value {
    let age_range = value.age_range().map(|range| {
        serde_json::json!({
            "minimum": range.minimum().map(age_bound), "maximum": range.maximum().map(age_bound)
        })
    });
    let criteria = value.criteria().map(|rows| rows.iter().map(|row| {
        let classification = match row.classification() {
            biodata::ClinicalTrialEligibilityClassification::Inclusion => serde_json::json!({"kind":"inclusion"}),
            biodata::ClinicalTrialEligibilityClassification::Exclusion => serde_json::json!({"kind":"exclusion"}),
            biodata::ClinicalTrialEligibilityClassification::Other(value) => serde_json::json!({"kind":"other", "source": code(value)}),
        };
        serde_json::json!({"id": row.id().get(), "description": row.description(), "classification": classification})
    }).collect::<Vec<_>>());
    serde_json::json!({
        "registry_text": value.registry_text(), "age_range": age_range,
        "sexes": value.sexes().map(|rows| rows.iter().map(code).collect::<Vec<_>>()),
        "includes_healthy_subjects": value.includes_healthy_subjects(), "criteria": criteria,
    })
}

#[derive(Serialize)]
struct TrialSiteContactView<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phone: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phone_extension: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<&'a str>,
}

impl<'a> From<&'a ClinicalTrialContact> for TrialSiteContactView<'a> {
    fn from(contact: &'a ClinicalTrialContact) -> Self {
        Self {
            name: contact.name(),
            role: contact.role().map(|value| value.code()),
            phone: contact.phone(),
            phone_extension: contact.phone_extension(),
            email: contact.email(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct TrialLocationView<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    facility: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    city: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    postal_code: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    country: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    latitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    longitude: Option<f64>,
    contacts: Vec<TrialSiteContactView<'a>>,
}

impl<'a> TrialLocationView<'a> {
    pub(crate) fn new(site: &'a ClinicalTrialSite) -> Self {
        let coordinates = site.coordinates();
        Self {
            facility: site.facility(),
            city: site.city(),
            state: site.state(),
            postal_code: site.postal_code(),
            country: site.country(),
            status: site.status().map(|value| value.code()),
            latitude: coordinates.map(|value| value.latitude()),
            longitude: coordinates.map(|value| value.longitude()),
            contacts: site
                .contacts()
                .unwrap_or_default()
                .iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct TrialContactView<'a> {
    level: &'static str,
    #[serde(flatten)]
    contact: TrialSiteContactView<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    facility: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    city: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    country: Option<&'a str>,
}

impl<'a> TrialContactView<'a> {
    pub(crate) fn central(contact: &'a ClinicalTrialContact) -> Self {
        Self {
            level: "central",
            contact: contact.into(),
            facility: None,
            city: None,
            state: None,
            country: None,
        }
    }

    pub(crate) fn site(contact: &'a ClinicalTrialContact, site: &'a ClinicalTrialSite) -> Self {
        Self {
            level: "site",
            contact: contact.into(),
            facility: site.facility(),
            city: site.city(),
            state: site.state(),
            country: site.country(),
        }
    }
}

pub(crate) mod outcome_wire {
    use biodata::ClinicalTrialPlannedOutcome;
    use serde::Serialize;

    #[derive(Serialize)]
    pub(crate) struct RowView<'a> {
        measure: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        time_frame: Option<&'a str>,
    }

    impl<'a> From<&'a ClinicalTrialPlannedOutcome> for RowView<'a> {
        fn from(value: &'a ClinicalTrialPlannedOutcome) -> Self {
            Self {
                measure: value.measure(),
                description: value.description(),
                time_frame: value.time_frame(),
            }
        }
    }

    #[derive(Serialize)]
    pub(crate) struct GroupedView<'a> {
        primary: Vec<RowView<'a>>,
        secondary: Vec<RowView<'a>>,
        other: Vec<RowView<'a>>,
    }

    pub(crate) fn views(
        outcomes: Option<&[ClinicalTrialPlannedOutcome]>,
    ) -> Result<Option<GroupedView<'_>>, &'static str> {
        outcomes
            .map(|values| {
                let mut grouped = GroupedView {
                    primary: Vec::new(),
                    secondary: Vec::new(),
                    other: Vec::new(),
                };
                for outcome in values {
                    let classification = outcome.source_classification();
                    if classification.authority() != "clinicaltrials.gov" {
                        return Err("invalid clinical trial planned outcome");
                    }
                    let row = RowView::from(outcome);
                    match classification.code() {
                        "primaryOutcomes" => grouped.primary.push(row),
                        "secondaryOutcomes" => grouped.secondary.push(row),
                        "otherOutcomes" => grouped.other.push(row),
                        _ => return Err("invalid clinical trial planned outcome"),
                    }
                }
                Ok(grouped)
            })
            .transpose()
    }
}

pub(crate) mod reference_wire {
    use biodata::ClinicalTrialReference;
    use serde::Serialize;

    #[derive(Serialize)]
    pub(crate) struct View<'a> {
        pub(crate) pmid: Option<&'a str>,
        pub(crate) citation: Option<&'a str>,
        pub(crate) source_type_label: Option<&'a str>,
    }

    fn display_text(value: Option<&str>) -> Option<&str> {
        value.map(str::trim).filter(|value| !value.is_empty())
    }

    fn view(reference: &ClinicalTrialReference) -> View<'_> {
        let source_type_label = reference.source_type().and_then(|source_type| {
            display_text(source_type.display())
                .or_else(|| display_text(source_type.recognized_meaning()))
                .or_else(|| display_text(Some(source_type.code())))
        });
        View {
            pmid: display_text(reference.pmid()),
            citation: display_text(reference.citation()),
            source_type_label,
        }
    }

    pub(crate) fn views(references: Option<&[ClinicalTrialReference]>) -> Option<Vec<View<'_>>> {
        references.map(|values| values.iter().map(view).collect())
    }
}
