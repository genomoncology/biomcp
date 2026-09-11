//! Trial entity models and workflows exposed through the stable trial facade.

use biodata::{
    ClinicalTrialContact, ClinicalTrialEligibility, ClinicalTrialPlannedOutcome,
    ClinicalTrialReference, ClinicalTrialSite, ClinicalTrialSiteDirectory,
};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

use crate::error::BioMcpError;

mod design;
mod documents;
mod eligibility;
mod get;
mod search;
#[cfg(test)]
mod test_support;

pub use self::design::{TrialDesign, TrialDesignError};
pub use self::documents::{
    TrialDocumentsManifest, TrialEligibilityProvenance, trial_document_bytes,
    trial_documents_manifest,
};
pub use self::get::get;
#[cfg(test)]
pub(crate) use self::get::product_design;
pub use self::search::{count_all, search, search_page};

pub(crate) fn validate_search_filters(filters: &TrialSearchFilters) -> Result<(), BioMcpError> {
    search::validate_trial_search(filters).map(|_| ())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trial {
    #[serde(default)]
    pub identities: Vec<TrialIdentity>,
    pub nct_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub official_title: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub why_stopped: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(default)]
    pub phases: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub study_type: Option<String>,
    #[serde(default)]
    pub conditions: Vec<String>,
    #[serde(flatten)]
    pub design: TrialDesign,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrollment: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_date: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "eligibility_wire"
    )]
    pub eligibility: Option<ClinicalTrialEligibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eligibility_provenance: Option<TrialEligibilityProvenance>,
    #[serde(skip)]
    pub(crate) site_directory: Option<ClinicalTrialSiteDirectory>,
    #[serde(skip)]
    pub(crate) site_offset: usize,
    #[serde(skip)]
    pub(crate) site_limit: Option<usize>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "outcome_wire::serialize",
        deserialize_with = "outcome_wire::deserialize"
    )]
    pub outcomes: Option<Vec<ClinicalTrialPlannedOutcome>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "reference_wire"
    )]
    pub references: Option<Vec<ClinicalTrialReference>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrialIdentity {
    pub authority: String,
    pub identifier: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrialSectionState {
    NotRequested,
    Unavailable,
    Absent,
    Present,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrialSectionStates {
    pub arms: TrialSectionState,
    pub eligibility: TrialSectionState,
    pub outcomes: TrialSectionState,
    pub references: TrialSectionState,
    pub contacts: TrialSectionState,
    pub locations: TrialSectionState,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrialResponse {
    #[serde(flatten)]
    pub trial: Trial,
    pub section_states: TrialSectionStates,
}

impl Serialize for TrialResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Wire<'a> {
            #[serde(flatten)]
            trial: &'a Trial,
            #[serde(skip_serializing_if = "Option::is_none")]
            contacts: Option<Vec<TrialContactView<'a>>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            locations: Option<Vec<TrialLocationView<'a>>>,
            section_states: &'a TrialSectionStates,
        }
        Wire {
            trial: &self.trial,
            contacts: (self.section_states.contacts == TrialSectionState::Present)
                .then(|| self.trial.contact_views()),
            locations: (self.section_states.locations == TrialSectionState::Present)
                .then(|| self.trial.location_views()),
            section_states: &self.section_states,
        }
        .serialize(serializer)
    }
}

impl Deref for TrialResponse {
    type Target = Trial;

    fn deref(&self) -> &Self::Target {
        &self.trial
    }
}

impl DerefMut for TrialResponse {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.trial
    }
}

impl Trial {
    pub(crate) fn set_site_directory(&mut self, directory: Option<ClinicalTrialSiteDirectory>) {
        self.site_directory = directory;
    }

    pub(crate) fn location_count(&self) -> usize {
        self.site_directory
            .as_ref()
            .and_then(ClinicalTrialSiteDirectory::sites)
            .map_or(0, <[_]>::len)
    }

    pub(crate) fn set_site_page(&mut self, offset: usize, limit: usize) {
        self.site_offset = offset;
        self.site_limit = Some(limit);
    }

    pub(crate) fn returned_location_count(&self) -> usize {
        self.paged_sites().len()
    }

    fn paged_sites(&self) -> &[ClinicalTrialSite] {
        let sites = self
            .site_directory
            .as_ref()
            .and_then(ClinicalTrialSiteDirectory::sites)
            .unwrap_or_default();
        let start = self.site_offset.min(sites.len());
        let end = self.site_limit.map_or(sites.len(), |limit| {
            start.saturating_add(limit).min(sites.len())
        });
        &sites[start..end]
    }

    pub(crate) fn contact_views(&self) -> Vec<TrialContactView<'_>> {
        let mut views = Vec::new();
        if let Some(directory) = self.site_directory.as_ref() {
            views.extend(
                directory
                    .central_contacts()
                    .unwrap_or_default()
                    .iter()
                    .map(TrialContactView::central),
            );
        }
        for site in self.paged_sites() {
            views.extend(
                site.contacts()
                    .unwrap_or_default()
                    .iter()
                    .map(|contact| TrialContactView::site(contact, site)),
            );
        }
        views
    }

    pub(crate) fn location_views(&self) -> Vec<TrialLocationView<'_>> {
        self.paged_sites()
            .iter()
            .map(TrialLocationView::new)
            .collect()
    }

    pub(crate) fn contact_render_values(&self) -> serde_json::Value {
        serde_json::to_value(self.contact_views()).expect("borrowed contact views serialize")
    }

    pub(crate) fn location_render_values(&self) -> serde_json::Value {
        serde_json::to_value(self.location_views()).expect("borrowed location views serialize")
    }

    pub(crate) fn has_arms(&self) -> bool {
        self.design.arms().is_some()
    }

    pub(crate) fn has_eligibility_age(&self) -> bool {
        self.eligibility
            .as_ref()
            .is_some_and(|value| value.age_range().is_some())
    }
}

