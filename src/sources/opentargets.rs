use std::borrow::Cow;

use serde::Deserialize;
use serde::de::DeserializeOwned;
use tracing::warn;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const OPENTARGETS_BASE: &str = "https://api.platform.opentargets.org/api/v4";
const OPENTARGETS_API: &str = "opentargets";
const OPENTARGETS_BASE_ENV: &str = "BIOMCP_OPENTARGETS_BASE";
const GWAS_CREDIBLE_SETS_DATASOURCE_ID: &str = "gwas_credible_sets";
const SOMATIC_MUTATION_DATATYPE_ID: &str = "somatic_mutation";
// Derived from the current OpenTargets associationDatasources taxonomy.
const RARE_VARIANT_DATASOURCE_IDS: &[&str] =
    &["eva", "gene_burden", "orphanet", "uniprot_variants"];
const DRUG_SECTIONS_QUERY: &str = r#"
query DrugSections($chemblId: String!) {
  drug(chemblId: $chemblId) {
    id
    name
    indications {
      rows {
        maxClinicalStage
        disease { name }
      }
    }
    mechanismsOfAction {
      rows {
        targets {
          approvedSymbol
          approvedName
        }
      }
    }
  }
}
"#;
const DISEASE_GENES_QUERY: &str = r#"
query DiseaseGenes($efoId: String!, $size: Int!) {
  disease(efoId: $efoId) {
    id
    name
    associatedTargets(page: {index: 0, size: $size}) {
      rows {
        score
        datatypeScores {
          id
          score
        }
        datasourceScores {
          id
          score
        }
        target {
          approvedSymbol
        }
      }
    }
  }
}
"#;
const SEARCH_DISEASE_QUERY: &str = r#"
query SearchDisease($query: String!) {
  search(queryString: $query, entityNames: ["disease"], page: {index: 0, size: 5}) {
    hits {
      id
      name
      entity
    }
  }
}
"#;
const SEARCH_TARGET_QUERY: &str = r#"
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
"#;
const TARGET_DRUGGABILITY_CONTEXT_QUERY: &str = r#"
query TargetDruggabilityContext($ensemblId: String!) {
  target(ensemblId: $ensemblId) {
    tractability {
      label
      modality
      value
    }
    safetyLiabilities {
      event
      datasource
      effects {
        direction
        dosing
      }
      biosamples {
        tissueLabel
        cellLabel
        cellFormat
      }
    }
  }
}
"#;
const TARGET_CLINICAL_CONTEXT_QUERY: &str = r#"
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
    drugAndClinicalCandidates {
      rows {
        drug {
          id
          name
        }
      }
    }
  }
}
"#;
const DISEASE_PREVALENCE_QUERY: &str = r#"
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
"#;

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

    async fn post_plan_json<T: DeserializeOwned>(
        &self,
        plan: &RequestPlan,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(request_from_plan(
            &self.client,
            self.base.as_ref(),
            plan,
        ))
        .send()
        .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, OPENTARGETS_API).await?;

        decode_graphql_json(status, content_type.as_ref(), &bytes)
    }

    pub async fn drug_sections(
        &self,
        chembl_id: &str,
        limit: usize,
    ) -> Result<OpenTargetsDrugSections, BioMcpError> {
        let size = limit.clamp(1, 25);
        let plan = Self::drug_sections_plan(chembl_id)?;
        let resp: GraphQlResponse<DrugSectionsData> = self.post_plan_json(&plan).await?;

        drug_sections_from_response(resp, size)
    }

    fn drug_sections_plan(chembl_id: &str) -> Result<RequestPlan, BioMcpError> {
        let chembl_id = chembl_id.trim();
        if chembl_id.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "OpenTargets requires chemblId".into(),
            ));
        }
        Ok(RequestPlan::post("graphql").json(serde_json::json!({
            "query": DRUG_SECTIONS_QUERY,
            "variables": {
                "chemblId": chembl_id,
            },
        })))
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
        let plan = Self::disease_genes_plan(&efo_id, size)?;
        let resp: GraphQlResponse<DiseaseGenesData> = self.post_plan_json(&plan).await?;

        disease_associated_targets_from_response(resp, size)
    }

    fn disease_genes_plan(efo_id: &str, size: usize) -> Result<RequestPlan, BioMcpError> {
        let efo_id = efo_id.trim();
        if efo_id.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "OpenTargets disease ID is required".into(),
            ));
        }
        Ok(RequestPlan::post("graphql").json(serde_json::json!({
            "query": DISEASE_GENES_QUERY,
            "variables": {
                "efoId": efo_id,
                "size": size,
            },
        })))
    }

    pub async fn target_druggability_context(
        &self,
        symbol: &str,
    ) -> Result<OpenTargetsTargetDruggabilityContext, BioMcpError> {
        let symbol = symbol.trim();
        if symbol.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "OpenTargets target symbol is required".into(),
            ));
        }

        let Some(target_id) = self.resolve_target_id(symbol).await? else {
            return Ok(OpenTargetsTargetDruggabilityContext::default());
        };

        self.target_druggability_context_for_target_id(&target_id)
            .await
    }

    pub async fn target_druggability_context_for_target_id(
        &self,
        target_id: &str,
    ) -> Result<OpenTargetsTargetDruggabilityContext, BioMcpError> {
        let target_id = target_id.trim();
        if target_id.is_empty() {
            return Ok(OpenTargetsTargetDruggabilityContext::default());
        }

        let plan = Self::target_druggability_context_plan(target_id)?;
        let resp: GraphQlResponse<TargetDruggabilityData> = self.post_plan_json(&plan).await?;

        target_druggability_context_from_response(resp)
    }

    fn target_druggability_context_plan(target_id: &str) -> Result<RequestPlan, BioMcpError> {
        let target_id = target_id.trim();
        if target_id.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "OpenTargets target ID is required".into(),
            ));
        }
        Ok(RequestPlan::post("graphql").json(serde_json::json!({
            "query": TARGET_DRUGGABILITY_CONTEXT_QUERY,
            "variables": {
                "ensemblId": target_id,
            },
        })))
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

        self.target_clinical_context_for_target_id(&target_id, limit)
            .await
    }

    pub async fn target_clinical_context_for_target_id(
        &self,
        target_id: &str,
        limit: usize,
    ) -> Result<OpenTargetsTargetClinicalContext, BioMcpError> {
        let target_id = target_id.trim();
        if target_id.is_empty() {
            return Ok(OpenTargetsTargetClinicalContext::default());
        }

        let size = limit.clamp(1, 25);
        let plan = Self::target_clinical_context_plan(target_id, size)?;
        let resp: GraphQlResponse<TargetClinicalData> = self.post_plan_json(&plan).await?;

        target_clinical_context_from_response(resp, size)
    }

    fn target_clinical_context_plan(
        target_id: &str,
        size: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        let target_id = target_id.trim();
        if target_id.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "OpenTargets target ID is required".into(),
            ));
        }
        Ok(RequestPlan::post("graphql").json(serde_json::json!({
            "query": TARGET_CLINICAL_CONTEXT_QUERY,
            "variables": {
                "ensemblId": target_id,
                "size": size,
            },
        })))
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
        let plan = Self::disease_prevalence_plan(&efo_id, size)?;
        let resp: GraphQlResponse<DiseasePrevalenceData> = self.post_plan_json(&plan).await?;

        disease_prevalence_from_response(resp, size)
    }

    fn disease_prevalence_plan(efo_id: &str, size: usize) -> Result<RequestPlan, BioMcpError> {
        let efo_id = efo_id.trim();
        if efo_id.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "OpenTargets disease ID is required".into(),
            ));
        }
        Ok(RequestPlan::post("graphql").json(serde_json::json!({
            "query": DISEASE_PREVALENCE_QUERY,
            "variables": {
                "efoId": efo_id,
                "size": size,
            },
        })))
    }

    async fn resolve_disease_id(&self, disease_query: &str) -> Result<Option<String>, BioMcpError> {
        let prefixed = normalize_disease_id(disease_query);
        if let Some(id) = prefixed.as_deref().filter(|id| id.starts_with("EFO_")) {
            return Ok(Some(id.to_string()));
        }

        let plan = Self::search_disease_plan(disease_query)?;
        let resp: GraphQlResponse<SearchData> = self.post_plan_json(&plan).await?;
        let from_search = disease_id_from_search_response(resp)?;

        Ok(from_search.or(prefixed))
    }

    fn search_disease_plan(disease_query: &str) -> Result<RequestPlan, BioMcpError> {
        let disease_query = disease_query.trim();
        if disease_query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "OpenTargets disease query is required".into(),
            ));
        }
        Ok(RequestPlan::post("graphql").json(serde_json::json!({
            "query": SEARCH_DISEASE_QUERY,
            "variables": {
                "query": disease_query,
            },
        })))
    }

    async fn resolve_target_id(&self, symbol: &str) -> Result<Option<String>, BioMcpError> {
        let symbol = symbol.trim();
        if symbol.is_empty() {
            return Ok(None);
        }

        let plan = Self::search_target_plan(symbol)?;
        let resp: GraphQlResponse<TargetSearchData> = self.post_plan_json(&plan).await?;

        target_id_from_search_response(resp, symbol)
    }

    fn search_target_plan(symbol: &str) -> Result<RequestPlan, BioMcpError> {
        let symbol = symbol.trim();
        if symbol.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "OpenTargets target symbol is required".into(),
            ));
        }
        Ok(RequestPlan::post("graphql").json(serde_json::json!({
            "query": SEARCH_TARGET_QUERY,
            "variables": {
                "query": symbol,
            },
        })))
    }
}

