use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::interpro::InterProClient;
use crate::sources::mygene::MyGeneClient;
use crate::sources::string::StringClient;
use crate::sources::uniprot::UniProtClient;
use crate::transform;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Protein {
    pub accession: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_id: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gene_symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organism: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,
    #[serde(default)]
    pub structures: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structure_count: Option<usize>,
    #[serde(default)]
    pub domains: Vec<ProteinDomain>,
    #[serde(default)]
    pub interactions: Vec<ProteinInteraction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProteinDomain {
    pub accession: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProteinInteraction {
    pub partner: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProteinSearchResult {
    pub accession: String,
    pub uniprot_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gene_symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub species: Option<String>,
}

const PROTEIN_SECTION_DOMAINS: &str = "domains";
const PROTEIN_SECTION_INTERACTIONS: &str = "interactions";
const PROTEIN_SECTION_STRUCTURES: &str = "structures";
const PROTEIN_SECTION_ALL: &str = "all";
const DEFAULT_STRUCTURE_LIMIT: usize = 10;
const MAX_STRUCTURE_LIMIT: usize = 100;

pub const PROTEIN_SECTION_NAMES: &[&str] = &[
    PROTEIN_SECTION_DOMAINS,
    PROTEIN_SECTION_INTERACTIONS,
    PROTEIN_SECTION_STRUCTURES,
    PROTEIN_SECTION_ALL,
];

fn validate_structure_limit(limit: usize) -> Result<usize, BioMcpError> {
    if limit == 0 || limit > MAX_STRUCTURE_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "Protein structures --limit must be between 1 and {MAX_STRUCTURE_LIMIT}"
        )));
    }
    Ok(limit)
}

fn paginate_structures(rows: Vec<String>, limit: usize, offset: usize) -> Vec<String> {
    rows.into_iter().skip(offset).take(limit).collect()
}

fn uniprot_accession_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^(?:[OPQ][0-9][A-Z0-9]{3}[0-9]|[A-NR-Z][0-9](?:[A-Z][A-Z0-9]{2}[0-9]){1,2})(?:-\d+)?$")
            .expect("valid uniprot accession regex")
    })
}

fn is_uniprot_accession(value: &str) -> bool {
    uniprot_accession_re().is_match(value.trim())
}

async fn resolve_accession(value: &str) -> Result<String, BioMcpError> {
    let value = value.trim();
    if is_uniprot_accession(value) {
        return Ok(value.to_string());
    }

    let client = MyGeneClient::new()?;
    match client.resolve_uniprot_accession(value).await {
        Ok(accession) => Ok(accession),
        Err(BioMcpError::NotFound { .. }) => Err(BioMcpError::NotFound {
            entity: "protein".into(),
            id: value.to_string(),
            suggestion: format!("Try searching: biomcp search protein -q {value}"),
        }),
        Err(BioMcpError::InvalidArgument(_)) => Err(BioMcpError::InvalidArgument(
            "Protein input must be a UniProt accession or HGNC symbol. Examples: biomcp get protein P15056, biomcp get protein BRAF".into(),
        )),
        Err(err) => Err(err),
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct ProteinSections {
    include_domains: bool,
    include_interactions: bool,
    include_structures: bool,
}

fn parse_sections(sections: &[String]) -> Result<ProteinSections, BioMcpError> {
    let mut out = ProteinSections::default();
    let mut include_all = false;

    for raw in sections {
        let section = raw.trim().to_ascii_lowercase();
        if section.is_empty() {
            continue;
        }
        if section == "--json" || section == "-j" {
            continue;
        }

        match section.as_str() {
            PROTEIN_SECTION_DOMAINS => out.include_domains = true,
            PROTEIN_SECTION_INTERACTIONS => out.include_interactions = true,
            PROTEIN_SECTION_STRUCTURES => out.include_structures = true,
            PROTEIN_SECTION_ALL => include_all = true,
            _ => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown section \"{section}\" for protein. Available: {}",
                    PROTEIN_SECTION_NAMES.join(", ")
                )));
            }
        }
    }

    if include_all {
        out.include_domains = true;
        out.include_interactions = true;
        out.include_structures = true;
    }

    Ok(out)
}

#[allow(dead_code)]
pub async fn search(
    query: &str,
    limit: usize,
    all_species: bool,
) -> Result<Vec<ProteinSearchResult>, BioMcpError> {
    Ok(
        search_page(query, limit, 0, None, all_species, false, None, None)
            .await?
            .results,
    )
}

