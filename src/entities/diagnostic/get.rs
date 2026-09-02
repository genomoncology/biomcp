use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::future::Future;
#[cfg(test)]
use std::path::Path;
use std::time::Duration;

use crate::entities::section_outcome::SectionOutcome;
use crate::error::BioMcpError;
use crate::sources::gtr::{GtrClient, GtrIndex, GtrRecord, GtrSyncMode};
use crate::sources::openfda::{Fda510kResult, FdaPmaResult, OpenFdaClient};
use crate::sources::who_ivd::{WhoIvdClient, WhoIvdRecord, WhoIvdSyncMode};

use super::{
    DIAGNOSTIC_SECTION_CONDITIONS, DIAGNOSTIC_SECTION_GENES, DIAGNOSTIC_SECTION_METHODS,
    DIAGNOSTIC_SECTION_NAMES, DIAGNOSTIC_SECTION_REGULATORY, DIAGNOSTIC_SOURCE_GTR,
    DIAGNOSTIC_SOURCE_WHO_IVD, Diagnostic, DiagnosticRegulatoryRecord,
    default_diagnostic_section_outcomes, diagnostic_source_label, looks_like_gtr_accession,
    optional_text, preferred_diagnostic_name, supported_diagnostic_sections_for_source,
};

pub(super) const OPTIONAL_REGULATORY_TIMEOUT: Duration = Duration::from_secs(8);
const REGULATORY_ALIAS_LIMIT: usize = 6;
const REGULATORY_ENDPOINT_LIMIT: usize = 25;
const REGULATORY_RESULT_LIMIT: usize = 8;

#[derive(Debug, Clone, Copy, Default)]
struct DiagnosticSections {
    include_genes: bool,
    include_conditions: bool,
    include_methods: bool,
    include_regulatory: bool,
    include_all: bool,
}

#[derive(Debug, Clone)]
pub(super) struct DiagnosticRegulatoryLookupContext {
    pub(super) display_name: String,
    pub(super) aliases: Vec<String>,
    pub(super) manufacturer: Option<String>,
}

#[derive(Debug, Clone)]
struct RankedRegulatoryRecord {
    record: DiagnosticRegulatoryRecord,
    alias_rank: usize,
    manufacturer_overlap: bool,
    decision_key: String,
}

fn parse_sections(sections: &[String]) -> Result<DiagnosticSections, BioMcpError> {
    let mut out = DiagnosticSections::default();

    for raw in sections {
        let section = raw.trim().to_ascii_lowercase();
        if section.is_empty() || section == "--json" || section == "-j" {
            continue;
        }

        match section.as_str() {
            DIAGNOSTIC_SECTION_GENES => out.include_genes = true,
            DIAGNOSTIC_SECTION_CONDITIONS => out.include_conditions = true,
            DIAGNOSTIC_SECTION_METHODS => out.include_methods = true,
            DIAGNOSTIC_SECTION_REGULATORY => out.include_regulatory = true,
            "all" => out.include_all = true,
            _ => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown section \"{section}\" for diagnostic. Available: {}",
                    DIAGNOSTIC_SECTION_NAMES.join(", ")
                )));
            }
        }
    }

    Ok(out)
}

fn quote_identifier(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return String::new();
    }
    if value.chars().any(|c| c.is_whitespace()) {
        format!("\"{}\"", value.replace('\"', "\\\""))
    } else {
        value.to_string()
    }
}

fn unsupported_diagnostic_section_error(
    section: &str,
    source: &str,
    accession: &str,
) -> BioMcpError {
    let quoted = quote_identifier(accession);
    BioMcpError::InvalidArgument(format!(
        "Section `{section}` is not available for {} diagnostic records. Try `biomcp get diagnostic {quoted} conditions`",
        diagnostic_source_label(source)
    ))
}