fn decode_graphql_json<T: DeserializeOwned>(
    status: reqwest::StatusCode,
    content_type: Option<&reqwest::header::HeaderValue>,
    bytes: &[u8],
) -> Result<T, BioMcpError> {
    if !status.is_success() {
        let excerpt = crate::sources::body_excerpt(bytes);
        return Err(BioMcpError::Api {
            api: OPENTARGETS_API.to_string(),
            message: format!("HTTP {status}: {excerpt}"),
        });
    }

    crate::sources::ensure_json_content_type(OPENTARGETS_API, content_type, bytes)?;
    serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
        api: OPENTARGETS_API.to_string(),
        source,
    })
}

fn graphql_error_message<T>(resp: &GraphQlResponse<T>) -> Option<String> {
    resp.errors.as_ref().and_then(|errors| {
        let msg = errors
            .iter()
            .filter_map(|e| e.message.as_deref())
            .collect::<Vec<_>>()
            .join("; ");
        (!msg.is_empty()).then_some(msg)
    })
}

fn drug_sections_from_response(
    resp: GraphQlResponse<DrugSectionsData>,
    size: usize,
) -> Result<OpenTargetsDrugSections, BioMcpError> {
    if let Some(msg) = graphql_error_message(&resp) {
        return Err(BioMcpError::Api {
            api: OPENTARGETS_API.to_string(),
            message: msg,
        });
    }

    let Some(drug) = resp.data.and_then(|d| d.drug) else {
        warn_missing_field("DrugSections", "data.drug");
        return Ok(OpenTargetsDrugSections::default());
    };

    let mut indications = Vec::new();
    if let Some(ind) = drug.indications {
        for row in ind.rows.into_iter().take(size) {
            let Some(disease) = row.disease else {
                continue;
            };
            let Some(name) = disease.name.map(|v| v.trim().to_string()) else {
                continue;
            };
            if name.is_empty() {
                continue;
            }
            indications.push(OpenTargetsIndication {
                disease_name: name,
                max_clinical_stage: row.max_clinical_stage,
            });
        }
    } else {
        warn_missing_field("DrugSections", "data.drug.indications");
    }

    let mut targets = Vec::new();
    if let Some(mechanisms) = drug.mechanisms_of_action {
        for row in mechanisms
            .rows
            .into_iter()
            .flat_map(|row| row.targets)
            .take(size)
        {
            let Some(symbol) = row.approved_symbol.map(|v| v.trim().to_string()) else {
                continue;
            };
            if symbol.is_empty() {
                continue;
            }
            targets.push(OpenTargetsTarget {
                approved_symbol: symbol,
                approved_name: row
                    .approved_name
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty()),
            });
        }
    } else {
        warn_missing_field("DrugSections", "data.drug.mechanismsOfAction");
    }

    Ok(OpenTargetsDrugSections {
        indications,
        targets,
    })
}

