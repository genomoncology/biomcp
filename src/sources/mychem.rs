use std::borrow::Cow;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};

use crate::error::BioMcpError;

const MYCHEM_BASE: &str = "https://mychem.info/v1";
const MYCHEM_API: &str = "mychem.info";
const MYCHEM_BASE_ENV: &str = "BIOMCP_MYCHEM_BASE";

pub(crate) const MYCHEM_FIELDS_SEARCH: &str = "_id,_score,drugbank.id,drugbank.name,chembl.molecule_chembl_id,chembl.molecule_type,chembl.pref_name,chembl.drug_mechanisms.action_type,chembl.drug_mechanisms.target_name,chembl.drug_mechanisms.mechanism_of_action,gtopdb.name,gtopdb.interaction_targets.symbol,unii.unii,unii.display_name,unii.substance_type,ndc.nonproprietaryname,ndc.pharm_classes,chebi.name";
pub(crate) const MYCHEM_FIELDS_GET: &str = "_id,_score,drugbank.id,drugbank.name,drugbank.synonyms,drugbank.drug_interactions,chembl.molecule_chembl_id,chembl.molecule_type,chembl.pref_name,chembl.drug_mechanisms.action_type,chembl.drug_mechanisms.target_name,chembl.drug_mechanisms.mechanism_of_action,gtopdb.name,gtopdb.interaction_targets.symbol,gtopdb.interaction_targets.name,drugcentral.drug_use.indication.concept_name,drugcentral.approval.agency,drugcentral.approval.date,ndc.nonproprietaryname,ndc.pharm_classes,unii.unii,unii.display_name,unii.substance_type,chebi.name";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

fn de_vec_or_single<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let value = Option::<OneOrMany<T>>::deserialize(deserializer)?;
    Ok(match value {
        Some(OneOrMany::One(v)) => vec![v],
        Some(OneOrMany::Many(v)) => v,
        None => Vec::new(),
    })
}

fn de_json_vec_or_single<'de, D>(deserializer: D) -> Result<Vec<serde_json::Value>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(match value {
        Some(serde_json::Value::Array(v)) => v,
        Some(v) => vec![v],
        None => Vec::new(),
    })
}