pub fn search_query_summary(
    query: &str,
    reviewed: bool,
    disease: Option<&str>,
    existence: Option<u8>,
    all_species: bool,
) -> String {
    let mut parts = Vec::new();
    let query = query.trim();
    if !query.is_empty() {
        parts.push(query.to_string());
    }
    // Reviewed entries are the default safety mode.
    if reviewed || !all_species {
        parts.push("reviewed=true".to_string());
    }
    if let Some(disease) = disease.map(str::trim).filter(|v| !v.is_empty()) {
        parts.push(format!("disease={disease}"));
    }
    if let Some(existence) = existence {
        parts.push(format!("existence={existence}"));
    }
    if all_species {
        parts.push("all_species=true".to_string());
    }
    parts.join(", ")
}

#[allow(clippy::too_many_arguments)]
pub async fn search_page(
    query: &str,
    limit: usize,
    offset: usize,
    next_page: Option<String>,
    all_species: bool,
    reviewed: bool,
    disease: Option<&str>,
    existence: Option<u8>,
) -> Result<SearchPage<ProteinSearchResult>, BioMcpError> {
    let query = query.trim();
    if query.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Query is required. Example: biomcp search protein -q kinase".into(),
        ));
    }

    if let Some(level) = existence
        && !(1..=5).contains(&level)
    {
        return Err(BioMcpError::InvalidArgument(
            "--existence must be an integer from 1 to 5".into(),
        ));
    }

    let mut scoped_terms = vec![format!("({query})")];
    if !all_species {
        scoped_terms.push("organism_id:9606".to_string());
    }
    if reviewed || !all_species {
        scoped_terms.push("reviewed:true".to_string());
    }
    if let Some(disease) = disease.map(str::trim).filter(|v| !v.is_empty()) {
        let disease = disease.replace('"', "\\\"");
        scoped_terms.push(format!("cc_disease:\"{disease}\""));
    }
    if let Some(level) = existence {
        scoped_terms.push(format!("existence:{level}"));
    }
    let scoped_query = scoped_terms.join(" AND ");

    let client = UniProtClient::new()?;
    if next_page
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
    {
        let page = client
            .search(&scoped_query, limit.clamp(1, 25), 0, next_page.as_deref())
            .await?;
        return Ok(SearchPage::cursor(
            page.results
                .into_iter()
                .map(transform::protein::from_uniprot_search_record)
                .collect(),
            page.total,
            page.next_page_token,
        ));
    }

    let limit = limit.clamp(1, 100);
    const API_PAGE_SIZE: usize = 25;
    const MAX_PAGE_FETCHES: usize = 50;
    let mut rows: Vec<ProteinSearchResult> = Vec::with_capacity(limit.min(25));
    let mut total: Option<usize> = None;
    let mut remaining_skip = offset;
    let mut page_token: Option<String> = None;
    let mut exhausted = false;

    for fetched_pages in 0..MAX_PAGE_FETCHES {
        if fetched_pages == 20 {
            tracing::warn!(
                "protein search exceeded 20 API page fetches, continuing up to {MAX_PAGE_FETCHES}"
            );
        }
        let page = client
            .search(&scoped_query, API_PAGE_SIZE, 0, page_token.as_deref())
            .await?;
        if total.is_none() {
            total = page.total;
            if total.is_some_and(|value| offset >= value) {
                exhausted = true;
                break;
            }
        }
        let page_count = page.results.len();
        if page_count == 0 {
            exhausted = true;
            break;
        }

        let mut consumed = 0usize;
        for row in page.results {
            consumed = consumed.saturating_add(1);
            if remaining_skip > 0 {
                remaining_skip -= 1;
                continue;
            }
            if rows.len() < limit {
                rows.push(transform::protein::from_uniprot_search_record(row));
            }
            if rows.len() >= limit {
                break;
            }
        }

        if rows.len() >= limit {
            if consumed >= page_count {
                page_token = page.next_page_token;
            } else {
                // Mid-page stops cannot be represented as a cursor.
                page_token = None;
            }
            break;
        }

        page_token = page.next_page_token;
        if page_token.is_none() {
            exhausted = true;
            break;
        }
    }

    if !exhausted && rows.len() < limit && remaining_skip > 0 {
        return Err(BioMcpError::InvalidArgument(format!(
            "--offset {} exceeds the maximum walkable range ({} records). Use --next-page for deep pagination.",
            offset,
            MAX_PAGE_FETCHES * API_PAGE_SIZE,
        )));
    }

    let resolved_total = total.or_else(|| Some(offset.saturating_add(rows.len())));
    let next = if exhausted { None } else { page_token };
    Ok(SearchPage::cursor(rows, resolved_total, next))
}

pub async fn get(accession: &str, sections: &[String]) -> Result<Protein, BioMcpError> {
    get_with_structure_limit(accession, sections, None, None).await
}

