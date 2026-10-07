use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use http_cache_reqwest::CacheMode;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};
use crate::xml::{ARTICLE_XML_NODE_LIMIT, parse_external_xml};

const NCBI_EFETCH_BASE: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils";
const NCBI_EFETCH_API: &str = "pubmed-eutils";
const NCBI_EFETCH_BASE_ENV: &str = "BIOMCP_PUBMED_BASE";

#[derive(Clone)]
pub struct NcbiEfetchClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    api_key: Option<String>,
}

impl NcbiEfetchClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(NCBI_EFETCH_BASE, NCBI_EFETCH_BASE_ENV),
            api_key: crate::sources::ncbi_api_key(),
        })
    }

    pub(crate) fn normalize_pmcid(pmcid: &str) -> Result<Option<String>, BioMcpError> {
        let pmcid = pmcid.trim();
        if pmcid.is_empty() {
            return Ok(None);
        }
        if pmcid.len() > 64 {
            return Err(BioMcpError::InvalidArgument("PMCID is too long.".into()));
        }

        let numeric = if pmcid
            .get(..3)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("PMC"))
        {
            &pmcid[3..]
        } else {
            pmcid
        };

        if numeric.is_empty() || !numeric.chars().all(|ch| ch.is_ascii_digit()) {
            return Err(BioMcpError::InvalidArgument(
                "PMCID must start with PMC and contain only digits after.".into(),
            ));
        }
        if numeric.len() > 32 {
            return Err(BioMcpError::InvalidArgument("PMCID is too long.".into()));
        }

        Ok(Some(numeric.to_string()))
    }

    pub(crate) fn full_text_xml_plan(
        pmcid: &str,
        api_key: Option<&str>,
    ) -> Result<Option<RequestPlan>, BioMcpError> {
        let Some(numeric_pmcid) = Self::normalize_pmcid(pmcid)? else {
            return Ok(None);
        };

        let mut plan = RequestPlan::get("efetch.fcgi")
            .query("db", "pmc")
            .query("id", numeric_pmcid)
            .query("rettype", "xml");
        if let Some(key) = api_key.map(str::trim).filter(|value| !value.is_empty()) {
            plan = plan.query("api_key", key);
        }
        Ok(Some(plan))
    }

    pub(crate) fn decode_text(
        status: reqwest::StatusCode,
        bytes: &[u8],
    ) -> Result<String, BioMcpError> {
        if matches!(
            status,
            reqwest::StatusCode::NOT_FOUND | reqwest::StatusCode::NO_CONTENT
        ) {
            return Ok(String::new());
        }
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(bytes);
            return Err(BioMcpError::Api {
                api: NCBI_EFETCH_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        std::str::from_utf8(bytes)
            .map(str::to_string)
            .map_err(|_| BioMcpError::Api {
                api: NCBI_EFETCH_API.to_string(),
                message: "Full text XML response was not valid UTF-8".to_string(),
            })
    }

    async fn get_text(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<String, BioMcpError> {
        let resp = req
            .with_extension(CacheMode::NoStore)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::NCBI_EFETCH,
            ))
            .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::NCBI_EFETCH),
        )
        .await?;
        Self::decode_text(status, &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::NCBI_EFETCH,
            ))
        })
    }

    pub async fn get_full_text_xml(&self, pmcid: &str) -> Result<Option<String>, BioMcpError> {
        let Some(plan) = Self::full_text_xml_plan(pmcid, self.api_key.as_deref())? else {
            return Ok(None);
        };

        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let xml = self.get_text(req).await?;
        normalize_article_xml(&xml)
    }
}

pub(crate) fn normalize_article_xml(xml: &str) -> Result<Option<String>, BioMcpError> {
    let trimmed = xml.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let doc = match parse_external_xml(trimmed, ARTICLE_XML_NODE_LIMIT) {
        Ok(doc) => doc,
        Err(_) => return Ok(Some(trimmed.to_string())),
    };

    let article = doc
        .descendants()
        .find(|node| node.is_element() && node.has_tag_name("article"));
    let Some(article) = article else {
        return Ok(Some(trimmed.to_string()));
    };

    Ok(Some(trimmed[article.range()].to_string()))
}

