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
    use std::collections::{HashMap, HashSet};

    use http_cache_reqwest::CacheMode;
    use reqwest::header::HeaderValue;
    use roxmltree::Node;

    use crate::entities::variant::{
        ClinvarAggregate, ClinvarCitation, ClinvarRecord, ClinvarSubmission,
    };
    use crate::error::{BioMcpError, SourceContext, SourceProvider};
    use crate::sources::{RequestBuilderSourceContextExt, RequestPlan, request_from_plan};
    use crate::xml::parse_external_xml;

    const CLINVAR_BASE: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils";
    const CLINVAR_BASE_ENV: &str = "BIOMCP_CLINVAR_BASE";
    pub(crate) const CLINVAR_MAX_BODY_BYTES: usize = 8 * 1024 * 1024;
    pub(crate) const CLINVAR_XML_NODE_LIMIT: u32 = 1_000_000;
    const MAX_RCV: usize = 512;
    const MAX_SCV: usize = 2_048;
    const MAX_CONDITIONS: usize = 256;
    const MAX_CITATIONS: usize = 128;
    const MAX_TEXT_BYTES: usize = 32 * 1024;
    const MAX_PUBLIC_TEXT_BYTES: usize = CLINVAR_MAX_BODY_BYTES;

    #[derive(Default)]
    struct PublicTextBudget(usize);

    impl PublicTextBudget {
        fn charge(&mut self, bytes: usize) -> Result<(), BioMcpError> {
            self.0 = self.0.checked_add(bytes).ok_or_else(provider_error)?;
            if self.0 > MAX_PUBLIC_TEXT_BYTES {
                return Err(provider_error());
            }
            Ok(())
        }

        fn repeated(&mut self, bytes: usize, count: usize) -> Result<(), BioMcpError> {
            self.charge(bytes.checked_mul(count).ok_or_else(provider_error)?)
        }
    }

    #[derive(Clone)]
    pub(crate) struct ClinvarClient {
        client: reqwest_middleware::ClientWithMiddleware,
        base: Cow<'static, str>,
        api_key: Option<String>,
    }

    #[cfg(test)]
    mod clinvar_render_tests {

        #[test]
        fn clinvar_markdown_keeps_vcv_rcv_and_scv_statuses_and_domains_distinct() {
            let variant: crate::entities::variant::Variant = serde_json::from_value(serde_json::json!({
        "id": "chr5:g.118860951A>G",
        "gene": "HSD17B4",
        "section_outcomes": {"clinvar": {"outcome": "data", "sources": ["NCBI ClinVar"]}},
        "clinvar": {
            "source": "NCBI ClinVar", "variation_id": 974782,
            "accession": "VCV000974782", "version": 2, "record_status": "current",
            "number_submitters": 2,
            "aggregates": [{
                "source": "NCBI ClinVar", "accession": "RCV001251043", "version": 2,
                "classification_domain": "germline", "classification": "Likely pathogenic",
                "review_status": "criteria provided, multiple submitters, no conflicts",
                "submission_count": 2, "conditions": ["Bifunctional peroxisomal enzyme deficiency"]
            }],
            "submissions": [{
                "source": "NCBI ClinVar", "accession": "SCV006072505", "version": 1,
                "classification_domain": "oncogenicity", "classification": "Oncogenic",
                "record_status": "current", "submitter": "LabCorp",
                "contributes_to_aggregate_classification": false, "conditions": []
            }]
        }
    }))
    .expect("variant");
            let markdown = crate::render::markdown::variant_markdown(&variant, &["clinvar".into()])
                .expect("markdown");
            assert!(markdown.contains("VCV record status: current"));
            assert!(markdown.contains("RCV001251043.2 [germline]"));
            assert!(markdown.contains("SCV006072505.1 [oncogenicity]"));
            assert!(markdown.contains("SCV status current"));
            assert!(markdown.contains("contributes to aggregate: false"));
        }
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
        ) -> Result<Option<ClinvarRecord>, BioMcpError> {
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
                CLINVAR_MAX_BODY_BYTES,
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
    ) -> Result<Option<ClinvarRecord>, BioMcpError> {
        if body.len() > CLINVAR_MAX_BODY_BYTES {
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
        let xml = std::str::from_utf8(body).map_err(|_| provider_error())?;
        parse_record(requested_id, xml)
    }

    fn provider_error() -> BioMcpError {
        BioMcpError::Api {
            api: "NCBI ClinVar".into(),
            message: "ClinVar record was unavailable".into(),
        }
    }

    fn value_ref<'a, 'input>(node: Node<'a, 'input>) -> Option<&'a str> {
        node.text().map(str::trim).filter(|text| !text.is_empty())
    }

    fn value(node: Node<'_, '_>) -> Option<String> {
        value_ref(node).map(str::to_string)
    }

    fn child<'a, 'input>(node: Node<'a, 'input>, name: &str) -> Option<Node<'a, 'input>> {
        node.children().find(|child| child.has_tag_name(name))
    }

    fn child_value(node: Node<'_, '_>, name: &str) -> Option<String> {
        child_value_ref(node, name).map(str::to_string)
    }

    fn child_value_ref<'a, 'input>(node: Node<'a, 'input>, name: &str) -> Option<&'a str> {
        child(node, name).and_then(value_ref)
    }

    fn rcv_record_status_ref<'a, 'input>(node: Node<'a, 'input>) -> Option<&'a str> {
        child_value_ref(node, "RecordStatus").or_else(|| node.attribute("RecordStatus"))
    }

    fn optional_u32_attr(node: Node<'_, '_>, name: &str) -> Result<Option<u32>, BioMcpError> {
        node.attribute(name)
            .map(|value| value.parse().map_err(|_| provider_error()))
            .transpose()
    }

    fn optional_bool_attr(node: Node<'_, '_>, name: &str) -> Result<Option<bool>, BioMcpError> {
        node.attribute(name)
            .map(|value| match value.to_ascii_lowercase().as_str() {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(provider_error()),
            })
            .transpose()
    }

    fn bounded_text(text: Option<String>) -> Result<Option<String>, BioMcpError> {
        match text {
            Some(text) if text.len() > MAX_TEXT_BYTES => Err(provider_error()),
            other => Ok(other),
        }
    }

    fn joined_bounded_text<'a, 'input: 'a>(
        nodes: impl Iterator<Item = Node<'a, 'input>>,
    ) -> Result<Option<String>, BioMcpError> {
        let joined = nodes.filter_map(value).collect::<Vec<_>>().join("; ");
        bounded_text((!joined.is_empty()).then_some(joined))
    }

    fn push_distinct(
        values: &mut Vec<String>,
        candidate: Option<String>,
    ) -> Result<(), BioMcpError> {
        if let Some(candidate) = candidate.filter(|value| !value.trim().is_empty())
            && !values.contains(&candidate)
        {
            if values.len() == MAX_CONDITIONS {
                return Err(provider_error());
            }
            values.push(candidate);
        }
        Ok(())
    }

    fn classification_domain(name: &str) -> String {
        match name {
            "GermlineClassification" => "germline".into(),
            "SomaticClinicalImpact" | "SomaticClinicalImpactClassification" => {
                "somatic clinical impact".into()
            }
            "OncogenicityClassification" => "oncogenicity".into(),
            other => other
                .strip_suffix("Classification")
                .unwrap_or(other)
                .to_string(),
        }
    }

    fn classification_domain_len(name: &str) -> usize {
        match name {
            "GermlineClassification" => "germline".len(),
            "SomaticClinicalImpact" | "SomaticClinicalImpactClassification" => {
                "somatic clinical impact".len()
            }
            "OncogenicityClassification" => "oncogenicity".len(),
            other => other.strip_suffix("Classification").unwrap_or(other).len(),
        }
    }

    fn classification_child<'a, 'input>(node: Node<'a, 'input>) -> Option<Node<'a, 'input>> {
        node.children().find(|child| {
            child.is_element()
                && !matches!(
                    child.tag_name().name(),
                    "ReviewStatus" | "Comment" | "Citation"
                )
                && (child.tag_name().name().ends_with("Classification")
                    || matches!(
                        child.tag_name().name(),
                        "SomaticClinicalImpact" | "Oncogenicity"
                    ))
        })
    }

    fn value_len(node: Node<'_, '_>) -> Option<usize> {
        value_ref(node).map(str::len)
    }

    fn child_value_len(node: Node<'_, '_>, name: &str) -> Option<usize> {
        child_value_ref(node, name).map(str::len)
    }

    fn joined_text_len<'a, 'input: 'a>(nodes: impl Iterator<Item = Node<'a, 'input>>) -> usize {
        let mut count = 0usize;
        let mut bytes = 0usize;
        for node in nodes {
            let Some(len) = value_len(node) else {
                continue;
            };
            bytes = bytes.saturating_add(len);
            count += 1;
        }
        bytes.saturating_add(count.saturating_sub(1).saturating_mul(2))
    }

    fn formatted_pair_len(node: Node<'_, '_>, first: &str, second: &str) -> usize {
        match (node.attribute(first), node.attribute(second)) {
            (Some(first), Some(second)) => {
                first.len().saturating_add(1).saturating_add(second.len())
            }
            _ => 0,
        }
    }

    fn rcv_condition_text_len(node: Node<'_, '_>) -> usize {
        node.descendants()
            .filter(|node| node.has_tag_name("ClassifiedCondition"))
            .map(|condition| {
                value_len(condition)
                    .unwrap_or(0)
                    .saturating_add(formatted_pair_len(condition, "DB", "ID"))
            })
            .fold(0usize, usize::saturating_add)
    }

    fn preflight_public_text(
        archive: Node<'_, '_>,
        rcv_nodes: &[Node<'_, '_>],
        scv_nodes: &[Node<'_, '_>],
    ) -> Result<(), BioMcpError> {
        let mut budget = PublicTextBudget::default();
        budget.charge("NCBI ClinVar".len())?;
        budget.charge(archive.attribute("Accession").map(str::len).unwrap_or(0))?;
        budget.charge(child_value_len(archive, "RecordStatus").unwrap_or(0))?;

        let mut aggregate_count = 0usize;
        for rcv in rcv_nodes {
            let Some(classifications) = child(*rcv, "RCVClassifications") else {
                continue;
            };
            let row_count = classifications
                .children()
                .filter(Node::is_element)
                .filter(|classification| child(*classification, "Description").is_some())
                .count();
            aggregate_count = aggregate_count
                .checked_add(row_count)
                .ok_or_else(provider_error)?;
            if aggregate_count > MAX_RCV {
                return Err(provider_error());
            }
            let shared = "NCBI ClinVar"
                .len()
                .saturating_add(rcv.attribute("Accession").map(str::len).unwrap_or(0))
                .saturating_add(rcv_record_status_ref(*rcv).map(str::len).unwrap_or(0))
                .saturating_add(rcv_condition_text_len(*rcv));
            budget.repeated(shared, row_count)?;
            for classification in classifications
                .children()
                .filter(Node::is_element)
                .filter(|classification| child(*classification, "Description").is_some())
            {
                let description = child(classification, "Description").expect("preflight row");
                budget.charge(classification_domain_len(classification.tag_name().name()))?;
                budget.charge(value_len(description).unwrap_or(0))?;
                budget.charge(child_value_len(classification, "ReviewStatus").unwrap_or(0))?;
                budget.charge(
                    description
                        .attribute("DateLastEvaluated")
                        .map(str::len)
                        .unwrap_or(0),
                )?;
            }
        }

        let mut mapping_text = HashMap::<&str, usize>::new();
        for mapping in archive
            .descendants()
            .filter(|node| node.has_tag_name("TraitMapping"))
        {
            let Some(id) = mapping.attribute("ClinicalAssertionID") else {
                continue;
            };
            let mut bytes = formatted_pair_len(mapping, "MappingRef", "MappingValue");
            for medgen in mapping
                .children()
                .filter(|node| node.has_tag_name("MedGen"))
            {
                bytes = bytes.saturating_add(medgen.attribute("Name").map(str::len).unwrap_or(0));
            }
            let entry = mapping_text.entry(id).or_default();
            *entry = entry.saturating_add(bytes);
        }
        for assertion in scv_nodes {
            if !child_value_ref(*assertion, "RecordStatus")
                .is_some_and(|status| status.trim().eq_ignore_ascii_case("current"))
            {
                continue;
            }
            budget.charge("NCBI ClinVar".len())?;
            budget.charge(
                child(*assertion, "ClinVarAccession").map_or(0, |accession| {
                    accession.attribute("Accession").map(str::len).unwrap_or(0)
                        + accession
                            .attribute("SubmitterName")
                            .map(str::len)
                            .unwrap_or(0)
                }),
            )?;
            budget.charge(child_value_len(*assertion, "RecordStatus").unwrap_or(0))?;
            if let Some(classification) = child(*assertion, "Classification") {
                if let Some(domain) = classification_child(classification) {
                    budget.charge(classification_domain_len(domain.tag_name().name()))?;
                    budget.charge(value_len(domain).unwrap_or(0))?;
                }
                budget.charge(child_value_len(classification, "ReviewStatus").unwrap_or(0))?;
                budget.charge(
                    classification
                        .attribute("DateLastEvaluated")
                        .map(str::len)
                        .unwrap_or(0),
                )?;
                budget.charge(joined_text_len(
                    classification
                        .children()
                        .filter(|node| node.has_tag_name("Comment")),
                ))?;
            }
            budget.charge(joined_text_len(assertion.descendants().filter(
                |candidate| {
                    candidate.has_tag_name("Attribute")
                        && matches!(
                            candidate.attribute("Type"),
                            Some("AssertionMethod" | "AssertionCriteria" | "ClassificationMethod")
                        )
                },
            )))?;
            for trait_node in assertion
                .descendants()
                .filter(|node| node.has_tag_name("Trait"))
            {
                budget.charge(
                    trait_node
                        .descendants()
                        .find(|node| {
                            node.has_tag_name("ElementValue")
                                && node.attribute("Type") == Some("Preferred")
                        })
                        .and_then(value_len)
                        .unwrap_or(0),
                )?;
                for xref in trait_node
                    .descendants()
                    .filter(|node| node.has_tag_name("XRef"))
                {
                    budget.charge(formatted_pair_len(xref, "DB", "ID"))?;
                }
            }
            if let Some(id) = assertion.attribute("ID") {
                budget.charge(mapping_text.get(id).copied().unwrap_or(0))?;
            }
            for citation in assertion
                .descendants()
                .filter(|node| node.has_tag_name("Citation"))
            {
                if let Some(id_node) = child(citation, "ID") {
                    budget.charge(id_node.attribute("Source").map(str::len).unwrap_or(0))?;
                    budget.charge(value_len(id_node).unwrap_or(0))?;
                }
                budget.charge(child_value_len(citation, "URL").unwrap_or(0))?;
            }
        }
        Ok(())
    }

    fn rcv_conditions(node: Node<'_, '_>) -> Result<Vec<String>, BioMcpError> {
        let mut conditions = Vec::new();
        for condition in node
            .descendants()
            .filter(|node| node.has_tag_name("ClassifiedCondition"))
        {
            push_distinct(&mut conditions, value(condition))?;
            if let (Some(db), Some(id)) = (condition.attribute("DB"), condition.attribute("ID")) {
                push_distinct(&mut conditions, Some(format!("{db}:{id}")))?;
            }
        }
        Ok(conditions)
    }

    fn parse_rcv(node: Node<'_, '_>) -> Result<Vec<ClinvarAggregate>, BioMcpError> {
        let accession = node.attribute("Accession").unwrap_or_default().trim();
        if accession.is_empty() {
            return Ok(Vec::new());
        }
        let version = optional_u32_attr(node, "Version")?;
        let record_status = rcv_record_status_ref(node).map(str::to_string);
        let conditions = rcv_conditions(node)?;
        let mut rows = Vec::new();
        let Some(classifications) = child(node, "RCVClassifications") else {
            return Ok(rows);
        };
        for classification in classifications.children().filter(Node::is_element) {
            let Some(description) = child(classification, "Description") else {
                continue;
            };
            rows.push(ClinvarAggregate {
                source: "NCBI ClinVar".into(),
                accession: accession.into(),
                version,
                classification_domain: classification_domain(classification.tag_name().name()),
                classification: value(description),
                review_status: child_value(classification, "ReviewStatus"),
                evaluation_date: description
                    .attribute("DateLastEvaluated")
                    .map(str::to_string),
                record_status: record_status.clone(),
                number_submitters: optional_u32_attr(description, "NumberOfSubmitters")?,
                submission_count: optional_u32_attr(description, "SubmissionCount")?,
                conditions: conditions.clone(),
            });
        }
        Ok(rows)
    }

    fn scv_conditions(
        assertion: Node<'_, '_>,
        assertion_id: Option<&str>,
        trait_mappings: &HashMap<String, Vec<Node<'_, '_>>>,
    ) -> Result<Vec<String>, BioMcpError> {
        let mut conditions = Vec::new();
        for trait_node in assertion
            .descendants()
            .filter(|node| node.has_tag_name("Trait"))
        {
            let preferred = trait_node.descendants().find(|node| {
                node.has_tag_name("ElementValue") && node.attribute("Type") == Some("Preferred")
            });
            push_distinct(&mut conditions, preferred.and_then(value))?;
            for xref in trait_node
                .descendants()
                .filter(|node| node.has_tag_name("XRef"))
            {
                if let (Some(db), Some(id)) = (xref.attribute("DB"), xref.attribute("ID")) {
                    push_distinct(&mut conditions, Some(format!("{db}:{id}")))?;
                }
            }
        }
        for mapping in assertion_id
            .and_then(|id| trait_mappings.get(id))
            .into_iter()
            .flatten()
        {
            if let (Some(db), Some(id)) = (
                mapping.attribute("MappingRef"),
                mapping.attribute("MappingValue"),
            ) {
                push_distinct(&mut conditions, Some(format!("{db}:{id}")))?;
            }
            for medgen in mapping
                .children()
                .filter(|node| node.has_tag_name("MedGen"))
            {
                push_distinct(
                    &mut conditions,
                    medgen.attribute("Name").map(str::to_string),
                )?;
            }
        }
        Ok(conditions)
    }

    fn citations(assertion: Node<'_, '_>) -> Result<Vec<ClinvarCitation>, BioMcpError> {
        let mut rows = Vec::new();
        for citation in assertion
            .descendants()
            .filter(|node| node.has_tag_name("Citation"))
        {
            if rows.len() == MAX_CITATIONS {
                return Err(provider_error());
            }
            let id_node = child(citation, "ID");
            let row = ClinvarCitation {
                source: id_node
                    .and_then(|node| node.attribute("Source"))
                    .map(str::to_string),
                id: id_node.and_then(value),
                url: child_value(citation, "URL"),
            };
            if row.source.is_some() || row.id.is_some() || row.url.is_some() {
                rows.push(row);
            }
        }
        Ok(rows)
    }

    fn parse_scv(
        node: Node<'_, '_>,
        trait_mappings: &HashMap<String, Vec<Node<'_, '_>>>,
    ) -> Result<Option<ClinvarSubmission>, BioMcpError> {
        let record_status = child_value(node, "RecordStatus").unwrap_or_default();
        if !record_status.eq_ignore_ascii_case("current") {
            return Ok(None);
        }
        let accession_node = child(node, "ClinVarAccession").ok_or_else(provider_error)?;
        let accession = accession_node
            .attribute("Accession")
            .unwrap_or_default()
            .trim();
        if accession.is_empty() {
            return Err(provider_error());
        }
        let classification = child(node, "Classification").ok_or_else(provider_error)?;
        let domain_node = classification_child(classification).ok_or_else(provider_error)?;
        let criteria = node.descendants().filter(|candidate| {
            candidate.has_tag_name("Attribute")
                && matches!(
                    candidate.attribute("Type"),
                    Some("AssertionMethod" | "AssertionCriteria" | "ClassificationMethod")
                )
        });
        Ok(Some(ClinvarSubmission {
            source: "NCBI ClinVar".into(),
            accession: accession.into(),
            version: optional_u32_attr(accession_node, "Version")?,
            classification_domain: classification_domain(domain_node.tag_name().name()),
            classification: value(domain_node),
            review_status: child_value(classification, "ReviewStatus"),
            evaluation_date: classification
                .attribute("DateLastEvaluated")
                .map(str::to_string),
            record_status,
            submitter: accession_node
                .attribute("SubmitterName")
                .map(str::to_string),
            contributes_to_aggregate_classification: optional_bool_attr(
                node,
                "ContributesToAggregateClassification",
            )?,
            conditions: scv_conditions(node, node.attribute("ID"), trait_mappings)?,
            criteria: joined_bounded_text(criteria)?,
            citations: citations(node)?,
            public_comment: joined_bounded_text(
                classification
                    .children()
                    .filter(|node| node.has_tag_name("Comment")),
            )?,
        }))
    }

    pub(crate) fn parse_record(
        requested_id: u64,
        xml: &str,
    ) -> Result<Option<ClinvarRecord>, BioMcpError> {
        let doc = parse_external_xml(xml, CLINVAR_XML_NODE_LIMIT).map_err(|_| provider_error())?;
        if !doc.root_element().has_tag_name("ClinVarResult-Set") {
            return Err(provider_error());
        }
        let archives = doc
            .descendants()
            .filter(|node| node.has_tag_name("VariationArchive"))
            .collect::<Vec<_>>();
        if archives.is_empty() {
            return Ok(None);
        }
        if archives.len() != 1 {
            return Err(provider_error());
        }
        let archive = archives[0];
        let variation_id = archive
            .attribute("VariationID")
            .and_then(|value| value.parse().ok())
            .ok_or_else(provider_error)?;
        if variation_id != requested_id {
            return Err(provider_error());
        }
        let record_status = child_value(archive, "RecordStatus");
        if !record_status
            .as_deref()
            .is_some_and(|status| status.eq_ignore_ascii_case("current"))
        {
            return Ok(None);
        }
        let rcv_nodes = archive
            .descendants()
            .filter(|node| node.has_tag_name("RCVAccession"))
            .collect::<Vec<_>>();
        let scv_nodes = archive
            .descendants()
            .filter(|node| node.has_tag_name("ClinicalAssertion"))
            .collect::<Vec<_>>();
        if rcv_nodes.len() > MAX_RCV || scv_nodes.len() > MAX_SCV {
            return Err(provider_error());
        }
        let mut assertion_ids = HashSet::new();
        for node in &scv_nodes {
            if let Some(id) = node.attribute("ID")
                && !assertion_ids.insert(id)
            {
                return Err(provider_error());
            }
        }
        preflight_public_text(archive, &rcv_nodes, &scv_nodes)?;
        let mut trait_mappings: HashMap<String, Vec<Node<'_, '_>>> = HashMap::new();
        for mapping in archive
            .descendants()
            .filter(|node| node.has_tag_name("TraitMapping"))
        {
            if let Some(id) = mapping.attribute("ClinicalAssertionID") {
                trait_mappings
                    .entry(id.to_string())
                    .or_default()
                    .push(mapping);
            }
        }
        let mut aggregates = Vec::new();
        for node in rcv_nodes {
            aggregates.extend(parse_rcv(node)?);
            if aggregates.len() > MAX_RCV {
                return Err(provider_error());
            }
        }
        let mut submissions = Vec::new();
        for node in scv_nodes {
            if let Some(row) = parse_scv(node, &trait_mappings)? {
                submissions.push(row);
            }
        }
        Ok(Some(ClinvarRecord {
            source: "NCBI ClinVar".into(),
            variation_id,
            accession: archive.attribute("Accession").map(str::to_string),
            version: optional_u32_attr(archive, "Version")?,
            record_status,
            number_submissions: optional_u32_attr(archive, "NumberOfSubmissions")?,
            number_submitters: optional_u32_attr(archive, "NumberOfSubmitters")?,
            aggregates,
            submissions,
        }))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        const FIXTURE: &str = r#"<ClinVarResult-Set><VariationArchive VariationID="974782" Accession="VCV000974782" Version="2"><RecordStatus>current</RecordStatus><ClassifiedRecord><RCVList><RCVAccession Accession="RCV001251043" Version="2"><ClassifiedConditionList><ClassifiedCondition DB="MedGen" ID="C1">Disease one</ClassifiedCondition></ClassifiedConditionList><RCVClassifications><GermlineClassification><ReviewStatus>multiple submitters</ReviewStatus><Description DateLastEvaluated="2025-03-18" SubmissionCount="2">Likely pathogenic</Description></GermlineClassification><SomaticClinicalImpact><ReviewStatus>single submitter</ReviewStatus><Description>Tier II</Description></SomaticClinicalImpact></RCVClassifications></RCVAccession></RCVList><ClinicalAssertionList><ClinicalAssertion ID="1" ContributesToAggregateClassification="true"><ClinVarAccession Accession="SCV1" Version="3" SubmitterName="Lab A"/><RecordStatus>current</RecordStatus><Classification DateLastEvaluated="2025-01-01"><ReviewStatus>criteria provided</ReviewStatus><GermlineClassification>Pathogenic</GermlineClassification><Citation><ID Source="PubMed">123</ID></Citation><Comment>public</Comment></Classification><AttributeSet><Attribute Type="AssertionMethod">ACMG</Attribute></AttributeSet><TraitSet><Trait><XRef DB="OMIM" ID="1"/></Trait></TraitSet></ClinicalAssertion><ClinicalAssertion ID="2" ContributesToAggregateClassification="false"><ClinVarAccession Accession="SCV2" SubmitterName="Lab B"/><RecordStatus>current</RecordStatus><Classification><OncogenicityClassification>Oncogenic</OncogenicityClassification></Classification></ClinicalAssertion><ClinicalAssertion ID="3"><ClinVarAccession Accession="SCV3"/><RecordStatus>replaced</RecordStatus><Classification><GermlineClassification>Benign</GermlineClassification></Classification></ClinicalAssertion></ClinicalAssertionList><TraitMappingList><TraitMapping ClinicalAssertionID="1" MappingRef="MedGen" MappingValue="C1"><MedGen Name="Disease one"/></TraitMapping></TraitMappingList></ClassifiedRecord></VariationArchive></ClinVarResult-Set>"#;
        const HSD17B4: &str = r#"<ClinVarResult-Set><VariationArchive VariationID="974782" Accession="VCV000974782" Version="2" NumberOfSubmissions="2" NumberOfSubmitters="2"><RecordStatus>current</RecordStatus><ClassifiedRecord><RCVList><RCVAccession Accession="RCV001251043" Version="2"><ClassifiedConditionList><ClassifiedCondition DB="MedGen" ID="C0342870">Bifunctional peroxisomal enzyme deficiency</ClassifiedCondition></ClassifiedConditionList><RCVClassifications><GermlineClassification><ReviewStatus>criteria provided, multiple submitters, no conflicts</ReviewStatus><Description DateLastEvaluated="2025-03-18" SubmissionCount="2">Likely pathogenic</Description></GermlineClassification></RCVClassifications></RCVAccession></RCVList><ClinicalAssertionList><ClinicalAssertion ID="2814546" ContributesToAggregateClassification="true"><ClinVarAccession Accession="SCV001426412" Version="1" SubmitterName="Rady Children's Institute for Genomic Medicine"/><RecordStatus>current</RecordStatus><Classification DateLastEvaluated="2020-08-04"><ReviewStatus>criteria provided, single submitter</ReviewStatus><GermlineClassification>Likely pathogenic</GermlineClassification></Classification></ClinicalAssertion><ClinicalAssertion ID="11678995" ContributesToAggregateClassification="true"><ClinVarAccession Accession="SCV006072505" Version="1" SubmitterName="LabCorp"/><RecordStatus>current</RecordStatus><Classification DateLastEvaluated="2025-03-18"><ReviewStatus>criteria provided, single submitter</ReviewStatus><GermlineClassification>Likely pathogenic</GermlineClassification></Classification></ClinicalAssertion></ClinicalAssertionList></ClassifiedRecord></VariationArchive></ClinVarResult-Set>"#;

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
        fn hsd17b4_returns_current_two_submitter_aggregate_and_both_scvs() {
            let record = parse_record(974782, HSD17B4).unwrap().unwrap();
            assert_eq!(record.number_submitters, Some(2));
            assert_eq!(record.aggregates[0].submission_count, Some(2));
            assert_eq!(
                record.aggregates[0].review_status.as_deref(),
                Some("criteria provided, multiple submitters, no conflicts")
            );
            assert_eq!(
                record
                    .submissions
                    .iter()
                    .map(|row| row.accession.as_str())
                    .collect::<Vec<_>>(),
                ["SCV001426412", "SCV006072505"]
            );
        }

        #[test]
        fn parses_domains_current_noncontributing_rows_and_trait_mappings() {
            let record = parse_record(974782, FIXTURE).unwrap().unwrap();
            assert_eq!(record.record_status.as_deref(), Some("current"));
            assert_eq!(record.aggregates.len(), 2);
            assert_eq!(record.aggregates[0].submission_count, Some(2));
            assert_eq!(
                record.aggregates[1].classification_domain,
                "somatic clinical impact"
            );
            assert_eq!(record.submissions.len(), 2);
            assert_eq!(
                record.submissions[1].contributes_to_aggregate_classification,
                Some(false)
            );
            assert_eq!(record.submissions[1].classification_domain, "oncogenicity");
            assert!(
                record.submissions[0]
                    .conditions
                    .contains(&"Disease one".into())
            );
            assert_eq!(record.submissions[0].criteria.as_deref(), Some("ACMG"));
            assert_eq!(
                record.submissions[0].citations[0].id.as_deref(),
                Some("123")
            );
        }

        #[test]
        fn rejects_hostile_invalid_and_mismatched_documents() {
            for xml in [
                "<html/>",
                "<ClinVarResult-Set>",
                "<!DOCTYPE x [<!ENTITY x 'y'>]><ClinVarResult-Set>&x;</ClinVarResult-Set>",
            ] {
                assert!(parse_record(974782, xml).is_err());
            }
            assert!(parse_record(1, FIXTURE).is_err());
        }

        #[test]
        fn supplied_invalid_numeric_and_contribution_attributes_fail_closed() {
            for xml in [
                HSD17B4.replace(
                    "Version=\"2\" NumberOfSubmissions",
                    "Version=\"two\" NumberOfSubmissions",
                ),
                HSD17B4.replace("NumberOfSubmissions=\"2\"", "NumberOfSubmissions=\"many\""),
                HSD17B4.replace("NumberOfSubmitters=\"2\"", "NumberOfSubmitters=\"many\""),
                HSD17B4.replacen(
                    "Version=\"2\"><ClassifiedConditionList",
                    "Version=\"two\"><ClassifiedConditionList",
                    1,
                ),
                HSD17B4.replace("SubmissionCount=\"2\"", "SubmissionCount=\"many\""),
                HSD17B4.replacen(
                    "Version=\"1\" SubmitterName",
                    "Version=\"one\" SubmitterName",
                    1,
                ),
                HSD17B4.replacen(
                    "ContributesToAggregateClassification=\"true\"",
                    "ContributesToAggregateClassification=\"sometimes\"",
                    1,
                ),
            ] {
                assert!(parse_record(974782, &xml).is_err());
            }
        }

        #[test]
        fn duplicate_assertion_ids_reject_trait_mapping_amplification() {
            let scv = "<ClinicalAssertion ID=\"same\"><ClinVarAccession Accession=\"SCV1\"/><RecordStatus>current</RecordStatus><Classification><GermlineClassification>Pathogenic</GermlineClassification></Classification></ClinicalAssertion>";
            let mapping_value = "x".repeat(MAX_TEXT_BYTES);
            let xml = archive_with(&format!(
                "<ClinicalAssertionList>{}</ClinicalAssertionList><TraitMappingList><TraitMapping ClinicalAssertionID=\"same\" MappingRef=\"provider\" MappingValue=\"{mapping_value}\"/></TraitMappingList>",
                scv.repeat(MAX_SCV)
            ));
            assert!(xml.len() < CLINVAR_MAX_BODY_BYTES);
            assert!(parse_record(7, &xml).is_err());
        }

        #[test]
        fn cumulative_public_text_budget_accepts_exact_and_rejects_plus_one() {
            let classifications =
                "<GermlineClassification><Description>P</Description></GermlineClassification>"
                    .repeat(MAX_RCV);
            let fixed = "NCBI ClinVar".len()
                + "current".len()
                + MAX_RCV * ("NCBI ClinVar".len() + "germline".len() + "P".len());
            let shared_accession_len = (MAX_PUBLIC_TEXT_BYTES - fixed) / MAX_RCV;
            let archive_accession_len = (MAX_PUBLIC_TEXT_BYTES - fixed) % MAX_RCV;
            let body = |archive_extra: usize| {
                format!(
                    "<ClinVarResult-Set><VariationArchive VariationID=\"7\" Accession=\"{}\"><RecordStatus>current</RecordStatus><ClassifiedRecord><RCVList><RCVAccession Accession=\"{}\"><RCVClassifications>{classifications}</RCVClassifications></RCVAccession></RCVList></ClassifiedRecord></VariationArchive></ClinVarResult-Set>",
                    "v".repeat(archive_accession_len + archive_extra),
                    "r".repeat(shared_accession_len),
                )
            };
            let exact = body(0);
            assert!(exact.len() < CLINVAR_MAX_BODY_BYTES);
            assert_eq!(
                parse_record(7, &exact).unwrap().unwrap().aggregates.len(),
                MAX_RCV
            );
            let over = body(1);
            assert!(over.len() < CLINVAR_MAX_BODY_BYTES);
            assert!(parse_record(7, &over).is_err());
        }

        #[test]
        fn many_classification_children_fail_before_shared_text_is_cloned() {
            let classifications =
                "<GermlineClassification><Description>P</Description></GermlineClassification>"
                    .repeat(MAX_RCV);
            let shared = "r".repeat(2 * 1024 * 1024);
            let xml = archive_with(&format!(
                "<RCVList><RCVAccession Accession=\"{shared}\"><RCVClassifications>{classifications}</RCVClassifications></RCVAccession></RCVList>"
            ));
            assert!(xml.len() < CLINVAR_MAX_BODY_BYTES);
            assert!(parse_record(7, &xml).is_err());

            let too_many =
                "<GermlineClassification><Description>P</Description></GermlineClassification>"
                    .repeat(MAX_RCV + 1);
            let xml = archive_with(&format!(
                "<RCVList><RCVAccession Accession=\"RCV1\"><RCVClassifications>{too_many}</RCVClassifications></RCVAccession></RCVList>"
            ));
            assert!(parse_record(7, &xml).is_err());
        }

        #[test]
        fn empty_status_child_cannot_bypass_shared_attribute_budget() {
            let classifications =
                "<GermlineClassification><Description>P</Description></GermlineClassification>"
                    .repeat(MAX_RCV);
            let shared_status = "s".repeat(2 * 1024 * 1024);
            let xml = archive_with(&format!(
                "<RCVList><RCVAccession Accession=\"RCV1\" RecordStatus=\"{shared_status}\"><RecordStatus> \n </RecordStatus><RCVClassifications>{classifications}</RCVClassifications></RCVAccession></RCVList>"
            ));
            assert!(xml.len() < CLINVAR_MAX_BODY_BYTES);
            assert!(parse_record(7, &xml).is_err());
        }

        #[test]
        fn exact_text_and_list_boundaries_are_enforced_without_partial_records() {
            let text = "x".repeat(MAX_TEXT_BYTES);
            let exact = FIXTURE.replace("public", &text);
            assert!(parse_record(974782, &exact).is_ok());
            let over = FIXTURE.replace("public", &(text + "x"));
            assert!(parse_record(974782, &over).is_err());

            let criteria = "c".repeat(MAX_TEXT_BYTES);
            let exact = FIXTURE.replace(">ACMG<", &format!(">{criteria}<"));
            assert!(parse_record(974782, &exact).is_ok());
            let over = FIXTURE.replace(">ACMG<", &format!(">{criteria}c<"));
            assert!(parse_record(974782, &over).is_err());

            let citations = "<Citation><ID>1</ID></Citation>".repeat(MAX_CITATIONS);
            let exact = FIXTURE.replace(
                "<Citation><ID Source=\"PubMed\">123</ID></Citation>",
                &citations,
            );
            assert_eq!(
                parse_record(974782, &exact).unwrap().unwrap().submissions[0]
                    .citations
                    .len(),
                MAX_CITATIONS
            );
            let over = FIXTURE.replace(
                "<Citation><ID Source=\"PubMed\">123</ID></Citation>",
                &(citations + "<Citation><ID>2</ID></Citation>"),
            );
            assert!(parse_record(974782, &over).is_err());
        }

        fn archive_with(body: &str) -> String {
            format!(
                "<ClinVarResult-Set><VariationArchive VariationID=\"7\"><RecordStatus>current</RecordStatus><ClassifiedRecord>{body}</ClassifiedRecord></VariationArchive></ClinVarResult-Set>"
            )
        }

        #[test]
        fn exact_rcv_and_scv_boundaries_are_accepted_and_plus_one_rejected() {
            let rcv = "<RCVAccession Accession=\"RCV1\"><RCVClassifications><GermlineClassification><Description>Pathogenic</Description></GermlineClassification></RCVClassifications></RCVAccession>";
            let exact = archive_with(&format!("<RCVList>{}</RCVList>", rcv.repeat(MAX_RCV)));
            assert_eq!(
                parse_record(7, &exact).unwrap().unwrap().aggregates.len(),
                MAX_RCV
            );
            let over = archive_with(&format!("<RCVList>{}</RCVList>", rcv.repeat(MAX_RCV + 1)));
            assert!(parse_record(7, &over).is_err());

            let scv = "<ClinicalAssertion ContributesToAggregateClassification=\"false\"><ClinVarAccession Accession=\"SCV1\"/><RecordStatus>current</RecordStatus><Classification><GermlineClassification>Uncertain significance</GermlineClassification></Classification></ClinicalAssertion>";
            let exact = archive_with(&format!(
                "<ClinicalAssertionList>{}</ClinicalAssertionList>",
                scv.repeat(MAX_SCV)
            ));
            assert_eq!(
                parse_record(7, &exact).unwrap().unwrap().submissions.len(),
                MAX_SCV
            );
            let over = archive_with(&format!(
                "<ClinicalAssertionList>{}</ClinicalAssertionList>",
                scv.repeat(MAX_SCV + 1)
            ));
            assert!(parse_record(7, &over).is_err());
        }

        #[test]
        fn exact_condition_boundary_is_accepted_and_plus_one_rejected() {
            let rcv = |count| {
                format!(
                    "<RCVList><RCVAccession Accession=\"RCV1\"><ClassifiedConditionList>{}</ClassifiedConditionList><RCVClassifications><GermlineClassification><Description>Pathogenic</Description></GermlineClassification></RCVClassifications></RCVAccession></RCVList>",
                    (0..count)
                        .map(|index| format!(
                            "<ClassifiedCondition>condition {index}</ClassifiedCondition>"
                        ))
                        .collect::<String>()
                )
            };
            let exact = parse_record(7, &archive_with(&rcv(MAX_CONDITIONS)))
                .unwrap()
                .unwrap();
            assert_eq!(exact.aggregates[0].conditions.len(), MAX_CONDITIONS);
            assert!(parse_record(7, &archive_with(&rcv(MAX_CONDITIONS + 1))).is_err());
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
            let padding = " ".repeat(CLINVAR_MAX_BODY_BYTES - prefix.len() - suffix.len());
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

        #[test]
        fn exact_node_boundary_is_accepted_and_plus_one_rejected() {
            let base = archive_with("");
            let base_nodes = parse_external_xml(&base, u32::MAX)
                .expect("base XML")
                .descendants()
                .count();
            let padding = "<x/>".repeat(CLINVAR_XML_NODE_LIMIT as usize - base_nodes);
            assert!(parse_record(7, &archive_with(&padding)).is_ok());
            assert!(parse_record(7, &archive_with(&(padding + "<x/>"))).is_err());
        }
    }
}