pub async fn get_with_structure_limit(
    accession: &str,
    sections: &[String],
    structure_limit: Option<usize>,
    structure_offset: Option<usize>,
) -> Result<Protein, BioMcpError> {
    let query = accession.trim();
    if query.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Protein accession is required. Example: biomcp get protein P15056".into(),
        ));
    }

    let parsed_sections = parse_sections(sections)?;
    let accession = resolve_accession(query).await?;

    let uniprot = UniProtClient::new()?;
    let record = uniprot.get_record(&accession).await?;
    let mut protein = transform::protein::from_uniprot_record_base(record.clone());

    if parsed_sections.include_structures {
        let structure_limit =
            validate_structure_limit(structure_limit.unwrap_or(DEFAULT_STRUCTURE_LIMIT))?;
        let structure_offset = structure_offset.unwrap_or(0);
        let fetch_limit = structure_limit
            .saturating_add(structure_offset)
            .max(structure_limit);
        protein.structure_count = Some(record.structure_count());
        protein.structures = paginate_structures(
            record.structure_summaries(fetch_limit),
            structure_limit,
            structure_offset,
        );
    }

    let interaction_query = protein
        .gene_symbol
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or(&protein.accession)
        .to_string();

    let domains_fut = async {
        if !parsed_sections.include_domains {
            return Ok::<Vec<ProteinDomain>, BioMcpError>(Vec::new());
        }

        let domains = InterProClient::new()?
            .domains(&protein.accession, 20)
            .await?;
        Ok(domains
            .into_iter()
            .map(|d| ProteinDomain {
                accession: d.accession,
                name: d.name,
                domain_type: d.domain_type,
            })
            .collect::<Vec<_>>())
    };

    let interactions_fut = async {
        if !parsed_sections.include_interactions {
            return Ok::<Vec<ProteinInteraction>, BioMcpError>(Vec::new());
        }

        let rows = StringClient::new()?
            .interactions(&interaction_query, 9606, 10)
            .await?;

        let mut interactions = Vec::new();
        for r in rows {
            let a = r.preferred_name_a.unwrap_or_default();
            let b = r.preferred_name_b.unwrap_or_default();
            let partner = if a.eq_ignore_ascii_case(&interaction_query) {
                b
            } else {
                a
            };
            let partner = partner.trim().to_string();
            if partner.is_empty() {
                continue;
            }
            if interactions
                .iter()
                .any(|v: &ProteinInteraction| v.partner.eq_ignore_ascii_case(&partner))
            {
                continue;
            }
            interactions.push(ProteinInteraction {
                partner,
                score: r.score,
            });
        }
        interactions.sort_by(|a, b| {
            b.score
                .unwrap_or_default()
                .partial_cmp(&a.score.unwrap_or_default())
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.partner.cmp(&b.partner))
        });
        interactions.truncate(15);
        Ok(interactions)
    };

    let (domains_res, interactions_res) = tokio::join!(domains_fut, interactions_fut);

    match domains_res {
        Ok(domains) => protein.domains = domains,
        Err(err) => warn!("InterPro unavailable for protein domains: {err}"),
    }

    match interactions_res {
        Ok(rows) => protein.interactions = rows,
        Err(err) => warn!("STRING unavailable for protein interactions: {err}"),
    }

    Ok(protein)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sections_supports_all_and_reports_unknown_values() {
        let flags = parse_sections(&["all".to_string()]).unwrap();
        assert!(flags.include_domains);
        assert!(flags.include_interactions);
        assert!(flags.include_structures);

        let err = parse_sections(&["unexpected".to_string()]).unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn search_rejects_empty_query() {
        let err = search(" ", 5, false).await.unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[test]
    fn uniprot_accession_validation_accepts_accessions_and_rejects_symbols() {
        assert!(is_uniprot_accession("P15056"));
        assert!(is_uniprot_accession("Q9Y243"));
        assert!(!is_uniprot_accession("BRAF"));
        assert!(!is_uniprot_accession("BRAF V600E"));
    }

    #[test]
    fn validate_structure_limit_enforces_bounds() {
        assert_eq!(validate_structure_limit(1).unwrap(), 1);
        assert_eq!(validate_structure_limit(25).unwrap(), 25);
        assert!(validate_structure_limit(0).is_err());
        assert!(validate_structure_limit(MAX_STRUCTURE_LIMIT + 1).is_err());
    }

    #[test]
    fn paginate_structures_applies_offset_then_limit() {
        let rows = vec![
            "1abc".to_string(),
            "2abc".to_string(),
            "3abc".to_string(),
            "4abc".to_string(),
        ];
        let page = paginate_structures(rows, 2, 1);
        assert_eq!(page, vec!["2abc".to_string(), "3abc".to_string()]);
    }
}
