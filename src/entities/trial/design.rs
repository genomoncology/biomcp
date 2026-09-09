use self::shared::{
    ClinicalTrialArm, ClinicalTrialArmId, ClinicalTrialArmInterventionAssignment,
    ClinicalTrialArmRelationshipError, ClinicalTrialArms, ClinicalTrialIntervention,
    ClinicalTrialInterventionId, ExtensibleCode,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A failure validating BioMCP's trial design representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TrialDesignError {
    /// Arms and their assignments were not both present or absent.
    SectionPresenceMismatch,
    /// A typed arm relationship failed local validation.
    InvalidRelationship(ClinicalTrialArmRelationshipError),
}

impl TrialDesignError {
    /// Returns the typed relationship failure when one caused this error.
    pub const fn relationship_error(&self) -> Option<&ClinicalTrialArmRelationshipError> {
        match self {
            Self::SectionPresenceMismatch => None,
            Self::InvalidRelationship(source) => Some(source),
        }
    }
}

pub(crate) mod shared {
    use std::collections::HashSet;
    use std::error::Error;
    use std::fmt::{self, Display, Formatter};

    use serde::{Deserialize, Serialize};

    const MAX_PORTABLE_JSON_INTEGER: u64 = 9_007_199_254_740_991;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct ClinicalTrialIdentityError;

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub(crate) struct ExtensibleCode {
        authority: String,
        code: String,
        display: Option<String>,
        vocabulary_version: Option<String>,
        recognized_meaning: Option<String>,
    }

    impl ExtensibleCode {
        pub(crate) fn new(
            authority: impl Into<String>,
            code: impl Into<String>,
            display: Option<impl Into<String>>,
            vocabulary_version: Option<impl Into<String>>,
            recognized_meaning: Option<impl Into<String>>,
        ) -> Result<Self, ()> {
            let value = Self {
                authority: authority.into(),
                code: code.into(),
                display: display.map(Into::into),
                vocabulary_version: vocabulary_version.map(Into::into),
                recognized_meaning: recognized_meaning.map(Into::into),
            };
            if value.authority.is_empty()
                || value.code.is_empty()
                || [
                    &value.display,
                    &value.vocabulary_version,
                    &value.recognized_meaning,
                ]
                .into_iter()
                .flatten()
                .any(String::is_empty)
            {
                return Err(());
            }
            Ok(value)
        }

        pub(crate) fn authority(&self) -> &str {
            &self.authority
        }
        pub(crate) fn code(&self) -> &str {
            &self.code
        }
        pub(crate) fn display(&self) -> Option<&str> {
            self.display.as_deref()
        }
        pub(crate) fn vocabulary_version(&self) -> Option<&str> {
            self.vocabulary_version.as_deref()
        }
        pub(crate) fn recognized_meaning(&self) -> Option<&str> {
            self.recognized_meaning.as_deref()
        }
    }

    macro_rules! entity_id {
        ($name:ident) => {
            #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
            pub struct $name(u64);
            impl $name {
                pub fn new(value: u64) -> Result<Self, ClinicalTrialIdentityError> {
                    (value != 0 && value <= MAX_PORTABLE_JSON_INTEGER)
                        .then_some(Self(value))
                        .ok_or(ClinicalTrialIdentityError)
                }
                pub const fn get(self) -> u64 {
                    self.0
                }
            }
        };
    }

    entity_id!(ClinicalTrialArmId);
    entity_id!(ClinicalTrialInterventionId);
    entity_id!(ClinicalTrialEligibilityCriterionId);

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub(crate) struct ClinicalTrialIntervention {
        id: ClinicalTrialInterventionId,
        name: String,
        source_type: Option<ExtensibleCode>,
        description: Option<String>,
        other_names: Option<Vec<String>>,
    }

