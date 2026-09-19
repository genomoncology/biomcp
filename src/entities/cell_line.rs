//! The cell-line entity. Cellosaurus is the identity source, and the CVCL
//! accession is the join key every other cell line source uses.

use serde::{Deserialize, Serialize};

use crate::entities::section_outcome::{SectionOutcome, SectionOutcomes};
use crate::entities::source_state_registry::outcome_keys;
use crate::error::BioMcpError;
pub(crate) mod chembl;
pub(crate) mod pharmacodb;

use self::chembl::CellLineChembl;
use crate::entities::pharmacodb::PharmacoDbCounts;
use crate::sources::cellosaurus::{
    CARD_FIELDS, CellLineXrefDatabase, CellosaurusClient, CellosaurusRecord, CellosaurusSearchPage,
};

pub(crate) const CELL_LINE_SOURCE: &str = "Cellosaurus";
pub(crate) const CELL_LINE_CITATION: &str = "Cite Bairoch A. J. Biomol. Tech. 29:25-38 (2018).";

const CELL_LINE_SECTION_VARIANTS: &str = "variants";
const CELL_LINE_SECTION_XREFS: &str = "xrefs";
const CELL_LINE_SECTION_CHEMBL: &str = "chembl";
pub(crate) const CELL_LINE_SECTION_DRUG_RESPONSE: &str = "drug_response";
const CELL_LINE_SECTION_ALL: &str = "all";

pub const CELL_LINE_SECTION_NAMES: &[&str] = &[
    CELL_LINE_SECTION_VARIANTS,
    CELL_LINE_SECTION_XREFS,
    CELL_LINE_SECTION_CHEMBL,
    CELL_LINE_SECTION_DRUG_RESPONSE,
    CELL_LINE_SECTION_ALL,
];

/// The note a filled search window adds.
pub(crate) const CELL_LINE_FULL_WINDOW_NOTE: &str = "Cellosaurus returned 1000 rows; an exact match may lie past this window. \
Use a more specific name or the CVCL accession.";

fn default_cell_line_section_outcomes() -> SectionOutcomes {
    SectionOutcomes::with_keys(&outcome_keys("cell_line"))
}

fn deserialize_cell_line_section_outcomes<'de, D>(
    deserializer: D,
) -> Result<SectionOutcomes, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let outcomes = SectionOutcomes::deserialize(deserializer)?;
    outcomes
        .validate_keys(&outcome_keys("cell_line"))
        .map_err(serde::de::Error::custom)?;
    Ok(outcomes)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellLine {
    #[serde(
        default = "default_cell_line_section_outcomes",
        deserialize_with = "deserialize_cell_line_section_outcomes"
    )]
    pub section_outcomes: SectionOutcomes,
    pub source: String,
    pub data_as_of: String,
    pub data_as_of_kind: String,
    pub accession: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secondary_accessions: Vec<String>,
    pub rrid: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub synonyms: Vec<String>,
    #[serde(default)]
    pub species: Vec<CellLineSpecies>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diseases: Vec<CellLineDisease>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<String>,
    /// The source ID a reverse lookup resolved from, when there was one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_from: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<CellLineVariant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xrefs: Option<CellLineXrefs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chembl: Option<CellLineChembl>,
    /// The PharmacoDB counts, asked for by name. `all` leaves it out.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drug_response: Option<PharmacoDbCounts>,
}

/// One ChEMBL cell line record. BioMCP lists the count and no assays.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellLineChemblRecord {
    pub chembl_id: String,
    pub name: String,
    pub efo_id: Option<String>,
    pub clo_id: Option<String>,
    pub assay_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellLineSpecies {
    pub taxon_id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellLineDisease {
    pub database: String,
    pub accession: String,
    pub label: String,
}

/// One curated sequence variation, as Cellosaurus published it. BioMCP adds no
/// interpretation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellLineVariant {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gene: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hgnc_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variation_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutation_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zygosity: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pubmed_ids: Vec<String>,
}

