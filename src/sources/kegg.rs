use std::borrow::Cow;

use crate::error::BioMcpError;

const KEGG_BASE: &str = "https://rest.kegg.jp";
const KEGG_API: &str = "kegg";
const KEGG_BASE_ENV: &str = "BIOMCP_KEGG_BASE";

pub struct KeggClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl KeggClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(KEGG_BASE, KEGG_BASE_ENV),
        })
    }

    #[cfg(test)]
    pub(crate) fn new_for_test(base: String) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::test_client()?,
            base: Cow::Owned(base),
        })
    }

    fn build_segment_url(&self, segments: &[&str]) -> Result<String, BioMcpError> {
        let mut url = reqwest::Url::parse(self.base.as_ref()).map_err(|err| BioMcpError::Api {
            api: KEGG_API.to_string(),
            message: format!("Invalid KEGG base URL: {err}"),
        })?;
        {
            let mut path = url.path_segments_mut().map_err(|_| BioMcpError::Api {
                api: KEGG_API.to_string(),
                message: "Invalid KEGG base URL path".to_string(),
            })?;
            path.pop_if_empty();
            for segment in segments {
                path.push(segment);
            }
        }
        Ok(url.to_string())
    }

    async fn get_text(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<String, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req).send().await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_body(resp, KEGG_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: KEGG_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        String::from_utf8(bytes).map_err(|err| BioMcpError::Api {
            api: KEGG_API.to_string(),
            message: format!("Response was not valid UTF-8: {err}"),
        })
    }

    pub async fn search_pathways(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<KeggPathwayHit>, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "KEGG query is required".into(),
            ));
        }

        let url = self.build_segment_url(&["find", "pathway", query])?;
        let body = self.get_text(self.client.get(url)).await?;
        Ok(parse_search_response(&body, limit.clamp(1, 25)))
    }

    pub async fn get_pathway(&self, pathway_id: &str) -> Result<KeggPathwayRecord, BioMcpError> {
        let pathway_id = pathway_id.trim();
        if pathway_id.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "KEGG pathway ID is required".into(),
            ));
        }

        let url = self.build_segment_url(&["get", pathway_id])?;
        let body = self.get_text(self.client.get(url)).await?;
        parse_pathway_record(&body)
    }
}

#[derive(Debug, Clone)]
pub struct KeggPathwayHit {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct KeggPathwayRecord {
    pub id: String,
    pub name: String,
    pub summary: Option<String>,
    pub genes: Vec<String>,
}

fn parse_search_response(body: &str, limit: usize) -> Vec<KeggPathwayHit> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let Some((raw_id, raw_name)) = line.split_once('\t') else {
            continue;
        };
        let Some(name) = raw_name
            .trim()
            .split(" - ")
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let Some(id) = normalize_search_pathway_id(raw_id, raw_name) else {
            continue;
        };
        if !seen.insert(id.clone()) {
            continue;
        }

        out.push(KeggPathwayHit {
            id,
            name: name.to_string(),
        });
        if out.len() >= limit {
            break;
        }
    }

    out
}

fn parse_pathway_record(body: &str) -> Result<KeggPathwayRecord, BioMcpError> {
    let mut id = None;
    let mut name = None;
    let mut description = String::new();
    let mut genes = Vec::new();
    let mut active_field = String::new();

    for line in body.lines() {
        if line.trim() == "///" {
            break;
        }

        let (field, value) = split_flat_file_line(line);
        if let Some(field) = field {
            active_field = field.to_string();
        }

        let value = value.trim();
        if value.is_empty() {
            continue;
        }

        match active_field.as_str() {
            "ENTRY" => {
                let candidate = value.split_whitespace().next().unwrap_or("").trim();
                if is_human_pathway_id(candidate) {
                    id = Some(candidate.to_string());
                }
            }
            "NAME" if name.is_none() => {
                let cleaned = value.trim_end_matches(';').trim();
                if !cleaned.is_empty() {
                    name = Some(cleaned.to_string());
                }
            }
            "DESCRIPTION" => {
                if !description.is_empty() {
                    description.push(' ');
                }
                description.push_str(value);
            }
            "GENE" => {
                if let Some(symbol) = parse_gene_symbol(value) {
                    genes.push(symbol);
                }
            }
            _ => {}
        }
    }

    let id = id.ok_or_else(|| BioMcpError::Api {
        api: KEGG_API.to_string(),
        message: "KEGG pathway record missing ENTRY".to_string(),
    })?;
    let name = name.ok_or_else(|| BioMcpError::Api {
        api: KEGG_API.to_string(),
        message: "KEGG pathway record missing NAME".to_string(),
    })?;

    Ok(KeggPathwayRecord {
        id,
        name,
        summary: (!description.trim().is_empty()).then(|| description.trim().to_string()),
        genes: dedupe_preserving_order(genes),
    })
}