fn disease_id_from_search_response(
    resp: GraphQlResponse<SearchData>,
) -> Result<Option<String>, BioMcpError> {
    if let Some(msg) = graphql_error_message(&resp) {
        return Err(BioMcpError::Api {
            api: OPENTARGETS_API.to_string(),
            message: msg,
        });
    }

    let hits = resp
        .data
        .and_then(|d| d.search)
        .map(|s| s.hits)
        .unwrap_or_default();

    let from_search = hits
        .iter()
        .find(|h| {
            h.entity.as_deref() == Some("disease")
                && h.id.as_deref().is_some_and(|id| id.starts_with("EFO_"))
        })
        .and_then(|h| h.id.clone())
        .or_else(|| {
            hits.into_iter()
                .find(|h| h.entity.as_deref() == Some("disease"))
                .and_then(|h| h.id)
        });

    Ok(from_search)
}

fn disease_associated_targets_from_response(
    resp: GraphQlResponse<DiseaseGenesData>,
    size: usize,
) -> Result<Vec<OpenTargetsAssociatedGene>, BioMcpError> {
    if let Some(msg) = graphql_error_message(&resp) {
        return Err(BioMcpError::Api {
            api: OPENTARGETS_API.to_string(),
            message: msg,
        });
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
        let Some(target) = row.target else {
            continue;
        };
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
            overall_score: row.score,
            gwas_score: score_for_id(&row.datasource_scores, GWAS_CREDIBLE_SETS_DATASOURCE_ID),
            rare_variant_score: max_score_for_ids(
                &row.datasource_scores,
                RARE_VARIANT_DATASOURCE_IDS,
            ),
            somatic_mutation_score: score_for_id(
                &row.datatype_scores,
                SOMATIC_MUTATION_DATATYPE_ID,
            ),
        });
    }

    Ok(out)
}