/// The join keys other cell line sources use. Every key is present, and a
/// missing link is an empty list.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellLineXrefs {
    pub depmap: Vec<String>,
    pub cosmic_clp: Vec<String>,
    pub chembl: Vec<String>,
    pub cell_model_passport: Vec<String>,
    pub gdsc: Vec<String>,
    pub pharmacodb: Vec<String>,
    pub lincs_ldp: Vec<String>,
}

impl CellLineXrefs {
    fn push(&mut self, database: CellLineXrefDatabase, accession: String) {
        let slot = match database {
            CellLineXrefDatabase::DepMap => &mut self.depmap,
            CellLineXrefDatabase::CosmicClp => &mut self.cosmic_clp,
            CellLineXrefDatabase::Chembl => &mut self.chembl,
            CellLineXrefDatabase::CellModelPassport => &mut self.cell_model_passport,
            CellLineXrefDatabase::Gdsc => &mut self.gdsc,
            CellLineXrefDatabase::PharmacoDb => &mut self.pharmacodb,
            CellLineXrefDatabase::LincsLdp => &mut self.lincs_ldp,
        };
        if !slot.contains(&accession) {
            slot.push(accession);
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.depmap.is_empty()
            && self.cosmic_clp.is_empty()
            && self.chembl.is_empty()
            && self.cell_model_passport.is_empty()
            && self.gdsc.is_empty()
            && self.pharmacodb.is_empty()
            && self.lincs_ldp.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellLineSearchResult {
    pub accession: String,
    pub name: String,
    #[serde(default)]
    pub species: Vec<CellLineSpecies>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disease: Option<String>,
    /// `exact` when the normalized query equals the normalized name or any
    /// normalized synonym, otherwise `partial`.
    #[serde(rename = "match")]
    pub match_kind: String,
    /// `name` or `synonym`, set on exact rows only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_on: Option<String>,
}

/// One search window after ranking, with the total and any notes.
#[derive(Debug, Clone)]
pub(crate) struct CellLineSearchPage {
    pub results: Vec<CellLineSearchResult>,
    pub total: Option<usize>,
    pub notes: Vec<String>,
    pub data_as_of: String,
    pub data_as_of_kind: String,
}

/// Lowercase the value and drop every character that is not a letter or a
/// digit. `MOLM13`, `molm-13` and `Molm 13` all normalize to `molm13`.
pub(crate) fn normalize_cell_line_name(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// A query that already names a Cellosaurus accession.
pub(crate) fn is_cvcl_accession(value: &str) -> bool {
    let value = value.trim();
    let Some(suffix) = value
        .strip_prefix("CVCL_")
        .or_else(|| value.strip_prefix("cvcl_"))
        .or_else(|| strip_prefix_ignore_ascii_case(value, "CVCL_"))
    else {
        return false;
    };
    suffix.len() == 4 && suffix.chars().all(|ch| ch.is_ascii_alphanumeric())
}

fn strip_prefix_ignore_ascii_case<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    (value.len() >= prefix.len() && value[..prefix.len()].eq_ignore_ascii_case(prefix))
        .then(|| &value[prefix.len()..])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum MatchRank {
    ExactName,
    ExactSynonym,
    Partial,
}

fn match_rank(record: &CellosaurusRecord, normalized_query: &str) -> MatchRank {
    let identifier_matches = record
        .identifier()
        .is_some_and(|name| normalize_cell_line_name(name) == normalized_query);
    if identifier_matches {
        return MatchRank::ExactName;
    }
    let synonym_matches = record
        .synonyms()
        .iter()
        .any(|name| normalize_cell_line_name(name) == normalized_query);
    if synonym_matches {
        return MatchRank::ExactSynonym;
    }
    MatchRank::Partial
}

/// The one accession a cell line name resolves to, or `None`.
///
/// A name resolves only when exactly one record in the window carries it as its
/// identifier after normalization. Synonyms never answer, so `NB4` cannot land
/// on SJNB-4 and `HL-60` cannot land on HL-60(TB). The caller restricts the
/// window to human records. This is the one name matcher; every cell line join
/// by name goes through it.
pub(crate) fn unique_identifier_accession(
    records: &[CellosaurusRecord],
    name: &str,
) -> Option<String> {
    let normalized = normalize_cell_line_name(name);
    if normalized.is_empty() {
        return None;
    }
    let mut matches = records
        .iter()
        .filter(|record| match_rank(record, &normalized) == MatchRank::ExactName)
        .filter_map(|record| record.primary_accession().map(str::to_string));
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}

fn is_human(species: &[CellLineSpecies]) -> bool {
    species.iter().any(|entry| entry.taxon_id == "9606")
}

fn species_of(record: &CellosaurusRecord) -> Vec<CellLineSpecies> {
    record
        .species_list
        .iter()
        .filter_map(|entry| {
            Some(CellLineSpecies {
                taxon_id: entry.accession.clone()?,
                label: entry.label.clone().unwrap_or_default(),
            })
        })
        .collect()
}

fn diseases_of(record: &CellosaurusRecord) -> Vec<CellLineDisease> {
    record
        .disease_list
        .iter()
        .filter_map(|entry| {
            Some(CellLineDisease {
                database: entry.database.clone()?,
                accession: entry.accession.clone()?,
                label: entry.label.clone().unwrap_or_default(),
            })
        })
        .collect()
}

fn search_row(record: &CellosaurusRecord, normalized_query: &str) -> Option<CellLineSearchResult> {
    let accession = record.primary_accession()?.to_string();
    let rank = match_rank(record, normalized_query);
    let (match_kind, matched_on) = match rank {
        MatchRank::ExactName => ("exact", Some("name".to_string())),
        MatchRank::ExactSynonym => ("exact", Some("synonym".to_string())),
        MatchRank::Partial => ("partial", None),
    };
    Some(CellLineSearchResult {
        accession,
        name: record.identifier().unwrap_or_default().to_string(),
        species: species_of(record),
        category: record.category.clone(),
        disease: diseases_of(record).first().map(|entry| entry.label.clone()),
        match_kind: match_kind.to_string(),
        matched_on,
    })
}

/// Rank the merged window: exact identifier matches, then exact synonym-only
/// matches, then partial matches. Within each group human lines come first and
/// upstream order decides the rest.
pub(crate) fn rank_cell_line_rows(
    records: &[CellosaurusRecord],
    normalized_query: &str,
) -> Vec<CellLineSearchResult> {
    let mut rows: Vec<(MatchRank, bool, CellLineSearchResult)> = records
        .iter()
        .filter_map(|record| {
            let row = search_row(record, normalized_query)?;
            Some((
                match_rank(record, normalized_query),
                is_human(&row.species),
                row,
            ))
        })
        .collect();
    rows.sort_by_key(|(rank, human, _)| (*rank, !*human));
    rows.into_iter().map(|(_, _, row)| row).collect()
}

/// Merge two windows by accession. The raw-query window keeps its order, then
/// the rows seen only in the second window keep theirs.
fn merge_windows(
    first: CellosaurusSearchPage,
    second: Option<CellosaurusSearchPage>,
) -> (Vec<CellosaurusRecord>, bool) {
    let mut window_full = first.window_full;
    let mut records = first.records;
    if let Some(second) = second {
        window_full = window_full || second.window_full;
        let seen: Vec<String> = records
            .iter()
            .filter_map(|record| record.primary_accession().map(str::to_string))
            .collect();
        for record in second.records {
            let accession = record.primary_accession().unwrap_or_default().to_string();
            if seen.contains(&accession) {
                continue;
            }
            records.push(record);
        }
    }
    (records, window_full)
}

/// Read the Cellosaurus release behind every output. A failed read degrades to
/// the retrieval time rather than failing the command.
async fn resolve_data_as_of(client: &CellosaurusClient) -> (String, String) {
    match client.release_info().await {
        Ok(release) => (
            format!("{} ({})", release.version, release.updated),
            "release".to_string(),
        ),
        Err(error) => {
            tracing::debug!("Cellosaurus release info is unavailable: {error}");
            (chrono::Utc::now().to_rfc3339(), "retrieved".to_string())
        }
    }
}

/// The attribution line every cell line output carries.
pub(crate) fn cell_line_attribution(data_as_of: &str, data_as_of_kind: &str) -> String {
    if data_as_of_kind == "release" {
        format!("Cellosaurus {data_as_of}, CC BY 4.0. {CELL_LINE_CITATION}")
    } else {
        format!("Cellosaurus, retrieved {data_as_of}, CC BY 4.0. {CELL_LINE_CITATION}")
    }
}

pub(crate) async fn search(
    query: &str,
    fetch_limit: usize,
) -> Result<CellLineSearchPage, BioMcpError> {
    let query = query.trim();
    if query.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "A cell line name or accession is required".into(),
        ));
    }
    let client = CellosaurusClient::new()?;
    let (data_as_of, data_as_of_kind) = resolve_data_as_of(&client).await;

    let normalized_query = normalize_cell_line_name(query);
    let (records, window_full) = if is_cvcl_accession(query) {
        let page = client.search_accession(query).await?;
        (page.records, page.window_full)
    } else {
        let first = client.search_names(query).await?;
        let second = if query.contains('-') {
            Some(client.search_names(&query.replace('-', "")).await?)
        } else {
            None
        };
        merge_windows(first, second)
    };

    let mut results = if is_cvcl_accession(query) {
        records
            .iter()
            .filter_map(|record| {
                let mut row = search_row(record, &normalized_query)?;
                row.match_kind = "exact".to_string();
                row.matched_on = None;
                Some(row)
            })
            .collect()
    } else {
        rank_cell_line_rows(&records, &normalized_query)
    };
    results.truncate(fetch_limit);

    let mut notes = Vec::new();
    let total = if window_full {
        notes.push(CELL_LINE_FULL_WINDOW_NOTE.to_string());
        None
    } else {
        Some(records.len())
    };

    Ok(CellLineSearchPage {
        results,
        total,
        notes,
        data_as_of,
        data_as_of_kind,
    })
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct CellLineSections {
    pub include_variants: bool,
    pub include_xrefs: bool,
    /// The ChEMBL section is asked for by name. `all` leaves it out, because it
    /// costs one request per record.
    pub include_chembl: bool,
    /// The PharmacoDB section is asked for by name. `all` leaves it out, because
    /// it costs a join and one counts request.
    pub include_drug_response: bool,
}

pub(crate) fn parse_sections(sections: &[String]) -> Result<CellLineSections, BioMcpError> {
    let mut out = CellLineSections::default();
    for raw in sections {
        let section = raw.trim().to_ascii_lowercase();
        if section.is_empty() || section == "--json" || section == "-j" {
            continue;
        }
        match section.as_str() {
            CELL_LINE_SECTION_VARIANTS => out.include_variants = true,
            CELL_LINE_SECTION_XREFS => out.include_xrefs = true,
            CELL_LINE_SECTION_CHEMBL => out.include_chembl = true,
            CELL_LINE_SECTION_DRUG_RESPONSE => out.include_drug_response = true,
            CELL_LINE_SECTION_ALL => {
                out.include_variants = true;
                out.include_xrefs = true;
            }
            _ => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown section \"{section}\" for cell-line. Available: {}",
                    CELL_LINE_SECTION_NAMES.join(", ")
                )));
            }
        }
    }
    Ok(out)
}