fn resolve_sections_for_source(
    source: &str,
    accession: &str,
    raw_sections: &[String],
) -> Result<DiagnosticSections, BioMcpError> {
    let mut resolved = parse_sections(raw_sections)?;
    let supported = supported_diagnostic_sections_for_source(source);

    for raw in raw_sections {
        let section = raw.trim();
        if section.is_empty()
            || section.eq_ignore_ascii_case("--json")
            || section.eq_ignore_ascii_case("-j")
            || section.eq_ignore_ascii_case("all")
        {
            continue;
        }
        if !supported
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(section))
        {
            return Err(unsupported_diagnostic_section_error(
                section, source, accession,
            ));
        }
    }

    if resolved.include_all {
        resolved.include_genes = supported
            .iter()
            .any(|section| section.eq_ignore_ascii_case(DIAGNOSTIC_SECTION_GENES));
        resolved.include_conditions = supported
            .iter()
            .any(|section| section.eq_ignore_ascii_case(DIAGNOSTIC_SECTION_CONDITIONS));
        resolved.include_methods = supported
            .iter()
            .any(|section| section.eq_ignore_ascii_case(DIAGNOSTIC_SECTION_METHODS));
    }

    Ok(resolved)
}

fn diagnostic_from_record(
    record: &GtrRecord,
    index: &GtrIndex,
    sections: DiagnosticSections,
) -> Diagnostic {
    Diagnostic {
        section_outcomes: default_diagnostic_section_outcomes(),
        source: DIAGNOSTIC_SOURCE_GTR.to_string(),
        source_id: record.accession.clone(),
        accession: record.accession.clone(),
        name: preferred_diagnostic_name(record),
        test_type: optional_text(&record.test_type),
        manufacturer: optional_text(&record.manufacturer_test_name),
        target_marker: None,
        regulatory_version: None,
        prequalification_year: None,
        laboratory: optional_text(&record.name_of_laboratory),
        institution: optional_text(&record.name_of_institution),
        country: optional_text(&record.facility_country),
        clia_number: optional_text(&record.clia_number),
        state_licenses: optional_text(&record.state_licenses),
        current_status: optional_text(&record.test_curr_stat),
        public_status: optional_text(&record.test_pub_stat),
        method_categories: record.method_categories.clone(),
        genes: sections
            .include_genes
            .then(|| index.merged_genes(&record.accession)),
        conditions: sections
            .include_conditions
            .then(|| index.conditions(&record.accession)),
        methods: sections.include_methods.then(|| record.methods.clone()),
        regulatory: None,
    }
}

fn diagnostic_from_who_ivd_record(
    record: &WhoIvdRecord,
    sections: DiagnosticSections,
) -> Diagnostic {
    Diagnostic {
        section_outcomes: default_diagnostic_section_outcomes(),
        source: DIAGNOSTIC_SOURCE_WHO_IVD.to_string(),
        source_id: record.product_code.clone(),
        accession: record.product_code.clone(),
        name: optional_text(&record.product_name).unwrap_or_else(|| record.product_code.clone()),
        test_type: optional_text(&record.assay_format),
        manufacturer: optional_text(&record.manufacturer_name),
        target_marker: optional_text(&record.target_marker),
        regulatory_version: optional_text(&record.regulatory_version),
        prequalification_year: optional_text(&record.prequalification_year),
        laboratory: None,
        institution: None,
        country: None,
        clia_number: None,
        state_licenses: None,
        current_status: None,
        public_status: None,
        method_categories: Vec::new(),
        genes: None,
        conditions: sections
            .include_conditions
            .then(|| optional_text(&record.target_marker).into_iter().collect()),
        methods: None,
        regulatory: None,
    }
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_regulatory_alias(value: &str) -> String {
    collapse_whitespace(
        &value
            .replace(['®', '™'], " ")
            .replace("(R)", " ")
            .replace("(r)", " ")
            .replace("(TM)", " ")
            .replace("(tm)", " ")
            .replace("(Tm)", " ")
            .replace("(tM)", " "),
    )
}

fn normalize_overlap_text(value: &str) -> String {
    let mut out = String::new();
    let mut pending_space = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_space && !out.is_empty() {
                out.push(' ');
            }
            out.push(ch.to_ascii_lowercase());
            pending_space = false;
        } else if !out.is_empty() {
            pending_space = true;
        }
    }
    out
}