fn target_id_from_search_response(
    resp: GraphQlResponse<TargetSearchData>,
    symbol: &str,
) -> Result<Option<String>, BioMcpError> {
    if let Some(msg) = graphql_error_message(&resp) {
        return Err(BioMcpError::Api {
            api: OPENTARGETS_API.to_string(),
            message: msg,
        });
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

fn target_druggability_context_from_response(
    resp: GraphQlResponse<TargetDruggabilityData>,
) -> Result<OpenTargetsTargetDruggabilityContext, BioMcpError> {
    if let Some(msg) = graphql_error_message(&resp) {
        return Err(BioMcpError::Api {
            api: OPENTARGETS_API.to_string(),
            message: msg,
        });
    }

    let Some(target) = resp.data.and_then(|d| d.target) else {
        warn_missing_field("TargetDruggabilityContext", "data.target");
        return Ok(OpenTargetsTargetDruggabilityContext::default());
    };

    Ok(OpenTargetsTargetDruggabilityContext {
        tractability: summarize_tractability(target.tractability),
        safety_liabilities: summarize_safety_liabilities(target.safety_liabilities),
    })
}

fn target_clinical_context_from_response(
    resp: GraphQlResponse<TargetClinicalData>,
    size: usize,
) -> Result<OpenTargetsTargetClinicalContext, BioMcpError> {
    if let Some(msg) = graphql_error_message(&resp) {
        return Err(BioMcpError::Api {
            api: OPENTARGETS_API.to_string(),
            message: msg,
        });
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
    if let Some(drug_candidates) = target.drug_and_clinical_candidates {
        for row in drug_candidates.rows {
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
        warn_missing_field(
            "TargetClinicalContext",
            "data.target.drugAndClinicalCandidates",
        );
    }

    Ok(OpenTargetsTargetClinicalContext { diseases, drugs })
}

fn disease_prevalence_from_response(
    resp: GraphQlResponse<DiseasePrevalenceData>,
    size: usize,
) -> Result<Vec<OpenTargetsDiseasePrevalence>, BioMcpError> {
    if let Some(msg) = graphql_error_message(&resp) {
        return Err(BioMcpError::Api {
            api: OPENTARGETS_API.to_string(),
            message: msg,
        });
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
    pub max_clinical_stage: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OpenTargetsTarget {
    pub approved_symbol: String,
    pub approved_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OpenTargetsAssociatedGene {
    pub symbol: String,
    pub overall_score: Option<f64>,
    pub gwas_score: Option<f64>,
    pub rare_variant_score: Option<f64>,
    pub somatic_mutation_score: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct OpenTargetsTargetDruggabilityContext {
    pub tractability: Vec<OpenTargetsTractabilityModality>,
    pub safety_liabilities: Vec<OpenTargetsSafetyLiability>,
}

#[derive(Debug, Clone)]
pub struct OpenTargetsTractabilityModality {
    pub modality: String,
    pub tractable: bool,
    pub evidence_labels: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct OpenTargetsSafetyLiability {
    pub event: String,
    pub datasource: Option<String>,
    pub effect_direction: Option<String>,
    pub biosample: Option<String>,
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
    mechanisms_of_action: Option<DrugMechanismsOfAction>,
}

#[derive(Debug, Deserialize)]
struct DrugIndications {
    #[serde(default)]
    rows: Vec<IndicationRow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IndicationRow {
    max_clinical_stage: Option<String>,
    disease: Option<SimpleDisease>,
}

#[derive(Debug, Deserialize)]
struct SimpleDisease {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DrugMechanismsOfAction {
    #[serde(default)]
    rows: Vec<MechanismOfActionRow>,
}

#[derive(Debug, Deserialize)]
struct MechanismOfActionRow {
    #[serde(default)]
    targets: Vec<TargetNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TargetNode {
    approved_symbol: Option<String>,
    approved_name: Option<String>,
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
    // dead-code reason: opentargets::id preserves the provider shape used by source contract fixtures
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
#[serde(rename_all = "camelCase")]
struct AssociatedTargetRow {
    score: Option<f64>,
    #[serde(default)]
    datatype_scores: Vec<AssociationScoreRow>,
    #[serde(default)]
    datasource_scores: Vec<AssociationScoreRow>,
    target: Option<TargetNode>,
}

#[derive(Debug, Deserialize)]
struct AssociationScoreRow {
    id: Option<String>,
    score: Option<f64>,
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
struct TargetDruggabilityData {
    target: Option<TargetDruggabilityNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TargetClinicalNode {
    associated_diseases: Option<TargetAssociatedDiseases>,
    drug_and_clinical_candidates: Option<TargetDrugAndClinicalCandidates>,
}

#[derive(Debug, Deserialize)]
struct TargetAssociatedDiseases {
    #[serde(default)]
    rows: Vec<TargetAssociatedDiseaseRow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TargetAssociatedDiseaseRow {
    // dead-code reason: opentargets::score preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    score: Option<f64>,
    disease: Option<TargetDiseaseNode>,
}

#[derive(Debug, Deserialize)]
struct TargetDiseaseNode {
    // dead-code reason: opentargets::id preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    id: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TargetDrugAndClinicalCandidates {
    #[serde(default)]
    rows: Vec<TargetDrugCandidateRow>,
}

#[derive(Debug, Deserialize)]
struct TargetDrugCandidateRow {
    drug: Option<TargetDrugNode>,
}

#[derive(Debug, Deserialize)]
struct TargetDrugNode {
    // dead-code reason: opentargets::id preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    id: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TargetDruggabilityNode {
    #[serde(default)]
    tractability: Vec<TractabilityRow>,
    #[serde(default)]
    safety_liabilities: Vec<SafetyLiabilityRow>,
}

#[derive(Debug, Deserialize)]
struct TractabilityRow {
    label: Option<String>,
    modality: Option<String>,
    value: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SafetyLiabilityRow {
    event: Option<String>,
    datasource: Option<String>,
    #[serde(default)]
    effects: Vec<SafetyEffectRow>,
    #[serde(default)]
    biosamples: Vec<SafetyBiosampleRow>,
}

#[derive(Debug, Deserialize)]
struct SafetyEffectRow {
    direction: Option<String>,
    // dead-code reason: opentargets::dosing preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    dosing: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SafetyBiosampleRow {
    tissue_label: Option<String>,
    cell_label: Option<String>,
    cell_format: Option<String>,
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn score_for_id(rows: &[AssociationScoreRow], id: &str) -> Option<f64> {
    rows.iter().find_map(|row| {
        row.id
            .as_deref()
            .map(str::trim)
            .filter(|value| value.eq_ignore_ascii_case(id))
            .and(row.score)
    })
}

fn max_score_for_ids(rows: &[AssociationScoreRow], ids: &[&str]) -> Option<f64> {
    rows.iter()
        .filter_map(|row| {
            let id = row.id.as_deref().map(str::trim)?;
            if ids
                .iter()
                .any(|candidate| id.eq_ignore_ascii_case(candidate))
            {
                row.score
            } else {
                None
            }
        })
        .fold(None, |best, score| match best {
            Some(current) if current >= score => Some(current),
            _ => Some(score),
        })
}

fn summarize_tractability(rows: Vec<TractabilityRow>) -> Vec<OpenTargetsTractabilityModality> {
    if rows.is_empty() {
        return Vec::new();
    }

    const KNOWN_MODALITIES: [(&str, &str); 4] = [
        ("SM", "small molecule"),
        ("AB", "antibody"),
        ("PR", "PROTAC"),
        ("OC", "other modality"),
    ];

    #[derive(Default)]
    struct TractabilityAccumulator {
        tractable: bool,
        evidence_labels: Vec<String>,
    }

    let mut by_modality: std::collections::HashMap<String, TractabilityAccumulator> =
        std::collections::HashMap::new();
    let mut unknown_order: Vec<String> = Vec::new();

    for row in rows {
        let Some(modality_code) =
            clean_optional(row.modality).map(|value| value.to_ascii_uppercase())
        else {
            continue;
        };

        if !KNOWN_MODALITIES
            .iter()
            .any(|(code, _)| modality_code.eq_ignore_ascii_case(code))
            && !unknown_order.iter().any(|value| value == &modality_code)
        {
            unknown_order.push(modality_code.clone());
        }

        let accumulator = by_modality.entry(modality_code).or_default();
        if row.value.unwrap_or(false) {
            accumulator.tractable = true;
            if let Some(label) = clean_optional(row.label)
                && !accumulator
                    .evidence_labels
                    .iter()
                    .any(|existing| existing.eq_ignore_ascii_case(&label))
            {
                accumulator.evidence_labels.push(label);
            }
        }
    }

    let mut out = KNOWN_MODALITIES
        .into_iter()
        .map(|(code, label)| {
            let summary = by_modality.remove(code).unwrap_or_default();
            OpenTargetsTractabilityModality {
                modality: label.to_string(),
                tractable: summary.tractable,
                evidence_labels: summary.evidence_labels,
            }
        })
        .collect::<Vec<_>>();

    for modality_code in unknown_order {
        let summary = by_modality.remove(&modality_code).unwrap_or_default();
        out.push(OpenTargetsTractabilityModality {
            modality: modality_code.to_ascii_lowercase(),
            tractable: summary.tractable,
            evidence_labels: summary.evidence_labels,
        });
    }

    out
}

fn summarize_safety_liabilities(rows: Vec<SafetyLiabilityRow>) -> Vec<OpenTargetsSafetyLiability> {
    let mut out: Vec<OpenTargetsSafetyLiability> = Vec::new();
    let mut indices: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for row in rows {
        let Some(event) = clean_optional(row.event) else {
            continue;
        };
        let key = event.trim().to_ascii_lowercase();
        let idx = if let Some(idx) = indices.get(&key).copied() {
            idx
        } else {
            out.push(OpenTargetsSafetyLiability {
                event,
                datasource: None,
                effect_direction: None,
                biosample: None,
            });
            let idx = out.len() - 1;
            indices.insert(key, idx);
            idx
        };

        let liability = &mut out[idx];
        if liability.datasource.is_none() {
            liability.datasource = clean_optional(row.datasource);
        }
        if liability.effect_direction.is_none() {
            liability.effect_direction = row
                .effects
                .into_iter()
                .find_map(|effect| clean_optional(effect.direction));
        }
        if liability.biosample.is_none() {
            liability.biosample = row.biosamples.into_iter().find_map(|biosample| {
                clean_optional(biosample.tissue_label)
                    .or_else(|| clean_optional(biosample.cell_label))
                    .or_else(|| clean_optional(biosample.cell_format))
            });
        }
    }

    out.truncate(8);
    out
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

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

    fn decode_drug_sections(value: serde_json::Value) -> GraphQlResponse<DrugSectionsData> {
        serde_json::from_value(value).expect("drug sections response")
    }

    fn decode_disease_search(value: serde_json::Value) -> GraphQlResponse<SearchData> {
        serde_json::from_value(value).expect("disease search response")
    }

    fn decode_disease_genes(value: serde_json::Value) -> GraphQlResponse<DiseaseGenesData> {
        serde_json::from_value(value).expect("disease genes response")
    }

    fn decode_target_search(value: serde_json::Value) -> GraphQlResponse<TargetSearchData> {
        serde_json::from_value(value).expect("target search response")
    }

    fn decode_target_druggability(
        value: serde_json::Value,
    ) -> GraphQlResponse<TargetDruggabilityData> {
        serde_json::from_value(value).expect("target druggability response")
    }

    fn decode_target_clinical(value: serde_json::Value) -> GraphQlResponse<TargetClinicalData> {
        serde_json::from_value(value).expect("target clinical response")
    }

    fn decode_disease_prevalence(
        value: serde_json::Value,
    ) -> GraphQlResponse<DiseasePrevalenceData> {
        serde_json::from_value(value).expect("disease prevalence response")
    }

    #[test]
    fn drug_sections_plan_builds_graphql_request() {
        let plan = OpenTargetsClient::drug_sections_plan(" CHEMBL25 ").expect("plan");

        assert_eq!(plan.method, crate::sources::HttpMethod::Post);
        assert_eq!(plan.path, "graphql");
        let crate::sources::RequestBody::Json(body) = plan.body else {
            panic!("expected JSON body");
        };
        assert!(
            body["query"]
                .as_str()
                .expect("query")
                .contains("DrugSections")
        );
        assert_eq!(body["variables"]["chemblId"], "CHEMBL25");
    }

    #[test]
    fn drug_sections_maps_targets_and_indications() {
        let resp = decode_drug_sections(serde_json::json!({
            "data": {
                "drug": {
                    "indications": {
                        "rows": [
                            {
                                "maxClinicalStage": "APPROVAL",
                                "disease": {"name": "Melanoma"}
                            }
                        ]
                    },
                    "mechanismsOfAction": {
                        "rows": [
                            {
                                "targets": [
                                    {
                                        "approvedSymbol": "BRAF",
                                        "approvedName": "B-Raf proto-oncogene serine/threonine-protein kinase"
                                    }
                                ]
                            },
                            {
                                "targets": [
                                    {
                                        "approvedSymbol": "MAP2K1",
                                        "approvedName": "Dual specificity mitogen-activated protein kinase kinase 1"
                                    }
                                ]
                            }
                        ]
                    }
                }
            }
        }));
        let sections = drug_sections_from_response(resp, 5).unwrap();

        assert_eq!(sections.indications.len(), 1);
        assert_eq!(sections.indications[0].disease_name, "Melanoma");
        assert_eq!(
            sections.indications[0].max_clinical_stage.as_deref(),
            Some("APPROVAL")
        );
        assert_eq!(sections.targets.len(), 2);
        assert_eq!(sections.targets[0].approved_symbol, "BRAF");
        assert_eq!(
            sections.targets[0].approved_name.as_deref(),
            Some("B-Raf proto-oncogene serine/threonine-protein kinase")
        );
        assert_eq!(sections.targets[1].approved_symbol, "MAP2K1");
        assert_eq!(
            sections.targets[1].approved_name.as_deref(),
            Some("Dual specificity mitogen-activated protein kinase kinase 1")
        );
    }

    #[test]
    fn drug_sections_degrades_when_indications_missing() {
        let resp = decode_drug_sections(serde_json::json!({
            "data": {
                "drug": {
                    "mechanismsOfAction": {
                        "rows": [
                            {
                                "targets": [
                                    {"approvedSymbol": "BRAF"}
                                ]
                            }
                        ]
                    }
                }
            }
        }));
        let sections = drug_sections_from_response(resp, 5).unwrap();

        assert!(sections.indications.is_empty());
        assert_eq!(sections.targets.len(), 1);
    }

    #[test]
    fn search_disease_plan_builds_graphql_request() {
        let plan = OpenTargetsClient::search_disease_plan(" melanoma ").expect("plan");

        assert_eq!(plan.method, crate::sources::HttpMethod::Post);
        assert_eq!(plan.path, "graphql");
        let crate::sources::RequestBody::Json(body) = plan.body else {
            panic!("expected JSON body");
        };
        assert!(
            body["query"]
                .as_str()
                .expect("query")
                .contains("SearchDisease")
        );
        assert_eq!(body["variables"]["query"], "melanoma");
    }

    #[test]
    fn disease_genes_plan_builds_graphql_request() {
        let plan = OpenTargetsClient::disease_genes_plan(" EFO_0000311 ", 5).expect("plan");

        assert_eq!(plan.method, crate::sources::HttpMethod::Post);
        assert_eq!(plan.path, "graphql");
        let crate::sources::RequestBody::Json(body) = plan.body else {
            panic!("expected JSON body");
        };
        assert!(
            body["query"]
                .as_str()
                .expect("query")
                .contains("DiseaseGenes")
        );
        assert_eq!(body["variables"]["efoId"], "EFO_0000311");
        assert_eq!(body["variables"]["size"], 5);
    }

    #[test]
    fn disease_id_from_search_response_prefers_efo_hit() {
        let resp = decode_disease_search(serde_json::json!({
            "data": {
                "search": {
                    "hits": [
                        {
                            "id": "MONDO_0003864",
                            "name": "chronic lymphocytic leukemia/small lymphocytic lymphoma",
                            "entity": "disease"
                        },
                        {
                            "id": "EFO_0000095",
                            "name": "chronic lymphocytic leukemia",
                            "entity": "disease"
                        }
                    ]
                }
            }
        }));

        let id = disease_id_from_search_response(resp).expect("search response");

        assert_eq!(id.as_deref(), Some("EFO_0000095"));
    }

    #[test]
    fn disease_associated_targets_maps_scores() {
        let resp = decode_disease_genes(serde_json::json!({
            "data": {
                "disease": {
                    "associatedTargets": {
                        "rows": [
                            {
                                "score": 0.91,
                                "datatypeScores": [{"id": "somatic_mutation", "score": 0.67}],
                                "datasourceScores": [
                                    {"id": "gwas_credible_sets", "score": 0.42},
                                    {"id": "eva", "score": 0.88}
                                ],
                                "target": {"approvedSymbol": "BRAF"}
                            },
                            {
                                "score": 0.76,
                                "datatypeScores": [],
                                "datasourceScores": [],
                                "target": {"approvedSymbol": "KRAS"}
                            }
                        ]
                    }
                }
            }
        }));

        let genes = disease_associated_targets_from_response(resp, 5).unwrap();
        assert_eq!(genes.len(), 2);
        assert_eq!(genes[0].symbol, "BRAF");
        assert_eq!(genes[0].overall_score, Some(0.91));
        assert_eq!(genes[0].gwas_score, Some(0.42));
        assert_eq!(genes[0].rare_variant_score, Some(0.88));
        assert_eq!(genes[0].somatic_mutation_score, Some(0.67));
        assert_eq!(genes[1].symbol, "KRAS");
        assert_eq!(genes[1].overall_score, Some(0.76));
        assert_eq!(genes[1].gwas_score, None);
        assert_eq!(genes[1].rare_variant_score, None);
        assert_eq!(genes[1].somatic_mutation_score, None);
    }

    #[test]
    fn disease_associated_targets_maps_efo_lookup_result() {
        let resp = decode_disease_genes(serde_json::json!({
            "data": {
                "disease": {
                    "associatedTargets": {
                        "rows": [
                            {
                                "score": 0.99,
                                "datatypeScores": [],
                                "datasourceScores": [],
                                "target": {"approvedSymbol": "TP53"}
                            }
                        ]
                    }
                }
            }
        }));

        let genes = disease_associated_targets_from_response(resp, 5).unwrap();
        assert_eq!(genes.len(), 1);
        assert_eq!(genes[0].symbol, "TP53");
    }

    #[test]
    fn disease_associated_targets_degrades_when_associated_targets_missing() {
        let resp = decode_disease_genes(serde_json::json!({
            "data": {
                "disease": {}
            }
        }));

        let genes = disease_associated_targets_from_response(resp, 5).unwrap();
        assert!(genes.is_empty());
    }

    #[test]
    fn search_target_plan_builds_graphql_request() {
        let plan = OpenTargetsClient::search_target_plan(" EGFR ").expect("plan");

        assert_eq!(plan.method, crate::sources::HttpMethod::Post);
        assert_eq!(plan.path, "graphql");
        let crate::sources::RequestBody::Json(body) = plan.body else {
            panic!("expected JSON body");
        };
        assert!(
            body["query"]
                .as_str()
                .expect("query")
                .contains("SearchTarget")
        );
        assert_eq!(body["variables"]["query"], "EGFR");
    }

    #[test]
    fn target_druggability_context_plan_builds_graphql_request() {
        let plan =
            OpenTargetsClient::target_druggability_context_plan(" ENSG00000146648 ").expect("plan");

        assert_eq!(plan.method, crate::sources::HttpMethod::Post);
        assert_eq!(plan.path, "graphql");
        let crate::sources::RequestBody::Json(body) = plan.body else {
            panic!("expected JSON body");
        };
        assert!(
            body["query"]
                .as_str()
                .expect("query")
                .contains("TargetDruggabilityContext")
        );
        assert_eq!(body["variables"]["ensemblId"], "ENSG00000146648");
    }

    #[test]
    fn target_id_from_search_response_prefers_exact_symbol_match() {
        let resp = decode_target_search(serde_json::json!({
            "data": {
                "search": {
                    "hits": [
                        {"id": "ENSG00000000000", "entity": "target", "object": {"approvedSymbol": "ERBB1"}},
                        {"id": "ENSG00000146648", "entity": "target", "object": {"approvedSymbol": "EGFR"}}
                    ]
                }
            }
        }));

        let target_id = target_id_from_search_response(resp, "EGFR").expect("target search");

        assert_eq!(target_id.as_deref(), Some("ENSG00000146648"));
    }

    #[test]
    fn target_druggability_context_groups_modalities_and_safety_summary() {
        let resp = decode_target_druggability(serde_json::json!({
            "data": {
                "target": {
                    "tractability": [
                        {"label": "Approved Drug", "modality": "SM", "value": true},
                        {"label": "Clinical Precedence", "modality": "SM", "value": true},
                        {"label": "High-quality binder", "modality": "AB", "value": false},
                        {"label": "Clinical Precedence", "modality": "AB", "value": true},
                        {"label": "Discovery chemistry", "modality": "PR", "value": false},
                        {"label": "Ligand present", "modality": "OC", "value": true},
                        {"label": "Exploratory", "modality": "XX", "value": true}
                    ],
                    "safetyLiabilities": [
                        {
                            "event": "Skin rash",
                            "datasource": "ForceGenetics",
                            "effects": [{"direction": "activation", "dosing": "chronic"}],
                            "biosamples": [{"tissueLabel": "Skin", "cellLabel": null, "cellFormat": null}]
                        },
                        {
                            "event": "skin rash",
                            "datasource": "",
                            "effects": [{"direction": "", "dosing": null}],
                            "biosamples": [{"tissueLabel": null, "cellLabel": "Keratinocyte", "cellFormat": null}]
                        },
                        {
                            "event": "Cardiotoxicity",
                            "datasource": null,
                            "effects": [{"direction": "inhibition", "dosing": null}],
                            "biosamples": [{"tissueLabel": null, "cellLabel": null, "cellFormat": "iPSC cardiomyocyte"}]
                        }
                    ]
                }
            }
        }));

        let context = target_druggability_context_from_response(resp).unwrap();

        assert_eq!(context.tractability.len(), 5);
        assert_eq!(context.tractability[0].modality, "small molecule");
        assert!(context.tractability[0].tractable);
        assert_eq!(
            context.tractability[0].evidence_labels,
            vec!["Approved Drug", "Clinical Precedence"]
        );
        assert_eq!(context.tractability[1].modality, "antibody");
        assert!(context.tractability[1].tractable);
        assert_eq!(
            context.tractability[1].evidence_labels,
            vec!["Clinical Precedence"]
        );
        assert_eq!(context.tractability[2].modality, "PROTAC");
        assert!(!context.tractability[2].tractable);
        assert!(context.tractability[2].evidence_labels.is_empty());
        assert_eq!(context.tractability[3].modality, "other modality");
        assert!(context.tractability[3].tractable);
        assert_eq!(context.tractability[4].modality, "xx");
        assert!(context.tractability[4].tractable);

        assert_eq!(context.safety_liabilities.len(), 2);
        assert_eq!(context.safety_liabilities[0].event, "Skin rash");
        assert_eq!(
            context.safety_liabilities[0].datasource.as_deref(),
            Some("ForceGenetics")
        );
        assert_eq!(
            context.safety_liabilities[0].effect_direction.as_deref(),
            Some("activation")
        );
        assert_eq!(
            context.safety_liabilities[0].biosample.as_deref(),
            Some("Skin")
        );
        assert_eq!(context.safety_liabilities[1].event, "Cardiotoxicity");
        assert_eq!(
            context.safety_liabilities[1].biosample.as_deref(),
            Some("iPSC cardiomyocyte")
        );
    }

    #[test]
    fn target_druggability_context_returns_default_when_target_missing() {
        let resp = decode_target_druggability(serde_json::json!({
            "data": {
                "target": null
            }
        }));

        let context = target_druggability_context_from_response(resp).unwrap();
        assert!(context.tractability.is_empty());
        assert!(context.safety_liabilities.is_empty());
    }

    #[test]
    fn disease_prevalence_plan_builds_graphql_request() {
        let plan = OpenTargetsClient::disease_prevalence_plan(" MONDO_0007947 ", 5).expect("plan");

        assert_eq!(plan.method, crate::sources::HttpMethod::Post);
        assert_eq!(plan.path, "graphql");
        let crate::sources::RequestBody::Json(body) = plan.body else {
            panic!("expected JSON body");
        };
        assert!(
            body["query"]
                .as_str()
                .expect("query")
                .contains("DiseasePrevalence")
        );
        assert_eq!(body["variables"]["efoId"], "MONDO_0007947");
        assert_eq!(body["variables"]["size"], 5);
    }

    #[test]
    fn disease_prevalence_maps_frequency_evidence() {
        let resp = decode_disease_prevalence(serde_json::json!({
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
        }));

        let rows = disease_prevalence_from_response(resp, 5).expect("prevalence rows");
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

    #[test]
    fn target_clinical_context_plan_builds_graphql_request() {
        let plan =
            OpenTargetsClient::target_clinical_context_plan(" ENSG00000157764 ", 5).expect("plan");

        assert_eq!(plan.method, crate::sources::HttpMethod::Post);
        assert_eq!(plan.path, "graphql");
        let crate::sources::RequestBody::Json(body) = plan.body else {
            panic!("expected JSON body");
        };
        assert!(
            body["query"]
                .as_str()
                .expect("query")
                .contains("TargetClinicalContext")
        );
        assert_eq!(body["variables"]["ensemblId"], "ENSG00000157764");
        assert_eq!(body["variables"]["size"], 5);
    }

    #[test]
    fn target_clinical_context_collects_diseases_and_drugs() {
        let resp = decode_target_clinical(serde_json::json!({
            "data": {
                "target": {
                    "associatedDiseases": {
                        "rows": [
                            {"score": 0.8, "disease": {"id": "EFO_1", "name": "Melanoma"}},
                            {"score": 0.7, "disease": {"id": "EFO_2", "name": "Colorectal cancer"}}
                        ]
                    },
                    "drugAndClinicalCandidates": {
                        "rows": [
                            {"drug": {"id": "CHEMBL1", "name": "Dabrafenib"}},
                            {"drug": {"id": "CHEMBL2", "name": "Vemurafenib"}}
                        ]
                    }
                }
            }
        }));

        let context = target_clinical_context_from_response(resp, 5).unwrap();
        assert_eq!(context.diseases, vec!["Melanoma", "Colorectal cancer"]);
        assert_eq!(context.drugs, vec!["Dabrafenib", "Vemurafenib"]);
    }

    #[test]
    fn target_clinical_context_degrades_when_drug_candidates_missing() {
        let resp = decode_target_clinical(serde_json::json!({
            "data": {
                "target": {
                    "associatedDiseases": {
                        "rows": [
                            {"score": 0.8, "disease": {"id": "EFO_1", "name": "Melanoma"}}
                        ]
                    }
                }
            }
        }));

        let context = target_clinical_context_from_response(resp, 5).unwrap();
        assert_eq!(context.diseases, vec!["Melanoma"]);
        assert!(context.drugs.is_empty());
    }

    #[test]
    fn drug_sections_propagates_graphql_error_message() {
        let resp = decode_drug_sections(serde_json::json!({
            "errors": [
                {"message": "Cannot query field linkedTargets on type Drug"}
            ]
        }));

        let err = drug_sections_from_response(resp, 5).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("linkedTargets"));
        assert!(msg.contains("opentargets"));
    }

    #[test]
    fn disease_associated_targets_egfr_lung() {
        let resp = decode_disease_genes(serde_json::json!({
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
        }));

        let genes = disease_associated_targets_from_response(resp, 3).unwrap();
        assert_eq!(genes.first().map(|g| g.symbol.as_str()), Some("EGFR"));
    }

    #[test]
    fn drug_sections_maps_osimertinib() {
        let resp = decode_drug_sections(serde_json::json!({
            "data": {
                "drug": {
                    "indications": {
                        "rows": [
                            {
                                "maxClinicalStage": "APPROVAL",
                                "disease": {"name": "Non-small cell lung cancer"}
                            }
                        ]
                    },
                    "mechanismsOfAction": {
                        "rows": [
                            {
                                "targets": [
                                    {"approvedSymbol": "EGFR"}
                                ]
                            }
                        ]
                    }
                }
            }
        }));

        let sections = drug_sections_from_response(resp, 5).unwrap();
        assert_eq!(
            sections
                .indications
                .first()
                .map(|i| i.disease_name.as_str()),
            Some("Non-small cell lung cancer")
        );
        assert_eq!(
            sections
                .indications
                .first()
                .and_then(|i| i.max_clinical_stage.as_deref()),
            Some("APPROVAL")
        );
        assert_eq!(
            sections.targets.first().map(|t| t.approved_symbol.as_str()),
            Some("EGFR")
        );
    }
}