fn not_found(id: &str) -> BioMcpError {
    BioMcpError::NotFound {
        entity: "cell-line".to_string(),
        id: id.to_string(),
        suggestion: format!("biomcp search cell-line {id}"),
    }
}

/// Resolve a source ID held in a cross-reference to one accession.
async fn resolve_source_id(client: &CellosaurusClient, id: &str) -> Result<String, BioMcpError> {
    let page = client.search_xref(id).await?;
    accession_for_source_id(id, &page.records)
}

/// Pick the one accession a cross-reference lookup names.
pub(crate) fn accession_for_source_id(
    id: &str,
    records: &[CellosaurusRecord],
) -> Result<String, BioMcpError> {
    let accessions: Vec<String> = records
        .iter()
        .filter_map(|record| record.primary_accession().map(str::to_string))
        .collect();
    match accessions.len() {
        0 => Err(not_found(id)),
        1 => Ok(accessions.into_iter().next().unwrap_or_default()),
        _ => Err(BioMcpError::InvalidArgument(format!(
            "`{id}` matches more than one Cellosaurus cell line: {}. \
Ask for one accession instead.",
            accessions.join(", ")
        ))),
    }
}

pub(crate) async fn get(id: &str, sections: &[String]) -> Result<CellLine, BioMcpError> {
    let parsed = parse_sections(sections)?;
    let id = id.trim();
    if id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "A cell line accession or source ID is required".into(),
        ));
    }
    let client = CellosaurusClient::new()?;
    let (data_as_of, data_as_of_kind) = resolve_data_as_of(&client).await;

    let (accession, resolved_from) = if is_cvcl_accession(id) {
        (id.to_ascii_uppercase(), None)
    } else {
        (resolve_source_id(&client, id).await?, Some(id.to_string()))
    };

    let mut fields: Vec<&str> = CARD_FIELDS.to_vec();
    if parsed.include_variants {
        fields.push("var");
    }
    // The PharmacoDB join reads the cross-reference list, so the section pays
    // for the `dr` field even when the card prints no cross-references.
    if parsed.include_xrefs || parsed.include_drug_response {
        fields.push("dr");
    }

    let record = client
        .get_record(&accession, &fields)
        .await?
        .ok_or_else(|| not_found(id))?;

    let mut cell_line = build_cell_line(
        &record,
        &accession,
        resolved_from,
        parsed,
        data_as_of,
        data_as_of_kind,
    );
    if parsed.include_chembl {
        chembl::attach_chembl_section(
            &mut cell_line,
            chembl::load_chembl_section(&accession).await,
        );
    }
    if parsed.include_drug_response {
        let ids = pharmacodb_ids(&record);
        pharmacodb::attach_drug_response_section(
            &mut cell_line,
            pharmacodb::load_drug_response_section(
                &accession,
                &ids,
                record.identifier().unwrap_or_default(),
            )
            .await,
        );
    }
    Ok(cell_line)
}

