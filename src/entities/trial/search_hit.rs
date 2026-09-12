use serde::Serialize;

use crate::error::BioMcpError;
use crate::render::trial_projection as wire;

#[derive(Clone)]
pub struct TrialSearchHit {
    projection: biodata::ClinicalTrialSearchProjection<biodata::Capture>,
    pub matched_intervention_label: Option<String>,
}

impl std::fmt::Debug for TrialSearchHit {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TrialSearchHit")
            .finish_non_exhaustive()
    }
}

impl TrialSearchHit {
    pub(crate) fn from_biodata(
        projection: biodata::ClinicalTrialSearchProjection<biodata::Capture>,
    ) -> Result<Self, BioMcpError> {
        projection
            .value()
            .identities()
            .first()
            .map(|_| ())
            .ok_or(BioMcpError::InternalProcessing)?;
        Ok(Self {
            projection,
            matched_intervention_label: None,
        })
    }

    pub(crate) fn value(&self) -> &biodata::ClinicalTrialSearchSummary {
        self.projection.value()
    }

    pub(crate) fn nct_id(&self) -> &str {
        self.value()
            .identities()
            .first()
            .map(|identity| identity.identifier())
            .unwrap_or_default()
    }

    pub(crate) fn status(&self) -> &str {
        self.value().overall_status().code()
    }

    #[cfg(test)]
    pub(crate) fn test(nct_id: &str, status: &str) -> Self {
        let filters = biodata::ClinicalTrialSearchFilters::new(
            biodata::ClinicalTrialSearchFilterFields {
                condition: Some("test".into()),
                ..Default::default()
            },
            Default::default(),
        )
        .expect("test filters");
        let plan = biodata::ClinicalTrialsGovApiV2SearchPlan::new(&filters, 1, None, true)
            .expect("test plan");
        let input = format!(
            r#"{{"studies":[{{"protocolSection":{{"identificationModule":{{"nctId":"{nct_id}","briefTitle":"Trial {nct_id}"}},"statusModule":{{"overallStatus":"{status}"}}}}}}],"totalCount":1}}"#
        );
        let page = biodata::ClinicalTrialsGovApiV2SearchPage::parse(
            &plan,
            input.as_bytes(),
            &Default::default(),
        )
        .expect("test page");
        let projection = page.results().expect("test results")[0]
            .clone()
            .into_projection();
        Self::from_biodata(projection).expect("test hit")
    }
}

impl Serialize for TrialSearchHit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let shared = self.value();
        let mut output = serde_json::Map::new();
        output.insert("nct_id".into(), self.nct_id().into());
        output.insert("title".into(), shared.brief_title().into());
        output.insert("status".into(), shared.overall_status().code().into());
        let phases: Vec<_> = shared.phases().iter().map(|value| value.code()).collect();
        if !phases.is_empty() {
            output.insert("phase".into(), phases.join("/").into());
        }
        output.insert("phases".into(), phases.into());
        output.insert("conditions".into(), shared.conditions().into());
        if let Some(value) = shared.lead_sponsor_name() {
            output.insert("sponsor".into(), value.into());
        }
        if let Some(value) = &self.matched_intervention_label {
            output.insert("matched_intervention_label".into(), value.clone().into());
        }
        output.insert("capture".into(), wire::capture(self.projection.capture()));
        output.insert(
            "conversion_report".into(),
            wire::report(self.projection.report()),
        );
        serde_json::Value::Object(output).serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::TrialSearchHit;

    #[test]
    fn shared_summary_maps_to_stable_product_keys_and_keeps_projection_metadata() {
        let filters = biodata::ClinicalTrialSearchFilters::new(
            biodata::ClinicalTrialSearchFilterFields {
                condition: Some("fixture".into()),
                ..Default::default()
            },
            Default::default(),
        )
        .unwrap();
        let plan = biodata::ClinicalTrialsGovApiV2SearchPlan::new(&filters, 1, None, true).unwrap();
        let page = biodata::ClinicalTrialsGovApiV2SearchPage::parse(
            &plan,
            br#"{"studies":[{"protocolSection":{"identificationModule":{"nctId":"NCT00000001","briefTitle":" title "},"statusModule":{"overallStatus":" status "},"designModule":{"phases":["PHASE1","PHASE2"]},"conditionsModule":{"conditions":[" A "," A "]},"sponsorCollaboratorsModule":{"leadSponsor":{"name":" sponsor "}}}}],"totalCount":1}"#,
            &Default::default(),
        )
        .unwrap();
        let result =
            TrialSearchHit::from_biodata(page.results().unwrap()[0].clone().into_projection())
                .unwrap();
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["nct_id"], "NCT00000001");
        assert_eq!(json["title"], " title ");
        assert_eq!(json["status"], " status ");
        assert_eq!(json["phase"], "PHASE1/PHASE2");
        assert_eq!(json["phases"], serde_json::json!(["PHASE1", "PHASE2"]));
        assert_eq!(json["conditions"], serde_json::json!([" A ", " A "]));
        assert_eq!(json["sponsor"], " sponsor ");
        assert!(json.get("capture").is_some());
        assert!(json.get("conversion_report").is_some());
        assert!(!format!("{result:?}").contains(" title "));
    }
}
