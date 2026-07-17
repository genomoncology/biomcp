use std::borrow::Cow;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const MYDISEASE_BASE: &str = "https://mydisease.info/v1";
const MYDISEASE_API: &str = "mydisease.info";
const MYDISEASE_BASE_ENV: &str = "BIOMCP_MYDISEASE_BASE";

pub(crate) const MYDISEASE_SEARCH_FIELDS: &str = "_id,mondo.name,mondo.synonym,disease_ontology.name,disease_ontology.synonyms,hpo.inheritance.hpo_id,hpo.inheritance.hpo_name,hpo.phenotype_related_to_disease.hpo_id,hpo.clinical_course.hpo_name";
pub(crate) const MYDISEASE_GET_FIELDS: &str = "_id,mondo.name,mondo.definition,mondo.parents,mondo.synonym,mondo.xrefs,disease_ontology.name,disease_ontology.doid,disease_ontology.def,disease_ontology.parents,disease_ontology.synonyms,disease_ontology.xrefs,umls.mesh,umls.nci,umls.snomed,umls.icd10am,disgenet.genes_related_to_disease,hpo.phenotype_related_to_disease.hpo_id,hpo.phenotype_related_to_disease.evidence,hpo.phenotype_related_to_disease.hp_freq,hpo.inheritance.hpo_id";

