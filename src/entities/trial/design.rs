use biodata::{
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
    /// A typed arm relationship failed BioData validation.
    InvalidRelationship(ClinicalTrialArmRelationshipError),
}

impl TrialDesignError {
    /// Returns the typed BioData relationship failure when one caused this error.
    pub const fn relationship_error(&self) -> Option<&ClinicalTrialArmRelationshipError> {
        match self {
            Self::SectionPresenceMismatch => None,
            Self::InvalidRelationship(source) => Some(source),
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

#[cfg(test)]
mod tests {
    use biodata::{
        ClinicalTrialArm, ClinicalTrialArmId, ClinicalTrialArmInterventionAssignment,
        ClinicalTrialArmRelationshipError, ClinicalTrialIntervention, ClinicalTrialInterventionId,
    };

    use super::{TrialDesign, TrialDesignError};
    use crate::error::BioMcpError;

    fn arm(id: u64) -> ClinicalTrialArm {
        ClinicalTrialArm::new(
            ClinicalTrialArmId::new(id).expect("arm identity"),
            format!("arm {id}"),
            None,
            None,
        )
        .expect("arm")
    }

    fn intervention(id: u64) -> ClinicalTrialIntervention {
        ClinicalTrialIntervention::new(
            ClinicalTrialInterventionId::new(id).expect("intervention identity"),
            format!("intervention {id}"),
            None,
            None,
            None,
        )
        .expect("intervention")
    }

    fn assignment(arm_id: u64, intervention_id: u64) -> ClinicalTrialArmInterventionAssignment {
        ClinicalTrialArmInterventionAssignment::new(
            ClinicalTrialArmId::new(arm_id).expect("arm identity"),
            ClinicalTrialInterventionId::new(intervention_id).expect("intervention identity"),
        )
    }

    fn relationship_error(
        interventions: Vec<ClinicalTrialIntervention>,
        arms: Vec<ClinicalTrialArm>,
        assignments: Vec<ClinicalTrialArmInterventionAssignment>,
    ) -> ClinicalTrialArmRelationshipError {
        let error = TrialDesign::new(interventions, Some(arms), Some(assignments))
            .expect_err("invalid relationship");
        error
            .relationship_error()
            .copied()
            .expect("typed relationship error")
    }

    #[test]
    fn trial_design_distinguishes_section_presence_from_relationship_failures() {
        assert_eq!(
            TrialDesign::new(Vec::new(), Some(Vec::new()), None).unwrap_err(),
            TrialDesignError::SectionPresenceMismatch
        );
        assert!(
            TrialDesignError::SectionPresenceMismatch
                .relationship_error()
                .is_none()
        );
    }

    #[test]
    fn trial_design_preserves_all_relationship_failure_variants() {
        let arm_id = ClinicalTrialArmId::new(1).unwrap();
        let intervention_id = ClinicalTrialInterventionId::new(1).unwrap();
        let missing_arm_id = ClinicalTrialArmId::new(2).unwrap();
        let missing_intervention_id = ClinicalTrialInterventionId::new(2).unwrap();

        assert_eq!(
            relationship_error(vec![intervention(1)], vec![arm(1), arm(1)], Vec::new()),
            ClinicalTrialArmRelationshipError::DuplicateArmId { arm_id }
        );
        assert_eq!(
            relationship_error(
                vec![intervention(1), intervention(1)],
                vec![arm(1)],
                Vec::new()
            ),
            ClinicalTrialArmRelationshipError::DuplicateInterventionId { intervention_id }
        );
        assert_eq!(
            relationship_error(vec![intervention(1)], vec![arm(1)], vec![assignment(2, 1)]),
            ClinicalTrialArmRelationshipError::MissingArmEndpoint {
                arm_id: missing_arm_id
            }
        );
        assert_eq!(
            relationship_error(vec![intervention(1)], vec![arm(1)], vec![assignment(1, 2)]),
            ClinicalTrialArmRelationshipError::MissingInterventionEndpoint {
                intervention_id: missing_intervention_id
            }
        );
        assert_eq!(
            relationship_error(
                vec![intervention(1)],
                vec![arm(1)],
                vec![assignment(1, 1), assignment(1, 1)]
            ),
            ClinicalTrialArmRelationshipError::DuplicateAssignment {
                arm_id,
                intervention_id
            }
        );
    }

    #[test]
    fn missing_intervention_is_checked_against_the_supplied_collection() {
        assert_eq!(
            relationship_error(vec![intervention(2)], vec![arm(1)], vec![assignment(1, 1)]),
            ClinicalTrialArmRelationshipError::MissingInterventionEndpoint {
                intervention_id: ClinicalTrialInterventionId::new(1).unwrap()
            }
        );
    }

    #[test]
    fn trial_design_error_retains_sources_and_safe_public_projections() {
        let relationship = ClinicalTrialArmRelationshipError::MissingInterventionEndpoint {
            intervention_id: ClinicalTrialInterventionId::new(42).unwrap(),
        };
        let design = TrialDesignError::InvalidRelationship(relationship);
        let error = BioMcpError::TrialDesign(design);

        assert_eq!(design.relationship_error(), Some(&relationship));
        assert_eq!(error.code(), "internal_processing");
        assert_eq!(error.exit_code(), 1);
        assert_eq!(error.to_string(), "Internal processing failed.");
        assert_eq!(
            error.public_projection().message,
            "Internal processing failed."
        );
        assert_eq!(error.public_projection().source, None);
        assert_eq!(error.public_projection().recovery, None);
        let design_source = std::error::Error::source(&error)
            .and_then(|source| source.downcast_ref::<TrialDesignError>())
            .expect("typed design source");
        assert_eq!(
            std::error::Error::source(design_source)
                .and_then(|source| source.downcast_ref::<ClinicalTrialArmRelationshipError>()),
            Some(&relationship)
        );

        let json = crate::render::json::to_error_json(&error).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["error"]["code"], "internal_processing");
        assert_eq!(value["error"]["message"], "Internal processing failed.");
        assert!(value["error"].get("source").is_none());
        assert!(value["error"].get("recovery").is_none());
        assert!(!json.contains("MissingInterventionEndpoint"));
        assert!(!json.contains("42"));
    }

    #[test]
    fn trial_design_round_trips_shared_relationships() {
        for design in [
            TrialDesign::default(),
            TrialDesign::from_names_and_arm(
                &["drug", "device"],
                "arm",
                Some("future"),
                Some("description"),
            ),
        ] {
            let encoded = serde_json::to_value(&design).expect("serialize design");
            let decoded: TrialDesign =
                serde_json::from_value(encoded.clone()).expect("deserialize design");
            assert_eq!(serde_json::to_value(decoded).unwrap(), encoded);
        }
    }

    #[test]
    fn flattened_design_keeps_assignment_member() {
        #[derive(serde::Serialize)]
        struct Flat<'a> {
            name: &'a str,
            #[serde(flatten)]
            design: &'a TrialDesign,
        }
        let design = TrialDesign::from_names_and_arm(&["drug"], "arm", None, None);
        let value = serde_json::to_value(Flat {
            name: "trial",
            design: &design,
        })
        .unwrap();
        assert_eq!(
            value["arm_intervention_assignments"]
                .as_array()
                .map(Vec::len),
            Some(1)
        );
    }

    #[test]
    fn type_wire_rejects_missing_unknown_and_empty_members() {
        let complete = serde_json::json!({"interventions": [{"id": 1, "name": "study drug", "type": {"authority": "future.registry", "code": "FUTURE", "display": null, "vocabulary_version": null, "recognized_meaning": null}, "description": null, "other_names": []}]});
        let decoded: TrialDesign = serde_json::from_value(complete.clone()).expect("complete code");
        assert_eq!(
            decoded.interventions()[0].source_type().unwrap().code(),
            "FUTURE"
        );
        assert_eq!(serde_json::to_value(decoded).unwrap(), complete);
        for invalid in [
            serde_json::json!({"interventions": [{"id": 1, "name": "x", "type": {"authority": "", "code": "X", "display": null, "vocabulary_version": null, "recognized_meaning": null}, "description": null, "other_names": []}]}),
            serde_json::json!({"interventions": [{"id": 1, "name": "x", "type": {"authority": "a", "code": "X", "display": null, "vocabulary_version": null}, "description": null, "other_names": []}]}),
            serde_json::json!({"interventions": [{"id": 1, "name": "x", "type": {"authority": "a", "code": "X", "display": null, "vocabulary_version": null, "recognized_meaning": null, "extra": null}, "description": null, "other_names": []}]}),
            serde_json::json!({"interventions": [{"id": 1, "name": "x", "type": {"authority": "a", "code": "", "display": null, "vocabulary_version": null, "recognized_meaning": null}, "description": null, "other_names": []}]}),
            serde_json::json!({"interventions": [{"id": 1, "name": "x", "type": {"authority": "a", "code": "X", "display": "", "vocabulary_version": null, "recognized_meaning": null}, "description": null, "other_names": []}]}),
            serde_json::json!({"interventions": [{"id": 1, "name": "x", "type": {"authority": "a", "code": "X", "display": null, "vocabulary_version": "", "recognized_meaning": null}, "description": null, "other_names": []}]}),
            serde_json::json!({"interventions": [{"id": 1, "name": "x", "type": {"authority": "a", "code": "X", "display": null, "vocabulary_version": null, "recognized_meaning": ""}, "description": null, "other_names": []}]}),
            serde_json::json!({"interventions": [], "unexpected": true}),
        ] {
            assert!(serde_json::from_value::<TrialDesign>(invalid).is_err());
        }
    }

    #[test]
    fn deserialization_rejects_duplicate_and_dangling_relationships() {
        let intervention = serde_json::json!({
            "id": 1, "name": "drug", "type": null, "description": null, "other_names": []
        });
        let arm = serde_json::json!({
            "id": 1, "name": "arm", "type": null, "description": null
        });
        for invalid in [
            serde_json::json!({
                "interventions": [intervention.clone(), intervention.clone()],
                "arms": [arm.clone()], "arm_intervention_assignments": []
            }),
            serde_json::json!({
                "interventions": [intervention.clone()], "arms": [arm.clone()],
                "arm_intervention_assignments": [{"arm_id": 1, "intervention_id": 2}]
            }),
            serde_json::json!({
                "interventions": [intervention], "arms": [arm],
                "arm_intervention_assignments": [
                    {"arm_id": 1, "intervention_id": 1},
                    {"arm_id": 1, "intervention_id": 1}
                ]
            }),
            serde_json::json!({
                "interventions": [],
                "arms": [
                    {"id": 1, "name": "first", "type": null, "description": null},
                    {"id": 1, "name": "second", "type": null, "description": null}
                ],
                "arm_intervention_assignments": []
            }),
            serde_json::json!({
                "interventions": [{"id": 1, "name": "drug", "type": null, "description": null, "other_names": []}],
                "arms": [{"id": 1, "name": "arm", "type": null, "description": null}],
                "arm_intervention_assignments": [{"arm_id": 2, "intervention_id": 1}]
            }),
            serde_json::json!({"interventions": [], "arms": []}),
            serde_json::json!({"interventions": [], "arm_intervention_assignments": []}),
        ] {
            assert!(serde_json::from_value::<TrialDesign>(invalid).is_err());
        }
    }
}
