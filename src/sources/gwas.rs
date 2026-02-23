use std::borrow::Cow;

use serde::Deserialize;
use serde::Deserializer;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;

const GWAS_BASE: &str = "https://www.ebi.ac.uk/gwas/rest/api";
const GWAS_API: &str = "gwas";
const GWAS_BASE_ENV: &str = "BIOMCP_GWAS_BASE";

pub struct GwasClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl GwasClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(GWAS_BASE, GWAS_BASE_ENV),
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

    async fn get_json_optional<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<Option<T>, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req).send().await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, GWAS_API).await?;

        if status == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: GWAS_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        crate::sources::ensure_json_content_type(GWAS_API, content_type.as_ref(), &bytes)?;

        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|source| BioMcpError::ApiJson {
                api: GWAS_API.to_string(),
                source,
            })
    }

    pub async fn associations_by_rsid(
        &self,
        rsid: &str,
        limit: usize,
    ) -> Result<Vec<GwasAssociation>, BioMcpError> {
        let rsid = normalize_rsid(rsid)?;
        let limit = limit.clamp(1, 200);
        let url = self.endpoint(&format!(
            "singleNucleotidePolymorphisms/{rsid}/associations"
        ));

        let req = self.client.get(&url).query(&[
            ("projection", "associationByStudy"),
            ("page", "0"),
            ("size", &limit.to_string()),
        ]);

        let Some(resp): Option<GwasAssociationsResponse> = self.get_json_optional(req).await?
        else {
            return Ok(Vec::new());
        };

        Ok(resp.embedded.associations)
    }

    pub async fn snps_by_gene(
        &self,
        gene_symbol: &str,
        limit: usize,
    ) -> Result<Vec<GwasSnp>, BioMcpError> {
        let gene_symbol = normalize_gene_symbol(gene_symbol)?;
        let limit = limit.clamp(1, 200);
        let url = self.endpoint("singleNucleotidePolymorphisms/search/findByGene");

        let req = self.client.get(&url).query(&[
            ("geneName", gene_symbol.as_str()),
            ("page", "0"),
            ("size", &limit.to_string()),
        ]);

        let Some(resp): Option<GwasSnpsResponse> = self.get_json_optional(req).await? else {
            return Ok(Vec::new());
        };

        Ok(resp.embedded.snps)
    }

    pub async fn snps_by_trait(
        &self,
        trait_query: &str,
        limit: usize,
    ) -> Result<Vec<GwasSnp>, BioMcpError> {
        let trait_query = normalize_trait_query(trait_query)?;
        let limit = limit.clamp(1, 200);
        let url = self.endpoint("singleNucleotidePolymorphisms/search/findByDiseaseTrait");

        let req = self.client.get(&url).query(&[
            ("diseaseTrait", trait_query.as_str()),
            ("page", "0"),
            ("size", &limit.to_string()),
        ]);

        let Some(resp): Option<GwasSnpsResponse> = self.get_json_optional(req).await? else {
            return Ok(Vec::new());
        };

        Ok(resp.embedded.snps)
    }

    pub async fn studies_by_trait(
        &self,
        trait_query: &str,
        limit: usize,
    ) -> Result<Vec<GwasStudy>, BioMcpError> {
        let trait_query = normalize_trait_query(trait_query)?;
        let limit = limit.clamp(1, 200);
        let url = self.endpoint("studies/search/findByDiseaseTrait");

        let req = self.client.get(&url).query(&[
            ("diseaseTrait", trait_query.as_str()),
            ("page", "0"),
            ("size", &limit.to_string()),
        ]);

        let Some(resp): Option<GwasStudiesResponse> = self.get_json_optional(req).await? else {
            return Ok(Vec::new());
        };

        Ok(resp.embedded.studies)
    }

    pub async fn associations_by_study(
        &self,
        study_accession: &str,
        limit: usize,
    ) -> Result<Vec<GwasAssociation>, BioMcpError> {
        let study_accession = normalize_study_accession(study_accession)?;
        let limit = limit.clamp(1, 200);

        let search_url = self.endpoint("associations/search/findByStudyAccessionId");
        let search_req = self.client.get(&search_url).query(&[
            ("studyAccessionId", study_accession.as_str()),
            ("page", "0"),
            ("size", &limit.to_string()),
            ("projection", "associationByStudy"),
        ]);

        if let Some(search_resp) = self
            .get_json_optional::<GwasAssociationsResponse>(search_req)
            .await?
            && !search_resp.embedded.associations.is_empty()
        {
            return Ok(search_resp.embedded.associations);
        }

        let fallback_url = self.endpoint(&format!("studies/{study_accession}/associations"));
        let fallback_req = self.client.get(&fallback_url).query(&[
            ("projection", "associationByStudy"),
            ("page", "0"),
            ("size", &limit.to_string()),
        ]);

        let Some(fallback_resp): Option<GwasAssociationsResponse> =
            self.get_json_optional(fallback_req).await?
        else {
            return Ok(Vec::new());
        };

        Ok(fallback_resp.embedded.associations)
    }
}