fn clean_option_text(value: Option<&str>) -> Option<String> {
    value
        .map(collapse_whitespace)
        .filter(|text| !text.is_empty())
}

fn push_lookup_alias(aliases: &mut Vec<String>, seen: &mut HashSet<String>, value: &str) {
    if aliases.len() >= REGULATORY_ALIAS_LIMIT {
        return;
    }

    let original = collapse_whitespace(value);
    if original.is_empty() {
        return;
    }

    let original_key = original.to_ascii_lowercase();
    if seen.insert(original_key) {
        aliases.push(original.clone());
        if aliases.len() >= REGULATORY_ALIAS_LIMIT {
            return;
        }
    }

    let normalized = normalize_regulatory_alias(&original);
    if normalized.is_empty() {
        return;
    }
    let normalized_key = normalized.to_ascii_lowercase();
    if seen.insert(normalized_key) {
        aliases.push(normalized);
    }
}

pub(super) fn build_gtr_regulatory_lookup_context(
    record: &GtrRecord,
) -> DiagnosticRegulatoryLookupContext {
    let display_name = preferred_diagnostic_name(record);
    let mut aliases = Vec::new();
    let mut seen = HashSet::new();
    push_lookup_alias(&mut aliases, &mut seen, &display_name);
    push_lookup_alias(&mut aliases, &mut seen, &record.lab_test_name);
    push_lookup_alias(&mut aliases, &mut seen, &record.manufacturer_test_name);
    if aliases.is_empty() {
        aliases.push(display_name.clone());
    }
    aliases.truncate(REGULATORY_ALIAS_LIMIT);

    DiagnosticRegulatoryLookupContext {
        display_name,
        aliases,
        manufacturer: optional_text(&record.name_of_laboratory)
            .or_else(|| optional_text(&record.name_of_institution))
            .or_else(|| optional_text(&record.manufacturer_test_name)),
    }
}

pub(super) fn build_who_regulatory_lookup_context(
    record: &WhoIvdRecord,
) -> DiagnosticRegulatoryLookupContext {
    let display_name =
        optional_text(&record.product_name).unwrap_or_else(|| record.product_code.clone());
    let mut aliases = Vec::new();
    let mut seen = HashSet::new();
    push_lookup_alias(&mut aliases, &mut seen, &display_name);
    if aliases.is_empty() {
        aliases.push(display_name.clone());
    }

    DiagnosticRegulatoryLookupContext {
        display_name,
        aliases,
        manufacturer: optional_text(&record.manufacturer_name),
    }
}

fn build_device_query(field: &str, aliases: &[String]) -> String {
    aliases
        .iter()
        .take(REGULATORY_ALIAS_LIMIT)
        .filter_map(|alias| {
            let alias = alias.trim();
            (!alias.is_empty())
                .then(|| format!("{field}:\"{}\"", OpenFdaClient::escape_query_value(alias)))
        })
        .collect::<Vec<_>>()
        .join(" OR ")
}

fn alias_match_rank(ctx: &DiagnosticRegulatoryLookupContext, candidate: &str) -> usize {
    let candidate = normalize_regulatory_alias(candidate).to_ascii_lowercase();
    if candidate.is_empty() {
        return usize::MAX;
    }

    ctx.aliases
        .iter()
        .position(|alias| normalize_regulatory_alias(alias).eq_ignore_ascii_case(&candidate))
        .unwrap_or(usize::MAX)
}

fn has_manufacturer_overlap(manufacturer: Option<&str>, applicant: Option<&str>) -> bool {
    let Some(manufacturer) = manufacturer
        .map(normalize_overlap_text)
        .filter(|value| !value.is_empty())
    else {
        return false;
    };
    let Some(applicant) = applicant
        .map(normalize_overlap_text)
        .filter(|value| !value.is_empty())
    else {
        return false;
    };

    manufacturer.contains(&applicant) || applicant.contains(&manufacturer)
}

fn decision_key(value: Option<&str>) -> String {
    value
        .unwrap_or_default()
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .collect()
}

