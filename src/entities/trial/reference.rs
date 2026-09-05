//! Product serialization around the shared clinical-trial reference value.

use biodata::{ClinicalTrialReference, ExtensibleCode};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::BioMcpError;

#[derive(Debug, Clone)]
pub struct TrialReference {
    shared: ClinicalTrialReference,
}

impl TrialReference {
    pub fn new(
        pmid: Option<String>,
        citation: String,
        reference_type: Option<String>,
    ) -> Result<Self, BioMcpError> {
        let citation = normalize(Some(citation)).ok_or_else(invalid_reference)?;
        let source_type = normalize(reference_type)
            .map(|code| {
                ExtensibleCode::new(
                    "clinicaltrials.gov",
                    code,
                    None::<String>,
                    None::<String>,
                    None::<String>,
                )
            })
            .transpose()
            .map_err(|_| invalid_reference())?;
        let shared = ClinicalTrialReference::new(normalize(pmid), Some(citation), source_type)
            .map_err(|_| invalid_reference())?;
        Ok(Self { shared })
    }

    pub fn shared(&self) -> &ClinicalTrialReference {
        &self.shared
    }
}

fn normalize(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

fn invalid_reference() -> BioMcpError {
    BioMcpError::Api {
        api: "ClinicalTrials.gov".into(),
        message: "Invalid trial reference data".into(),
    }
}

impl Serialize for TrialReference {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Wire<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            pmid: Option<&'a str>,
            citation: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            reference_type: Option<&'a str>,
        }
        let shared = self.shared();
        Wire {
            pmid: shared.pmid(),
            citation: shared
                .citation()
                .ok_or_else(|| serde::ser::Error::custom("Invalid trial reference data"))?,
            reference_type: shared.source_type().map(ExtensibleCode::code),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TrialReference {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Wire {
            pmid: Option<String>,
            citation: String,
            reference_type: Option<String>,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.pmid, wire.citation, wire.reference_type)
            .map_err(|_| serde::de::Error::custom("Invalid trial reference data"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_owns_a_shared_value_and_preserves_wire_information() {
        let reference = TrialReference::new(
            Some(" 123 ".into()),
            " Étude α. ".into(),
            Some(" DERIVED ".into()),
        )
        .expect("usable citation");
        let shared: &ClinicalTrialReference = reference.shared();
        assert_eq!(shared.pmid(), Some("123"));
        assert_eq!(shared.citation(), Some("Étude α."));
        let code = shared.source_type().expect("source type");
        assert_eq!(code.authority(), "clinicaltrials.gov");
        assert_eq!(code.code(), "DERIVED");
        let wire = serde_json::to_value(&reference).expect("serialize");
        assert_eq!(
            wire,
            serde_json::json!({"pmid":"123", "citation":"Étude α.", "reference_type":"DERIVED"})
        );
        let decoded: TrialReference = serde_json::from_value(wire).expect("decode");
        assert_eq!(decoded.shared(), shared);
    }

    #[test]
    fn decoding_normalizes_optional_fields_and_rejects_unusable_citations() {
        for value in [serde_json::Value::Null, serde_json::json!(" \t ")] {
            let wire =
                serde_json::json!({"citation":" Citation ", "pmid":value, "reference_type":value});
            let reference: TrialReference = serde_json::from_value(wire).expect("optional absence");
            assert_eq!(
                serde_json::to_value(reference).expect("serialize"),
                serde_json::json!({"citation":"Citation"})
            );
        }
        for wire in [
            serde_json::json!({}),
            serde_json::json!({"citation":null}),
            serde_json::json!({"citation":""}),
            serde_json::json!({"citation":" \t "}),
        ] {
            assert!(serde_json::from_value::<TrialReference>(wire).is_err());
        }
        let error = TrialReference::new(Some("private input".into()), " ".into(), None)
            .expect_err("citation is required");
        assert!(matches!(error, BioMcpError::Api { .. }));
        assert!(!error.to_string().contains("private input"));
    }
}
