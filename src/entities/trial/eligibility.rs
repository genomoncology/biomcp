use biodata::{
    Bound, ClinicalTrialAgeBound, ClinicalTrialAgeBoundForm, ClinicalTrialAgeRange,
    ClinicalTrialEligibility, ClinicalTrialEligibilityClassification,
    ClinicalTrialEligibilityCriterion, ClinicalTrialEligibilityCriterionId, DurationUnit,
    ExtensibleCode, ParseOutcome, TemporalParser,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Value, json};

const NO_LIMIT_RULE: &str = "nci-cts-v2-999-years-no-upper-bound";

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum UnitWire {
    Years,
    Months,
    Weeks,
    Days,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum BoundWire {
    Minimum,
    Maximum,
}

impl From<UnitWire> for DurationUnit {
    fn from(value: UnitWire) -> Self {
        match value {
            UnitWire::Years => Self::Years,
            UnitWire::Months => Self::Months,
            UnitWire::Weeks => Self::Weeks,
            UnitWire::Days => Self::Days,
        }
    }
}

impl From<BoundWire> for Bound {
    fn from(value: BoundWire) -> Self {
        match value {
            BoundWire::Minimum => Self::Minimum,
            BoundWire::Maximum => Self::Maximum,
        }
    }
}

fn unit(value: DurationUnit) -> &'static str {
    match value {
        DurationUnit::Years => "years",
        DurationUnit::Months => "months",
        DurationUnit::Weeks => "weeks",
        DurationUnit::Days => "days",
    }
}

fn bound(value: Bound) -> &'static str {
    match value {
        Bound::Minimum => "minimum",
        Bound::Maximum => "maximum",
    }
}

fn code_value(value: &ExtensibleCode) -> Value {
    json!({
        "authority": value.authority(),
        "code": value.code(),
        "display": value.display(),
        "vocabulary_version": value.vocabulary_version(),
        "recognized_meaning": value.recognized_meaning()
    })
}

fn age_bound_value(value: &ClinicalTrialAgeBound) -> Value {
    let source = value.source();
    let mut out = json!({
        "kind": "limited",
        "source": source.source(),
        "source_quantity": source.source_quantity(),
        "source_unit": unit(source.source_unit()),
        "bound": bound(source.bound())
    });
    if value.form() == ClinicalTrialAgeBoundForm::SourceStatedNoLimit {
        out["kind"] = json!("source_stated_no_limit");
        out["rule"] = json!({"name": NO_LIMIT_RULE, "version": "1"});
    }
    out
}

fn classification_value(value: &ClinicalTrialEligibilityClassification) -> Value {
    match value {
        ClinicalTrialEligibilityClassification::Inclusion => json!({"kind": "inclusion"}),
        ClinicalTrialEligibilityClassification::Exclusion => json!({"kind": "exclusion"}),
        ClinicalTrialEligibilityClassification::Other(source) => {
            json!({"kind": "other", "source": code_value(source)})
        }
    }
}

fn eligibility_value(value: &ClinicalTrialEligibility) -> Value {
    json!({
        "registry_text": value.registry_text(),
        "age_range": value.age_range().map(|range| json!({
            "minimum": range.minimum().map(age_bound_value),
            "maximum": range.maximum().map(age_bound_value)
        })),
        "sexes": value.sexes().map(|values| values.iter().map(code_value).collect::<Vec<_>>()),
        "includes_healthy_subjects": value.includes_healthy_subjects(),
        "criteria": value.criteria().map(|values| values.iter().map(|criterion| json!({
            "id": criterion.id().get(),
            "description": criterion.description(),
            "classification": classification_value(criterion.classification())
        })).collect::<Vec<_>>())
    })
}

pub(super) fn serialize<S: Serializer>(
    value: &Option<ClinicalTrialEligibility>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    value.as_ref().map(eligibility_value).serialize(serializer)
}

fn required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

