use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::error::BioMcpError;
use crate::sources::clingen_erepo::ERepoClient;

const MAX_CAIDS: usize = 50;
const GENE_PREVIEW_BYTES: usize = 256;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum ERepoBatchInput {
    Caids(Vec<String>),
    Object { caids: Vec<String> },
}

impl ERepoBatchInput {
    pub(crate) fn into_caids(self) -> Vec<String> {
        match self {
            Self::Caids(caids) | Self::Object { caids } => caids,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ERepoResponse {
    pub items: Vec<ERepoItem>,
    pub complete: bool,
    pub source_status: Vec<ERepoSourceStatus>,
    pub provider: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ERepoSourceStatus {
    pub source: &'static str,
    pub status: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ERepoGenePage {
    pub results: Vec<ERepoGeneResult>,
    pub returned: usize,
    pub offset: usize,
    pub limit: usize,
    pub has_more: bool,
    pub total: Option<usize>,
    pub source_status: Vec<ERepoSourceStatus>,
    pub provider: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ERepoGeneResult {
    pub caid: Option<String>,
    pub gene: Option<String>,
    pub condition: Option<String>,
    pub classification: Option<String>,
    pub guideline_label: Option<String>,
    pub expert_panel: Option<String>,
    pub published_date: Option<String>,
    pub hgvs: Vec<String>,
    pub hgvs_count: usize,
    pub met_evidence_codes: Vec<String>,
    pub truncated_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ERepoItem {
    pub caid: String,
    pub assertions: Vec<ERepoAssertion>,
    pub complete: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ERepoAssertion {
    pub assertion_id: String,
    pub doc_version: String,
    pub guideline_label: Option<String>,
    pub guideline_version: Option<String>,
    pub versions: Vec<String>,
    pub classification: Option<String>,
    pub condition: Option<String>,
    pub mondo_id: Option<String>,
    pub moi: Option<String>,
    pub vcep: Option<String>,
    pub gene: Option<String>,
    pub gene_ncbi_id: Option<String>,
    pub hgvs: Vec<String>,
    pub preferred_variant_title: Option<String>,
    pub approved_date: Option<String>,
    pub published_date: Option<String>,
    pub retracted: Option<bool>,
    pub pcer_doc_id: Option<String>,
    pub summary_description: Option<String>,
    pub source_url: String,
    pub criteria: Vec<ERepoCriterion>,
    pub unmet_codes_state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<ERepoDetail>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ERepoCriterion {
    pub source_token: String,
    pub code: String,
    pub status: &'static str,
    pub explicit_strength: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ERepoDetail {
    pub source_url: String,
    pub assertion_uuid: String,
    pub provider_entity_id: Option<String>,
    pub provider_at_id: Option<String>,
    pub body_sha256: String,
    pub body_bytes: usize,
    pub response_version: Option<String>,
    pub service_version: Option<String>,
    pub template_version: Option<String>,
    pub criteria: Vec<ERepoDetailCriterion>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ERepoDetailCriterion {
    pub code: Option<String>,
    pub default_strength: Option<String>,
    pub statement_outcome: Option<String>,
    pub comments: Vec<String>,
    pub curator_facts: Vec<Value>,
    pub pmids: Vec<ERepoPmid>,
    pub locator: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ERepoPmid {
    pub pmid: u64,
    pub assertion_uuid: String,
    pub provider_entity_id: Option<String>,
    pub locator: String,
}

pub(crate) async fn retrieve(
    caids: Vec<String>,
    detail: bool,
    assertion_id: Option<&str>,
    version: Option<&str>,
) -> Result<ERepoResponse, BioMcpError> {
    if caids.is_empty() || caids.len() > MAX_CAIDS {
        return Err(BioMcpError::InvalidArgument(
            "variant erepo requires between 1 and 50 CAids".into(),
        ));
    }
    if caids.iter().any(|caid| caid.trim().is_empty()) {
        return Err(BioMcpError::InvalidArgument(
            "variant erepo CAid must not be empty".into(),
        ));
    }
    if !detail && (assertion_id.is_some() || version.is_some()) {
        return Err(BioMcpError::InvalidArgument(
            "variant erepo --assertion and --version require --detail".into(),
        ));
    }
    if detail && caids.len() != 1 {
        return Err(BioMcpError::InvalidArgument(
            "variant erepo --detail is only available for one CAid".into(),
        ));
    }
    retrieve_with_client(caids, detail, assertion_id, version, ERepoClient::new()?).await
}

pub(crate) async fn search_gene(
    gene: &str,
    limit: usize,
    offset: usize,
) -> Result<ERepoGenePage, BioMcpError> {
    if !crate::sources::is_valid_gene_symbol(gene) {
        return Err(BioMcpError::InvalidArgument(
            "variant erepo --gene must be a gene symbol".into(),
        ));
    }
    if !(1..=100).contains(&limit) {
        return Err(BioMcpError::InvalidArgument(
            "variant erepo --limit must be between 1 and 100".into(),
        ));
    }
    let value = ERepoClient::new()?.gene(gene, limit, offset).await?;
    gene_page_from_value(&value, offset, limit)
}

fn gene_page_from_value(
    value: &Value,
    offset: usize,
    limit: usize,
) -> Result<ERepoGenePage, BioMcpError> {
    let rows = value
        .get("variantInterpretations")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("gene response has no variantInterpretations array"))?;
    if rows.len() > limit.saturating_add(1) {
        return Err(invalid("gene response exceeded requested page"));
    }
    let has_more = rows.len() > limit;
    let results = rows
        .iter()
        .take(limit)
        .map(gene_result)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ERepoGenePage {
        returned: results.len(),
        results,
        offset,
        limit,
        has_more,
        total: None,
        source_status: vec![ERepoSourceStatus {
            source: "clingen_erepo",
            status: "available",
        }],
        provider: "ClinGen ERepo",
    })
}

fn gene_result(row: &Value) -> Result<ERepoGeneResult, BioMcpError> {
    let guidelines = row
        .get("guidelines")
        .and_then(Value::as_array)
        .and_then(|rows| rows.first());
    let agents = guidelines
        .and_then(|value| value.get("agents"))
        .and_then(Value::as_array);
    let agent = agents.and_then(|rows| rows.first());
    let mut truncated_fields = Vec::new();
    let hgvs_values = row
        .get("hgvs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    let hgvs = hgvs_values
        .iter()
        .take(3)
        .enumerate()
        .filter_map(|(index, value)| {
            bounded_preview(
                Some(value),
                &format!("hgvs[{index}]"),
                &mut truncated_fields,
            )
        })
        .collect();
    let met_evidence_codes = agent
        .and_then(|value| value.get("evidenceCodes"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|code| code.get("status").and_then(Value::as_str) == Some("Met"))
        .filter_map(|code| code.get("label").and_then(Value::as_str))
        .enumerate()
        .filter_map(|(index, value)| {
            bounded_preview(
                Some(value),
                &format!("met_evidence_codes[{index}]"),
                &mut truncated_fields,
            )
        })
        .collect();
    let caid = row
        .get("caid")
        .and_then(Value::as_str)
        .map(|value| value.strip_prefix("CAR:").unwrap_or(value));
    Ok(ERepoGeneResult {
        caid: bounded_preview(caid, "caid", &mut truncated_fields),
        gene: bounded_preview(
            row.pointer("/gene/label").and_then(Value::as_str),
            "gene",
            &mut truncated_fields,
        ),
        condition: bounded_preview(
            row.pointer("/condition/label").and_then(Value::as_str),
            "condition",
            &mut truncated_fields,
        ),
        classification: bounded_preview(
            guidelines
                .and_then(|value| value.pointer("/outcome/label"))
                .and_then(Value::as_str),
            "classification",
            &mut truncated_fields,
        ),
        guideline_label: bounded_preview(
            guidelines
                .and_then(|value| value.get("label"))
                .and_then(Value::as_str),
            "guideline_label",
            &mut truncated_fields,
        ),
        expert_panel: bounded_preview(
            agent.and_then(|value| {
                value
                    .get("affiliation")
                    .or_else(|| value.get("label"))
                    .and_then(Value::as_str)
            }),
            "expert_panel",
            &mut truncated_fields,
        ),
        published_date: bounded_preview(
            row.get("publishedDate").and_then(Value::as_str),
            "published_date",
            &mut truncated_fields,
        ),
        hgvs,
        hgvs_count: hgvs_values.len(),
        met_evidence_codes,
        truncated_fields,
    })
}

fn bounded_preview(
    value: Option<&str>,
    field: &str,
    truncated_fields: &mut Vec<String>,
) -> Option<String> {
    let value = value?;
    if value.len() > GENE_PREVIEW_BYTES {
        truncated_fields.push(field.to_owned());
        None
    } else {
        Some(value.to_owned())
    }
}

async fn retrieve_with_client(
    caids: Vec<String>,
    detail: bool,
    assertion_id: Option<&str>,
    version: Option<&str>,
    client: ERepoClient,
) -> Result<ERepoResponse, BioMcpError> {
    let mut items = Vec::with_capacity(caids.len());
    for caid in caids {
        let envelope = client.summary(&caid).await?;
        let rows = envelope
            .get("data")
            .and_then(Value::as_array)
            .ok_or_else(|| invalid("summary data must be an array"))?;
        if rows.len() > 25 {
            return Err(invalid("summary exceeded the 25 assertion bound"));
        }
        let mut assertions = rows
            .iter()
            .map(|row| summary(row, &client))
            .collect::<Result<Vec<_>, _>>()?;
        assertions.sort_by(|a, b| {
            a.assertion_id
                .cmp(&b.assertion_id)
                .then(a.doc_version.cmp(&b.doc_version))
        });
        if detail {
            let selected = select(&assertions, assertion_id, version)?;
            let selected_id = selected.assertion_id.clone();
            let detail_version = version.unwrap_or(&selected.doc_version).to_owned();
            let (envelope, bytes) = client.detail(&selected_id, &detail_version).await?;
            let guideline_bytes = client.guideline_page(&selected_id, &detail_version).await?;
            let data = envelope
                .get("data")
                .and_then(Value::as_object)
                .ok_or_else(|| invalid("detail data must be one object"))?;
            let url = client.detail_url(&selected_id, &detail_version);
            validate_detail_identity(data, &selected_id, &url)?;
            let selected_index = assertions
                .iter()
                .position(|row| {
                    row.assertion_id == selected_id && row.doc_version == selected.doc_version
                })
                .expect("selected summary exists");
            let guideline_label = parse_guideline_label(&guideline_bytes)?;
            assertions[selected_index].guideline_version =
                guideline_label.as_deref().and_then(parse_guideline_version);
            assertions[selected_index].guideline_label = guideline_label;
            assertions[selected_index].detail =
                Some(detail_projection(data, &selected_id, &url, &bytes));
        }
        items.push(ERepoItem {
            caid,
            assertions,
            complete: true,
        });
    }
    Ok(ERepoResponse {
        items,
        complete: true,
        source_status: vec![ERepoSourceStatus {
            source: "clingen_erepo",
            status: "available",
        }],
        provider: "ClinGen ERepo",
    })
}

fn summary(row: &Value, client: &ERepoClient) -> Result<ERepoAssertion, BioMcpError> {
    let assertion_id = required_string(row, "uuid")?;
    let doc_version = required_string(row, "docVersion")?;
    let versions = strings(row.get("versionsList"));
    if !versions.iter().any(|value| value == &doc_version) {
        return Err(invalid("summary docVersion is not in versionsList"));
    }
    let mut criteria = tokens(row.get("metCodes"), "met");
    let unmet_provided = row.get("unMetCodes").is_some();
    criteria.extend(tokens(row.get("unMetCodes"), "unmet"));
    Ok(ERepoAssertion {
        source_url: client.detail_url(&assertion_id, &doc_version),
        assertion_id,
        doc_version,
        guideline_label: None,
        guideline_version: None,
        versions,
        classification: string(row, "classification"),
        condition: string(row, "condition"),
        mondo_id: string(row, "mondoId"),
        moi: string(row, "moi"),
        vcep: string(row, "ep"),
        gene: string(row, "gene"),
        gene_ncbi_id: string(row, "geneNcbiId"),
        hgvs: strings(row.get("hgvs")),
        preferred_variant_title: string(row, "preferredVarTitle"),
        approved_date: string(row, "approvedDate"),
        published_date: string(row, "publishedDate"),
        retracted: row.get("retracted").and_then(Value::as_bool),
        pcer_doc_id: string(row, "PCERDocID"),
        summary_description: string(row, "summaryDesc"),
        criteria,
        unmet_codes_state: if unmet_provided {
            "provided"
        } else {
            "not_provided"
        },
        detail: None,
    })
}

fn parse_guideline_label(bytes: &[u8]) -> Result<Option<String>, BioMcpError> {
    let html = std::str::from_utf8(bytes).map_err(|_| invalid("guideline page is not UTF-8"))?;
    let document = scraper::Html::parse_document(html);
    let selector = scraper::Selector::parse("p.cspec-svi-text em")
        .expect("static ERepo guideline selector is valid");
    let labels = document
        .select(&selector)
        .map(|element| element.text().collect::<String>().trim().to_owned())
        .filter(|label| !label.is_empty())
        .collect::<Vec<_>>();
    match labels.as_slice() {
        [] => Ok(None),
        [label] => Ok(Some(label.clone())),
        _ => Err(invalid("guideline page returned multiple labels")),
    }
}

fn parse_guideline_version(label: &str) -> Option<String> {
    let mut tokens = label.split_whitespace();
    while let Some(token) = tokens.next() {
        if token.trim_matches(|ch: char| !ch.is_ascii_alphanumeric()) == "Version" {
            let candidate = tokens
                .next()?
                .trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '.' && ch != '-');
            return semver::Version::parse(candidate)
                .ok()
                .map(|version| version.to_string());
        }
    }
    None
}

fn select<'a>(
    rows: &'a [ERepoAssertion],
    assertion_id: Option<&str>,
    version: Option<&str>,
) -> Result<&'a ERepoAssertion, BioMcpError> {
    let candidates = match assertion_id {
        Some(id) => rows
            .iter()
            .filter(|row| row.assertion_id == id)
            .collect::<Vec<_>>(),
        None if rows.len() == 1 => rows.iter().collect(),
        None => {
            return Err(BioMcpError::InvalidArgument(format!(
                "variant erepo --detail requires --assertion; choices: {}",
                rows.iter()
                    .map(|row| row.assertion_id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
    };
    let row = candidates.first().copied().ok_or_else(|| {
        BioMcpError::InvalidArgument("variant erepo assertion was not found for this CAid".into())
    })?;
    if let Some(version) = version
        && !row.versions.iter().any(|candidate| candidate == version)
    {
        return Err(BioMcpError::InvalidArgument(
            "variant erepo --version must exactly match a summary versionsList value".into(),
        ));
    }
    Ok(row)
}

fn detail_projection(
    data: &serde_json::Map<String, Value>,
    uuid: &str,
    url: &str,
    bytes: &[u8],
) -> ERepoDetail {
    let provider_entity_id = data.get("id").and_then(Value::as_str).map(str::to_owned);
    let provider_at_id = data.get("@id").and_then(Value::as_str).map(str::to_owned);
    let mut criteria = Vec::new();
    for (line_index, line) in data
        .get("evidenceLine")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        for (item_index, item) in line
            .get("evidenceItem")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            if item.get("type").and_then(Value::as_str) != Some("CriterionAssessment") {
                continue;
            }
            let locator = format!("/evidenceLine/{line_index}/evidenceItem/{item_index}");
            let curator_facts = contribution_facts(item);
            let comments = curator_facts
                .iter()
                .filter_map(|contribution| contribution.get("comments"))
                .flat_map(comment_strings)
                .collect::<Vec<_>>();
            let mut pmids = structured_pmids(item, uuid, provider_entity_id.clone(), &locator);
            for comment in &comments {
                pmids.extend(comment_pmids(
                    comment,
                    uuid,
                    provider_entity_id.clone(),
                    &format!("{locator}/contribution/comments"),
                ));
            }
            let mut seen = std::collections::BTreeSet::new();
            pmids.retain(|pmid| seen.insert(pmid.pmid));
            criteria.push(ERepoDetailCriterion {
                code: item
                    .pointer("/criterion/label")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                default_strength: item
                    .pointer("/criterion/defaultStrength/label")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                statement_outcome: item
                    .pointer("/statementOutcome/label")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                comments,
                curator_facts,
                pmids,
                locator,
            });
        }
    }
    let metadata = data.get("metadata");
    ERepoDetail {
        source_url: url.into(),
        assertion_uuid: uuid.into(),
        provider_entity_id,
        provider_at_id,
        body_sha256: format!("{:x}", Sha256::digest(bytes)),
        body_bytes: bytes.len(),
        response_version: metadata
            .and_then(|value| value.get("version"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        service_version: metadata
            .and_then(|value| value.get("serviceVersion"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        template_version: metadata
            .and_then(|value| value.get("templateVersion"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        criteria,
    }
}

fn structured_pmids(
    item: &Value,
    uuid: &str,
    entity: Option<String>,
    locator: &str,
) -> Vec<ERepoPmid> {
    let mut out = Vec::new();
    collect_structured_pmids(item, uuid, entity, locator, &mut out);
    out
}
fn collect_structured_pmids(
    value: &Value,
    uuid: &str,
    entity: Option<String>,
    locator: &str,
    out: &mut Vec<ERepoPmid>,
) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if matches!(key.as_str(), "pmid" | "PMID") {
                    for pmid in comment_pmids(
                        value.as_str().unwrap_or_default(),
                        uuid,
                        entity.clone(),
                        &format!("{locator}/{key}"),
                    ) {
                        out.push(pmid);
                    }
                } else {
                    collect_structured_pmids(
                        value,
                        uuid,
                        entity.clone(),
                        &format!("{locator}/{key}"),
                        out,
                    );
                }
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                collect_structured_pmids(
                    value,
                    uuid,
                    entity.clone(),
                    &format!("{locator}/{index}"),
                    out,
                );
            }
        }
        _ => {}
    }
}
fn comment_pmids(text: &str, uuid: &str, entity: Option<String>, locator: &str) -> Vec<ERepoPmid> {
    let lower = text.to_ascii_lowercase();
    let mut out = Vec::new();
    let mut start = 0;
    while let Some(found) = lower[start..]
        .find("pmid:")
        .or_else(|| lower[start..].find("pmids:"))
    {
        let at = start + found;
        let tail = &text[at..];
        let values = tail
            .split_once(':')
            .map(|(_, values)| values)
            .unwrap_or_default();
        for token in values
            .split(|ch: char| !(ch.is_ascii_digit() || matches!(ch, ',' | ';' | ' ' | '\t')))
            .next()
            .unwrap_or_default()
            .split(|ch: char| !ch.is_ascii_digit())
        {
            if let Ok(pmid) = token.parse::<u64>()
                && pmid > 0
            {
                out.push(ERepoPmid {
                    pmid,
                    assertion_uuid: uuid.into(),
                    provider_entity_id: entity.clone(),
                    locator: locator.into(),
                });
            }
        }
        start = at + 5;
    }
    out
}
fn contribution_facts(value: &Value) -> Vec<Value> {
    let mut facts = Vec::new();
    collect_contribution_facts(value, &mut facts);
    facts
}

fn collect_contribution_facts(value: &Value, facts: &mut Vec<Value>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if key == "contribution"
                    && let Some(contributions) = value.as_array()
                {
                    facts.extend(contributions.iter().cloned());
                }
                collect_contribution_facts(value, facts);
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_contribution_facts(value, facts);
            }
        }
        _ => {}
    }
}

fn comment_strings(value: &Value) -> Vec<String> {
    match value {
        Value::String(value) => vec![value.clone()],
        Value::Array(values) => values
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect(),
        _ => Vec::new(),
    }
}
fn tokens(value: Option<&Value>, status: &'static str) -> Vec<ERepoCriterion> {
    strings(value)
        .into_iter()
        .map(|source_token| {
            let (code, explicit_strength) = source_token
                .split_once('_')
                .map_or((source_token.clone(), None), |(code, strength)| {
                    (code.into(), Some(strength.into()))
                });
            ERepoCriterion {
                source_token,
                code,
                status,
                explicit_strength,
            }
        })
        .collect()
}
fn required_string(value: &Value, key: &str) -> Result<String, BioMcpError> {
    string(value, key).ok_or_else(|| invalid(&format!("summary {key} is required")))
}
fn string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_owned)
}
fn strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}
fn validate_detail_identity(
    data: &serde_json::Map<String, Value>,
    assertion_id: &str,
    requested_url: &str,
) -> Result<(), BioMcpError> {
    let requested_path = reqwest::Url::parse(requested_url)
        .ok()
        .map(|url| url.path().to_owned());
    let returned_path = data
        .get("@id")
        .and_then(Value::as_str)
        .and_then(|id| reqwest::Url::parse(id).ok())
        .map(|url| url.path().to_owned());
    if data.get("uuid").and_then(Value::as_str) == Some(assertion_id)
        && returned_path == requested_path
    {
        Ok(())
    } else {
        Err(BioMcpError::InternalProcessing)
    }
}

fn invalid(_message: &str) -> BioMcpError {
    BioMcpError::InternalProcessing
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    #[test]
    fn receipted_pten_gene_page_is_compact_bounded_and_reports_exact_count() {
        let value: Value = serde_json::from_slice(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/clingen_erepo/pten-gene-limit-26.json"
        )))
        .expect("PTEN gene receipt");
        let page = gene_page_from_value(&value, 0, 25).expect("gene page");
        assert_eq!(page.returned, 25);
        assert!(page.has_more);
        assert_eq!(page.total, None);
        let complete_page = gene_page_from_value(&value, 0, 26).expect("complete gene page");
        assert_eq!(complete_page.total, Some(26));
        assert_eq!(page.results[0].gene.as_deref(), Some("PTEN"));
        assert!(
            page.results[0]
                .caid
                .as_deref()
                .is_some_and(|id| id.starts_with("CA"))
        );
        assert!(page.results.iter().all(|row| row.hgvs.len() <= 3));
        assert!(
            page.results
                .iter()
                .flat_map(|row| row.hgvs.iter())
                .all(|value| value.len() <= GENE_PREVIEW_BYTES)
        );
    }
    #[test]
    fn gene_preview_omits_oversized_strings_without_cutting_them() {
        let oversized = "α".repeat(129);
        let value = serde_json::json!({
            "variantInterpretations": [{
                "caid": "CAR:CA1",
                "gene": {"label": "PTEN"},
                "condition": {"label": oversized},
                "hgvs": [oversized, "NM_1:c.1A>G"],
                "guidelines": []
            }]
        });
        let page = gene_page_from_value(&value, 0, 25).expect("bounded page");
        let row = &page.results[0];
        assert_eq!(row.condition, None);
        assert_eq!(row.hgvs, vec!["NM_1:c.1A>G"]);
        assert_eq!(row.hgvs_count, 2);
        assert_eq!(row.truncated_fields, vec!["hgvs[0]", "condition"]);
    }

    #[tokio::test]
    async fn selectors_require_detail() {
        let error = retrieve(vec!["CA015543".into()], false, Some("assertion"), None)
            .await
            .expect_err("selection without detail must be rejected before a source request");
        assert!(
            error
                .to_string()
                .contains("--assertion and --version require --detail")
        );
    }

    #[test]
    fn selector_accepts_an_exact_historical_version() {
        let value: Value = serde_json::from_slice(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/clingen_erepo/atm-summary.json"
        )))
        .expect("fixture JSON");
        let client = ERepoClient::new().expect("client");
        let row = summary(&value["data"][0], &client).expect("summary");

        assert_eq!(row.doc_version, "2.0.0");
        assert!(select(&[row], None, Some("1.0.0")).is_ok());
    }

    #[test]
    fn receipted_apc_detail_preserves_the_provider_at_id() {
        let bytes = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/clingen_erepo/apc-detail.json"
        ));
        let envelope: Value = serde_json::from_slice(bytes).expect("receipt JSON");
        let data = envelope["data"].as_object().expect("receipt detail data");
        let uuid = "34ea9707-51d8-44df-818d-f69b075295c5";
        let requested_url = "https://erepo.clinicalgenome.org/evrepo/api/summary/classification/34ea9707-51d8-44df-818d-f69b075295c5/doc/sepio/version/1.0.0";

        validate_detail_identity(data, uuid, requested_url).expect("receipt identity");
        let detail = detail_projection(data, uuid, requested_url, bytes);

        assert_eq!(
            detail.provider_at_id.as_deref(),
            data.get("@id").and_then(Value::as_str)
        );
        assert_eq!(
            detail.body_sha256,
            "f6b1e4bfd2359a4d648626a87d487c4d92e5f2cc723de9347139218c03abad46"
        );
        assert!(detail.body_bytes > 0);
        assert!(
            detail
                .criteria
                .iter()
                .any(|criterion| criterion.pmids.iter().any(|pmid| pmid.pmid == 12_901_799))
        );
    }

    #[test]
    fn receipted_guideline_pages_preserve_labels_and_parse_only_semver() {
        let versioned = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/clingen_erepo/gp1ba-guideline.html"
        ));
        let legacy = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/clingen_erepo/mtdna-guideline.html"
        ));

        let versioned_label = parse_guideline_label(versioned)
            .expect("versioned receipt parses")
            .expect("versioned label");
        assert_eq!(
            versioned_label,
            "ClinGen Platelet Disorders Expert Panel Specifications to the ACMG/AMP Variant Interpretation Guidelines for GP1BA Version 1.1.0"
        );
        assert_eq!(
            parse_guideline_version(&versioned_label).as_deref(),
            Some("1.1.0")
        );

        let legacy_label = parse_guideline_label(legacy)
            .expect("legacy receipt parses")
            .expect("legacy label");
        assert_eq!(parse_guideline_version(&legacy_label), None);
        assert!(legacy_label.ends_with("Version 1_mtDNA"));
    }

    #[tokio::test]
    async fn detail_retrieval_consumes_the_summary_selected_plan() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ERepo fixture");
        let base = format!("http://{}", listener.local_addr().expect("fixture address"));
        let summary = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/clingen_erepo/apc-summary.json"
        ));
        let detail = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/clingen_erepo/apc-detail.json"
        ));
        let server = tokio::spawn(async move {
            let mut requests = Vec::new();
            let guideline = include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/testdata/sources/clingen_erepo/gp1ba-guideline.html"
            ));
            for (body, content_type) in [
                (summary, "application/json"),
                (detail, "application/json"),
                (guideline, "text/html; charset=utf-8"),
            ] {
                let (mut stream, _) = listener.accept().await.expect("accept ERepo request");
                let mut request = Vec::new();
                loop {
                    let mut chunk = [0_u8; 4096];
                    let read = stream.read(&mut chunk).await.expect("read ERepo request");
                    if read == 0 {
                        break;
                    }
                    request.extend_from_slice(&chunk[..read]);
                    if request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                        break;
                    }
                }
                requests.push(String::from_utf8_lossy(&request).into_owned());
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body,
                );
                stream
                    .write_all(response.as_bytes())
                    .await
                    .expect("write response");
            }
            requests
        });
        let client = ERepoClient::with_test_client(
            crate::sources::test_client().expect("test client"),
            base,
        );

        let response = retrieve_with_client(vec!["CA015543".into()], true, None, None, client)
            .await
            .expect("receipt-backed detail retrieval");
        assert!(response.items[0].assertions[0].detail.is_some());

        let requests = server.await.expect("ERepo fixture server");
        assert!(requests[0].starts_with(
            "GET /evrepo/api/summary/classifications?columns=caId&values=CA015543&matchTypes=exact&pgSize=25&pg=1 "
        ));
        assert!(requests[1].starts_with(
            "GET /evrepo/api/summary/classification/34ea9707-51d8-44df-818d-f69b075295c5/doc/sepio/version/1.0.0 "
        ));
        assert!(requests[2].starts_with(
            "GET /evrepo/ui/classification/34ea9707-51d8-44df-818d-f69b075295c5?version=1.0.0 "
        ));
    }

    #[test]
    fn detail_identity_uses_path_and_reports_mismatches_as_internal() {
        let requested_url = "https://erepo.clinicalgenome.org/evrepo/api/summary/classification/34ea9707-51d8-44df-818d-f69b075295c5/doc/sepio/version/1.0.0";
        let mut data = serde_json::json!({
            "uuid": "34ea9707-51d8-44df-818d-f69b075295c5",
            "@id": "https://cgerepoapi/evrepo/api/summary/classification/34ea9707-51d8-44df-818d-f69b075295c5/doc/sepio/version/1.0.0"
        });
        let detail = data.as_object().expect("detail object");

        assert!(
            validate_detail_identity(
                detail,
                "34ea9707-51d8-44df-818d-f69b075295c5",
                requested_url,
            )
            .is_ok()
        );

        data["@id"] = Value::String(
            "https://cgerepoapi/evrepo/api/summary/classification/34ea9707-51d8-44df-818d-f69b075295c5/doc/sepio/version/2.0.0".into(),
        );
        let error = validate_detail_identity(
            data.as_object().expect("detail object"),
            "34ea9707-51d8-44df-818d-f69b075295c5",
            requested_url,
        )
        .expect_err("wrong detail version must fail identity validation");

        assert!(matches!(error, BioMcpError::InternalProcessing));
    }
}