fn rank_record(
    ctx: &DiagnosticRegulatoryLookupContext,
    record: DiagnosticRegulatoryRecord,
) -> RankedRegulatoryRecord {
    RankedRegulatoryRecord {
        alias_rank: alias_match_rank(ctx, &record.display_name),
        manufacturer_overlap: has_manufacturer_overlap(
            ctx.manufacturer.as_deref(),
            record.applicant.as_deref(),
        ),
        decision_key: decision_key(record.decision_date.as_deref()),
        record,
    }
}

fn compare_ranked_records(
    left: &RankedRegulatoryRecord,
    right: &RankedRegulatoryRecord,
) -> Ordering {
    left.alias_rank
        .cmp(&right.alias_rank)
        .then_with(|| right.manufacturer_overlap.cmp(&left.manufacturer_overlap))
        .then_with(|| right.decision_key.cmp(&left.decision_key))
        .then_with(|| left.record.number.cmp(&right.record.number))
}

fn should_replace_record(
    existing: &RankedRegulatoryRecord,
    candidate: &RankedRegulatoryRecord,
) -> bool {
    if candidate.decision_key != existing.decision_key {
        return candidate.decision_key > existing.decision_key;
    }
    compare_ranked_records(candidate, existing).is_lt()
}

fn merge_510k_results(
    ctx: &DiagnosticRegulatoryLookupContext,
    rows: &[Fda510kResult],
) -> Vec<RankedRegulatoryRecord> {
    let mut deduped = HashMap::new();

    for row in rows {
        let Some(number) = clean_option_text(row.k_number.as_deref()) else {
            continue;
        };
        let record = DiagnosticRegulatoryRecord {
            submission_type: "510(k)".to_string(),
            number: number.clone(),
            display_name: clean_option_text(row.device_name.as_deref())
                .unwrap_or_else(|| ctx.display_name.clone()),
            trade_name: None,
            generic_name: None,
            applicant: clean_option_text(row.applicant.as_deref()),
            decision_date: clean_option_text(row.decision_date.as_deref()),
            decision_description: clean_option_text(row.decision_description.as_deref()),
            advisory_committee: clean_option_text(row.advisory_committee_description.as_deref()),
            product_code: clean_option_text(row.product_code.as_deref()),
            supplement_count: None,
        };
        let candidate = rank_record(ctx, record);
        match deduped.get(&number) {
            Some(existing) if !should_replace_record(existing, &candidate) => {}
            _ => {
                deduped.insert(number, candidate);
            }
        }
    }

    deduped.into_values().collect()
}

fn merge_pma_results(
    ctx: &DiagnosticRegulatoryLookupContext,
    rows: &[FdaPmaResult],
) -> Vec<RankedRegulatoryRecord> {
    let mut deduped = HashMap::new();
    let mut supplement_counts: HashMap<String, HashSet<String>> = HashMap::new();

    for row in rows {
        let Some(number) = clean_option_text(row.pma_number.as_deref()) else {
            continue;
        };
        if let Some(supplement_number) = clean_option_text(row.supplement_number.as_deref()) {
            supplement_counts
                .entry(number.clone())
                .or_default()
                .insert(supplement_number);
        }

        let trade_name = clean_option_text(row.trade_name.as_deref());
        let generic_name = clean_option_text(row.generic_name.as_deref());
        let record = DiagnosticRegulatoryRecord {
            submission_type: "PMA".to_string(),
            number: number.clone(),
            display_name: trade_name
                .clone()
                .or_else(|| generic_name.clone())
                .unwrap_or_else(|| ctx.display_name.clone()),
            trade_name,
            generic_name,
            applicant: clean_option_text(row.applicant.as_deref()),
            decision_date: clean_option_text(row.decision_date.as_deref()),
            decision_description: clean_option_text(row.decision_description.as_deref()),
            advisory_committee: clean_option_text(row.advisory_committee_description.as_deref()),
            product_code: clean_option_text(row.product_code.as_deref()),
            supplement_count: None,
        };
        let candidate = rank_record(ctx, record);
        match deduped.get(&number) {
            Some(existing) if !should_replace_record(existing, &candidate) => {}
            _ => {
                deduped.insert(number, candidate);
            }
        }
    }

    let mut out = deduped.into_values().collect::<Vec<_>>();
    for row in &mut out {
        row.record.supplement_count = supplement_counts
            .get(&row.record.number)
            .map(HashSet::len)
            .filter(|count| *count > 0);
    }
    out
}

