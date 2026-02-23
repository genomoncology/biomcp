use std::borrow::Cow;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::error::BioMcpError;

const OPENTARGETS_BASE: &str = "https://api.platform.opentargets.org/api/v4";
const OPENTARGETS_API: &str = "opentargets";
const OPENTARGETS_BASE_ENV: &str = "BIOMCP_OPENTARGETS_BASE";

pub struct OpenTargetsClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl OpenTargetsClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(OPENTARGETS_BASE, OPENTARGETS_BASE_ENV),
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

    async fn post_json<T: DeserializeOwned, B: Serialize>(
        &self,
        req: reqwest_middleware::RequestBuilder,
        body: &B,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req.json(body))
            .send()
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, OPENTARGETS_API).await?;

        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: OPENTARGETS_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        crate::sources::ensure_json_content_type(OPENTARGETS_API, content_type.as_ref(), &bytes)?;
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: OPENTARGETS_API.to_string(),
            source,
        })
    }

    pub async fn drug_sections(
        &self,
        chembl_id: &str,
        limit: usize,
    ) -> Result<OpenTargetsDrugSections, BioMcpError> {
        let chembl_id = chembl_id.trim();
        if chembl_id.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "OpenTargets requires chemblId".into(),
            ));
        }

        let size = limit.clamp(1, 25);
        let url = self.endpoint("graphql");
        let body = GraphQlRequest {
            query: r#"
query DrugSections($chemblId: String!) {
  drug(chemblId: $chemblId) {
    id
    name
    indications {
      rows {
        maxPhaseForIndication
        disease { name }
      }
    }
    linkedTargets {
      rows {
        approvedSymbol
      }
    }
  }
}
"#,
            variables: serde_json::json!({
                "chemblId": chembl_id,
            }),
        };

        let resp: GraphQlResponse<DrugSectionsData> =
            self.post_json(self.client.post(&url), &body).await?;

        if let Some(errors) = resp.errors {
            let msg = errors
                .into_iter()
                .filter_map(|e| e.message)
                .collect::<Vec<_>>()
                .join("; ");
            if !msg.is_empty() {
                return Err(BioMcpError::Api {
                    api: OPENTARGETS_API.to_string(),
                    message: msg,
                });
            }
        }

        let Some(drug) = resp.data.and_then(|d| d.drug) else {
            warn_missing_field("DrugSections", "data.drug");
            return Ok(OpenTargetsDrugSections::default());
        };

        let mut indications = Vec::new();
        if let Some(ind) = drug.indications {
            for row in ind.rows.into_iter().take(size) {
                let Some(disease) = row.disease else { continue };
                let Some(name) = disease.name.map(|v| v.trim().to_string()) else {
                    continue;
                };
                if name.is_empty() {
                    continue;
                }
                indications.push(OpenTargetsIndication {
                    disease_name: name,
                    max_phase: row.max_phase_for_indication,
                });
            }
        } else {
            warn_missing_field("DrugSections", "data.drug.indications");
        }

        let mut targets = Vec::new();
        if let Some(linked) = drug.linked_targets {
            for row in linked.rows.into_iter().take(size) {
                let Some(symbol) = row.approved_symbol.map(|v| v.trim().to_string()) else {
                    continue;
                };
                if symbol.is_empty() {
                    continue;
                }
                targets.push(OpenTargetsTarget {
                    approved_symbol: symbol,
                });
            }
        } else {
            warn_missing_field("DrugSections", "data.drug.linkedTargets");
        }

        Ok(OpenTargetsDrugSections {
            indications,
            targets,
        })
    }

    pub async fn disease_associated_targets(
        &self,
        disease_query: &str,
        limit: usize,
    ) -> Result<Vec<OpenTargetsAssociatedGene>, BioMcpError> {
        let disease_query = disease_query.trim();
        if disease_query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "OpenTargets disease query is required".into(),
            ));
        }

        let efo_id = self.resolve_disease_id(disease_query).await?;
        let Some(efo_id) = efo_id else {
            return Ok(Vec::new());
        };

        let size = limit.clamp(1, 25);
        let url = self.endpoint("graphql");
        let body = GraphQlRequest {
            query: r#"
query DiseaseGenes($efoId: String!, $size: Int!) {
  disease(efoId: $efoId) {
    id
    name
    associatedTargets(page: {index: 0, size: $size}) {
      rows {
        target {
          approvedSymbol
        }
      }
    }
  }
}
"#,
            variables: serde_json::json!({
                "efoId": efo_id,
                "size": size,
            }),
        };

        let resp: GraphQlResponse<DiseaseGenesData> =
            self.post_json(self.client.post(&url), &body).await?;

        if let Some(errors) = resp.errors {
            let msg = errors
                .into_iter()
                .filter_map(|e| e.message)
                .collect::<Vec<_>>()
                .join("; ");
            if !msg.is_empty() {
                return Err(BioMcpError::Api {
                    api: OPENTARGETS_API.to_string(),
                    message: msg,
                });
            }
        }

        let Some(disease) = resp.data.and_then(|d| d.disease) else {
            warn_missing_field("DiseaseGenes", "data.disease");
            return Ok(Vec::new());
        };

        let Some(rows) = disease.associated_targets.map(|v| v.rows) else {
            warn_missing_field("DiseaseGenes", "data.disease.associatedTargets");
            return Ok(Vec::new());
        };

        let mut out = Vec::new();
        for row in rows.into_iter().take(size) {
            let Some(target) = row.target else { continue };
            let Some(symbol) = target
                .approved_symbol
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
            else {
                continue;
            };
            out.push(OpenTargetsAssociatedGene {
                symbol: symbol.to_string(),
            });
        }

        Ok(out)
    }

    pub async fn target_clinical_context(
        &self,
        symbol: &str,
        limit: usize,
    ) -> Result<OpenTargetsTargetClinicalContext, BioMcpError> {
        let symbol = symbol.trim();
        if symbol.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "OpenTargets target symbol is required".into(),
            ));
        }

        let Some(target_id) = self.resolve_target_id(symbol).await? else {
            return Ok(OpenTargetsTargetClinicalContext::default());
        };

        let size = limit.clamp(1, 25);
        let url = self.endpoint("graphql");
        let body = GraphQlRequest {
            query: r#"
query TargetClinicalContext($ensemblId: String!, $size: Int!) {
  target(ensemblId: $ensemblId) {
    associatedDiseases(page: {index: 0, size: $size}) {
      rows {
        score
        disease {
          id
          name
        }
      }
    }
    knownDrugs(size: $size) {
      rows {
        phase
        drug {
          id
          name
        }
        disease {
          id
          name
        }
      }
    }
  }
}
"#,
            variables: serde_json::json!({
                "ensemblId": target_id,
                "size": size,
            }),
        };

        let resp: GraphQlResponse<TargetClinicalData> =
            self.post_json(self.client.post(&url), &body).await?;

        if let Some(errors) = resp.errors {
            let msg = errors
                .into_iter()
                .filter_map(|e| e.message)
                .collect::<Vec<_>>()
                .join("; ");
            if !msg.is_empty() {
                return Err(BioMcpError::Api {
                    api: OPENTARGETS_API.to_string(),
                    message: msg,
                });
            }
        }

        let Some(target) = resp.data.and_then(|d| d.target) else {
            warn_missing_field("TargetClinicalContext", "data.target");
            return Ok(OpenTargetsTargetClinicalContext::default());
        };

        let mut diseases: Vec<String> = Vec::new();
        let mut disease_seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        if let Some(associated) = target.associated_diseases {
            for row in associated.rows {
                let Some(name) = row
                    .disease
                    .and_then(|d| d.name)
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty())
                else {
                    continue;
                };
                let key = name.to_ascii_lowercase();
                if !disease_seen.insert(key) {
                    continue;
                }
                diseases.push(name);
                if diseases.len() >= size {
                    break;
                }
            }
        } else {
            warn_missing_field("TargetClinicalContext", "data.target.associatedDiseases");
        }

        let mut drugs: Vec<String> = Vec::new();
        let mut drug_seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        if let Some(known_drugs) = target.known_drugs {
            for row in known_drugs.rows {
                let Some(name) = row
                    .drug
                    .and_then(|d| d.name)
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty())
                else {
                    continue;
                };
                let key = name.to_ascii_lowercase();
                if !drug_seen.insert(key) {
                    continue;
                }
                drugs.push(name);
                if drugs.len() >= size {
                    break;
                }
            }
        } else {
            warn_missing_field("TargetClinicalContext", "data.target.knownDrugs");
        }

        Ok(OpenTargetsTargetClinicalContext { diseases, drugs })
    }

    pub async fn disease_prevalence(
        &self,
        disease_query: &str,
        limit: usize,
    ) -> Result<Vec<OpenTargetsDiseasePrevalence>, BioMcpError> {
        let disease_query = disease_query.trim();
        if disease_query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "OpenTargets disease query is required".into(),
            ));
        }

        let Some(efo_id) = self.resolve_disease_id(disease_query).await? else {
            return Ok(Vec::new());
        };
        let size = limit.clamp(1, 20);
        let url = self.endpoint("graphql");
        let body = GraphQlRequest {
            query: r#"
query DiseasePrevalence($efoId: String!, $size: Int!) {
  disease(efoId: $efoId) {
    phenotypes(page: {index: 0, size: $size}) {
      rows {
        phenotypeHPO { id name }
        evidence {
          frequency
          frequencyHPO { id name }
          resource
          evidenceType
          sex
          onset { id name }
        }
      }
    }
  }
}
"#,
            variables: serde_json::json!({
                "efoId": efo_id,
                "size": size,
            }),
        };

        let resp: GraphQlResponse<DiseasePrevalenceData> =
            self.post_json(self.client.post(&url), &body).await?;

        if let Some(errors) = resp.errors {
            let msg = errors
                .into_iter()
                .filter_map(|e| e.message)
                .collect::<Vec<_>>()
                .join("; ");
            if !msg.is_empty() {
                return Err(BioMcpError::Api {
                    api: OPENTARGETS_API.to_string(),
                    message: msg,
                });
            }
        }

        let Some(rows) = resp
            .data
            .and_then(|d| d.disease)
            .and_then(|d| d.phenotypes)
            .map(|p| p.rows)
        else {
            return Ok(Vec::new());
        };

        let mut out: Vec<OpenTargetsDiseasePrevalence> = Vec::new();
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

        for row in rows {
            let phenotype_name = row
                .phenotype_hpo
                .as_ref()
                .and_then(|h| h.name.as_deref())
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string);

            for ev in row.evidence {
                let estimate = ev
                    .frequency
                    .as_deref()
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(str::to_string)
                    .or_else(|| {
                        ev.frequency_hpo
                            .as_ref()
                            .and_then(|h| h.name.as_deref())
                            .map(str::trim)
                            .filter(|v| !v.is_empty())
                            .map(str::to_string)
                    });
                let Some(estimate) = estimate else {
                    continue;
                };

                let mut context_parts: Vec<String> = Vec::new();
                if let Some(name) = phenotype_name.as_deref() {
                    context_parts.push(format!("Phenotype: {name}"));
                }
                if let Some(sex) = ev.sex.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
                    context_parts.push(format!("Sex: {sex}"));
                }
                let onset = ev
                    .onset
                    .iter()
                    .filter_map(|o| o.name.as_deref())
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .collect::<Vec<_>>();
                if !onset.is_empty() {
                    context_parts.push(format!("Onset: {}", onset.join(", ")));
                }
                let context = if context_parts.is_empty() {
                    None
                } else {
                    Some(context_parts.join("; "))
                };

                let source = match (
                    ev.resource
                        .as_deref()
                        .map(str::trim)
                        .filter(|v| !v.is_empty()),
                    ev.evidence_type
                        .as_deref()
                        .map(str::trim)
                        .filter(|v| !v.is_empty()),
                ) {
                    (Some(resource), Some(kind)) => Some(format!("{resource} ({kind})")),
                    (Some(resource), None) => Some(resource.to_string()),
                    (None, Some(kind)) => Some(kind.to_string()),
                    (None, None) => None,
                };

                let dedupe = format!(
                    "{}|{}|{}",
                    estimate.to_ascii_lowercase(),
                    context.as_deref().unwrap_or("").to_ascii_lowercase(),
                    source.as_deref().unwrap_or("").to_ascii_lowercase()
                );
                if !seen.insert(dedupe) {
                    continue;
                }

                out.push(OpenTargetsDiseasePrevalence {
                    estimate,
                    context,
                    source,
                });
                if out.len() >= size {
                    return Ok(out);
                }
            }
        }

        Ok(out)
    }

    async fn resolve_disease_id(&self, disease_query: &str) -> Result<Option<String>, BioMcpError> {
        let prefixed = normalize_disease_id(disease_query);
        if let Some(id) = prefixed.as_deref().filter(|id| id.starts_with("EFO_")) {
            return Ok(Some(id.to_string()));
        }

        let url = self.endpoint("graphql");
        let body = GraphQlRequest {
            query: r#"
query SearchDisease($query: String!) {
  search(queryString: $query, entityNames: ["disease"], page: {index: 0, size: 5}) {
    hits {
      id
      name
      entity
    }
  }
}
"#,
            variables: serde_json::json!({ "query": disease_query }),
        };

        let resp: GraphQlResponse<SearchData> =
            self.post_json(self.client.post(&url), &body).await?;

        if let Some(errors) = resp.errors {
            let msg = errors
                .into_iter()
                .filter_map(|e| e.message)
                .collect::<Vec<_>>()
                .join("; ");
            if !msg.is_empty() {
                return Err(BioMcpError::Api {
                    api: OPENTARGETS_API.to_string(),
                    message: msg,
                });
            }
        }

        let from_search = resp
            .data
            .and_then(|d| d.search)
            .and_then(|s| {
                s.hits
                    .into_iter()
                    .find(|h| h.entity.as_deref() == Some("disease"))
            })
            .and_then(|h| h.id);

        Ok(from_search.or(prefixed))
    }

    async fn resolve_target_id(&self, symbol: &str) -> Result<Option<String>, BioMcpError> {
        let symbol = symbol.trim();
        if symbol.is_empty() {
            return Ok(None);
        }

        let url = self.endpoint("graphql");
        let body = GraphQlRequest {
            query: r#"
query SearchTarget($query: String!) {
  search(queryString: $query, entityNames: ["target"], page: {index: 0, size: 10}) {
    hits {
      id
      entity
      object {
        ... on Target {
          approvedSymbol
        }
      }
    }
  }
}
"#,
            variables: serde_json::json!({ "query": symbol }),
        };

        let resp: GraphQlResponse<TargetSearchData> =
            self.post_json(self.client.post(&url), &body).await?;

        if let Some(errors) = resp.errors {
            let msg = errors
                .into_iter()
                .filter_map(|e| e.message)
                .collect::<Vec<_>>()
                .join("; ");
            if !msg.is_empty() {
                return Err(BioMcpError::Api {
                    api: OPENTARGETS_API.to_string(),
                    message: msg,
                });
            }
        }

        let hits = resp
            .data
            .and_then(|d| d.search)
            .map(|s| s.hits)
            .unwrap_or_default();

        for hit in &hits {
            let approved_symbol = hit
                .object
                .as_ref()
                .and_then(|o| o.approved_symbol.as_deref())
                .map(str::trim);
            if hit.entity.as_deref() == Some("target")
                && approved_symbol.is_some_and(|v| v.eq_ignore_ascii_case(symbol))
                && let Some(id) = hit.id.as_deref().map(str::trim).filter(|v| !v.is_empty())
            {
                return Ok(Some(id.to_string()));
            }
        }

        Ok(hits
            .into_iter()
            .find(|h| h.entity.as_deref() == Some("target"))
            .and_then(|h| h.id)
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty()))
    }
}