pub struct MyChemClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl MyChemClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(MYCHEM_BASE, MYCHEM_BASE_ENV),
        })
    }

    #[cfg(test)]
    fn new_for_test(base: String) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: Cow::Owned(base),
        })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base.as_ref().trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    pub(crate) fn escape_query_value(value: &str) -> String {
        crate::utils::query::escape_lucene_value(value)
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req).send().await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, MYCHEM_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: MYCHEM_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        crate::sources::ensure_json_content_type(MYCHEM_API, content_type.as_ref(), &bytes)?;
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: MYCHEM_API.to_string(),
            source,
        })
    }

    pub async fn query_with_fields(
        &self,
        q: &str,
        limit: usize,
        offset: usize,
        fields: &str,
    ) -> Result<MyChemQueryResponse, BioMcpError> {
        let q = q.trim();
        if q.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Query is required. Example: biomcp search drug -q pembrolizumab".into(),
            ));
        }
        if q.len() > 1024 {
            return Err(BioMcpError::InvalidArgument("Query is too long.".into()));
        }
        if limit == 0 || limit > 50 {
            return Err(BioMcpError::InvalidArgument(
                "--limit must be between 1 and 50".into(),
            ));
        }
        crate::sources::validate_biothings_result_window("MyChem search", limit, offset)?;

        let url = self.endpoint("query");
        let size = limit.to_string();
        let from = offset.to_string();
        self.get_json(self.client.get(&url).query(&[
            ("q", q),
            ("size", size.as_str()),
            ("from", from.as_str()),
            ("fields", fields),
        ]))
        .await
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MyChemQueryResponse {
    #[allow(dead_code)]
    pub total: usize,
    pub hits: Vec<MyChemHit>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyChemHit {
    #[serde(rename = "_id")]
    #[allow(dead_code)]
    pub id: String,
    #[serde(rename = "_score")]
    #[allow(dead_code)]
    pub score: f64,

    #[serde(default)]
    pub drugbank: Option<MyChemDrugBank>,
    #[serde(default)]
    pub chembl: Option<MyChemChembl>,
    #[serde(default)]
    pub drugcentral: Option<MyChemDrugCentral>,
    #[serde(default)]
    pub gtopdb: Option<MyChemGtoPdb>,
    #[serde(default)]
    pub ndc: Option<MyChemNdcField>,
    #[serde(default)]
    pub unii: Option<MyChemUniiField>,
    #[serde(default)]
    pub chebi: Option<MyChemChebiField>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyChemDrugBank {
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(default, deserialize_with = "de_vec_or_single")]
    pub synonyms: Vec<String>,
    #[serde(default, deserialize_with = "de_json_vec_or_single")]
    pub drug_interactions: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyChemChembl {
    pub molecule_chembl_id: Option<String>,
    pub molecule_type: Option<String>,
    pub pref_name: Option<String>,
    #[serde(default, deserialize_with = "de_vec_or_single")]
    pub drug_mechanisms: Vec<MyChemChemblDrugMechanism>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyChemChemblDrugMechanism {
    pub action_type: Option<String>,
    pub target_name: Option<String>,
    pub mechanism_of_action: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyChemDrugCentral {
    pub drug_use: Option<MyChemDrugCentralDrugUse>,
    #[serde(default, deserialize_with = "de_vec_or_single")]
    pub approval: Vec<MyChemDrugCentralApproval>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyChemDrugCentralDrugUse {
    #[serde(default, deserialize_with = "de_vec_or_single")]
    pub indication: Vec<MyChemDrugCentralIndication>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyChemDrugCentralIndication {
    pub concept_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyChemDrugCentralApproval {
    pub agency: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyChemGtoPdb {
    pub name: Option<String>,
    #[serde(default, deserialize_with = "de_vec_or_single")]
    pub interaction_targets: Vec<MyChemGtoPdbTarget>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyChemGtoPdbTarget {
    pub symbol: Option<String>,
    #[allow(dead_code)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MyChemNdcField {
    Many(Vec<MyChemNdc>),
    One(MyChemNdc),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyChemNdc {
    pub nonproprietaryname: Option<String>,
    #[serde(default, deserialize_with = "de_vec_or_single")]
    pub pharm_classes: Vec<MyChemPharmClass>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyChemUnii {
    pub unii: Option<String>,
    pub display_name: Option<String>,
    #[allow(dead_code)]
    pub substance_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MyChemUniiField {
    Many(Vec<MyChemUnii>),
    One(MyChemUnii),
}

impl MyChemUniiField {
    pub fn unii(&self) -> Option<&str> {
        match self {
            Self::Many(v) => v.iter().find_map(|u| u.unii.as_deref()),
            Self::One(v) => v.unii.as_deref(),
        }
    }

    pub fn display_name(&self) -> Option<&str> {
        match self {
            Self::Many(v) => v.iter().find_map(|u| u.display_name.as_deref()),
            Self::One(v) => v.display_name.as_deref(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyChemChebi {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MyChemChebiField {
    Many(Vec<MyChemChebi>),
    One(MyChemChebi),
}

impl MyChemChebiField {
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Many(v) => v.iter().find_map(|c| c.name.as_deref()),
            Self::One(v) => v.name.as_deref(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MyChemPharmClass {
    Str(String),
    Map(serde_json::Map<String, serde_json::Value>),
}

impl MyChemPharmClass {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(value) => Some(value.as_str()),
            Self::Map(map) => {
                for key in ["classname", "class_name", "name", "value", "term", "label"] {
                    if let Some(v) = map.get(key).and_then(|v| v.as_str()) {
                        return Some(v);
                    }
                }
                map.values().find_map(|v| v.as_str())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pharm_class_supports_string_and_map() {
        let input = r#"
        {
          "total": 1,
          "hits": [
            {
              "_id": "X",
              "_score": 1.0,
              "ndc": {
                "pharm_classes": [
                  "Kinase inhibitor [MoA]",
                  { "classname": "Alkylating agent [MoA]" }
                ]
              }
            }
          ]
        }
        "#;
        let parsed: MyChemQueryResponse = serde_json::from_str(input).expect("parse");
        let hit = parsed.hits.first().expect("hit");
        let ndc = hit.ndc.as_ref().expect("ndc");
        let MyChemNdcField::One(ndc) = ndc else {
            panic!("expected one ndc entry");
        };
        let classes = ndc
            .pharm_classes
            .iter()
            .filter_map(MyChemPharmClass::as_str)
            .collect::<Vec<_>>();
        assert_eq!(
            classes,
            vec!["Kinase inhibitor [MoA]", "Alkylating agent [MoA]"]
        );
    }

    #[test]
    fn chebi_name_round_trips() {
        let input = r#"
        {
          "total": 2,
          "hits": [
            { "_id": "CHEBI:1", "_score": 1.0, "chebi": { "name": "Example inhibitor" } },
            { "_id": "CHEBI:2", "_score": 1.0, "chebi": [{ "name": "Example inhibitor 2" }] }
          ]
        }
        "#;
        let parsed: MyChemQueryResponse = serde_json::from_str(input).expect("parse");
        let hit = parsed.hits.first().expect("hit");
        assert_eq!(
            hit.chebi.as_ref().and_then(MyChemChebiField::name),
            Some("Example inhibitor")
        );

        let hit = parsed.hits.get(1).expect("hit");
        assert_eq!(
            hit.chebi.as_ref().and_then(MyChemChebiField::name),
            Some("Example inhibitor 2")
        );
    }

    #[test]
    fn unii_supports_object_and_list() {
        let input = r#"
        {
          "total": 2,
          "hits": [
            { "_id": "X", "_score": 1.0, "unii": { "unii": "ABC", "display_name": "Example" } },
            { "_id": "Y", "_score": 1.0, "unii": [{ "unii": "DEF", "display_name": "Example 2" }] }
          ]
        }
        "#;

        let parsed: MyChemQueryResponse = serde_json::from_str(input).expect("parse");
        let hit = parsed.hits.first().expect("hit");
        let unii = hit.unii.as_ref().expect("unii");
        assert_eq!(unii.unii(), Some("ABC"));
        assert_eq!(unii.display_name(), Some("Example"));

        let hit = parsed.hits.get(1).expect("hit");
        let unii = hit.unii.as_ref().expect("unii");
        assert_eq!(unii.unii(), Some("DEF"));
        assert_eq!(unii.display_name(), Some("Example 2"));
    }

    #[test]
    fn drugcentral_approval_supports_object_and_list() {
        let input = r#"
        {
          "total": 2,
          "hits": [
            {
              "_id": "A",
              "_score": 1.0,
              "drugcentral": {
                "approval": { "agency": "FDA", "date": "2011-08-17" }
              }
            },
            {
              "_id": "B",
              "_score": 1.0,
              "drugcentral": {
                "approval": [
                  { "agency": "FDA", "date": "2014-09-04" },
                  { "agency": "EMA", "date": "2015-07-01" }
                ]
              }
            }
          ]
        }
        "#;

        let parsed: MyChemQueryResponse = serde_json::from_str(input).expect("parse");
        let first = parsed
            .hits
            .first()
            .and_then(|hit| hit.drugcentral.as_ref())
            .map(|dc| dc.approval.len());
        assert_eq!(first, Some(1));

        let second = parsed
            .hits
            .get(1)
            .and_then(|hit| hit.drugcentral.as_ref())
            .map(|dc| dc.approval.len());
        assert_eq!(second, Some(2));
    }

    #[test]
    fn drugbank_interactions_support_object_and_list() {
        let parsed: MyChemQueryResponse = serde_json::from_value(serde_json::json!({
            "total": 2,
            "hits": [
                {
                    "_id": "A",
                    "_score": 1.0,
                    "drugbank": {
                        "id": "DB1",
                        "drug_interactions": {
                            "name": "Aspirin",
                            "description": "May increase bleeding risk."
                        }
                    }
                },
                {
                    "_id": "B",
                    "_score": 1.0,
                    "drugbank": {
                        "id": "DB2",
                        "drug_interactions": [
                            {"name": "Clopidogrel", "description": "Monitor bleeding."}
                        ]
                    }
                }
            ]
        }))
        .expect("parse");

        let first = parsed
            .hits
            .first()
            .and_then(|h| h.drugbank.as_ref())
            .map(|d| d.drug_interactions.len());
        let second = parsed
            .hits
            .get(1)
            .and_then(|h| h.drugbank.as_ref())
            .map(|d| d.drug_interactions.len());
        assert_eq!(first, Some(1));
        assert_eq!(second, Some(1));
    }

    #[tokio::test]
    async fn query_with_fields_rejects_offset_at_biothings_window() {
        let client = MyChemClient::new_for_test("http://127.0.0.1".into()).unwrap();
        let err = client
            .query_with_fields("imatinib", 5, 10_000, MYCHEM_FIELDS_SEARCH)
            .await
            .unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(err.to_string().contains("--offset must be less than 10000"));
    }

    #[tokio::test]
    async fn query_with_fields_rejects_offset_limit_window_overflow() {
        let client = MyChemClient::new_for_test("http://127.0.0.1".into()).unwrap();
        let err = client
            .query_with_fields("imatinib", 30, 9_980, MYCHEM_FIELDS_SEARCH)
            .await
            .unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(
            err.to_string()
                .contains("--offset + --limit must be <= 10000")
        );
    }
}