async fn fetch_fda_regulatory(
    ctx: &DiagnosticRegulatoryLookupContext,
) -> Result<Vec<DiagnosticRegulatoryRecord>, BioMcpError> {
    if ctx.aliases.is_empty() {
        return Ok(Vec::new());
    }

    let device_query = build_device_query("device_name", &ctx.aliases);
    let pma_query = build_device_query("trade_name", &ctx.aliases);
    if device_query.is_empty() && pma_query.is_empty() {
        return Ok(Vec::new());
    }

    // This optional overlay must not pay the synchronous managed-cache scan before
    // its timeout can be polled.
    let client = OpenFdaClient::new_uncached()?;
    let (device_rows, pma_rows) = tokio::join!(
        async {
            if device_query.is_empty() {
                Ok(None)
            } else {
                client
                    .device_510k_search(&device_query, REGULATORY_ENDPOINT_LIMIT)
                    .await
            }
        },
        async {
            if pma_query.is_empty() {
                Ok(None)
            } else {
                client
                    .device_pma_search(&pma_query, REGULATORY_ENDPOINT_LIMIT)
                    .await
            }
        }
    );

    Ok(regulatory_records_from_fda_rows(
        ctx,
        &device_rows?.map(|resp| resp.results).unwrap_or_default(),
        &pma_rows?.map(|resp| resp.results).unwrap_or_default(),
    ))
}

pub(super) fn regulatory_records_from_fda_rows(
    ctx: &DiagnosticRegulatoryLookupContext,
    device_rows: &[Fda510kResult],
    pma_rows: &[FdaPmaResult],
) -> Vec<DiagnosticRegulatoryRecord> {
    let mut ranked = Vec::new();
    ranked.extend(merge_510k_results(ctx, device_rows));
    ranked.extend(merge_pma_results(ctx, pma_rows));

    ranked.sort_by(compare_ranked_records);
    ranked.truncate(REGULATORY_RESULT_LIMIT);
    ranked.into_iter().map(|row| row.record).collect()
}

pub(super) async fn load_regulatory_records_with<F>(
    lookup: F,
    timeout: Duration,
) -> (Vec<DiagnosticRegulatoryRecord>, SectionOutcome)
where
    F: Future<Output = Result<Vec<DiagnosticRegulatoryRecord>, BioMcpError>>,
{
    match tokio::time::timeout(timeout, lookup).await {
        Ok(Ok(records)) => {
            let outcome = if records.is_empty() {
                SectionOutcome::empty("OpenFDA Device 510(k) / PMA")
            } else {
                SectionOutcome::data("OpenFDA Device 510(k) / PMA")
            };
            (records, outcome)
        }
        Ok(Err(_)) | Err(_) => (
            Vec::new(),
            SectionOutcome::unavailable(
                "OpenFDA diagnostic regulatory data is temporarily unavailable.",
            ),
        ),
    }
}

async fn load_regulatory_records(
    ctx: &DiagnosticRegulatoryLookupContext,
) -> (Vec<DiagnosticRegulatoryRecord>, SectionOutcome) {
    load_regulatory_records_with(fetch_fda_regulatory(ctx), OPTIONAL_REGULATORY_TIMEOUT).await
}

pub async fn get(accession: &str, sections: &[String]) -> Result<Diagnostic, BioMcpError> {
    let accession = accession.trim();
    if accession.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Diagnostic accession or WHO IVD product code is required. Example: biomcp get diagnostic GTR000006692.3".into(),
        ));
    }

    if looks_like_gtr_accession(accession) {
        let client = GtrClient::ready(GtrSyncMode::Auto).await?;
        let index = client.load_index()?;
        return get_from_data(accession, sections, Some(index), None).await;
    }

    let client = WhoIvdClient::ready(WhoIvdSyncMode::Auto).await?;
    get_from_data(accession, sections, None, Some(client.read_rows()?)).await
}

