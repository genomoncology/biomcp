use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::value::RawValue;

use super::shared::{
    Bound, ClinicalTrialAgeBound, ClinicalTrialAgeRange, ClinicalTrialEligibility,
    ClinicalTrialEligibilityClassification, ClinicalTrialEligibilityCriterion,
    ClinicalTrialEligibilityCriterionId, DurationUnit, ExtensibleCode,
};

const INVALID_ELIGIBILITY: &str = "invalid clinical trial eligibility";
const NO_LIMIT_RULE: &str = "nci-cts-v2-999-years-no-upper-bound";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CodeWire {
    authority: String,
    code: String,
    #[serde(deserialize_with = "required_nullable")]
    display: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    vocabulary_version: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    recognized_meaning: Option<String>,
}

fn required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

impl CodeWire {
    fn from_value(value: &ExtensibleCode) -> Self {
        Self {
            authority: value.authority().to_string(),
            code: value.code().to_string(),
            display: value.display().map(str::to_owned),
            vocabulary_version: value.vocabulary_version().map(str::to_owned),
            recognized_meaning: value.recognized_meaning().map(str::to_owned),
        }
    }

    fn into_value(self) -> Result<ExtensibleCode, ()> {
        ExtensibleCode::new(
            self.authority,
            self.code,
            self.display,
            self.vocabulary_version,
            self.recognized_meaning,
        )
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum UnitWire {
    Years,
    Months,
    Weeks,
    Days,
    Hours,
    Minutes,
    Seconds,
}
impl From<DurationUnit> for UnitWire {
    fn from(value: DurationUnit) -> Self {
        match value {
            DurationUnit::Years => Self::Years,
            DurationUnit::Months => Self::Months,
            DurationUnit::Weeks => Self::Weeks,
            DurationUnit::Days => Self::Days,
            DurationUnit::Hours => Self::Hours,
            DurationUnit::Minutes => Self::Minutes,
            DurationUnit::Seconds => Self::Seconds,
        }
    }
}
impl From<UnitWire> for DurationUnit {
    fn from(value: UnitWire) -> Self {
        match value {
            UnitWire::Years => Self::Years,
            UnitWire::Months => Self::Months,
            UnitWire::Weeks => Self::Weeks,
            UnitWire::Days => Self::Days,
            UnitWire::Hours => Self::Hours,
            UnitWire::Minutes => Self::Minutes,
            UnitWire::Seconds => Self::Seconds,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleWire {
    name: String,
    version: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum AgeBoundWire {
    Limited {
        source: String,
        source_quantity: String,
        source_unit: UnitWire,
        bound: Bound,
    },
    SourceStatedNoLimit {
        source: String,
        source_quantity: String,
        source_unit: UnitWire,
        bound: Bound,
        rule: RuleWire,
    },
}

impl AgeBoundWire {
    fn from_value(value: &ClinicalTrialAgeBound) -> Self {
        let source = value.source();
        let fields = (
            source.source().to_string(),
            source.source_quantity().to_string(),
            source.source_unit().into(),
            source.bound(),
        );
        match value.form() {
            super::shared::ClinicalTrialAgeBoundForm::Limited => Self::Limited {
                source: fields.0,
                source_quantity: fields.1,
                source_unit: fields.2,
                bound: fields.3,
            },
            super::shared::ClinicalTrialAgeBoundForm::SourceStatedNoLimit => {
                Self::SourceStatedNoLimit {
                    source: fields.0,
                    source_quantity: fields.1,
                    source_unit: fields.2,
                    bound: fields.3,
                    rule: RuleWire {
                        name: NO_LIMIT_RULE.to_string(),
                        version: "1".to_string(),
                    },
                }
            }
        }
    }

    fn into_value(self) -> Result<ClinicalTrialAgeBound, ()> {
        let (source, source_quantity, source_unit, bound, no_limit) = match self {
            Self::Limited {
                source,
                source_quantity,
                source_unit,
                bound,
            } => (source, source_quantity, source_unit, bound, false),
            Self::SourceStatedNoLimit {
                source,
                source_quantity,
                source_unit,
                bound,
                rule,
            } => {
                if rule.name != NO_LIMIT_RULE || rule.version != "1" {
                    return Err(());
                }
                (source, source_quantity, source_unit, bound, true)
            }
        };
        let parsed = super::shared::TemporalParser::default().parse_duration(&source, bound);
        let super::shared::ParseOutcome::Parsed(duration) = parsed else {
            return Err(());
        };
        if duration.source_quantity() != source_quantity
            || duration.source_unit() != DurationUnit::from(source_unit)
        {
            return Err(());
        }
        if no_limit {
            ClinicalTrialAgeBound::source_stated_no_limit(duration)
        } else {
            ClinicalTrialAgeBound::limited(duration)
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AgeRangeWire {
    #[serde(deserialize_with = "required_nullable")]
    minimum: Option<AgeBoundWire>,
    #[serde(deserialize_with = "required_nullable")]
    maximum: Option<AgeBoundWire>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ClassificationWire {
    Inclusion,
    Exclusion,
    Other { source: CodeWire },
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CriterionWire {
    id: u64,
    description: String,
    classification: ClassificationWire,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EligibilityWire {
    #[serde(deserialize_with = "required_nullable")]
    registry_text: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    age_range: Option<AgeRangeWire>,
    #[serde(deserialize_with = "required_nullable")]
    sexes: Option<Vec<CodeWire>>,
    #[serde(deserialize_with = "required_nullable")]
    includes_healthy_subjects: Option<bool>,
    #[serde(deserialize_with = "required_nullable")]
    criteria: Option<Vec<CriterionWire>>,
}

#[derive(Serialize)]
struct EligibilityRef {
    registry_text: Option<String>,
    age_range: Option<AgeRangeWire>,
    sexes: Option<Vec<CodeWire>>,
    includes_healthy_subjects: Option<bool>,
    criteria: Option<Vec<CriterionWire>>,
}

impl ClinicalTrialEligibility {
    pub(crate) fn to_json(&self) -> Result<String, ()> {
        let age_range = self.age_range().map(|value| AgeRangeWire {
            minimum: value.minimum().map(AgeBoundWire::from_value),
            maximum: value.maximum().map(AgeBoundWire::from_value),
        });
        let criteria = self.criteria().map(|rows| {
            rows.iter()
                .map(|value| CriterionWire {
                    id: value.id().get(),
                    description: value.description().to_string(),
                    classification: match value.classification() {
                        ClinicalTrialEligibilityClassification::Inclusion => {
                            ClassificationWire::Inclusion
                        }
                        ClinicalTrialEligibilityClassification::Exclusion => {
                            ClassificationWire::Exclusion
                        }
                        ClinicalTrialEligibilityClassification::Other(source) => {
                            ClassificationWire::Other {
                                source: CodeWire::from_value(source),
                            }
                        }
                    },
                })
                .collect()
        });
        serde_json::to_string(&EligibilityRef {
            registry_text: self.registry_text().map(str::to_owned),
            age_range,
            sexes: self
                .sexes()
                .map(|rows| rows.iter().map(CodeWire::from_value).collect()),
            includes_healthy_subjects: self.includes_healthy_subjects(),
            criteria,
        })
        .map_err(|_| ())
    }

    pub(crate) fn from_json_bytes(input: &[u8]) -> Result<Self, ()> {
        super::strict_json::validate(input).map_err(|_| ())?;
        let wire: EligibilityWire = serde_json::from_slice(input).map_err(|_| ())?;
        let registry_text = wire.registry_text;
        let age_range = wire
            .age_range
            .map(|range| {
                ClinicalTrialAgeRange::new(
                    range.minimum.map(AgeBoundWire::into_value).transpose()?,
                    range.maximum.map(AgeBoundWire::into_value).transpose()?,
                )
            })
            .transpose()?;
        let sexes = wire
            .sexes
            .map(|rows| rows.into_iter().map(CodeWire::into_value).collect())
            .transpose()?;
        let criteria = wire
            .criteria
            .map(|rows| {
                rows.into_iter()
                    .map(|value| {
                        let classification = match value.classification {
                            ClassificationWire::Inclusion => {
                                ClinicalTrialEligibilityClassification::Inclusion
                            }
                            ClassificationWire::Exclusion => {
                                ClinicalTrialEligibilityClassification::Exclusion
                            }
                            ClassificationWire::Other { source } => {
                                ClinicalTrialEligibilityClassification::Other(source.into_value()?)
                            }
                        };
                        ClinicalTrialEligibilityCriterion::new(
                            ClinicalTrialEligibilityCriterionId::new(value.id).map_err(|_| ())?,
                            value.description,
                            classification,
                        )
                    })
                    .collect()
            })
            .transpose()?;
        ClinicalTrialEligibility::new(
            registry_text,
            age_range,
            sexes,
            wire.includes_healthy_subjects,
            criteria,
        )
    }
}

pub(super) fn serialize<S: Serializer>(
    value: &Option<ClinicalTrialEligibility>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let Some(value) = value else {
        return serializer.serialize_none();
    };
    let json = value
        .to_json()
        .map_err(|_| serde::ser::Error::custom(INVALID_ELIGIBILITY))?;
    RawValue::from_string(json)
        .map_err(|_| serde::ser::Error::custom(INVALID_ELIGIBILITY))?
        .serialize(serializer)
}

pub(super) fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<ClinicalTrialEligibility>, D::Error> {
    Option::<Box<RawValue>>::deserialize(deserializer)?
        .map(|raw| ClinicalTrialEligibility::from_json_bytes(raw.get().as_bytes()))
        .transpose()
        .map_err(|_| serde::de::Error::custom(INVALID_ELIGIBILITY))
}

#[cfg(test)]
mod tests {
    use super::super::shared::ClinicalTrialEligibility;
    use serde::{Deserialize, Serialize};
    use serde_json::{Value, json};

    #[derive(Deserialize, Serialize)]
    struct EligibilityHolder {
        #[serde(with = "super")]
        eligibility: Option<ClinicalTrialEligibility>,
    }

    fn code(authority: &str, code: &str, display: Option<&str>) -> Value {
        json!({
            "authority": authority,
            "code": code,
            "display": display,
            "vocabulary_version": null,
            "recognized_meaning": null
        })
    }

    fn limited(source: &str, quantity: &str, bound: &str) -> Value {
        json!({
            "kind": "limited",
            "source": source,
            "source_quantity": quantity,
            "source_unit": "years",
            "bound": bound
        })
    }

    fn complete() -> Value {
        json!({
            "registry_text": "Include β participants.",
            "age_range": {
                "minimum": limited("18 Years", "18", "minimum"),
                "maximum": {
                    "kind": "source_stated_no_limit",
                    "source": "999 Years",
                    "source_quantity": "999",
                    "source_unit": "years",
                    "bound": "maximum",
                    "rule": {"name": "nci-cts-v2-999-years-no-upper-bound", "version": "1"}
                }
            },
            "sexes": [code("nci", "ALL", Some("All"))],
            "includes_healthy_subjects": false,
            "criteria": [
                {"id": 1, "description": "First", "classification": {"kind": "inclusion"}},
                {"id": 2, "description": "Second", "classification": {"kind": "exclusion"}},
                {"id": 3, "description": "Third", "classification": {
                    "kind": "other", "source": code("registry", "OTHER", None)
                }}
            ]
        })
    }

    fn round_trip(eligibility: Value) -> Result<Value, serde_json::Error> {
        let holder: EligibilityHolder =
            serde_json::from_value(json!({"eligibility": eligibility}))?;
        serde_json::to_value(holder).map(|value| value["eligibility"].clone())
    }

    #[test]
    fn direct_eligibility_wire_preserves_every_value_and_explicit_empty_collection() {
        let expected = complete();
        assert_eq!(round_trip(expected.clone()).unwrap(), expected);
        let empty = json!({
            "registry_text": null,
            "age_range": null,
            "sexes": [],
            "includes_healthy_subjects": false,
            "criteria": []
        });
        assert_eq!(round_trip(empty.clone()).unwrap(), empty);
        assert_eq!(round_trip(Value::Null).unwrap(), Value::Null);
    }

    #[test]
    fn direct_eligibility_wire_rejects_bad_identity_text_codes_and_members() {
        let mut cases = Vec::new();
        for id in [0, 9_007_199_254_740_992_u64] {
            let mut value = complete();
            value["criteria"][0]["id"] = json!(id);
            cases.push(value);
        }
        let mut duplicate = complete();
        duplicate["criteria"][1]["id"] = json!(1);
        cases.push(duplicate);
        for path in ["registry_text", "criteria"] {
            let mut value = complete();
            if path == "registry_text" {
                value[path] = json!(" \t");
            } else {
                value[path][0]["description"] = json!(" ");
            }
            cases.push(value);
        }
        let mut bad_code = complete();
        bad_code["sexes"][0]["authority"] = json!("");
        cases.push(bad_code);
        let mut unknown = complete();
        unknown["extra"] = json!(true);
        cases.push(unknown);
        let mut missing = complete();
        missing.as_object_mut().unwrap().remove("sexes");
        cases.push(missing);
        let mut wrong = complete();
        wrong["criteria"] = json!({});
        cases.push(wrong);
        for value in cases {
            assert!(round_trip(value.clone()).is_err(), "accepted {value}");
        }
    }

    #[test]
    fn direct_eligibility_wire_rejects_malformed_age_and_any_other_no_limit_rule() {
        let mut cases = Vec::new();
        for (path, replacement) in [
            ("source_quantity", json!("018")),
            ("source_unit", json!("months")),
            ("bound", json!("maximum")),
        ] {
            let mut value = complete();
            value["age_range"]["minimum"][path] = replacement;
            cases.push(value);
        }
        for (path, replacement) in [("name", json!("another-rule")), ("version", json!("2"))] {
            let mut value = complete();
            value["age_range"]["maximum"]["rule"][path] = replacement;
            cases.push(value);
        }
        let mut missing_bound = complete();
        missing_bound["age_range"]
            .as_object_mut()
            .unwrap()
            .remove("maximum");
        cases.push(missing_bound);
        for value in cases {
            assert!(round_trip(value.clone()).is_err(), "accepted {value}");
        }
    }

    #[test]
    fn direct_eligibility_wire_rejects_duplicate_members() {
        let input = r#"{"eligibility":{"registry_text":null,"registry_text":null,"age_range":null,"sexes":null,"includes_healthy_subjects":null,"criteria":null}}"#;
        assert!(serde_json::from_str::<EligibilityHolder>(input).is_err());
    }

    #[test]
    fn trial_wire_distinguishes_a_missing_field_from_an_all_null_aggregate() {
        let root = json!({
            "nct_id": "NCT00000001",
            "title": "Eligibility wire test",
            "status": "RECRUITING",
            "conditions": [],
            "interventions": []
        });
        let missing: super::super::Trial = serde_json::from_value(root.clone()).unwrap();
        assert!(missing.eligibility.is_none());
        assert!(
            serde_json::to_value(missing)
                .unwrap()
                .get("eligibility")
                .is_none()
        );

        let mut present = root;
        let expected = json!({
            "registry_text": null,
            "age_range": null,
            "sexes": null,
            "includes_healthy_subjects": null,
            "criteria": null
        });
        present["eligibility"] = expected.clone();
        let present: super::super::Trial = serde_json::from_value(present).unwrap();
        assert!(present.eligibility.is_some());
        assert_eq!(
            serde_json::to_value(present).unwrap()["eligibility"],
            expected
        );
    }
}

pub(crate) mod strict_json {
    use std::cell::Cell;
    use std::collections::HashSet;
    use std::fmt;
    use std::rc::Rc;

    use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};

    const MAX_BYTES: usize = 8 * 1024 * 1024;
    const MAX_DEPTH: usize = 128;
    const MAX_STRING_CHARS: usize = 4 * 1024 * 1024;
    const MAX_COLLECTION_ITEMS: usize = 1_000_000;
    const MAX_OBJECT_MEMBERS: usize = 16_384;
    const MAX_EVENTS: usize = 1_000_000;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum StrictJsonError {
        Malformed,
        Unsupported,
        Resource,
    }

    #[derive(Clone)]
    struct Seed {
        depth: usize,
        events: Rc<Cell<usize>>,
        count_event: bool,
    }

    impl Seed {
        fn child(&self) -> Self {
            Self {
                depth: self.depth + 1,
                events: self.events.clone(),
                count_event: true,
            }
        }
        fn string<E: serde::de::Error>(&self, value: &str) -> Result<(), E> {
            if value.chars().take(MAX_STRING_CHARS + 1).count() > MAX_STRING_CHARS {
                Err(E::custom("__resource_limit__"))
            } else {
                Ok(())
            }
        }
    }

    impl<'de> DeserializeSeed<'de> for Seed {
        type Value = ();
        fn deserialize<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
            if self.depth > MAX_DEPTH {
                return Err(serde::de::Error::custom("__resource_limit__"));
            }
            if self.count_event {
                let events = self
                    .events
                    .get()
                    .checked_add(1)
                    .ok_or_else(|| serde::de::Error::custom("__resource_limit__"))?;
                if events > MAX_EVENTS {
                    return Err(serde::de::Error::custom("__resource_limit__"));
                }
                self.events.set(events);
            }
            deserializer.deserialize_any(self)
        }
    }

    impl<'de> Visitor<'de> for Seed {
        type Value = ();
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a bounded JSON value")
        }
        fn visit_bool<E: serde::de::Error>(self, _: bool) -> Result<(), E> {
            Ok(())
        }
        fn visit_i64<E: serde::de::Error>(self, _: i64) -> Result<(), E> {
            Ok(())
        }
        fn visit_u64<E: serde::de::Error>(self, _: u64) -> Result<(), E> {
            Ok(())
        }
        fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<(), E> {
            Ok(())
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<(), E> {
            Ok(())
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<(), E> {
            Ok(())
        }
        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<(), E> {
            self.string(value)
        }
        fn visit_string<E: serde::de::Error>(self, value: String) -> Result<(), E> {
            self.string(&value)
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<(), A::Error> {
            let mut count = 0;
            while sequence.next_element_seed(self.child())?.is_some() {
                count += 1;
                if count > MAX_COLLECTION_ITEMS {
                    return Err(serde::de::Error::custom("__resource_limit__"));
                }
            }
            Ok(())
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
            let mut keys = HashSet::new();
            let mut count = 0;
            let mut duplicate = false;
            while let Some(key) = map.next_key::<String>()? {
                self.string::<A::Error>(&key)?;
                count += 1;
                if count > MAX_OBJECT_MEMBERS {
                    return Err(serde::de::Error::custom("__resource_limit__"));
                }
                duplicate |= !keys.insert(key);
                map.next_value_seed(self.child())?;
            }
            if duplicate {
                Err(serde::de::Error::custom("__duplicate_member__"))
            } else {
                Ok(())
            }
        }
        fn visit_newtype_struct<D: serde::Deserializer<'de>>(
            self,
            deserializer: D,
        ) -> Result<(), D::Error> {
            self.child().deserialize(deserializer)
        }
        fn visit_some<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
            self.child().deserialize(deserializer)
        }
        fn visit_bytes<E: serde::de::Error>(self, _: &[u8]) -> Result<(), E> {
            Err(E::custom("__unsupported_json__"))
        }
        fn visit_byte_buf<E: serde::de::Error>(self, _: Vec<u8>) -> Result<(), E> {
            Err(E::custom("__unsupported_json__"))
        }
    }

    pub(crate) fn validate(input: &[u8]) -> Result<(), StrictJsonError> {
        if input.len() > MAX_BYTES {
            return Err(StrictJsonError::Resource);
        }
        let mut deserializer = serde_json::Deserializer::from_slice(input);
        deserializer.disable_recursion_limit();
        let seed = Seed {
            depth: 0,
            events: Rc::new(Cell::new(0)),
            count_event: false,
        };
        seed.deserialize(&mut deserializer)
            .and_then(|()| deserializer.end())
            .map_err(|error| {
                let text = error.to_string();
                if text.contains("__duplicate_member__") || text.contains("__unsupported_json__") {
                    StrictJsonError::Unsupported
                } else if text.contains("__resource_limit__")
                    || text.contains("recursion limit exceeded")
                {
                    StrictJsonError::Resource
                } else {
                    StrictJsonError::Malformed
                }
            })
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn rejects_duplicate_members_at_any_depth() {
            assert_eq!(
                validate(br#"{"a":{"b":1,"b":2}}"#),
                Err(StrictJsonError::Unsupported)
            );
        }
        #[test]
        fn rejects_trailing_and_oversized_input() {
            assert_eq!(validate(b"{}{}"), Err(StrictJsonError::Malformed));
            let mut exact = b"[]".to_vec();
            exact.resize(MAX_BYTES, b' ');
            assert_eq!(validate(&exact), Ok(()));
            assert_eq!(
                validate(&vec![b' '; MAX_BYTES + 1]),
                Err(StrictJsonError::Resource)
            );
        }
        #[test]
        fn depth_limit_is_inclusive_and_rejects_one_more_container() {
            let nested = |depth: usize| format!("{}0{}", "[".repeat(depth), "]".repeat(depth));
            assert_eq!(validate(nested(MAX_DEPTH).as_bytes()), Ok(()));
            assert_eq!(
                validate(nested(MAX_DEPTH + 1).as_bytes()),
                Err(StrictJsonError::Resource)
            );
        }
        #[test]
        fn string_scalar_limit_is_inclusive() {
            let at_limit = format!("\"{}\"", "x".repeat(MAX_STRING_CHARS));
            let over_limit = format!("\"{}\"", "x".repeat(MAX_STRING_CHARS + 1));
            assert_eq!(validate(at_limit.as_bytes()), Ok(()));
            assert_eq!(
                validate(over_limit.as_bytes()),
                Err(StrictJsonError::Resource)
            );
        }

        fn null_array(items: usize) -> Vec<u8> {
            let mut json = Vec::with_capacity(items.saturating_mul(5) + 2);
            json.push(b'[');
            for index in 0..items {
                if index != 0 {
                    json.push(b',');
                }
                json.extend_from_slice(b"null");
            }
            json.push(b']');
            json
        }
        fn null_object(members: usize, duplicate_keys: bool) -> Vec<u8> {
            let mut json = Vec::with_capacity(members.saturating_mul(16) + 2);
            json.push(b'{');
            for index in 0..members {
                if index != 0 {
                    json.push(b',');
                }
                let key = if duplicate_keys { 0 } else { index };
                json.extend_from_slice(format!("\"{key}\":null").as_bytes());
            }
            json.push(b'}');
            json
        }
        #[test]
        fn collection_element_limit_is_inclusive() {
            assert_eq!(validate(&null_array(MAX_COLLECTION_ITEMS)), Ok(()));
            assert_eq!(
                validate(&null_array(MAX_COLLECTION_ITEMS + 1)),
                Err(StrictJsonError::Resource)
            );
        }
        #[test]
        fn object_member_limit_is_inclusive_and_counts_duplicates() {
            assert_eq!(validate(&null_object(MAX_OBJECT_MEMBERS, false)), Ok(()));
            assert_eq!(
                validate(&null_object(MAX_OBJECT_MEMBERS + 1, false)),
                Err(StrictJsonError::Resource)
            );
            assert_eq!(
                validate(&null_object(MAX_OBJECT_MEMBERS, true)),
                Err(StrictJsonError::Unsupported)
            );
            assert_eq!(
                validate(&null_object(MAX_OBJECT_MEMBERS + 1, true)),
                Err(StrictJsonError::Resource)
            );
        }
        #[test]
        fn normalized_event_limit_is_inclusive() {
            let wrap = |items: usize| {
                let mut json = Vec::with_capacity(items.saturating_mul(5) + 4);
                json.push(b'[');
                json.extend_from_slice(&null_array(items));
                json.push(b']');
                json
            };
            assert_eq!(validate(&wrap(MAX_EVENTS - 1)), Ok(()));
            assert_eq!(validate(&wrap(MAX_EVENTS)), Err(StrictJsonError::Resource));
        }
    }
}
