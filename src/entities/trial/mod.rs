//! Trial entity models and workflows exposed through the stable trial facade.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;

mod documents;
mod get;
mod reference;
mod search;
#[cfg(test)]
mod test_support;

pub use self::documents::{
    TrialDocumentsManifest, TrialEligibilityProvenance, trial_document_bytes,
    trial_documents_manifest,
};
pub use self::get::get;
pub use self::reference::TrialReference;
pub use self::search::{count_all, search, search_page};

pub(crate) fn validate_search_filters(filters: &TrialSearchFilters) -> Result<(), BioMcpError> {
    search::validate_trial_search(filters).map(|_| ())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trial {
    pub nct_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub title: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub why_stopped: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub study_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age_range: Option<String>,
    #[serde(default)]
    pub conditions: Vec<String>,
    #[serde(default)]
    pub interventions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub intervention_details: Vec<TrialIntervention>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrollment: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eligibility_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eligibility: Option<TrialEligibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eligibility_provenance: Option<TrialEligibilityProvenance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contacts: Option<Vec<TrialContact>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<TrialLocation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcomes: Option<TrialOutcomes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arms: Option<Vec<TrialArm>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<Vec<TrialReference>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialIntervention {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intervention_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub other_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialSiteContact {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialLocation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contacts: Vec<TrialSiteContact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialContact {
    pub level: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

#[derive(Debug, Eq, Hash, PartialEq)]
struct SiteContactKey {
    facility: Option<String>,
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
    name: String,
    role: Option<String>,
    phone: Option<String>,
    email: Option<String>,
}

pub(crate) fn project_contacts_to_locations(
    contacts: &mut Option<Vec<TrialContact>>,
    locations: &[TrialLocation],
) {
    let Some(current_contacts) = contacts.take() else {
        return;
    };
    let mut authorized = HashMap::<SiteContactKey, usize>::new();
    for location in locations {
        if location.contacts.is_empty() {
            if let Some(name) = location
                .contact_name
                .as_deref()
                .filter(|name| !name.trim().is_empty())
            {
                *authorized
                    .entry(SiteContactKey {
                        facility: location.facility.clone(),
                        city: location.city.clone(),
                        state: location.state.clone(),
                        country: location.country.clone(),
                        name: name.to_string(),
                        role: location.contact_role.clone(),
                        phone: location.contact_phone.clone(),
                        email: location.contact_email.clone(),
                    })
                    .or_default() += 1;
            }
        } else {
            for contact in &location.contacts {
                *authorized
                    .entry(SiteContactKey {
                        facility: location.facility.clone(),
                        city: location.city.clone(),
                        state: location.state.clone(),
                        country: location.country.clone(),
                        name: contact.name.clone(),
                        role: contact.role.clone(),
                        phone: contact.phone.clone(),
                        email: contact.email.clone(),
                    })
                    .or_default() += 1;
            }
        }
    }

    let retained: Vec<_> = current_contacts
        .into_iter()
        .filter(|contact| {
            if !contact.level.eq_ignore_ascii_case("site") {
                return true;
            }
            let key = SiteContactKey {
                facility: contact.facility.clone(),
                city: contact.city.clone(),
                state: contact.state.clone(),
                country: contact.country.clone(),
                name: contact.name.clone(),
                role: contact.role.clone(),
                phone: contact.phone.clone(),
                email: contact.email.clone(),
            };
            let Some(remaining) = authorized.get_mut(&key) else {
                return false;
            };
            if *remaining == 0 {
                return false;
            }
            *remaining -= 1;
            true
        })
        .collect();
    *contacts = (!retained.is_empty()).then_some(retained);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialEligibility {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_age: Option<TrialAge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum_age: Option<TrialAge>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrialAge {
    number: Option<f64>,
    unit: Option<TrialAgeUnit>,
    original: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TrialAgeUnit {
    Years,
    Months,
    Weeks,
    Days,
    Hours,
    Minutes,
}

impl TrialAge {
    pub(crate) fn from_provider(value: &str) -> Option<Self> {
        let original = value.trim();
        if original.is_empty() {
            return None;
        }
        if original.eq_ignore_ascii_case("n/a") {
            return Some(Self::unparsed(original.to_string()));
        }

        let mut tokens = original.split_whitespace();
        let number_token = tokens.next().expect("nonblank age has a token");
        let unit_token = tokens.next();
        if tokens.next().is_some() || !valid_age_number_token(number_token) {
            return Some(Self::unparsed(original.to_string()));
        }
        let Ok(number) = number_token.parse::<f64>() else {
            return Some(Self::unparsed(original.to_string()));
        };
        if !number.is_finite() {
            return Some(Self::unparsed(original.to_string()));
        }
        let unit = match unit_token {
            None => TrialAgeUnit::Years,
            Some(value)
                if value.eq_ignore_ascii_case("year") || value.eq_ignore_ascii_case("years") =>
            {
                TrialAgeUnit::Years
            }
            Some(value)
                if value.eq_ignore_ascii_case("month") || value.eq_ignore_ascii_case("months") =>
            {
                TrialAgeUnit::Months
            }
            Some(value)
                if value.eq_ignore_ascii_case("week") || value.eq_ignore_ascii_case("weeks") =>
            {
                TrialAgeUnit::Weeks
            }
            Some(value)
                if value.eq_ignore_ascii_case("day") || value.eq_ignore_ascii_case("days") =>
            {
                TrialAgeUnit::Days
            }
            Some(value)
                if value.eq_ignore_ascii_case("hour") || value.eq_ignore_ascii_case("hours") =>
            {
                TrialAgeUnit::Hours
            }
            Some(value)
                if value.eq_ignore_ascii_case("minute")
                    || value.eq_ignore_ascii_case("minutes") =>
            {
                TrialAgeUnit::Minutes
            }
            Some(_) => return Some(Self::unparsed(original.to_string())),
        };
        Some(Self {
            number: Some(number),
            unit: Some(unit),
            original: original.to_string(),
        })
    }

    fn unparsed(original: String) -> Self {
        Self {
            number: None,
            unit: None,
            original,
        }
    }

    pub(crate) fn retained_unparsed(original: String) -> Option<Self> {
        (!original.trim().is_empty()).then(|| Self::unparsed(original))
    }

    pub fn number(&self) -> Option<f64> {
        self.number
    }

    pub fn unit(&self) -> Option<&'static str> {
        self.unit.map(TrialAgeUnit::as_str)
    }

    pub fn original(&self) -> &str {
        &self.original
    }

    pub(crate) fn comparable_years(&self) -> Option<f64> {
        let number = self.number?;
        match self.unit? {
            TrialAgeUnit::Years => Some(number),
            TrialAgeUnit::Months => Some(number / 12.0),
            TrialAgeUnit::Weeks => Some(number / 52.0),
            TrialAgeUnit::Days => Some(number / 365.0),
            TrialAgeUnit::Hours | TrialAgeUnit::Minutes => None,
        }
    }

    pub(crate) fn is_no_limit(&self) -> bool {
        self.number.is_none()
            && (self.original.eq_ignore_ascii_case("n/a")
                || self.original.eq_ignore_ascii_case("999 Years"))
    }
}

impl TrialAgeUnit {
    fn as_str(self) -> &'static str {
        match self {
            Self::Years => "years",
            Self::Months => "months",
            Self::Weeks => "weeks",
            Self::Days => "days",
            Self::Hours => "hours",
            Self::Minutes => "minutes",
        }
    }
}

fn valid_age_number_token(value: &str) -> bool {
    let mut pieces = value.split('.');
    let Some(integer) = pieces.next() else {
        return false;
    };
    if integer.is_empty() || !integer.bytes().all(|byte| byte.is_ascii_digit()) {
        return false;
    }
    match (pieces.next(), pieces.next()) {
        (None, None) => true,
        (Some(fraction), None) => {
            !fraction.is_empty() && fraction.bytes().all(|byte| byte.is_ascii_digit())
        }
        _ => false,
    }
}

pub(crate) fn format_age_range(
    minimum: Option<&TrialAge>,
    maximum: Option<&TrialAge>,
) -> Option<String> {
    let minimum = minimum
        .filter(|age| !age.is_no_limit())
        .map(TrialAge::original);
    let maximum = maximum
        .filter(|age| !age.is_no_limit())
        .map(TrialAge::original);
    match (minimum, maximum) {
        (Some(minimum), Some(maximum)) => Some(format!("{minimum} to {maximum}")),
        (Some(minimum), None) => Some(format!("{minimum} to Any age")),
        (None, Some(maximum)) => Some(format!("Any age to {maximum}")),
        (None, None) => None,
    }
}

impl Serialize for TrialAge {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.original.trim().is_empty()
            || self.number.is_some() != self.unit.is_some()
            || self
                .number
                .is_some_and(|number| !number.is_finite() || number < 0.0)
        {
            return Err(serde::ser::Error::custom("invalid trial age"));
        }
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("TrialAge", 3)?;
        state.serialize_field("number", &self.number())?;
        state.serialize_field("unit", &self.unit())?;
        state.serialize_field("original", &self.original)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for TrialAge {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let object = value
            .as_object()
            .filter(|object| {
                object.len() == 3
                    && object.contains_key("number")
                    && object.contains_key("unit")
                    && object.contains_key("original")
            })
            .ok_or_else(|| serde::de::Error::custom("trial age must have exactly three members"))?;
        let number: Option<f64> =
            serde_json::from_value(object["number"].clone()).map_err(serde::de::Error::custom)?;
        let unit: Option<TrialAgeUnit> =
            serde_json::from_value(object["unit"].clone()).map_err(serde::de::Error::custom)?;
        let original: String =
            serde_json::from_value(object["original"].clone()).map_err(serde::de::Error::custom)?;
        if original.trim().is_empty()
            || number.is_some() != unit.is_some()
            || number.is_some_and(|number| !number.is_finite() || number < 0.0)
        {
            return Err(serde::de::Error::custom("invalid trial age"));
        }
        Ok(Self {
            number,
            unit,
            original,
        })
    }
}

#[cfg(test)]
mod age_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn provider_age_grammar_is_exact_and_fail_open() {
        for (input, number, unit, comparable) in [
            ("0", 0.0, "years", Some(0.0)),
            ("18.25", 18.25, "years", Some(18.25)),
            ("1 Year", 1.0, "years", Some(1.0)),
            ("2 YEARS", 2.0, "years", Some(2.0)),
            ("6 mOnTh", 6.0, "months", Some(0.5)),
            ("12 mOnThS", 12.0, "months", Some(1.0)),
            ("1 Week", 1.0, "weeks", Some(1.0 / 52.0)),
            ("2 Weeks", 2.0, "weeks", Some(2.0 / 52.0)),
            ("1 Day", 1.0, "days", Some(1.0 / 365.0)),
            ("30 DAYS", 30.0, "days", Some(30.0 / 365.0)),
            ("1 Hour", 1.0, "hours", None),
            ("4 Hours", 4.0, "hours", None),
            ("1 Minute", 1.0, "minutes", None),
            ("5 Minutes", 5.0, "minutes", None),
        ] {
            let age = TrialAge::from_provider(input).unwrap();
            assert_eq!(age.number(), Some(number), "{input}");
            assert_eq!(age.unit(), Some(unit), "{input}");
            assert_eq!(age.comparable_years(), comparable, "{input}");
            assert_eq!(
                serde_json::to_value(&age).unwrap(),
                json!({"number":number,"unit":unit,"original":input}),
                "{input}"
            );
        }
        let spaced = TrialAge::from_provider("\u{2003}6\u{2002}Months\n").unwrap();
        assert_eq!(spaced.original(), "6\u{2002}Months");
        for input in [
            "+18",
            "-1",
            ".5",
            "5.",
            "1e2",
            "NaN",
            "inf",
            "Infinity",
            "1e9999",
            "18, Years",
            "18 Years,",
            "18 Years old",
            "18 Fortnights",
        ] {
            let age = TrialAge::from_provider(input).unwrap();
            assert_eq!(
                (age.number(), age.unit(), age.comparable_years()),
                (None, None, None),
                "{input}"
            );
            assert_eq!(age.original(), input);
        }
        let overflow = "9".repeat(400);
        let age = TrialAge::from_provider(&overflow).unwrap();
        assert_eq!(
            (age.number(), age.unit(), age.comparable_years()),
            (None, None, None)
        );
        assert_eq!(age.original(), overflow);
        assert_eq!(
            serde_json::to_value(age).unwrap(),
            json!({"number":null,"unit":null,"original":overflow})
        );
        assert!(TrialAge::from_provider(" \t\n").is_none());
        let sentinel = TrialAge::from_provider(" n/A ").unwrap();
        assert!(sentinel.is_no_limit());
        assert_eq!(sentinel.original(), "n/A");
    }

    #[test]
    fn public_age_serde_is_object_only_and_validated() {
        let exact = json!({"number":6.0,"unit":"months","original":"6 Months"});
        let age: TrialAge = serde_json::from_value(exact.clone()).unwrap();
        assert_eq!(serde_json::to_value(age).unwrap(), exact);
        let nulls = json!({"number":null,"unit":null,"original":"N/A"});
        assert_eq!(
            serde_json::to_value(serde_json::from_value::<TrialAge>(nulls.clone()).unwrap())
                .unwrap(),
            nulls
        );
        let malformed = json!({"number":null,"unit":null,"original":"18 Years old"});
        assert_eq!(
            serde_json::to_value(serde_json::from_value::<TrialAge>(malformed.clone()).unwrap())
                .unwrap(),
            malformed
        );
        for invalid in [
            json!("6 Months"),
            json!({"number":6.0,"unit":null,"original":"6 Months"}),
            json!({"number":null,"unit":"months","original":"6 Months"}),
            json!({"number":-1.0,"unit":"years","original":"-1 Years"}),
            json!({"number":1.0,"unit":"fortnights","original":"1 Fortnight"}),
            json!({"number":1.0,"unit":"years","original":" "}),
            json!({"number":1.0,"unit":"years","original":"1 Year","extra":true}),
            json!({"number":1.0,"original":"1 Year"}),
        ] {
            assert!(serde_json::from_value::<TrialAge>(invalid).is_err());
        }
        for invalid_memory in [
            TrialAge {
                number: Some(f64::NAN),
                unit: Some(TrialAgeUnit::Years),
                original: "NaN Years".into(),
            },
            TrialAge {
                number: Some(f64::INFINITY),
                unit: Some(TrialAgeUnit::Years),
                original: "+Infinity Years".into(),
            },
            TrialAge {
                number: Some(f64::NEG_INFINITY),
                unit: Some(TrialAgeUnit::Years),
                original: "-Infinity Years".into(),
            },
            TrialAge {
                number: Some(1.0),
                unit: None,
                original: "1 Year".into(),
            },
            TrialAge {
                number: None,
                unit: Some(TrialAgeUnit::Years),
                original: "1 Year".into(),
            },
            TrialAge {
                number: Some(-1.0),
                unit: Some(TrialAgeUnit::Years),
                original: "-1 Years".into(),
            },
            TrialAge {
                number: Some(1.0),
                unit: Some(TrialAgeUnit::Years),
                original: " \t".into(),
            },
        ] {
            assert!(serde_json::to_value(invalid_memory).is_err());
        }
    }

    #[test]
    fn eligibility_null_and_missing_bounds_reserialize_as_omission() {
        for input in [
            json!({"sex":"All"}),
            json!({"sex":"All","minimum_age":null}),
        ] {
            let value =
                serde_json::to_value(serde_json::from_value::<TrialEligibility>(input).unwrap())
                    .unwrap();
            assert_eq!(value, json!({"sex":"All"}));
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialOutcomes {
    #[serde(default)]
    pub primary: Vec<TrialOutcome>,
    #[serde(default)]
    pub secondary: Vec<TrialOutcome>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialOutcome {
    pub measure: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_frame: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialArm {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arm_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub interventions: Vec<String>,
}

#[cfg(test)]
mod reference_wire_tests {
    use super::TrialReference;

    #[test]
    fn reference_wire_rejects_blank_required_citation() {
        let wire = serde_json::json!({"citation": " \t "});
        assert!(serde_json::from_value::<TrialReference>(wire).is_err());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialSearchResult {
    pub nct_id: String,
    pub title: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(default)]
    pub conditions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_intervention_label: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TrialSearchFilters {
    pub condition: Option<String>,
    pub intervention: Option<String>,
    pub no_alias_expand: bool,
    pub no_count_total: bool,
    pub facility: Option<String>,
    pub status: Option<String>,
    pub phase: Option<String>,
    pub study_type: Option<String>,
    pub age: Option<f64>,
    pub sex: Option<String>,
    pub sponsor: Option<String>,
    pub sponsor_type: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub mutation: Option<String>,
    pub criteria: Option<String>,
    pub biomarker: Option<String>,
    pub prior_therapies: Option<String>,
    pub progression_on: Option<String>,
    pub line_of_therapy: Option<String>,
    pub results_available: bool,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub distance: Option<u32>,
    pub source: TrialSource,
}

#[derive(Debug, Clone, Default, Copy)]
pub enum TrialSource {
    #[default]
    ClinicalTrialsGov,
    NciCts,
}

impl TrialSource {
    pub fn from_flag(value: &str) -> Result<Self, BioMcpError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "ctgov" | "clinicaltrials" | "clinicaltrials.gov" => Ok(Self::ClinicalTrialsGov),
            "nci" | "nci_cts" | "cts" => Ok(Self::NciCts),
            other => Err(BioMcpError::InvalidArgument(format!(
                "Unknown --source '{other}'. Expected 'ctgov' or 'nci'."
            ))),
        }
    }
}

const TRIAL_SECTION_ELIGIBILITY: &str = "eligibility";
const TRIAL_SECTION_CONTACTS: &str = "contacts";
const TRIAL_SECTION_LOCATIONS: &str = "locations";
const TRIAL_SECTION_OUTCOMES: &str = "outcomes";
const TRIAL_SECTION_ARMS: &str = "arms";
const TRIAL_SECTION_REFERENCES: &str = "references";
const TRIAL_SECTION_ALL: &str = "all";

pub const TRIAL_SECTION_NAMES: &[&str] = &[
    TRIAL_SECTION_ELIGIBILITY,
    TRIAL_SECTION_CONTACTS,
    TRIAL_SECTION_LOCATIONS,
    TRIAL_SECTION_OUTCOMES,
    TRIAL_SECTION_ARMS,
    TRIAL_SECTION_REFERENCES,
    TRIAL_SECTION_ALL,
];

/// Describes the precision of a trial `--count-only` result.
#[derive(Debug, PartialEq)]
pub enum TrialCount {
    /// Exact post-filtered count.
    Exact(usize),
    /// Upstream CTGov total before client-side age post-filtering.
    Approximate(usize),
    /// The total is unknown for the stated reason.
    Unknown(TrialCountUnknownReason),
}

/// Explains why a trial count could not be stated numerically.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrialCountUnknownReason {
    /// ClinicalTrials.gov omitted the requested total from its response.
    ProviderOmittedTotal,
    /// Bounded post-filter traversal reached its page limit.
    TraversalLimitReached,
    /// An expanded ClinicalTrials.gov worker failed, leaving coverage incomplete.
    IncompleteCoverage,
}