pub(crate) fn build_cell_line(
    record: &CellosaurusRecord,
    accession: &str,
    resolved_from: Option<String>,
    sections: CellLineSections,
    data_as_of: String,
    data_as_of_kind: String,
) -> CellLine {
    let accession = record.primary_accession().unwrap_or(accession).to_string();
    let variants: Vec<CellLineVariant> = record
        .sequence_variation_list
        .iter()
        .map(|entry| {
            let hgnc = entry
                .xref_list
                .iter()
                .find(|xref| xref.database.as_deref() == Some("HGNC"));
            CellLineVariant {
                gene: hgnc.and_then(|xref| xref.label.clone()),
                hgnc_id: hgnc.and_then(|xref| xref.accession.clone()),
                variation_type: entry.variation_type.clone(),
                mutation_type: entry.mutation_type.clone(),
                description: entry.mutation_description.clone(),
                zygosity: entry.zygosity_type.clone(),
                pubmed_ids: entry
                    .source_list
                    .iter()
                    .filter_map(|source| source.pubmed_id())
                    .collect(),
            }
        })
        .collect();

    let mut xrefs = CellLineXrefs::default();
    for entry in &record.xref_list {
        let (Some(database), Some(value)) = (entry.database.as_deref(), entry.accession.as_deref())
        else {
            continue;
        };
        if let Some(database) = CellLineXrefDatabase::from_name(database) {
            xrefs.push(database, value.to_string());
        }
    }

    let mut section_outcomes = default_cell_line_section_outcomes();
    if sections.include_variants {
        section_outcomes.complete(
            CELL_LINE_SECTION_VARIANTS,
            if variants.is_empty() {
                SectionOutcome::empty(CELL_LINE_SOURCE)
            } else {
                SectionOutcome::data(CELL_LINE_SOURCE)
            },
        );
    }
    if sections.include_xrefs {
        section_outcomes.complete(
            CELL_LINE_SECTION_XREFS,
            if xrefs.is_empty() {
                SectionOutcome::empty(CELL_LINE_SOURCE)
            } else {
                SectionOutcome::data(CELL_LINE_SOURCE)
            },
        );
    }

    CellLine {
        section_outcomes,
        source: CELL_LINE_SOURCE.to_string(),
        data_as_of,
        data_as_of_kind,
        rrid: format!("RRID:{accession}"),
        accession,
        secondary_accessions: record.secondary_accessions(),
        name: record.identifier().unwrap_or_default().to_string(),
        synonyms: record.synonyms(),
        species: species_of(record),
        diseases: diseases_of(record),
        category: record.category.clone(),
        sex: record.sex.clone(),
        age: record.age.clone(),
        resolved_from,
        variants: if sections.include_variants {
            variants
        } else {
            Vec::new()
        },
        xrefs: sections.include_xrefs.then_some(xrefs),
        chembl: None,
        drug_response: None,
    }
}