#[cfg(test)]
mod tests {
    use biodata::ClinicalTrialEligibility;
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EligibilityOwned {
    #[serde(deserialize_with = "required_nullable")]
    registry_text: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    age_range: Option<AgeRangeOwned>,
    #[serde(deserialize_with = "required_nullable")]
    sexes: Option<Vec<CodeOwned>>,
    #[serde(deserialize_with = "required_nullable")]
    includes_healthy_subjects: Option<bool>,
    #[serde(deserialize_with = "required_nullable")]
    criteria: Option<Vec<CriterionOwned>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AgeRangeOwned {
    #[serde(deserialize_with = "required_nullable")]
    minimum: Option<AgeBoundOwned>,
    #[serde(deserialize_with = "required_nullable")]
    maximum: Option<AgeBoundOwned>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum AgeBoundOwned {
    Limited {
        source: String,
        source_quantity: String,
        source_unit: UnitWire,
        bound: BoundWire,
    },
    SourceStatedNoLimit {
        source: String,
        source_quantity: String,
        source_unit: UnitWire,
        bound: BoundWire,
        rule: RuleOwned,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleOwned {
    name: String,
    version: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CodeOwned {
    authority: String,
    code: String,
    #[serde(deserialize_with = "required_nullable")]
    display: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    vocabulary_version: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    recognized_meaning: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CriterionOwned {
    id: u64,
    description: String,
    classification: ClassificationOwned,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ClassificationOwned {
    Inclusion,
    Exclusion,
    Other { source: CodeOwned },
}

fn code(value: CodeOwned) -> Result<ExtensibleCode, ()> {
    ExtensibleCode::new(
        value.authority,
        value.code,
        value.display,
        value.vocabulary_version,
        value.recognized_meaning,
    )
    .map_err(|_| ())
}

fn parsed_bound(
    source: String,
    source_quantity: String,
    source_unit: UnitWire,
    bound: BoundWire,
) -> Result<biodata::Duration, ()> {
    let expected_unit: DurationUnit = source_unit.into();
    let expected_bound: Bound = bound.into();
    let ParseOutcome::Parsed(parsed) =
        TemporalParser::default().parse_duration(&source, expected_bound)
    else {
        return Err(());
    };
    if parsed.source_quantity() != source_quantity || parsed.source_unit() != expected_unit {
        return Err(());
    }
    Ok(parsed)
}

fn age_bound(value: AgeBoundOwned) -> Result<ClinicalTrialAgeBound, ()> {
    match value {
        AgeBoundOwned::Limited {
            source,
            source_quantity,
            source_unit,
            bound,
        } => ClinicalTrialAgeBound::limited(parsed_bound(
            source,
            source_quantity,
            source_unit,
            bound,
        )?)
        .map_err(|_| ()),
        AgeBoundOwned::SourceStatedNoLimit {
            source,
            source_quantity,
            source_unit,
            bound,
            rule,
        } => {
            if rule.name != NO_LIMIT_RULE || rule.version != "1" {
                return Err(());
            }
            ClinicalTrialAgeBound::source_stated_no_limit(parsed_bound(
                source,
                source_quantity,
                source_unit,
                bound,
            )?)
            .map_err(|_| ())
        }
    }
}

fn age_range(value: AgeRangeOwned) -> Result<ClinicalTrialAgeRange, ()> {
    ClinicalTrialAgeRange::new(
        value.minimum.map(age_bound).transpose()?,
        value.maximum.map(age_bound).transpose()?,
    )
    .map_err(|_| ())
}

fn criterion(value: CriterionOwned) -> Result<ClinicalTrialEligibilityCriterion, ()> {
    let classification = match value.classification {
        ClassificationOwned::Inclusion => ClinicalTrialEligibilityClassification::Inclusion,
        ClassificationOwned::Exclusion => ClinicalTrialEligibilityClassification::Exclusion,
        ClassificationOwned::Other { source } => {
            ClinicalTrialEligibilityClassification::Other(code(source)?)
        }
    };
    ClinicalTrialEligibilityCriterion::new(
        ClinicalTrialEligibilityCriterionId::new(value.id).map_err(|_| ())?,
        value.description,
        classification,
    )
    .map_err(|_| ())
}

fn eligibility(value: EligibilityOwned) -> Result<ClinicalTrialEligibility, ()> {
    ClinicalTrialEligibility::new(
        value.registry_text,
        value.age_range.map(age_range).transpose()?,
        value
            .sexes
            .map(|values| values.into_iter().map(code).collect())
            .transpose()?,
        value.includes_healthy_subjects,
        value
            .criteria
            .map(|values| values.into_iter().map(criterion).collect())
            .transpose()?,
    )
    .map_err(|_| ())
}

pub(super) fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<ClinicalTrialEligibility>, D::Error> {
    Option::<EligibilityOwned>::deserialize(deserializer)?
        .map(eligibility)
        .transpose()
        .map_err(|()| serde::de::Error::custom("invalid clinical trial eligibility"))
}