#[cfg(test)]
pub(super) async fn get_with_roots(
    accession: &str,
    sections: &[String],
    gtr_root: Option<&Path>,
    who_ivd_root: Option<&Path>,
) -> Result<Diagnostic, BioMcpError> {
    let accession = accession.trim();
    if accession.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Diagnostic accession or WHO IVD product code is required. Example: biomcp get diagnostic GTR000006692.3".into(),
        ));
    }

    let gtr_index = if looks_like_gtr_accession(accession) {
        let root = gtr_root.ok_or_else(|| {
            BioMcpError::InvalidArgument("diagnostic GTR test root is required".to_string())
        })?;
        Some(GtrClient::from_root(root).load_index()?)
    } else {
        None
    };
    let who_rows = if !looks_like_gtr_accession(accession) {
        let root = who_ivd_root.ok_or_else(|| {
            BioMcpError::InvalidArgument("diagnostic WHO IVD test root is required".to_string())
        })?;
        Some(WhoIvdClient::from_root(root).read_rows()?)
    } else {
        None
    };

    get_from_data(accession, sections, gtr_index, who_rows).await
}

async fn get_from_data(
    accession: &str,
    sections: &[String],
    gtr_index: Option<GtrIndex>,
    who_rows: Option<Vec<WhoIvdRecord>>,
) -> Result<Diagnostic, BioMcpError> {
    if looks_like_gtr_accession(accession) {
        let index = gtr_index.ok_or_else(|| {
            BioMcpError::InvalidArgument("diagnostic GTR data is required".to_string())
        })?;
        let record = index.record(accession).ok_or_else(|| BioMcpError::NotFound {
            entity: "diagnostic".to_string(),
            id: accession.to_string(),
            suggestion:
                "Try searching with a filter such as `biomcp search diagnostic --gene BRCA1` or inspect `biomcp list diagnostic`.".to_string(),
        })?;
        let sections = resolve_sections_for_source(DIAGNOSTIC_SOURCE_GTR, accession, sections)?;
        let regulatory_ctx = sections
            .include_regulatory
            .then(|| build_gtr_regulatory_lookup_context(record));
        let mut diagnostic = diagnostic_from_record(record, &index, sections);
        if let Some(ctx) = regulatory_ctx.as_ref() {
            let (records, outcome) = load_regulatory_records(ctx).await;
            diagnostic.regulatory = Some(records);
            diagnostic
                .section_outcomes
                .complete(DIAGNOSTIC_SECTION_REGULATORY, outcome);
        }
        return Ok(diagnostic);
    }

    let record = who_rows
        .ok_or_else(|| {
            BioMcpError::InvalidArgument("diagnostic WHO IVD data is required".to_string())
        })?
        .into_iter()
        .find(|record| record.product_code == accession)
        .ok_or_else(|| BioMcpError::NotFound {
        entity: "diagnostic".to_string(),
        id: accession.to_string(),
        suggestion:
            "Try searching with a filter such as `biomcp search diagnostic --disease HIV --source who-ivd` or inspect `biomcp list diagnostic`.".to_string(),
    })?;
    let sections = resolve_sections_for_source(DIAGNOSTIC_SOURCE_WHO_IVD, accession, sections)?;
    let regulatory_ctx = sections
        .include_regulatory
        .then(|| build_who_regulatory_lookup_context(&record));
    let mut diagnostic = diagnostic_from_who_ivd_record(&record, sections);
    if let Some(ctx) = regulatory_ctx.as_ref() {
        let (records, outcome) = load_regulatory_records(ctx).await;
        diagnostic.regulatory = Some(records);
        diagnostic
            .section_outcomes
            .complete(DIAGNOSTIC_SECTION_REGULATORY, outcome);
    }
    Ok(diagnostic)
}
