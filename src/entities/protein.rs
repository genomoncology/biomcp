use std::sync::OnceLock;

use crate::entities::SearchPage;
use crate::entities::section_outcome::{SectionOutcome, SectionOutcomes};
use crate::entities::source_state_registry::outcome_keys;
use crate::error::BioMcpError;
use crate::sources::complexportal::{ComplexPortalClient, ComplexPortalComplex};
use crate::sources::interpro::InterProClient;
use crate::sources::mygene::MyGeneClient;
use crate::sources::string::StringClient;
use crate::sources::uniprot::UniProtClient;
use crate::transform;
use regex::Regex;
use serde::{Deserialize, Serialize};

fn default_protein_section_outcomes() -> SectionOutcomes {
    SectionOutcomes::with_keys(&outcome_keys("protein"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Protein {
    #[serde(
        default = "default_protein_section_outcomes",
        deserialize_with = "deserialize_protein_section_outcomes"
    )]
    pub section_outcomes: SectionOutcomes,
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
    #[serde(default)]
    pub complexes: Vec<ProteinComplex>,
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
#[serde(rename_all = "lowercase")]
pub enum ProteinComplexCuration {
    Curated,
    Predicted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProteinComplexComponent {
    pub accession: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stoichiometry: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProteinComplex {
    pub accession: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub curation: ProteinComplexCuration,
    #[serde(default)]
    pub components: Vec<ProteinComplexComponent>,
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
const PROTEIN_SECTION_COMPLEXES: &str = "complexes";
const PROTEIN_SECTION_STRUCTURES: &str = "structures";
const PROTEIN_SECTION_ALL: &str = "all";
pub(crate) const PROTEIN_OUTCOME_KEYS: &[&str] = &[
    PROTEIN_SECTION_DOMAINS,
    PROTEIN_SECTION_INTERACTIONS,
    PROTEIN_SECTION_COMPLEXES,
    PROTEIN_SECTION_STRUCTURES,
];
fn deserialize_protein_section_outcomes<'de, D>(
    deserializer: D,
) -> Result<SectionOutcomes, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let outcomes = SectionOutcomes::deserialize(deserializer)?;
    outcomes
        .validate_keys(&outcome_keys("protein"))
        .map_err(serde::de::Error::custom)?;
    Ok(outcomes)
}

const DEFAULT_COMPLEX_LIMIT: usize = 10;
const DEFAULT_STRUCTURE_LIMIT: usize = 10;
const MAX_STRUCTURE_LIMIT: usize = 100;

pub const PROTEIN_SECTION_NAMES: &[&str] = &[
    PROTEIN_SECTION_DOMAINS,
    PROTEIN_SECTION_INTERACTIONS,
    PROTEIN_SECTION_COMPLEXES,
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

pub(crate) fn is_uniprot_accession(value: &str) -> bool {
    uniprot_accession_re().is_match(value.trim())
}

pub(crate) async fn resolve_accession(value: &str) -> Result<String, BioMcpError> {
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
    include_complexes: bool,
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
            PROTEIN_SECTION_COMPLEXES => out.include_complexes = true,
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
        out.include_complexes = true;
        out.include_structures = true;
    }

    Ok(out)
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
        protein.section_outcomes.complete(
            PROTEIN_SECTION_STRUCTURES,
            if protein.structures.is_empty() {
                SectionOutcome::empty("UniProt")
            } else {
                SectionOutcome::data("UniProt")
            },
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

    let complexes_fut = async {
        if !parsed_sections.include_complexes {
            return Ok::<Vec<ProteinComplex>, BioMcpError>(Vec::new());
        }

        let rows = ComplexPortalClient::new()?
            .complexes(&protein.accession, DEFAULT_COMPLEX_LIMIT)
            .await?;
        Ok(rows
            .into_iter()
            .map(map_complexportal_complex)
            .collect::<Vec<_>>())
    };

    let (domains_res, interactions_res, complexes_res) =
        tokio::join!(domains_fut, interactions_fut, complexes_fut);

    if parsed_sections.include_domains {
        match domains_res {
            Ok(domains) => {
                protein.section_outcomes.complete(
                    PROTEIN_SECTION_DOMAINS,
                    if domains.is_empty() {
                        SectionOutcome::empty("InterPro")
                    } else {
                        SectionOutcome::data("InterPro")
                    },
                );
                protein.domains = domains;
            }
            Err(_) => {
                protein.section_outcomes.complete(
                    PROTEIN_SECTION_DOMAINS,
                    SectionOutcome::unavailable("InterPro protein domains are unavailable."),
                );
            }
        }
    }

    if parsed_sections.include_interactions {
        match interactions_res {
            Ok(rows) => {
                protein.section_outcomes.complete(
                    PROTEIN_SECTION_INTERACTIONS,
                    if rows.is_empty() {
                        SectionOutcome::empty("STRING")
                    } else {
                        SectionOutcome::data("STRING")
                    },
                );
                protein.interactions = rows;
            }
            Err(_) => {
                protein.section_outcomes.complete(
                    PROTEIN_SECTION_INTERACTIONS,
                    SectionOutcome::unavailable("STRING protein interactions are unavailable."),
                );
            }
        }
    }

    if parsed_sections.include_complexes {
        match complexes_res {
            Ok(rows) => {
                protein
                    .section_outcomes
                    .complete(PROTEIN_SECTION_COMPLEXES, complexportal_outcome(&rows));
                protein.complexes = rows;
            }
            Err(_) => {
                protein.section_outcomes.complete(
                    PROTEIN_SECTION_COMPLEXES,
                    SectionOutcome::unavailable("ComplexPortal protein complexes are unavailable."),
                );
            }
        }
    }

    Ok(protein)
}

fn complexportal_outcome(rows: &[ProteinComplex]) -> SectionOutcome {
    if rows.is_empty() {
        SectionOutcome::empty("Complex Portal")
    } else {
        SectionOutcome::data("Complex Portal")
    }
}

fn map_complexportal_complex(row: ComplexPortalComplex) -> ProteinComplex {
    ProteinComplex {
        accession: row.accession,
        name: row.name,
        description: row.description,
        curation: if row.predicted_complex {
            ProteinComplexCuration::Predicted
        } else {
            ProteinComplexCuration::Curated
        },
        components: row
            .participants
            .into_iter()
            .map(|participant| ProteinComplexComponent {
                accession: participant.accession,
                name: participant.name,
                stoichiometry: participant.stoichiometry,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::complexportal::ComplexPortalParticipant;

    #[test]
    fn parse_sections_supports_all_and_reports_unknown_values() {
        let flags = parse_sections(&["complexes".to_string()]).unwrap();
        assert!(flags.include_complexes);
        assert!(!flags.include_domains);
        assert!(!flags.include_interactions);
        assert!(!flags.include_structures);

        let flags = parse_sections(&["all".to_string()]).unwrap();
        assert!(flags.include_complexes);
        assert!(flags.include_domains);
        assert!(flags.include_interactions);
        assert!(flags.include_structures);

        let err = parse_sections(&["unexpected".to_string()]).unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[test]
    fn complexportal_outcome_uses_registered_provider_label() {
        let outcome = complexportal_outcome(&[]);
        assert_eq!(outcome.sources(), &["Complex Portal"]);
        assert!(crate::entities::source_state_registry::allows_sources(
            "protein",
            PROTEIN_SECTION_COMPLEXES,
            outcome.sources()
        ));
    }

    #[test]
    fn map_complexportal_complex_uses_explicit_curation_and_components() {
        let row = ComplexPortalComplex {
            accession: "CPX-1234".to_string(),
            name: "BRAF signaling complex".to_string(),
            description: Some("Complex description".to_string()),
            predicted_complex: true,
            participants: vec![ComplexPortalParticipant {
                accession: "P15056".to_string(),
                name: "BRAF".to_string(),
                stoichiometry: Some("minValue: 1, maxValue: 1".to_string()),
            }],
        };

        let mapped = map_complexportal_complex(row);
        assert!(matches!(mapped.curation, ProteinComplexCuration::Predicted));
        assert_eq!(mapped.components.len(), 1);
        assert_eq!(mapped.components[0].accession, "P15056");
        assert_eq!(mapped.components[0].name, "BRAF");
    }

    #[tokio::test]
    async fn search_rejects_empty_query() {
        let err = search_page(" ", 5, 0, None, false, false, None, None)
            .await
            .unwrap_err();
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

    fn test_protein() -> Protein {
        Protein {
            section_outcomes: default_protein_section_outcomes(),
            accession: "P15056".to_string(),
            entry_id: Some("BRAF_HUMAN".to_string()),
            name: "Serine/threonine-protein kinase B-raf".to_string(),
            gene_symbol: Some("BRAF".to_string()),
            organism: Some("Homo sapiens".to_string()),
            length: Some(766),
            function: None,
            structures: Vec::new(),
            structure_count: None,
            domains: Vec::new(),
            interactions: Vec::new(),
            complexes: Vec::new(),
        }
    }

    fn injected_protein_failure(source: &str, kind: &str) -> BioMcpError {
        match kind {
            "connection-refused" => BioMcpError::Api {
                api: source.to_string(),
                message: "connection refused".to_string(),
            },
            "timeout" => BioMcpError::SourceUnavailable {
                source_name: source.to_string(),
                reason: "request timed out".to_string(),
                suggestion: "retry".to_string(),
            },
            "malformed-body" => BioMcpError::ApiJson {
                api: source.to_string(),
                source: serde_json::from_str::<serde_json::Value>("{")
                    .expect_err("fixture body must be malformed"),
            },
            other => panic!("unknown injected failure: {other}"),
        }
    }

    fn assert_protein_outcome(
        protein: &Protein,
        key: &str,
        expected: crate::entities::section_outcome::SectionOutcomeState,
        successful_source: &str,
    ) {
        use crate::entities::section_outcome::SectionOutcomeState;

        let outcome = protein
            .section_outcomes
            .get(key)
            .expect("registered protein outcome");
        assert_eq!(outcome.outcome(), expected, "wrong outcome for {key}");
        if expected == SectionOutcomeState::Unavailable {
            assert!(outcome.sources().is_empty());
            assert_ne!(outcome.outcome(), SectionOutcomeState::Empty);
        } else {
            assert_eq!(outcome.sources(), &[successful_source.to_string()]);
        }
    }

    #[test]
    fn interpro_and_string_failure_state_matrix() {
        use crate::entities::section_outcome::SectionOutcomeState;

        for failure in ["connection-refused", "timeout", "malformed-body"] {
            let mut protein = test_protein();
            apply_domains_result(
                &mut protein,
                Err::<Vec<ProteinDomain>, _>(injected_protein_failure("InterPro", failure)),
            );
            assert!(protein.domains.is_empty());
            assert_protein_outcome(
                &protein,
                PROTEIN_SECTION_DOMAINS,
                SectionOutcomeState::Unavailable,
                "InterPro",
            );

            let mut protein = test_protein();
            apply_protein_interactions_result(
                &mut protein,
                Err::<Vec<ProteinInteraction>, _>(injected_protein_failure("STRING", failure)),
            );
            assert!(protein.interactions.is_empty());
            assert_protein_outcome(
                &protein,
                PROTEIN_SECTION_INTERACTIONS,
                SectionOutcomeState::Unavailable,
                "STRING",
            );
        }

        let mut empty = test_protein();
        apply_domains_result(&mut empty, Ok(Vec::new()));
        assert_protein_outcome(
            &empty,
            PROTEIN_SECTION_DOMAINS,
            SectionOutcomeState::Empty,
            "InterPro",
        );
        let mut data = test_protein();
        apply_domains_result(
            &mut data,
            Ok(vec![ProteinDomain {
                accession: "IPR000719".to_string(),
                name: Some("Protein kinase domain".to_string()),
                domain_type: Some("domain".to_string()),
            }]),
        );
        assert_eq!(data.domains[0].accession, "IPR000719");
        assert_protein_outcome(
            &data,
            PROTEIN_SECTION_DOMAINS,
            SectionOutcomeState::Data,
            "InterPro",
        );

        let mut empty = test_protein();
        apply_protein_interactions_result(&mut empty, Ok(Vec::new()));
        assert_protein_outcome(
            &empty,
            PROTEIN_SECTION_INTERACTIONS,
            SectionOutcomeState::Empty,
            "STRING",
        );
        let mut data = test_protein();
        apply_protein_interactions_result(
            &mut data,
            Ok(vec![ProteinInteraction {
                partner: "MAP2K1".to_string(),
                score: Some(0.99),
            }]),
        );
        assert_eq!(data.interactions[0].partner, "MAP2K1");
        assert_protein_outcome(
            &data,
            PROTEIN_SECTION_INTERACTIONS,
            SectionOutcomeState::Data,
            "STRING",
        );
    }
}