// dead-code reason: mydisease::MyDiseaseQueryRequestPlan preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
pub struct MyDiseaseQueryRequestPlan {
    pub method: &'static str,
    pub path: &'static str,
    pub query_params: Vec<(&'static str, String)>,
    pub cache_mode: &'static str,
    pub status_expectation: &'static str,
}

// dead-code reason: mydisease::MyDiseaseXrefLookupRequestPlan preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
pub struct MyDiseaseXrefLookupRequestPlan {
    pub method: &'static str,
    pub path: &'static str,
    pub query_params: Vec<(&'static str, String)>,
    pub cache_mode: &'static str,
    pub status_expectation: &'static str,
}

// dead-code reason: mydisease::MyDiseaseGetRequestPlan preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
pub struct MyDiseaseGetRequestPlan {
    pub method: &'static str,
    pub path: String,
    pub query_params: Vec<(&'static str, String)>,
    pub cache_mode: &'static str,
    pub status_expectation: &'static str,
}

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

#[derive(Clone)]
pub struct MyDiseaseClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl MyDiseaseClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(MYDISEASE_BASE, MYDISEASE_BASE_ENV),
        })
    }

    #[cfg(test)]
    pub(crate) fn new_for_test(base: String) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::test_client()?,
            base: Cow::Owned(base),
        })
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req).send().await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, MYDISEASE_API).await?;
        crate::sources::decode_json(MYDISEASE_API, status, content_type.as_ref(), &bytes, false)
    }

    // dead-code reason: mydisease::legacy_plan preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    fn legacy_plan(
        plan: &RequestPlan,
        status_expectation: &'static str,
    ) -> MyDiseaseQueryRequestPlan {
        MyDiseaseQueryRequestPlan {
            method: "GET",
            path: "/query",
            query_params: plan
                .query
                .iter()
                .map(|(key, value)| {
                    let key = match key.as_str() {
                        "q" => "q",
                        "size" => "size",
                        "from" => "from",
                        "fields" => "fields",
                        _ => "",
                    };
                    (key, value.clone())
                })
                .collect(),
            cache_mode: "default",
            status_expectation,
        }
    }

    // dead-code reason: mydisease::legacy_xref_plan preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    fn legacy_xref_plan(
        plan: &RequestPlan,
        status_expectation: &'static str,
    ) -> MyDiseaseXrefLookupRequestPlan {
        MyDiseaseXrefLookupRequestPlan {
            method: "GET",
            path: "/query",
            query_params: plan
                .query
                .iter()
                .map(|(key, value)| {
                    let key = match key.as_str() {
                        "q" => "q",
                        "size" => "size",
                        "from" => "from",
                        "fields" => "fields",
                        _ => "",
                    };
                    (key, value.clone())
                })
                .collect(),
            cache_mode: "default",
            status_expectation,
        }
    }

    // dead-code reason: mydisease::legacy_get_plan preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    fn legacy_get_plan(plan: &RequestPlan) -> MyDiseaseGetRequestPlan {
        MyDiseaseGetRequestPlan {
            method: "GET",
            path: format!("/{}", plan.path.trim_start_matches('/')),
            query_params: plan
                .query
                .iter()
                .map(|(key, value)| {
                    let key = match key.as_str() {
                        "fields" => "fields",
                        _ => "",
                    };
                    (key, value.clone())
                })
                .collect(),
            cache_mode: "default",
            status_expectation: "404 => NotFound; other non-2xx => Api",
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn query_plan(
        q: &str,
        size: usize,
        offset: usize,
        source: Option<&str>,
        inheritance: Option<&str>,
        phenotype: Option<&str>,
        onset: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let q = q.trim();
        if q.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Query is required. Example: biomcp search disease -q melanoma".into(),
            ));
        }
        if q.len() > 512 {
            return Err(BioMcpError::InvalidArgument("Query is too long.".into()));
        }
        crate::sources::validate_biothings_result_window("MyDisease search", size, offset)?;

        let size = size.to_string();
        let from = offset.to_string();
        let escaped = crate::utils::query::escape_lucene_value(q);
        let mut scoped_query = if q.contains(':') && !q.chars().any(|c| c.is_whitespace()) {
            format!("(_id:\"{escaped}\" OR disease_ontology.doid:\"{escaped}\")")
        } else {
            // Keep the legacy name search semantics (tokenized by backend) to avoid
            // over-constraining common disease names like "lung cancer".
            format!(
                "(disease_ontology.name:{escaped} OR disease_ontology.synonyms:{escaped} OR mondo.name:{escaped} OR mondo.synonym:{escaped})"
            )
        };
        if let Some(source) = source.map(str::trim).filter(|v| !v.is_empty()) {
            let source_clause = match source.to_ascii_lowercase().as_str() {
                "mondo" => "(mondo.parents:* OR mondo.xrefs:*)",
                "doid" => "(disease_ontology.doid:* OR mondo.xrefs.doid:*)",
                "mesh" => "(disease_ontology.xrefs.mesh:* OR mondo.xrefs.mesh:* OR umls.mesh:*)",
                other => {
                    return Err(BioMcpError::InvalidArgument(format!(
                        "Unknown --source '{other}'. Expected one of: mondo, doid, mesh"
                    )));
                }
            };
            scoped_query = format!("{scoped_query} AND {source_clause}");
        }
        if let Some(inheritance) = inheritance.map(str::trim).filter(|v| !v.is_empty()) {
            let escaped = crate::utils::query::escape_lucene_value(inheritance);
            scoped_query = format!(
                "{scoped_query} AND (hpo.inheritance.hpo_name:*{escaped}* OR hpo.inheritance.hpo_id:*{escaped}*)"
            );
        }
        if let Some(phenotype) = phenotype.map(str::trim).filter(|v| !v.is_empty()) {
            let escaped = crate::utils::query::escape_lucene_value(phenotype);
            scoped_query =
                format!("{scoped_query} AND hpo.phenotype_related_to_disease.hpo_id:*{escaped}*");
        }
        if let Some(onset) = onset.map(str::trim).filter(|v| !v.is_empty()) {
            let escaped = crate::utils::query::escape_lucene_value(onset);
            scoped_query = format!("{scoped_query} AND hpo.clinical_course.hpo_name:*{escaped}*");
        }
        Ok(RequestPlan::get("query")
            .query("q", scoped_query)
            .query("size", size)
            .query("from", from)
            .query("fields", MYDISEASE_SEARCH_FIELDS))
    }

    #[allow(clippy::too_many_arguments)]
    // dead-code reason: mydisease::query_request_plan preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub fn query_request_plan(
        &self,
        q: &str,
        size: usize,
        offset: usize,
        source: Option<&str>,
        inheritance: Option<&str>,
        phenotype: Option<&str>,
        onset: Option<&str>,
    ) -> Result<MyDiseaseQueryRequestPlan, BioMcpError> {
        let plan = Self::query_plan(q, size, offset, source, inheritance, phenotype, onset)?;
        Ok(Self::legacy_plan(&plan, "non-2xx => Api"))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn query(
        &self,
        q: &str,
        size: usize,
        offset: usize,
        source: Option<&str>,
        inheritance: Option<&str>,
        phenotype: Option<&str>,
        onset: Option<&str>,
    ) -> Result<MyDiseaseQueryResponse, BioMcpError> {
        let plan = Self::query_plan(q, size, offset, source, inheritance, phenotype, onset)?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.get_json(req).await
    }

    pub(crate) fn lookup_disease_by_xref_plan(
        kind: &str,
        value: &str,
        size: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        let value = value.trim();
        if value.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Disease crosswalk ID is required.".into(),
            ));
        }
        crate::sources::validate_biothings_result_window("MyDisease search", size, 0)?;

        let escaped = crate::utils::query::escape_lucene_value(value);
        let query = match kind.trim().to_ascii_lowercase().as_str() {
            "mesh" => format!(
                "(mondo.xrefs.mesh:\"{escaped}\" OR disease_ontology.xrefs.mesh:\"{escaped}\" OR umls.mesh:\"{escaped}\")"
            ),
            "omim" => format!(
                "(mondo.xrefs.omim:\"{escaped}\" OR disease_ontology.xrefs.omim:\"{escaped}\")"
            ),
            "icd10cm" => {
                let prefixed = format!("ICD10:{escaped}");
                format!(
                    "(mondo.xrefs.icd10:\"{escaped}\" OR mondo.xrefs.icd10:\"{prefixed}\" OR disease_ontology.xrefs.icd10:\"{escaped}\" OR disease_ontology.xrefs.icd10:\"{prefixed}\" OR umls.icd10am:\"{escaped}\" OR umls.icd10am:\"{prefixed}\")"
                )
            }
            other => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown disease xref kind '{other}'. Expected one of: mesh, omim, icd10cm"
                )));
            }
        };

        Ok(RequestPlan::get("query")
            .query("q", query)
            .query("size", size.to_string())
            .query("from", "0")
            .query("fields", MYDISEASE_SEARCH_FIELDS))
    }

    // dead-code reason: mydisease::lookup_disease_by_xref_request_plan preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub fn lookup_disease_by_xref_request_plan(
        &self,
        kind: &str,
        value: &str,
        size: usize,
    ) -> Result<MyDiseaseXrefLookupRequestPlan, BioMcpError> {
        let plan = Self::lookup_disease_by_xref_plan(kind, value, size)?;
        Ok(Self::legacy_xref_plan(&plan, "non-2xx => Api"))
    }

    pub async fn lookup_disease_by_xref(
        &self,
        kind: &str,
        value: &str,
        size: usize,
    ) -> Result<MyDiseaseQueryResponse, BioMcpError> {
        let plan = Self::lookup_disease_by_xref_plan(kind, value, size)?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.get_json(req).await
    }

    pub(crate) fn get_plan(id: &str) -> Result<RequestPlan, BioMcpError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Disease ID is required. Example: biomcp get disease MONDO:0005105".into(),
            ));
        }
        if id.len() > 128 {
            return Err(BioMcpError::InvalidArgument(
                "Disease ID is too long.".into(),
            ));
        }
        if id.contains(['/', '\\', '?', '#']) {
            return Err(BioMcpError::InvalidArgument(
                "Disease ID must not contain path or query separators.".into(),
            ));
        }

        Ok(RequestPlan::get(format!("disease/{id}")).query("fields", MYDISEASE_GET_FIELDS))
    }

    // dead-code reason: mydisease::get_request_plan preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub fn get_request_plan(&self, id: &str) -> Result<MyDiseaseGetRequestPlan, BioMcpError> {
        let plan = Self::get_plan(id)?;
        Ok(Self::legacy_get_plan(&plan))
    }

    pub(crate) fn decode_get_hit(
        status: reqwest::StatusCode,
        content_type: Option<&reqwest::header::HeaderValue>,
        bytes: &[u8],
        id: &str,
    ) -> Result<MyDiseaseHit, BioMcpError> {
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(BioMcpError::NotFound {
                entity: "disease".into(),
                id: id.trim().into(),
                suggestion: format!("Try searching: biomcp search disease -q \"{}\"", id.trim()),
            });
        }
        crate::sources::decode_json(MYDISEASE_API, status, content_type, bytes, false)
    }

    pub async fn get(&self, id: &str) -> Result<MyDiseaseHit, BioMcpError> {
        let plan = Self::get_plan(id)?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let resp = crate::sources::apply_cache_mode(req).send().await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, MYDISEASE_API).await?;
        Self::decode_get_hit(status, content_type.as_ref(), &bytes, id)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MyDiseaseQueryResponse {
    // dead-code reason: mydisease::total preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub total: usize,
    pub hits: Vec<MyDiseaseHit>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MyDiseaseHit {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(default)]
    pub mondo: Option<serde_json::Value>,
    #[serde(default, rename = "disease_ontology")]
    pub disease_ontology: Option<serde_json::Value>,
    #[serde(default)]
    pub umls: Option<serde_json::Value>,
    #[serde(default)]
    pub disgenet: Option<serde_json::Value>,
    #[serde(default)]
    pub hpo: Option<MyDiseaseHpo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyDiseaseHpo {
    #[serde(default, deserialize_with = "de_vec_or_single")]
    pub phenotype_related_to_disease: Vec<MyDiseasePhenotypeRelatedToDisease>,
    #[serde(default, deserialize_with = "de_vec_or_single")]
    pub inheritance: Vec<MyDiseaseInheritance>,
    #[serde(default, deserialize_with = "de_vec_or_single")]
    pub clinical_course: Vec<MyDiseaseClinicalCourse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyDiseasePhenotypeRelatedToDisease {
    pub hpo_id: Option<String>,
    pub evidence: Option<String>,
    #[serde(rename = "hp_freq")]
    pub hp_freq: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyDiseaseInheritance {
    pub hpo_id: Option<String>,
    pub hpo_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyDiseaseClinicalCourse {
    pub hpo_name: Option<String>,
}

#[cfg(test)]
mod tests;