fn normalize_rsid(value: &str) -> Result<String, BioMcpError> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() || !normalized.starts_with("rs") {
        return Err(BioMcpError::InvalidArgument(
            "GWAS lookup requires an rsID (e.g., rs7903146).".into(),
        ));
    }
    if !normalized.chars().skip(2).all(|c| c.is_ascii_digit()) {
        return Err(BioMcpError::InvalidArgument(format!(
            "Invalid rsID: {value}"
        )));
    }
    Ok(normalized)
}

fn normalize_gene_symbol(value: &str) -> Result<String, BioMcpError> {
    let normalized = value.trim().to_ascii_uppercase();
    if normalized.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Gene symbol is required. Example: biomcp search gwas -g TCF7L2".into(),
        ));
    }
    if !crate::sources::is_valid_gene_symbol(&normalized) {
        return Err(BioMcpError::InvalidArgument(format!(
            "Invalid gene symbol: {value}"
        )));
    }
    Ok(normalized)
}

fn normalize_trait_query(value: &str) -> Result<String, BioMcpError> {
    let normalized = value.trim().to_string();
    if normalized.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Trait query is required. Example: biomcp search gwas --trait \"type 2 diabetes\""
                .into(),
        ));
    }
    if normalized.len() > 256 {
        return Err(BioMcpError::InvalidArgument(
            "Trait query is too long.".into(),
        ));
    }
    Ok(normalized)
}

fn normalize_study_accession(value: &str) -> Result<String, BioMcpError> {
    let normalized = value.trim().to_ascii_uppercase();
    if normalized.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Study accession is required (e.g., GCST000796).".into(),
        ));
    }
    if !normalized.starts_with("GCST") {
        return Err(BioMcpError::InvalidArgument(format!(
            "Invalid study accession: {value}"
        )));
    }
    Ok(normalized)
}

fn de_opt_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumberLike {
        Float(f64),
        Integer(i64),
        String(String),
    }

    let value = Option::<NumberLike>::deserialize(deserializer)?;
    Ok(match value {
        Some(NumberLike::Float(v)) => Some(v),
        Some(NumberLike::Integer(v)) => Some(v as f64),
        Some(NumberLike::String(v)) => v.trim().parse::<f64>().ok(),
        None => None,
    })
}

