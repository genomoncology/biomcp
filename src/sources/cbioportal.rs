use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;

const CBIOPORTAL_BASE: &str = "https://www.cbioportal.org/api";
const CBIOPORTAL_API: &str = "cbioportal";
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

    #[cfg(test)]
    fn new_for_test(base: String) -> Result<Self, BioMcpError> {
        let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new()).build();
        Ok(Self {
            client,
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

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req).send().await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_body(resp, CBIOPORTAL_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: CBIOPORTAL_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: CBIOPORTAL_API.to_string(),
            source,
        })
    }

    async fn post_json<T: serde::de::DeserializeOwned, B: Serialize>(
        &self,
        req: reqwest_middleware::RequestBuilder,
        body: &B,
    ) -> Result<T, BioMcpError> {
        self.get_json(req.json(body)).await
    }

    async fn resolve_entrez_gene_id(&self, gene: &str) -> Result<i32, BioMcpError> {
        let gene = gene.trim();
        if gene.is_empty() {
            return Err(BioMcpError::InvalidArgument("Gene is required".into()));
        }

        let url = self.endpoint("genes");
        let resp: Vec<CBioGene> = self
            .get_json(self.client.get(&url).query(&[
                ("keyword", gene),
                ("pageSize", "1"),
                ("pageNumber", "0"),
            ]))
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

    async fn get_study(&self, study_id: &str) -> Result<CBioStudy, BioMcpError> {
        let url = self.endpoint(&format!("studies/{study_id}"));
        self.get_json(self.client.get(&url)).await
    }

    async fn mutated_samples_in_profile(
        &self,
        molecular_profile_id: &str,
        sample_list_id: &str,
        entrez_gene_id: i32,
        max_unique_samples: usize,
    ) -> Result<HashSet<String>, BioMcpError> {
        let url = self.endpoint(&format!(
            "molecular-profiles/{molecular_profile_id}/mutations"
        ));

        let mut out: HashSet<String> = HashSet::new();
        let page_size: i32 = 500;

        for page_number in 0..30_i32 {
            let entrez = entrez_gene_id.to_string();
            let page_size_s = page_size.to_string();
            let page_number_s = page_number.to_string();
            let resp: Vec<CBioMutation> = self
                .get_json(self.client.get(&url).query(&[
                    ("sampleListId", sample_list_id),
                    ("entrezGeneId", entrez.as_str()),
                    ("pageSize", page_size_s.as_str()),
                    ("pageNumber", page_number_s.as_str()),
                ]))
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

    async fn cancer_type_distribution(
        &self,
        study_id: &str,
        sample_ids: &[String],
    ) -> Result<HashMap<String, usize>, BioMcpError> {
        let url = self.endpoint(&format!("studies/{study_id}/clinical-data/fetch"));
        let mut counts: HashMap<String, usize> = HashMap::new();

        // Avoid sending extremely large request bodies.
        for chunk in sample_ids.chunks(500) {
            let body = CBioClinicalDataSingleStudyFilter {
                attribute_ids: vec!["CANCER_TYPE_DETAILED".to_string()],
                ids: chunk.to_vec(),
            };

            let resp: Vec<CBioClinicalData> = self
                .post_json(
                    self.client
                        .post(&url)
                        .query(&[("clinicalDataType", "SAMPLE")]),
                    &body,
                )
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
        dist.sort_by(|a, b| b.sample_count.cmp(&a.sample_count));
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CBioClinicalDataSingleStudyFilter {
    attribute_ids: Vec<String>,
    ids: Vec<String>,
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
    #[allow(dead_code)]
    pub total_mutations: Option<i32>,
    #[allow(dead_code)]
    pub mutation_frequency: Option<f64>,
    pub cancer_distribution: Vec<CancerFrequency>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn gene_resolution_uses_keyword() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/genes"))
            .and(query_param("keyword", "BRAF"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"entrezGeneId": 673, "hugoGeneSymbol": "BRAF"}
            ])))
            .mount(&server)
            .await;

        let client = CBioPortalClient::new_for_test(server.uri()).unwrap();
        let id = client.resolve_entrez_gene_id("BRAF").await.unwrap();
        assert_eq!(id, 673);
    }

    #[tokio::test]
    async fn gene_resolution_surfaces_http_error_context() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/genes"))
            .respond_with(ResponseTemplate::new(500).set_body_string("upstream failure"))
            .mount(&server)
            .await;

        let client = CBioPortalClient::new_for_test(server.uri()).unwrap();
        let err = client.resolve_entrez_gene_id("BRAF").await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("cbioportal"));
        assert!(msg.contains("500"));
    }
}
