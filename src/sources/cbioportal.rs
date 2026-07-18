use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::BioMcpError;
use crate::sources::{RequestBody, RequestPlan, request_from_plan};

const CBIOPORTAL_BASE: &str = "https://www.cbioportal.org/api";
const CBIOPORTAL_BASE_ENV: &str = "BIOMCP_CBIOPORTAL_BASE";

// Pragmatic default: pan-cancer cohort with a public mutation profile.
const DEFAULT_STUDY_ID: &str = "msk_impact_2017";
const DEFAULT_SAMPLE_LIST_ID: &str = "msk_impact_2017_all";
const DEFAULT_MUTATION_PROFILE_ID: &str = "msk_impact_2017_mutations";

fn configured_study_id() -> String {
    std::env::var("BIOMCP_CBIOPORTAL_STUDY")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_STUDY_ID.to_string())
}

fn configured_sample_list_id() -> String {
    std::env::var("BIOMCP_CBIOPORTAL_SAMPLE_LIST")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_SAMPLE_LIST_ID.to_string())
}

fn configured_mutation_profile_id() -> String {
    std::env::var("BIOMCP_CBIOPORTAL_MUTATION_PROFILE")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_MUTATION_PROFILE_ID.to_string())
}

pub struct CBioPortalClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl CBioPortalClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(CBIOPORTAL_BASE, CBIOPORTAL_BASE_ENV),
        })
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::CBIOPORTAL,
            ))
            .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::CBIOPORTAL),
        )
        .await?;
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::CBIOPORTAL),
            status,
            None,
            &bytes,
            false,
        )
    }

    pub(crate) fn gene_resolution_plan(gene: &str) -> Result<RequestPlan, BioMcpError> {
        let gene = gene.trim();
        if gene.is_empty() {
            return Err(BioMcpError::InvalidArgument("Gene is required".into()));
        }
        Ok(RequestPlan::get("genes")
            .query("keyword", gene)
            .query("pageSize", "1")
            .query("pageNumber", "0"))
    }

    async fn resolve_entrez_gene_id(&self, gene: &str) -> Result<i32, BioMcpError> {
        let plan = Self::gene_resolution_plan(gene)?;
        let resp: Vec<CBioGene> = self
            .get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await?;

        resp.into_iter()
            .next()
            .map(|g| g.entrez_gene_id)
            .ok_or_else(|| BioMcpError::NotFound {
                entity: "gene".into(),
                id: gene.to_string(),
                suggestion: format!("Try searching: biomcp search gene -q {gene}"),
            })
    }

    pub(crate) fn study_plan(study_id: &str) -> RequestPlan {
        RequestPlan::get(format!("studies/{study_id}"))
    }

    async fn get_study(&self, study_id: &str) -> Result<CBioStudy, BioMcpError> {
        let plan = Self::study_plan(study_id);
        self.get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await
    }

    pub(crate) fn mutations_plan(
        molecular_profile_id: &str,
        sample_list_id: &str,
        entrez_gene_id: i32,
        page_size: i32,
        page_number: i32,
    ) -> RequestPlan {
        RequestPlan::get(format!(
            "molecular-profiles/{molecular_profile_id}/mutations"
        ))
        .query("sampleListId", sample_list_id)
        .query("entrezGeneId", entrez_gene_id.to_string())
        .query("pageSize", page_size.to_string())
        .query("pageNumber", page_number.to_string())
    }

    async fn mutated_samples_in_profile(
        &self,
        molecular_profile_id: &str,
        sample_list_id: &str,
        entrez_gene_id: i32,
        max_unique_samples: usize,
    ) -> Result<HashSet<String>, BioMcpError> {
        let mut out: HashSet<String> = HashSet::new();
        let page_size: i32 = 500;

        for page_number in 0..30_i32 {
            let plan = Self::mutations_plan(
                molecular_profile_id,
                sample_list_id,
                entrez_gene_id,
                page_size,
                page_number,
            );
            let resp: Vec<CBioMutation> = self
                .get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
                .await?;

            if resp.is_empty() {
                break;
            }
            let resp_len = resp.len();
            for m in resp {
                if let Some(sample_id) = m.sample_id
                    && !sample_id.trim().is_empty()
                {
                    out.insert(sample_id);
                    if out.len() >= max_unique_samples {
                        return Ok(out);
                    }
                }
            }

            if resp_len < page_size as usize {
                break;
            }
        }

        Ok(out)
    }

    pub(crate) fn clinical_data_plan(study_id: &str, sample_ids: &[String]) -> RequestPlan {
        let mut plan = RequestPlan::post(format!("studies/{study_id}/clinical-data/fetch"))
            .query("clinicalDataType", "SAMPLE");
        plan.body = RequestBody::Json(json!({
            "attributeIds": ["CANCER_TYPE_DETAILED"],
            "ids": sample_ids,
        }));
        plan
    }

    async fn cancer_type_distribution(
        &self,
        study_id: &str,
        sample_ids: &[String],
    ) -> Result<HashMap<String, usize>, BioMcpError> {
        let mut counts: HashMap<String, usize> = HashMap::new();

        // Avoid sending extremely large request bodies.
        for chunk in sample_ids.chunks(500) {
            let plan = Self::clinical_data_plan(study_id, chunk);
            let resp: Vec<CBioClinicalData> = self
                .get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
                .await?;

            for row in resp {
                let Some(sample_id) = row.sample_id else {
                    continue;
                };
                if sample_id.trim().is_empty() {
                    continue;
                }
                let Some(value) = row.value else { continue };
                let v = value.trim();
                if v.is_empty() {
                    continue;
                }
                *counts.entry(v.to_string()).or_insert(0) += 1;
            }
        }

        Ok(counts)
    }

    pub async fn get_mutation_summary(
        &self,
        gene: &str,
    ) -> Result<CBioMutationSummary, BioMcpError> {
        let study_id = configured_study_id();
        let sample_list_id = configured_sample_list_id();
        let mutation_profile_id = configured_mutation_profile_id();
        let entrez = self.resolve_entrez_gene_id(gene).await?;
        let study = self.get_study(&study_id).await?;

        let sample_ids = self
            .mutated_samples_in_profile(&mutation_profile_id, &sample_list_id, entrez, 2000)
            .await?;

        if sample_ids.is_empty() {
            return Ok(CBioMutationSummary {
                study_id,
                sample_list_id,
                mutation_profile_id,
                total_mutations: Some(0),
                mutation_frequency: Some(0.0),
                cancer_distribution: vec![],
            });
        }

        let mut sample_ids_vec = sample_ids.into_iter().collect::<Vec<_>>();
        sample_ids_vec.sort();

        let by_type = self
            .cancer_type_distribution(&study_id, &sample_ids_vec)
            .await?;

        let total_samples = sample_ids_vec.len();
        let sequenced = study.sequenced_sample_count.unwrap_or(0);
        let mutation_frequency = if sequenced > 0 {
            Some(total_samples as f64 / sequenced as f64)
        } else {
            None
        };

        let mut dist = by_type
            .into_iter()
            .map(|(cancer_type, sample_count)| CancerFrequency {
                cancer_type,
                frequency: sample_count as f64 / total_samples as f64,
                sample_count: sample_count as i32,
            })
            .collect::<Vec<_>>();
        dist.sort_by_key(|row| std::cmp::Reverse(row.sample_count));
        dist.truncate(5);

        Ok(CBioMutationSummary {
            study_id,
            sample_list_id,
            mutation_profile_id,
            total_mutations: Some(total_samples as i32),
            mutation_frequency,
            cancer_distribution: dist,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CBioGene {
    entrez_gene_id: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CBioStudy {
    sequenced_sample_count: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CBioMutation {
    sample_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CBioClinicalData {
    sample_id: Option<String>,
    value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancerFrequency {
    pub cancer_type: String,
    pub frequency: f64,
    pub sample_count: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CBioMutationSummary {
    pub study_id: String,
    pub sample_list_id: String,
    pub mutation_profile_id: String,
    // dead-code reason: cbioportal::total_mutations preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub total_mutations: Option<i32>,
    // dead-code reason: cbioportal::mutation_frequency preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub mutation_frequency: Option<f64>,
    pub cancer_distribution: Vec<CancerFrequency>,
}

#[cfg(test)]
mod tests;