#[cfg(test)]
mod tests;

pub(crate) mod clinvar {
    use std::borrow::Cow;

    use http_cache_reqwest::CacheMode;
    use reqwest::header::HeaderValue;

    use crate::error::{BioMcpError, SourceContext, SourceProvider};
    use crate::sources::{RequestBuilderSourceContextExt, RequestPlan, request_from_plan};
    use biodata::{ClinVarRecordProjection, NcbiClinVarVcv};

    const CLINVAR_BASE: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils";
    const CLINVAR_BASE_ENV: &str = "BIOMCP_CLINVAR_BASE";
    #[derive(Clone)]
    pub(crate) struct ClinvarClient {
        client: reqwest_middleware::ClientWithMiddleware,
        base: Cow<'static, str>,
        api_key: Option<String>,
    }

    impl ClinvarClient {
        pub(crate) fn new() -> Result<Self, BioMcpError> {
            Ok(Self {
                client: crate::sources::shared_client()?,
                base: crate::sources::env_base(CLINVAR_BASE, CLINVAR_BASE_ENV),
                api_key: crate::sources::ncbi_api_key(),
            })
        }

        pub(crate) fn variation_plan(id: u64, api_key: Option<&str>) -> RequestPlan {
            let mut plan = RequestPlan::get("efetch.fcgi")
                .query("db", "clinvar")
                .query("rettype", "vcv")
                .query("is_variationid", "true")
                .query("id", id.to_string());
            if let Some(key) = api_key.map(str::trim).filter(|key| !key.is_empty()) {
                plan = plan.query("api_key", key);
            }
            plan
        }

        pub(crate) async fn variation(
            &self,
            id: u64,
        ) -> Result<Option<ClinVarRecordProjection>, BioMcpError> {
            let plan = Self::variation_plan(id, self.api_key.as_deref());
            let response = request_from_plan(&self.client, self.base.as_ref(), &plan)
                .with_extension(CacheMode::NoStore)
                .send_with_source_context(SourceContext::retry(SourceProvider::NCBI_EFETCH))
                .await?;
            let status = response.status();
            let content_type = response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .cloned();
            let body = crate::sources::read_limited_source_body_with_limit(
                response,
                SourceContext::narrow(SourceProvider::NCBI_EFETCH),
                NcbiClinVarVcv::MAX_BODY_BYTES,
            )
            .await?;
            decode_response(id, status, content_type.as_ref(), &body)
        }
    }

    pub(crate) fn decode_response(
        requested_id: u64,
        status: reqwest::StatusCode,
        content_type: Option<&HeaderValue>,
        body: &[u8],
    ) -> Result<Option<ClinVarRecordProjection>, BioMcpError> {
        if body.len() > NcbiClinVarVcv::MAX_BODY_BYTES {
            return Err(provider_error());
        }
        if !status.is_success() {
            return Err(provider_error());
        }
        let content_type = content_type
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if !content_type.contains("xml") {
            return Err(provider_error());
        }
        NcbiClinVarVcv::parse_record_bytes(requested_id, body).map_err(|_| provider_error())
    }