    impl ClinicalTrialIntervention {
        pub(crate) fn new(
            id: ClinicalTrialInterventionId,
            name: impl Into<String>,
            source_type: Option<ExtensibleCode>,
            description: Option<String>,
            other_names: Option<Vec<String>>,
        ) -> Result<Self, ()> {
            let value = Self {
                id,
                name: name.into(),
                source_type,
                description,
                other_names,
            };
            if value.name.is_empty()
                || value.description.as_ref().is_some_and(String::is_empty)
                || value
                    .other_names
                    .as_ref()
                    .is_some_and(|rows| rows.iter().any(String::is_empty))
            {
                return Err(());
            }
            Ok(value)
        }
        pub(crate) const fn id(&self) -> ClinicalTrialInterventionId {
            self.id
        }
        pub(crate) fn name(&self) -> &str {
            &self.name
        }
        pub(crate) const fn source_type(&self) -> Option<&ExtensibleCode> {
            self.source_type.as_ref()
        }
        pub(crate) fn description(&self) -> Option<&str> {
            self.description.as_deref()
        }
        pub(crate) fn other_names(&self) -> Option<&[String]> {
            self.other_names.as_deref()
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub(crate) struct ClinicalTrialArm {
        id: ClinicalTrialArmId,
        name: String,
        source_type: Option<ExtensibleCode>,
        description: Option<String>,
    }

    impl ClinicalTrialArm {
        pub(crate) fn new(
            id: ClinicalTrialArmId,
            name: impl Into<String>,
            source_type: Option<ExtensibleCode>,
            description: Option<String>,
        ) -> Result<Self, ()> {
            let value = Self {
                id,
                name: name.into(),
                source_type,
                description,
            };
            if value.name.is_empty() || value.description.as_ref().is_some_and(String::is_empty) {
                return Err(());
            }
            Ok(value)
        }
        pub(crate) const fn id(&self) -> ClinicalTrialArmId {
            self.id
        }
        pub(crate) fn name(&self) -> &str {
            &self.name
        }
        pub(crate) const fn source_type(&self) -> Option<&ExtensibleCode> {
            self.source_type.as_ref()
        }
        pub(crate) fn description(&self) -> Option<&str> {
            self.description.as_deref()
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) struct ClinicalTrialArmInterventionAssignment {
        arm_id: ClinicalTrialArmId,
        intervention_id: ClinicalTrialInterventionId,
    }

    impl ClinicalTrialArmInterventionAssignment {
        pub(crate) const fn new(
            arm_id: ClinicalTrialArmId,
            intervention_id: ClinicalTrialInterventionId,
        ) -> Self {
            Self {
                arm_id,
                intervention_id,
            }
        }
        pub(crate) const fn arm_id(&self) -> ClinicalTrialArmId {
            self.arm_id
        }
        pub(crate) const fn intervention_id(&self) -> ClinicalTrialInterventionId {
            self.intervention_id
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum ClinicalTrialArmRelationshipError {
        DuplicateArmId {
            arm_id: ClinicalTrialArmId,
        },
        DuplicateInterventionId {
            intervention_id: ClinicalTrialInterventionId,
        },
        MissingArmEndpoint {
            arm_id: ClinicalTrialArmId,
        },
        MissingInterventionEndpoint {
            intervention_id: ClinicalTrialInterventionId,
        },
        DuplicateAssignment {
            arm_id: ClinicalTrialArmId,
            intervention_id: ClinicalTrialInterventionId,
        },
    }

    impl Display for ClinicalTrialArmRelationshipError {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
            formatter.write_str("clinical trial arm relationship is invalid")
        }
    }
    impl Error for ClinicalTrialArmRelationshipError {}

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub(crate) struct ClinicalTrialArms {
        arms: Vec<ClinicalTrialArm>,
        assignments: Vec<ClinicalTrialArmInterventionAssignment>,
    }
    impl ClinicalTrialArms {
        pub(crate) fn new(
            arms: Vec<ClinicalTrialArm>,
            interventions: &[ClinicalTrialIntervention],
            assignments: Vec<ClinicalTrialArmInterventionAssignment>,
        ) -> Result<Self, ClinicalTrialArmRelationshipError> {
            Self::validate(&arms, interventions, &assignments)?;
            Ok(Self { arms, assignments })
        }
        pub(crate) fn validate(
            arms: &[ClinicalTrialArm],
            interventions: &[ClinicalTrialIntervention],
            assignments: &[ClinicalTrialArmInterventionAssignment],
        ) -> Result<(), ClinicalTrialArmRelationshipError> {
            let mut arm_ids = HashSet::new();
            for arm in arms {
                if !arm_ids.insert(arm.id()) {
                    return Err(ClinicalTrialArmRelationshipError::DuplicateArmId {
                        arm_id: arm.id(),
                    });
                }
            }
            let mut intervention_ids = HashSet::new();
            for value in interventions {
                if !intervention_ids.insert(value.id()) {
                    return Err(ClinicalTrialArmRelationshipError::DuplicateInterventionId {
                        intervention_id: value.id(),
                    });
                }
            }
            let mut pairs = HashSet::new();
            for value in assignments {
                if !arm_ids.contains(&value.arm_id()) {
                    return Err(ClinicalTrialArmRelationshipError::MissingArmEndpoint {
                        arm_id: value.arm_id(),
                    });
                }
                if !intervention_ids.contains(&value.intervention_id()) {
                    return Err(
                        ClinicalTrialArmRelationshipError::MissingInterventionEndpoint {
                            intervention_id: value.intervention_id(),
                        },
                    );
                }
                if !pairs.insert((value.arm_id(), value.intervention_id())) {
                    return Err(ClinicalTrialArmRelationshipError::DuplicateAssignment {
                        arm_id: value.arm_id(),
                        intervention_id: value.intervention_id(),
                    });
                }
            }
            Ok(())
        }
        pub(crate) fn arms(&self) -> &[ClinicalTrialArm] {
            &self.arms
        }
        pub(crate) fn assignments(&self) -> &[ClinicalTrialArmInterventionAssignment] {
            &self.assignments
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub(crate) enum ClinicalTrialSection<T> {
        Present(T),
        Absent,
        NotRequested,
        Unavailable,
    }
    const _: ClinicalTrialSection<()> = ClinicalTrialSection::Unavailable;

    #[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub(crate) enum Bound {
        Minimum,
        Maximum,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub(crate) enum DurationUnit {
        Years,
        Months,
        Weeks,
        Days,
        Hours,
        Minutes,
        Seconds,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub(crate) struct Duration {
        source: String,
        source_quantity: String,
        source_unit: DurationUnit,
        bound: Bound,
    }
    impl Duration {
        pub(crate) fn source(&self) -> &str {
            &self.source
        }
        pub(crate) fn source_quantity(&self) -> &str {
            &self.source_quantity
        }
        pub(crate) const fn source_unit(&self) -> DurationUnit {
            self.source_unit
        }
        pub(crate) const fn bound(&self) -> Bound {
            self.bound
        }
    }

    #[derive(Debug)]
    pub(crate) enum ParseOutcome<T> {
        Parsed(T),
        Unparsed,
    }
    #[derive(Default)]
    pub(crate) struct TemporalParser {
        _private: (),
    }
    impl TemporalParser {
        pub(crate) fn parse_duration(&self, source: &str, bound: Bound) -> ParseOutcome<Duration> {
            let Some((quantity, unit)) = source.split_once(' ') else {
                return ParseOutcome::Unparsed;
            };
            if quantity.is_empty()
                || quantity
                    .parse::<f64>()
                    .ok()
                    .filter(|v| v.is_finite() && *v >= 0.0)
                    .is_none()
            {
                return ParseOutcome::Unparsed;
            }
            let source_unit = match unit.to_ascii_lowercase().as_str() {
                "year" | "years" => DurationUnit::Years,
                "month" | "months" => DurationUnit::Months,
                "week" | "weeks" => DurationUnit::Weeks,
                "day" | "days" => DurationUnit::Days,
                "hour" | "hours" => DurationUnit::Hours,
                "minute" | "minutes" => DurationUnit::Minutes,
                "second" | "seconds" => DurationUnit::Seconds,
                _ => return ParseOutcome::Unparsed,
            };
            ParseOutcome::Parsed(Duration {
                source: source.to_string(),
                source_quantity: quantity.to_string(),
                source_unit,
                bound,
            })
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum ClinicalTrialAgeBoundForm {
        Limited,
        SourceStatedNoLimit,
    }
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub(crate) struct ClinicalTrialAgeBound {
        source: Duration,
        form: ClinicalTrialAgeBoundForm,
    }
    impl ClinicalTrialAgeBound {
        pub(crate) fn limited(source: Duration) -> Result<Self, ()> {
            Ok(Self {
                source,
                form: ClinicalTrialAgeBoundForm::Limited,
            })
        }
        pub(crate) fn source_stated_no_limit(source: Duration) -> Result<Self, ()> {
            if source.bound != Bound::Maximum
                || source.source_quantity != "999"
                || source.source_unit != DurationUnit::Years
            {
                return Err(());
            }
            Ok(Self {
                source,
                form: ClinicalTrialAgeBoundForm::SourceStatedNoLimit,
            })
        }
        pub(crate) fn source(&self) -> &Duration {
            &self.source
        }
        pub(crate) const fn form(&self) -> ClinicalTrialAgeBoundForm {
            self.form
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub(crate) struct ClinicalTrialAgeRange {
        minimum: Option<ClinicalTrialAgeBound>,
        maximum: Option<ClinicalTrialAgeBound>,
    }
    impl ClinicalTrialAgeRange {
        pub(crate) fn new(
            minimum: Option<ClinicalTrialAgeBound>,
            maximum: Option<ClinicalTrialAgeBound>,
        ) -> Result<Self, ()> {
            if minimum
                .as_ref()
                .is_some_and(|v| v.source.bound != Bound::Minimum)
                || maximum
                    .as_ref()
                    .is_some_and(|v| v.source.bound != Bound::Maximum)
            {
                return Err(());
            }
            Ok(Self { minimum, maximum })
        }
        pub(crate) fn minimum(&self) -> Option<&ClinicalTrialAgeBound> {
            self.minimum.as_ref()
        }
        pub(crate) fn maximum(&self) -> Option<&ClinicalTrialAgeBound> {
            self.maximum.as_ref()
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub(crate) enum ClinicalTrialEligibilityClassification {
        Inclusion,
        Exclusion,
        Other(ExtensibleCode),
    }
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub(crate) struct ClinicalTrialEligibilityCriterion {
        id: ClinicalTrialEligibilityCriterionId,
        description: String,
        classification: ClinicalTrialEligibilityClassification,
    }
    impl ClinicalTrialEligibilityCriterion {
        pub(crate) fn new(
            id: ClinicalTrialEligibilityCriterionId,
            description: impl Into<String>,
            classification: ClinicalTrialEligibilityClassification,
        ) -> Result<Self, ()> {
            let description = description.into();
            if description.trim().is_empty() {
                return Err(());
            }
            Ok(Self {
                id,
                description,
                classification,
            })
        }
        pub(crate) const fn id(&self) -> ClinicalTrialEligibilityCriterionId {
            self.id
        }
        pub(crate) fn description(&self) -> &str {
            &self.description
        }
        pub(crate) const fn classification(&self) -> &ClinicalTrialEligibilityClassification {
            &self.classification
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub(crate) struct ClinicalTrialEligibility {
        registry_text: Option<String>,
        age_range: Option<ClinicalTrialAgeRange>,
        sexes: Option<Vec<ExtensibleCode>>,
        includes_healthy_subjects: Option<bool>,
        criteria: Option<Vec<ClinicalTrialEligibilityCriterion>>,
    }
    impl ClinicalTrialEligibility {
        pub(crate) fn new(
            registry_text: Option<String>,
            age_range: Option<ClinicalTrialAgeRange>,
            sexes: Option<Vec<ExtensibleCode>>,
            includes_healthy_subjects: Option<bool>,
            criteria: Option<Vec<ClinicalTrialEligibilityCriterion>>,
        ) -> Result<Self, ()> {
            if registry_text.as_ref().is_some_and(|v| v.trim().is_empty()) {
                return Err(());
            }
            let mut ids = HashSet::new();
            if criteria
                .as_ref()
                .is_some_and(|rows| rows.iter().any(|row| !ids.insert(row.id())))
            {
                return Err(());
            }
            Ok(Self {
                registry_text,
                age_range,
                sexes,
                includes_healthy_subjects,
                criteria,
            })
        }
        pub(crate) fn registry_text(&self) -> Option<&str> {
            self.registry_text.as_deref()
        }
        pub(crate) const fn age_range(&self) -> Option<&ClinicalTrialAgeRange> {
            self.age_range.as_ref()
        }
        pub(crate) fn sexes(&self) -> Option<&[ExtensibleCode]> {
            self.sexes.as_deref()
        }
        pub(crate) const fn includes_healthy_subjects(&self) -> Option<bool> {
            self.includes_healthy_subjects
        }
        pub(crate) fn criteria(&self) -> Option<&[ClinicalTrialEligibilityCriterion]> {
            self.criteria.as_deref()
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub(crate) struct ClinicalTrialReference {
        pmid: Option<String>,
        citation: Option<String>,
        source_type: Option<ExtensibleCode>,
    }
    impl ClinicalTrialReference {
        pub(crate) fn new(
            pmid: Option<String>,
            citation: Option<String>,
            source_type: Option<ExtensibleCode>,
        ) -> Result<Self, ()> {
            if [&pmid, &citation]
                .into_iter()
                .flatten()
                .any(String::is_empty)
            {
                return Err(());
            }
            Ok(Self {
                pmid,
                citation,
                source_type,
            })
        }
        pub(crate) fn pmid(&self) -> Option<&str> {
            self.pmid.as_deref()
        }
        pub(crate) fn citation(&self) -> Option<&str> {
            self.citation.as_deref()
        }
        pub(crate) const fn source_type(&self) -> Option<&ExtensibleCode> {
            self.source_type.as_ref()
        }
    }
}

impl std::fmt::Display for TrialDesignError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SectionPresenceMismatch => {
                formatter.write_str("trial design arm sections must be present together")
            }
            Self::InvalidRelationship(_) => {
                formatter.write_str("trial design arm relationship is invalid")
            }
        }
    }
}

impl std::error::Error for TrialDesignError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.relationship_error()
            .map(|source| source as &(dyn std::error::Error + 'static))
    }
}

#[derive(Clone, Debug, Default)]
pub struct TrialDesign {
    interventions: Vec<ClinicalTrialIntervention>,
    arms: Option<Vec<ClinicalTrialArm>>,
    assignments: Option<Vec<ClinicalTrialArmInterventionAssignment>>,
}

impl TrialDesign {
    #[cfg(test)]
    pub(crate) fn from_names(names: &[&str]) -> Self {
        let interventions = names
            .iter()
            .enumerate()
            .map(|(index, name)| {
                ClinicalTrialIntervention::new(
                    ClinicalTrialInterventionId::new((index + 1) as u64).expect("test identity"),
                    *name,
                    None,
                    None,
                    None,
                )
                .expect("test intervention")
            })
            .collect();
        Self::new(interventions, None, None).expect("test design")
    }

    #[cfg(test)]
    pub(crate) fn from_names_and_arm(
        names: &[&str],
        arm_name: &str,
        arm_type: Option<&str>,
        description: Option<&str>,
    ) -> Self {
        let plain = Self::from_names(names);
        let arm_id = ClinicalTrialArmId::new(1).expect("test arm identity");
        let source_type = arm_type.map(|code| {
            ExtensibleCode::new("test", code, None::<String>, None::<String>, None::<String>)
                .expect("test arm type")
        });
        let arm = ClinicalTrialArm::new(
            arm_id,
            arm_name,
            source_type,
            description.map(str::to_owned),
        )
        .expect("test arm");
        let assignments = plain
            .interventions
            .iter()
            .map(|value| ClinicalTrialArmInterventionAssignment::new(arm_id, value.id()))
            .collect();
        Self::new(plain.interventions, Some(vec![arm]), Some(assignments)).expect("test design")
    }

    pub fn new(
        interventions: Vec<ClinicalTrialIntervention>,
        arms: Option<Vec<ClinicalTrialArm>>,
        assignments: Option<Vec<ClinicalTrialArmInterventionAssignment>>,
    ) -> Result<Self, TrialDesignError> {
        if arms.is_some() != assignments.is_some() {
            return Err(TrialDesignError::SectionPresenceMismatch);
        }
        ClinicalTrialArms::validate(
            arms.as_deref().unwrap_or_default(),
            &interventions,
            assignments.as_deref().unwrap_or_default(),
        )
        .map_err(TrialDesignError::InvalidRelationship)?;
        Ok(Self {
            interventions,
            arms,
            assignments,
        })
    }

    pub fn interventions(&self) -> &[ClinicalTrialIntervention] {
        &self.interventions
    }

    pub fn arms(&self) -> Option<&[ClinicalTrialArm]> {
        self.arms.as_deref()
    }

    pub fn assignments(&self) -> Option<&[ClinicalTrialArmInterventionAssignment]> {
        self.assignments.as_deref()
    }
}

#[derive(Serialize)]
struct CodeWire {
    authority: String,
    code: String,
    display: Option<String>,
    vocabulary_version: Option<String>,
    recognized_meaning: Option<String>,
}

impl<'de> Deserialize<'de> for CodeWire {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RequiredCodeWire {
            authority: String,
            code: String,
            display: serde_json::Value,
            vocabulary_version: serde_json::Value,
            recognized_meaning: serde_json::Value,
        }
        let value = RequiredCodeWire::deserialize(deserializer)?;
        let nullable = |field: serde_json::Value| match field {
            serde_json::Value::Null => Ok(None),
            serde_json::Value::String(value) => Ok(Some(value)),
            _ => Err(serde::de::Error::custom(
                "code optional members must be strings or null",
            )),
        };
        Ok(Self {
            authority: value.authority,
            code: value.code,
            display: nullable(value.display)?,
            vocabulary_version: nullable(value.vocabulary_version)?,
            recognized_meaning: nullable(value.recognized_meaning)?,
        })
    }
}

impl CodeWire {
    fn from_shared(value: &ExtensibleCode) -> Self {
        Self {
            authority: value.authority().to_string(),
            code: value.code().to_string(),
            display: value.display().map(str::to_owned),
            vocabulary_version: value.vocabulary_version().map(str::to_owned),
            recognized_meaning: value.recognized_meaning().map(str::to_owned),
        }
    }

    fn into_shared(self) -> Result<ExtensibleCode, ()> {
        ExtensibleCode::new(
            self.authority,
            self.code,
            self.display,
            self.vocabulary_version,
            self.recognized_meaning,
        )
        .map_err(|_| ())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InterventionWire {
    id: u64,
    name: String,
    #[serde(rename = "type")]
    source_type: Option<CodeWire>,
    description: Option<String>,
    other_names: Vec<String>,
}

impl InterventionWire {
    fn from_shared(value: &ClinicalTrialIntervention) -> Self {
        Self {
            id: value.id().get(),
            name: value.name().to_string(),
            source_type: value.source_type().map(CodeWire::from_shared),
            description: value.description().map(str::to_owned),
            other_names: value.other_names().unwrap_or_default().to_vec(),
        }
    }

    fn into_shared(self) -> Result<ClinicalTrialIntervention, ()> {
        ClinicalTrialIntervention::new(
            ClinicalTrialInterventionId::new(self.id).map_err(|_| ())?,
            self.name,
            self.source_type.map(CodeWire::into_shared).transpose()?,
            self.description,
            Some(self.other_names),
        )
        .map_err(|_| ())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArmWire {
    id: u64,
    name: String,
    #[serde(rename = "type")]
    source_type: Option<CodeWire>,
    description: Option<String>,
}

impl ArmWire {
    fn from_shared(value: &ClinicalTrialArm) -> Self {
        Self {
            id: value.id().get(),
            name: value.name().to_string(),
            source_type: value.source_type().map(CodeWire::from_shared),
            description: value.description().map(str::to_owned),
        }
    }

    fn into_shared(self) -> Result<ClinicalTrialArm, ()> {
        ClinicalTrialArm::new(
            ClinicalTrialArmId::new(self.id).map_err(|_| ())?,
            self.name,
            self.source_type.map(CodeWire::into_shared).transpose()?,
            self.description,
        )
        .map_err(|_| ())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AssignmentWire {
    arm_id: u64,
    intervention_id: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DesignWire {
    #[serde(default)]
    interventions: Vec<InterventionWire>,
    #[serde(skip_serializing_if = "Option::is_none")]
    arms: Option<Vec<ArmWire>>,
    #[serde(
        rename = "arm_intervention_assignments",
        skip_serializing_if = "Option::is_none"
    )]
    assignments: Option<Vec<AssignmentWire>>,
}

impl Serialize for TrialDesign {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        DesignWire {
            interventions: self
                .interventions
                .iter()
                .map(InterventionWire::from_shared)
                .collect(),
            arms: self
                .arms
                .as_ref()
                .map(|values| values.iter().map(ArmWire::from_shared).collect()),
            assignments: self.assignments.as_ref().map(|values| {
                values
                    .iter()
                    .map(|value| AssignmentWire {
                        arm_id: value.arm_id().get(),
                        intervention_id: value.intervention_id().get(),
                    })
                    .collect()
            }),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TrialDesign {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = DesignWire::deserialize(deserializer)?;
        let interventions = wire
            .interventions
            .into_iter()
            .map(InterventionWire::into_shared)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|()| serde::de::Error::custom("invalid trial intervention"))?;
        let arms = wire
            .arms
            .map(|values| {
                values
                    .into_iter()
                    .map(ArmWire::into_shared)
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()
            .map_err(|()| serde::de::Error::custom("invalid trial arm"))?;
        let assignments = wire
            .assignments
            .map(|values| {
                values
                    .into_iter()
                    .map(|value| {
                        Ok(ClinicalTrialArmInterventionAssignment::new(
                            ClinicalTrialArmId::new(value.arm_id).map_err(|_| ())?,
                            ClinicalTrialInterventionId::new(value.intervention_id)
                                .map_err(|_| ())?,
                        ))
                    })
                    .collect::<Result<Vec<_>, ()>>()
            })
            .transpose()
            .map_err(|()| serde::de::Error::custom("invalid trial assignment"))?;
        Self::new(interventions, arms, assignments)
            .map_err(|_| serde::de::Error::custom("invalid trial relationships"))
    }
}
