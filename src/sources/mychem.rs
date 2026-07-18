use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};
use crate::utils::serde::StringOrVec;

const MYCHEM_BASE: &str = "https://mychem.info/v1";
const MYCHEM_BASE_ENV: &str = "BIOMCP_MYCHEM_BASE";

pub(crate) const MYCHEM_FIELDS_SEARCH: &str = "drugbank.id,drugbank.name,chembl.molecule_chembl_id,chembl.molecule_type,chembl.pref_name,chembl.drug_mechanisms.action_type,chembl.drug_mechanisms.target_name,chembl.drug_mechanisms.mechanism_of_action,chembl.atc_classifications,gtopdb.name,gtopdb.interaction_targets.symbol,unii.unii,unii.display_name,ndc.nonproprietaryname,ndc.pharm_classes,chebi.name,openfda.generic_name,openfda.brand_name";
pub(crate) const MYCHEM_FIELDS_GET: &str = "drugbank.id,drugbank.name,drugbank.synonyms,drugbank.drug_interactions,chembl.molecule_chembl_id,chembl.molecule_type,chembl.pref_name,chembl.drug_mechanisms.action_type,chembl.drug_mechanisms.target_name,chembl.drug_mechanisms.mechanism_of_action,gtopdb.name,gtopdb.interaction_targets.symbol,drugcentral.drug_use.indication.concept_name,drugcentral.approval.agency,drugcentral.approval.date,ndc.nonproprietaryname,ndc.pharm_classes,unii.unii,unii.display_name,chebi.name,openfda.generic_name,openfda.brand_name";

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

    pub(crate) fn escape_query_value(value: &str) -> String {
        crate::utils::query::escape_lucene_value(value)
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::MYCHEM,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::MYCHEM),
        )
        .await?;
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::MYCHEM),
            status,
            content_type.as_ref(),
            &bytes,
            true,
        )
    }

    pub(crate) fn query_with_fields_plan(
        q: &str,
        limit: usize,
        offset: usize,
        fields: &str,
    ) -> Result<RequestPlan, BioMcpError> {
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

        Ok(RequestPlan::get("query")
            .query("q", q)
            .query("size", limit.to_string())
            .query("from", offset.to_string())
            .query("fields", fields))
    }

    pub async fn query_with_fields(
        &self,
        q: &str,
        limit: usize,
        offset: usize,
        fields: &str,
    ) -> Result<MyChemQueryResponse, BioMcpError> {
        let plan = Self::query_with_fields_plan(q, limit, offset, fields)?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.get_json(req).await
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MyChemQueryResponse {
    pub total: usize,
    pub hits: Vec<MyChemHit>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyChemHit {
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
    #[serde(default)]
    pub openfda: Option<MyChemOpenfda>,
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
    #[serde(default)]
    pub atc_classifications: StringOrVec,
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
pub struct MyChemOpenfda {
    #[serde(default)]
    pub generic_name: StringOrVec,
    #[serde(default)]
    pub brand_name: StringOrVec,
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
mod tests;