#[derive(Debug, Clone, Deserialize, Default)]
struct GwasAssociationsResponse {
    #[serde(default, rename = "_embedded")]
    embedded: GwasAssociationsEmbedded,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct GwasAssociationsEmbedded {
    #[serde(default)]
    associations: Vec<GwasAssociation>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct GwasSnpsResponse {
    #[serde(default, rename = "_embedded")]
    embedded: GwasSnpsEmbedded,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct GwasSnpsEmbedded {
    #[serde(default, rename = "singleNucleotidePolymorphisms")]
    snps: Vec<GwasSnp>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct GwasStudiesResponse {
    #[serde(default, rename = "_embedded")]
    embedded: GwasStudiesEmbedded,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct GwasStudiesEmbedded {
    #[serde(default)]
    studies: Vec<GwasStudy>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasAssociation {
    #[serde(default)]
    pub snps: Vec<GwasSnp>,
    #[serde(default)]
    pub loci: Vec<GwasLocus>,
    #[serde(default, rename = "efoTraits")]
    pub efo_traits: Vec<GwasTrait>,
    #[serde(default)]
    pub study: Option<GwasStudy>,
    #[serde(default, deserialize_with = "de_opt_f64")]
    pub pvalue: Option<f64>,
    #[serde(default, rename = "orPerCopyNum", deserialize_with = "de_opt_f64")]
    pub or_per_copy_num: Option<f64>,
    #[serde(default, rename = "betaNum", deserialize_with = "de_opt_f64")]
    pub beta_num: Option<f64>,
    #[serde(default)]
    pub range: Option<String>,
    #[serde(default, rename = "riskFrequency", deserialize_with = "de_opt_f64")]
    pub risk_frequency: Option<f64>,
    #[serde(default)]
    #[allow(dead_code)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasSnp {
    #[serde(default, rename = "rsId")]
    pub rs_id: Option<String>,
    #[serde(default, rename = "genomicContexts")]
    #[allow(dead_code)]
    pub genomic_contexts: Vec<GwasGenomicContext>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasGenomicContext {
    #[serde(default)]
    #[allow(dead_code)]
    pub gene: Option<GwasGene>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasLocus {
    #[serde(default, rename = "strongestRiskAlleles")]
    pub strongest_risk_alleles: Vec<GwasRiskAllele>,
    #[serde(default, rename = "authorReportedGenes")]
    pub author_reported_genes: Vec<GwasGene>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasRiskAllele {
    #[serde(default, rename = "riskAlleleName")]
    pub risk_allele_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasGene {
    #[serde(default, rename = "geneName")]
    pub gene_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasTrait {
    #[serde(default, rename = "trait")]
    pub trait_field: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasStudy {
    #[serde(default, rename = "accessionId")]
    pub accession_id: Option<String>,
    #[serde(default, rename = "diseaseTrait")]
    pub disease_trait: Option<GwasDiseaseTrait>,
    #[serde(default, rename = "initialSampleSize")]
    pub initial_sample_size: Option<String>,
    #[serde(default, rename = "replicationSampleSize")]
    pub replication_sample_size: Option<String>,
    #[serde(default, rename = "publicationInfo")]
    pub publication_info: Option<GwasPublicationInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasDiseaseTrait {
    #[serde(default, rename = "trait")]
    pub trait_field: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasPublicationInfo {
    #[serde(default, rename = "pubmedId")]
    pub pubmed_id: Option<String>,
    #[serde(default)]
    pub author: Option<GwasAuthor>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasAuthor {
    #[serde(default, rename = "fullname")]
    pub fullname: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn de_opt_f64_accepts_string_numbers() {
        #[derive(Deserialize)]
        struct Wrapper {
            #[serde(deserialize_with = "de_opt_f64")]
            value: Option<f64>,
        }

        let parsed: Wrapper = serde_json::from_str("{\"value\":\"8e-12\"}").expect("parse");
        assert_eq!(parsed.value, Some(8e-12));
    }

    #[tokio::test]
    async fn associations_by_rsid_parses_rows() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(
                "/singleNucleotidePolymorphisms/rs7903146/associations",
            ))
            .and(query_param("projection", "associationByStudy"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "_embedded": {
                    "associations": [
                        {
                            "pvalue": "8e-12",
                            "orPerCopyNum": 1.54,
                            "riskFrequency": "0.04",
                            "loci": [
                                {
                                    "strongestRiskAlleles": [
                                        {"riskAlleleName": "rs7903146-T"}
                                    ],
                                    "authorReportedGenes": [
                                        {"geneName": "TCF7L2"}
                                    ]
                                }
                            ]
                        }
                    ]
                }
            })))
            .mount(&server)
            .await;

        let client = GwasClient::new_for_test(server.uri()).expect("client");
        let rows = client
            .associations_by_rsid("rs7903146", 5)
            .await
            .expect("associations");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].pvalue, Some(8e-12));
        assert_eq!(rows[0].or_per_copy_num, Some(1.54));
        assert_eq!(rows[0].risk_frequency, Some(0.04));
    }

    #[tokio::test]
    async fn associations_by_study_falls_back_when_search_is_empty() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/associations/search/findByStudyAccessionId"))
            .and(query_param("studyAccessionId", "GCST000796"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "_embedded": {"associations": []}
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/studies/GCST000796/associations"))
            .and(query_param("projection", "associationByStudy"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "_embedded": {
                    "associations": [
                        {
                            "pvalue": 1.0e-8
                        }
                    ]
                }
            })))
            .mount(&server)
            .await;

        let client = GwasClient::new_for_test(server.uri()).expect("client");
        let rows = client
            .associations_by_study("GCST000796", 5)
            .await
            .expect("associations");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].pvalue, Some(1.0e-8));
    }
}
