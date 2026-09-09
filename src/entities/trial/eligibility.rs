use biodata::ClinicalTrialEligibility;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::value::RawValue;

const INVALID_ELIGIBILITY: &str = "invalid clinical trial eligibility";

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