    fn provider_error() -> BioMcpError {
        BioMcpError::Api {
            api: "NCBI ClinVar".into(),
            message: "ClinVar record was unavailable".into(),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        const FIXTURE: &str = r#"<ClinVarResult-Set><VariationArchive VariationID="974782" Accession="VCV000974782" Version="2"><RecordStatus>current</RecordStatus><ClassifiedRecord><RCVList><RCVAccession Accession="RCV001251043" Version="2"><ClassifiedConditionList><ClassifiedCondition DB="MedGen" ID="C1">Disease one</ClassifiedCondition></ClassifiedConditionList><RCVClassifications><GermlineClassification><ReviewStatus>multiple submitters</ReviewStatus><Description DateLastEvaluated="2025-03-18" SubmissionCount="2">Likely pathogenic</Description></GermlineClassification><SomaticClinicalImpact><ReviewStatus>single submitter</ReviewStatus><Description>Tier II</Description></SomaticClinicalImpact></RCVClassifications></RCVAccession></RCVList><ClinicalAssertionList><ClinicalAssertion ID="1" ContributesToAggregateClassification="true"><ClinVarAccession Accession="SCV1" Version="3" SubmitterName="Lab A"/><RecordStatus>current</RecordStatus><Classification DateLastEvaluated="2025-01-01"><ReviewStatus>criteria provided</ReviewStatus><GermlineClassification>Pathogenic</GermlineClassification><Citation><ID Source="PubMed">123</ID></Citation><Comment>public</Comment></Classification><AttributeSet><Attribute Type="AssertionMethod">ACMG</Attribute></AttributeSet><TraitSet><Trait><XRef DB="OMIM" ID="1"/></Trait></TraitSet></ClinicalAssertion><ClinicalAssertion ID="2" ContributesToAggregateClassification="false"><ClinVarAccession Accession="SCV2" SubmitterName="Lab B"/><RecordStatus>current</RecordStatus><Classification><OncogenicityClassification>Oncogenic</OncogenicityClassification></Classification></ClinicalAssertion><ClinicalAssertion ID="3"><ClinVarAccession Accession="SCV3"/><RecordStatus>replaced</RecordStatus><Classification><GermlineClassification>Benign</GermlineClassification></Classification></ClinicalAssertion></ClinicalAssertionList><TraitMappingList><TraitMapping ClinicalAssertionID="1" MappingRef="MedGen" MappingValue="C1"><MedGen Name="Disease one"/></TraitMapping></TraitMappingList></ClassifiedRecord></VariationArchive></ClinVarResult-Set>"#;

        #[test]
        fn request_plan_uses_numeric_variation_identity_and_vcv_mode() {
            let plan = ClinvarClient::variation_plan(974782, Some("key"));
            assert_eq!(plan.path, "efetch.fcgi");
            assert_eq!(plan.query_value("db"), Some("clinvar"));
            assert_eq!(plan.query_value("rettype"), Some("vcv"));
            assert_eq!(plan.query_value("is_variationid"), Some("true"));
            assert_eq!(plan.query_value("id"), Some("974782"));
            assert_eq!(plan.query_value("api_key"), Some("key"));
        }

        #[test]
        fn response_requires_success_and_xml_content_type() {
            let xml = HeaderValue::from_static("application/xml");
            assert!(
                decode_response(
                    974782,
                    reqwest::StatusCode::OK,
                    Some(&xml),
                    FIXTURE.as_bytes()
                )
                .unwrap()
                .is_some()
            );
            assert!(
                decode_response(974782, reqwest::StatusCode::BAD_GATEWAY, Some(&xml), b"bad")
                    .is_err()
            );
            assert!(
                decode_response(
                    974782,
                    reqwest::StatusCode::OK,
                    Some(&HeaderValue::from_static("text/html")),
                    b"<html/>"
                )
                .is_err()
            );
        }

        #[test]
        fn exact_body_boundary_is_accepted_and_plus_one_rejected() {
            let prefix = "<ClinVarResult-Set>";
            let suffix = "</ClinVarResult-Set>";
            let padding = " ".repeat(NcbiClinVarVcv::MAX_BODY_BYTES - prefix.len() - suffix.len());
            let exact = format!("{prefix}{padding}{suffix}");
            let xml = HeaderValue::from_static("application/xml");
            assert!(
                decode_response(7, reqwest::StatusCode::OK, Some(&xml), exact.as_bytes())
                    .unwrap()
                    .is_none()
            );
            assert!(
                decode_response(
                    7,
                    reqwest::StatusCode::OK,
                    Some(&xml),
                    format!("{exact} ").as_bytes()
                )
                .is_err()
            );
        }
    }
}