fn normalize_disease_id(input: &str) -> Option<String> {
    let v = input.trim();
    if v.is_empty() {
        return None;
    }

    if v.contains('_') && v.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Some(v.to_string());
    }

    if let Some((prefix, rest)) = v.split_once(':') {
        let rest = rest.trim();
        if rest.is_empty() {
            return None;
        }
        return Some(format!("{}_{}", prefix.trim().to_ascii_uppercase(), rest));
    }

    None
}

fn warn_missing_field(operation: &str, field: &str) {
    warn!(
        source = OPENTARGETS_API,
        operation = operation,
        field = field,
        "Missing expected GraphQL field; degrading response"
    );
}

#[derive(Debug, Clone, Default)]
pub struct OpenTargetsDrugSections {
    pub indications: Vec<OpenTargetsIndication>,
    pub targets: Vec<OpenTargetsTarget>,
}

#[derive(Debug, Clone)]
pub struct OpenTargetsIndication {
    pub disease_name: String,
    pub max_phase: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct OpenTargetsTarget {
    pub approved_symbol: String,
}

#[derive(Debug, Clone)]
pub struct OpenTargetsAssociatedGene {
    pub symbol: String,
}

#[derive(Debug, Clone, Default)]
pub struct OpenTargetsTargetClinicalContext {
    pub diseases: Vec<String>,
    pub drugs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct OpenTargetsDiseasePrevalence {
    pub estimate: String,
    pub context: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Serialize)]
struct GraphQlRequest<'a> {
    query: &'a str,
    variables: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct GraphQlResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQlError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQlError {
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DrugSectionsData {
    drug: Option<DrugNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DrugNode {
    indications: Option<DrugIndications>,
    linked_targets: Option<LinkedTargets>,
}

#[derive(Debug, Deserialize)]
struct DrugIndications {
    #[serde(default)]
    rows: Vec<IndicationRow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IndicationRow {
    max_phase_for_indication: Option<f64>,
    disease: Option<SimpleDisease>,
}

#[derive(Debug, Deserialize)]
struct SimpleDisease {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LinkedTargets {
    #[serde(default)]
    rows: Vec<TargetNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TargetNode {
    approved_symbol: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DiseaseGenesData {
    disease: Option<DiseaseNode>,
}

#[derive(Debug, Deserialize)]
struct DiseasePrevalenceData {
    disease: Option<DiseasePrevalenceNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiseasePrevalenceNode {
    phenotypes: Option<DiseasePhenotypes>,
}

#[derive(Debug, Deserialize)]
struct DiseasePhenotypes {
    #[serde(default)]
    rows: Vec<DiseasePhenotypeRow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiseasePhenotypeRow {
    #[serde(rename = "phenotypeHPO")]
    phenotype_hpo: Option<HpoNode>,
    #[serde(default)]
    evidence: Vec<DiseaseHpoEvidence>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiseaseHpoEvidence {
    frequency: Option<String>,
    #[serde(rename = "frequencyHPO")]
    frequency_hpo: Option<HpoNode>,
    resource: Option<String>,
    evidence_type: Option<String>,
    sex: Option<String>,
    #[serde(default)]
    onset: Vec<HpoNode>,
}

#[derive(Debug, Deserialize)]
struct HpoNode {
    #[allow(dead_code)]
    id: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiseaseNode {
    associated_targets: Option<AssociatedTargets>,
}

#[derive(Debug, Deserialize)]
struct AssociatedTargets {
    #[serde(default)]
    rows: Vec<AssociatedTargetRow>,
}

#[derive(Debug, Deserialize)]
struct AssociatedTargetRow {
    target: Option<TargetNode>,
}

#[derive(Debug, Deserialize)]
struct SearchData {
    search: Option<SearchResult>,
}

#[derive(Debug, Deserialize)]
struct SearchResult {
    #[serde(default)]
    hits: Vec<SearchHit>,
}

#[derive(Debug, Deserialize)]
struct SearchHit {
    id: Option<String>,
    entity: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TargetSearchData {
    search: Option<TargetSearchResult>,
}

#[derive(Debug, Deserialize)]
struct TargetSearchResult {
    #[serde(default)]
    hits: Vec<TargetSearchHit>,
}

#[derive(Debug, Deserialize)]
struct TargetSearchHit {
    id: Option<String>,
    entity: Option<String>,
    object: Option<TargetSearchObject>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TargetSearchObject {
    approved_symbol: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TargetClinicalData {
    target: Option<TargetClinicalNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TargetClinicalNode {
    associated_diseases: Option<TargetAssociatedDiseases>,
    known_drugs: Option<TargetKnownDrugs>,
}

#[derive(Debug, Deserialize)]
struct TargetAssociatedDiseases {
    #[serde(default)]
    rows: Vec<TargetAssociatedDiseaseRow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TargetAssociatedDiseaseRow {
    #[allow(dead_code)]
    score: Option<f64>,
    disease: Option<TargetDiseaseNode>,
}

#[derive(Debug, Deserialize)]
struct TargetDiseaseNode {
    #[allow(dead_code)]
    id: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TargetKnownDrugs {
    #[serde(default)]
    rows: Vec<TargetKnownDrugRow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TargetKnownDrugRow {
    #[allow(dead_code)]
    phase: Option<f64>,
    drug: Option<TargetDrugNode>,
    #[allow(dead_code)]
    disease: Option<TargetDiseaseNode>,
}

#[derive(Debug, Deserialize)]
struct TargetDrugNode {
    #[allow(dead_code)]
    id: Option<String>,
    name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn normalize_disease_id_handles_known_forms() {
        assert_eq!(
            normalize_disease_id("MONDO:0005105").as_deref(),
            Some("MONDO_0005105")
        );
        assert_eq!(
            normalize_disease_id("EFO_0000311").as_deref(),
            Some("EFO_0000311")
        );
        assert_eq!(normalize_disease_id(""), None);
    }

    #[tokio::test]
    async fn drug_sections_maps_targets_and_indications() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_string_contains("DrugSections"))
            .and(body_string_contains("\"chemblId\":\"CHEMBL25\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "drug": {
                        "indications": {
                            "rows": [
                                {
                                    "maxPhaseForIndication": 4,
                                    "disease": {"name": "Melanoma"}
                                }
                            ]
                        },
                        "linkedTargets": {
                            "rows": [
                                {"approvedSymbol": "BRAF"},
                                {"approvedSymbol": "MAP2K1"}
                            ]
                        }
                    }
                }
            })))
            .mount(&server)
            .await;

        let client = OpenTargetsClient::new_for_test(server.uri()).unwrap();
        let sections = client.drug_sections("CHEMBL25", 5).await.unwrap();
        assert_eq!(sections.indications.len(), 1);
        assert_eq!(sections.indications[0].disease_name, "Melanoma");
        assert_eq!(sections.targets.len(), 2);
        assert_eq!(sections.targets[0].approved_symbol, "BRAF");
    }

    #[tokio::test]
    async fn drug_sections_degrades_when_indications_missing() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_string_contains("DrugSections"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "drug": {
                        "linkedTargets": {
                            "rows": [
                                {"approvedSymbol": "BRAF"}
                            ]
                        }
                    }
                }
            })))
            .mount(&server)
            .await;

        let client = OpenTargetsClient::new_for_test(server.uri()).unwrap();
        let sections = client.drug_sections("CHEMBL25", 5).await.unwrap();
        assert!(sections.indications.is_empty());
        assert_eq!(sections.targets.len(), 1);
    }

    #[tokio::test]
    async fn disease_associated_targets_runs_search_then_lookup() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_string_contains("SearchDisease"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "search": {
                        "hits": [
                            {"id": "EFO_0000311", "entity": "disease"}
                        ]
                    }
                }
            })))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_string_contains("DiseaseGenes"))
            .and(body_string_contains("\"efoId\":\"EFO_0000311\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "disease": {
                        "associatedTargets": {
                            "rows": [
                                {"target": {"approvedSymbol": "BRAF"}},
                                {"target": {"approvedSymbol": "KRAS"}}
                            ]
                        }
                    }
                }
            })))
            .mount(&server)
            .await;

        let client = OpenTargetsClient::new_for_test(server.uri()).unwrap();
        let genes = client
            .disease_associated_targets("melanoma", 5)
            .await
            .unwrap();
        assert_eq!(genes.len(), 2);
        assert_eq!(genes[0].symbol, "BRAF");
        assert_eq!(genes[1].symbol, "KRAS");
    }

    #[tokio::test]
    async fn disease_associated_targets_degrades_when_associated_targets_missing() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_string_contains("SearchDisease"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "search": {
                        "hits": [
                            {"id": "EFO_0000311", "entity": "disease"}
                        ]
                    }
                }
            })))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_string_contains("DiseaseGenes"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "disease": {}
                }
            })))
            .mount(&server)
            .await;

        let client = OpenTargetsClient::new_for_test(server.uri()).unwrap();
        let genes = client
            .disease_associated_targets("melanoma", 5)
            .await
            .unwrap();
        assert!(genes.is_empty());
    }

    #[tokio::test]
    async fn disease_prevalence_maps_frequency_evidence() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_string_contains("SearchDisease"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "search": {
                        "hits": [
                            {"id": "MONDO_0007947", "entity": "disease"}
                        ]
                    }
                }
            })))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_string_contains("DiseasePrevalence"))
            .and(body_string_contains("\"efoId\":\"MONDO_0007947\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "disease": {
                        "phenotypes": {
                            "rows": [
                                {
                                    "phenotypeHPO": {"id": "HP_0000278", "name": "Retrognathia"},
                                    "evidence": [
                                        {
                                            "frequency": "10/16",
                                            "resource": "HPO",
                                            "evidenceType": "PCS",
                                            "sex": null,
                                            "onset": []
                                        }
                                    ]
                                }
                            ]
                        }
                    }
                }
            })))
            .mount(&server)
            .await;

        let client = OpenTargetsClient::new_for_test(server.uri()).unwrap();
        let rows = client
            .disease_prevalence("Marfan syndrome", 5)
            .await
            .expect("prevalence rows");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].estimate, "10/16");
        assert!(
            rows[0]
                .context
                .as_deref()
                .is_some_and(|v| v.contains("Retrognathia"))
        );
        assert_eq!(rows[0].source.as_deref(), Some("HPO (PCS)"));
    }

    #[tokio::test]
    async fn target_clinical_context_collects_diseases_and_drugs() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_string_contains("SearchTarget"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "search": {
                        "hits": [
                            {"id": "ENSG00000157764", "entity": "target", "object": {"approvedSymbol": "BRAF"}}
                        ]
                    }
                }
            })))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_string_contains("TargetClinicalContext"))
            .and(body_string_contains("\"ensemblId\":\"ENSG00000157764\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "target": {
                        "associatedDiseases": {
                            "rows": [
                                {"score": 0.8, "disease": {"id": "EFO_1", "name": "Melanoma"}},
                                {"score": 0.7, "disease": {"id": "EFO_2", "name": "Colorectal cancer"}}
                            ]
                        },
                        "knownDrugs": {
                            "rows": [
                                {"phase": 4, "drug": {"id": "CHEMBL1", "name": "Dabrafenib"}, "disease": {"id": "EFO_1", "name": "Melanoma"}},
                                {"phase": 4, "drug": {"id": "CHEMBL2", "name": "Vemurafenib"}, "disease": {"id": "EFO_1", "name": "Melanoma"}}
                            ]
                        }
                    }
                }
            })))
            .mount(&server)
            .await;

        let client = OpenTargetsClient::new_for_test(server.uri()).unwrap();
        let context = client.target_clinical_context("BRAF", 5).await.unwrap();
        assert_eq!(context.diseases, vec!["Melanoma", "Colorectal cancer"]);
        assert_eq!(context.drugs, vec!["Dabrafenib", "Vemurafenib"]);
    }

    #[tokio::test]
    async fn target_clinical_context_degrades_when_known_drugs_missing() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_string_contains("SearchTarget"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "search": {
                        "hits": [
                            {"id": "ENSG00000157764", "entity": "target", "object": {"approvedSymbol": "BRAF"}}
                        ]
                    }
                }
            })))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_string_contains("TargetClinicalContext"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "target": {
                        "associatedDiseases": {
                            "rows": [
                                {"score": 0.8, "disease": {"id": "EFO_1", "name": "Melanoma"}}
                            ]
                        }
                    }
                }
            })))
            .mount(&server)
            .await;

        let client = OpenTargetsClient::new_for_test(server.uri()).unwrap();
        let context = client.target_clinical_context("BRAF", 5).await.unwrap();
        assert_eq!(context.diseases, vec!["Melanoma"]);
        assert!(context.drugs.is_empty());
    }

    #[tokio::test]
    async fn drug_sections_propagates_graphql_error_message() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_string_contains("DrugSections"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "errors": [
                    {"message": "Cannot query field linkedTargets on type Drug"}
                ]
            })))
            .mount(&server)
            .await;

        let client = OpenTargetsClient::new_for_test(server.uri()).unwrap();
        let err = client.drug_sections("CHEMBL25", 5).await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("linkedTargets"));
        assert!(msg.contains("opentargets"));
    }

    #[tokio::test]
    async fn disease_associated_targets_egfr_lung() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_string_contains("SearchDisease"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "search": {
                        "hits": [{"id": "EFO_0001071", "entity": "disease"}]
                    }
                }
            })))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_string_contains("DiseaseGenes"))
            .and(body_string_contains("\"efoId\":\"EFO_0001071\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "disease": {
                        "associatedTargets": {
                            "rows": [
                                {"target": {"approvedSymbol": "EGFR"}},
                                {"target": {"approvedSymbol": "ERBB2"}}
                            ]
                        }
                    }
                }
            })))
            .mount(&server)
            .await;

        let client = OpenTargetsClient::new_for_test(server.uri()).unwrap();
        let genes = client
            .disease_associated_targets("lung adenocarcinoma", 3)
            .await
            .unwrap();
        assert_eq!(genes.first().map(|g| g.symbol.as_str()), Some("EGFR"));
    }

    #[tokio::test]
    async fn drug_sections_maps_osimertinib() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_string_contains("DrugSections"))
            .and(body_string_contains("\"chemblId\":\"CHEMBL3353410\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "drug": {
                        "indications": {
                            "rows": [
                                {"maxPhaseForIndication": 4, "disease": {"name": "Non-small cell lung cancer"}}
                            ]
                        },
                        "linkedTargets": {
                            "rows": [
                                {"approvedSymbol": "EGFR"}
                            ]
                        }
                    }
                }
            })))
            .mount(&server)
            .await;

        let client = OpenTargetsClient::new_for_test(server.uri()).unwrap();
        let sections = client.drug_sections("CHEMBL3353410", 5).await.unwrap();
        assert_eq!(
            sections
                .indications
                .first()
                .map(|i| i.disease_name.as_str()),
            Some("Non-small cell lung cancer")
        );
        assert_eq!(
            sections.targets.first().map(|t| t.approved_symbol.as_str()),
            Some("EGFR")
        );
    }
}