/// The PharmacoDB cell line IDs one Cellosaurus record publishes.
pub(crate) fn pharmacodb_ids(record: &CellosaurusRecord) -> Vec<String> {
    record
        .xref_list
        .iter()
        .filter(|entry| {
            entry
                .database
                .as_deref()
                .and_then(CellLineXrefDatabase::from_name)
                == Some(CellLineXrefDatabase::PharmacoDb)
        })
        .filter_map(|entry| entry.accession.clone())
        .collect()
}

/// What a PharmacoDB helper needs to know about one cell line: the accession it
/// resolved to, the Cellosaurus name, and the PharmacoDB cross-references.
pub(crate) struct PharmacoDbIdentity {
    pub accession: String,
    pub name: String,
    pub pharmacodb_ids: Vec<String>,
}

/// Resolve any ID `get cell-line` accepts into a PharmacoDB join input.
pub(crate) async fn load_pharmacodb_identity(id: &str) -> Result<PharmacoDbIdentity, BioMcpError> {
    let id = id.trim();
    if id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "A cell line accession or source ID is required".into(),
        ));
    }
    let client = CellosaurusClient::new()?;
    let accession = if is_cvcl_accession(id) {
        id.to_ascii_uppercase()
    } else {
        resolve_source_id(&client, id).await?
    };
    let mut fields: Vec<&str> = CARD_FIELDS.to_vec();
    fields.push("dr");
    let record = client
        .get_record(&accession, &fields)
        .await?
        .ok_or_else(|| not_found(id))?;
    Ok(PharmacoDbIdentity {
        accession: record.primary_accession().unwrap_or(&accession).to_string(),
        name: record.identifier().unwrap_or_default().to_string(),
        pharmacodb_ids: pharmacodb_ids(&record),
    })
}

#[cfg(test)]
pub(crate) fn cell_line_sections(include_variants: bool, include_xrefs: bool) -> CellLineSections {
    CellLineSections {
        include_variants,
        include_xrefs,
        include_chembl: false,
        include_drug_response: false,
    }
}

#[cfg(test)]
mod tests;