fn split_flat_file_line(line: &str) -> (Option<&str>, &str) {
    let trimmed = line.trim_start();
    let field_len = trimmed
        .chars()
        .take_while(|ch| ch.is_ascii_uppercase())
        .count();
    let has_field = field_len > 0
        && trimmed
            .chars()
            .nth(field_len)
            .is_some_and(char::is_whitespace);
    if !has_field {
        return (None, trimmed);
    }

    let split_at = trimmed
        .char_indices()
        .nth(12)
        .map(|(idx, _)| idx)
        .unwrap_or(trimmed.len());
    let (raw_field, raw_value) = trimmed.split_at(split_at);
    let field = raw_field.trim();
    let field = (!field.is_empty()).then_some(field);
    (field, raw_value)
}

fn parse_gene_symbol(value: &str) -> Option<String> {
    let before_annotation = value.split(';').next()?.trim();
    if before_annotation.is_empty() {
        return None;
    }

    let mut parts = before_annotation.split_whitespace();
    let first = parts.next()?.trim();
    let second = parts.next().map(str::trim).filter(|part| !part.is_empty());

    let symbol = if first.chars().all(|ch| ch.is_ascii_digit()) {
        second?
    } else {
        first
    };

    Some(symbol.to_string())
}

fn dedupe_preserving_order(values: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for value in values {
        if out
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(&value))
        {
            continue;
        }
        out.push(value);
    }
    out
}

fn normalize_search_pathway_id(raw_id: &str, raw_name: &str) -> Option<String> {
    let id = raw_id.trim().strip_prefix("path:")?.trim();
    if is_human_pathway_id(id) {
        return Some(id.to_string());
    }
    if is_reference_map_id(id) && !raw_name.contains(" - ") {
        return Some(format!("hsa{}", &id[3..]));
    }
    None
}

pub(crate) fn is_human_pathway_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 8 && bytes.starts_with(b"hsa") && bytes[3..].iter().all(u8::is_ascii_digit)
}

fn is_reference_map_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 8 && bytes.starts_with(b"map") && bytes[3..].iter().all(u8::is_ascii_digit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn parse_search_response_keeps_human_rows_only() {
        let rows = parse_search_response(
            "path:hsa04010\tMAPK signaling pathway - Homo sapiens (human)\n\
             path:map04010\tMAPK signaling pathway - Reference pathway\n\
             path:mmu04010\tMAPK signaling pathway - Mus musculus (mouse)\n\
             bad line\n",
            10,
        );

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "hsa04010");
        assert_eq!(rows[0].name, "MAPK signaling pathway");
    }

    #[test]
    fn parse_search_response_normalizes_bare_reference_map_to_human() {
        // Live KEGG /find/pathway/MAPK returns a bare map##### row without an
        // organism suffix when no hsa##### row is present (documented deviation
        // from design in dev-log). This test guards that normalization path.
        let rows = parse_search_response("path:map04010\tMAPK signaling pathway\n", 10);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "hsa04010");
        assert_eq!(rows[0].name, "MAPK signaling pathway");
    }

    #[test]
    fn parse_search_response_dedupes_normalized_and_explicit_human_id() {
        // If the API returns both a bare map##### and an hsa##### for the same
        // pathway, the normalized entry arrives first and the explicit hsa#####
        // must be dropped as a duplicate.
        let rows = parse_search_response(
            "path:map04010\tMAPK signaling pathway\n\
             path:hsa04010\tMAPK signaling pathway - Homo sapiens (human)\n",
            10,
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "hsa04010");
    }

    #[test]
    fn parse_pathway_record_extracts_summary_and_genes() {
        let record = parse_pathway_record(
            "ENTRY       hsa05200           Pathway\n\
             NAME        Pathways in cancer\n\
             DESCRIPTION Cancer overview pathway.\n\
             GENE        673    BRAF; B-Raf proto-oncogene\n\
                         1956   EGFR; epidermal growth factor receptor\n\
             ///\n",
        )
        .expect("record");

        assert_eq!(record.id, "hsa05200");
        assert_eq!(record.name, "Pathways in cancer");
        assert_eq!(record.summary.as_deref(), Some("Cancer overview pathway."));
        assert_eq!(record.genes, vec!["BRAF".to_string(), "EGFR".to_string()]);
    }

    #[tokio::test]
    async fn search_pathways_reads_plain_text_body() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/find/pathway/MAPK"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string(
                    "path:hsa04010\tMAPK signaling pathway - Homo sapiens (human)\n",
                ),
            )
            .mount(&server)
            .await;

        let client = KeggClient::new_for_test(server.uri()).expect("client");
        let rows = client.search_pathways("MAPK", 5).await.expect("rows");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "hsa04010");
    }

    #[tokio::test]
    async fn get_pathway_parses_flat_file_record() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/get/hsa05200"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "ENTRY       hsa05200           Pathway\n\
                 NAME        Pathways in cancer\n\
                 DESCRIPTION Cancer overview pathway.\n\
                 GENE        673    BRAF; B-Raf proto-oncogene\n\
                             1956   EGFR; epidermal growth factor receptor\n\
                 ///\n",
            ))
            .mount(&server)
            .await;

        let client = KeggClient::new_for_test(server.uri()).expect("client");
        let record = client.get_pathway("hsa05200").await.expect("record");
        assert_eq!(record.id, "hsa05200");
        assert_eq!(record.genes, vec!["BRAF".to_string(), "EGFR".to_string()]);
    }
}