#[derive(Serialize)]
pub(crate) struct TrialSiteContactView<'a> {
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
    fn new(site: &'a ClinicalTrialSite) -> Self {
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
    fn central(contact: &'a ClinicalTrialContact) -> Self {
        Self {
            level: "central",
            contact: contact.into(),
            facility: None,
            city: None,
            state: None,
            country: None,
        }
    }

    fn site(contact: &'a ClinicalTrialContact, site: &'a ClinicalTrialSite) -> Self {
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

#[derive(Debug, Clone, PartialEq)]
pub struct TrialAge {
    number: Option<f64>,
    unit: Option<TrialAgeUnit>,
    original: String,
}

use eligibility as eligibility_wire;

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

    pub fn number(&self) -> Option<f64> {
        self.number
    }

    pub fn unit(&self) -> Option<&'static str> {
        self.unit.map(TrialAgeUnit::as_str)
    }

    #[cfg(test)]
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

    #[cfg(test)]
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
}

pub(crate) mod outcome_wire {
    use biodata::{ClinicalTrialPlannedOutcome, ExtensibleCode};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    const AUTHORITY: &str = "clinicaltrials.gov";
    const INVALID_OUTCOME: &str = "invalid clinical trial planned outcome";

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
        outcomes: &Option<Vec<ClinicalTrialPlannedOutcome>>,
    ) -> Result<Option<GroupedView<'_>>, &'static str> {
        outcomes
            .as_deref()
            .map(|values| {
                let mut grouped = GroupedView {
                    primary: Vec::new(),
                    secondary: Vec::new(),
                    other: Vec::new(),
                };
                for outcome in values {
                    let classification = outcome.source_classification();
                    if classification.authority() != AUTHORITY {
                        return Err(INVALID_OUTCOME);
                    }
                    let row = RowView::from(outcome);
                    match classification.code() {
                        "primaryOutcomes" => grouped.primary.push(row),
                        "secondaryOutcomes" => grouped.secondary.push(row),
                        "otherOutcomes" => grouped.other.push(row),
                        _ => return Err(INVALID_OUTCOME),
                    }
                }
                Ok(grouped)
            })
            .transpose()
    }

    pub(super) fn serialize<S: Serializer>(
        outcomes: &Option<Vec<ClinicalTrialPlannedOutcome>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        views(outcomes)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Grouped {
        #[serde(default)]
        primary: Vec<Row>,
        #[serde(default)]
        secondary: Vec<Row>,
        #[serde(default)]
        other: Vec<Row>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Row {
        measure: String,
        description: Option<String>,
        time_frame: Option<String>,
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Vec<ClinicalTrialPlannedOutcome>>, D::Error> {
        Option::<Grouped>::deserialize(deserializer)?
            .map(|grouped| {
                [
                    ("primaryOutcomes", grouped.primary),
                    ("secondaryOutcomes", grouped.secondary),
                    ("otherOutcomes", grouped.other),
                ]
                .into_iter()
                .flat_map(|(classification, rows)| {
                    rows.into_iter().map(move |row| (classification, row))
                })
                .map(|(classification, row)| {
                    let classification = ExtensibleCode::new(
                        AUTHORITY,
                        classification,
                        None::<String>,
                        None::<String>,
                        None::<String>,
                    )
                    .map_err(|_| serde::de::Error::custom(INVALID_OUTCOME))?;
                    ClinicalTrialPlannedOutcome::new(
                        row.measure,
                        row.description,
                        row.time_frame,
                        classification,
                    )
                    .map_err(|_| serde::de::Error::custom(INVALID_OUTCOME))
                })
                .collect()
            })
            .transpose()
    }
}

pub(crate) mod reference_wire {
    use biodata::ClinicalTrialReference;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde_json::value::RawValue;

    const INVALID_REFERENCE: &str = "invalid clinical trial reference";

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

    pub(crate) fn views(references: &Option<Vec<ClinicalTrialReference>>) -> Option<Vec<View<'_>>> {
        references
            .as_ref()
            .map(|values| values.iter().map(view).collect())
    }

    pub(super) fn serialize<S: Serializer>(
        references: &Option<Vec<ClinicalTrialReference>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let Some(references) = references else {
            return serializer.serialize_none();
        };
        let encoded = references
            .iter()
            .map(|reference| {
                let json = reference
                    .to_json()
                    .map_err(|_| serde::ser::Error::custom(INVALID_REFERENCE))?;
                RawValue::from_string(json)
                    .map_err(|_| serde::ser::Error::custom(INVALID_REFERENCE))
            })
            .collect::<Result<Vec<_>, S::Error>>()?;
        encoded.serialize(serializer)
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Vec<ClinicalTrialReference>>, D::Error> {
        Option::<Vec<Box<RawValue>>>::deserialize(deserializer)?
            .map(|values| {
                values
                    .into_iter()
                    .map(|value| {
                        ClinicalTrialReference::from_json_bytes(value.get().as_bytes())
                            .map_err(|_| serde::de::Error::custom(INVALID_REFERENCE))
                    })
                    .collect()
            })
            .transpose()
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
