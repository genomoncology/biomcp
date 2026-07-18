//! Drug retrieval workflows, section parsing, and region validation.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use regex::Regex;
use tracing::debug as warn;

use crate::entities::section_outcome::SectionOutcome;
use crate::error::BioMcpError;
use crate::sources::civic::{CivicClient, CivicContext};
use crate::sources::ema::{EmaClient, EmaSyncMode};
use crate::sources::mychem::MyChemHit;
use crate::sources::openfda::OpenFdaClient;
use crate::sources::who_pq::{WhoPqClient, WhoPqSyncMode, WhoProductTypeFilter};
use crate::transform;

use super::label::{extract_inline_label, extract_label_set_id, extract_label_warnings_text};
use super::metadata::{
    apply_openfda_metadata, fetch_shortage_entries, fetch_top_adverse_events,
    map_drugsfda_approvals,
};
use super::search::{search_page, search_results_from_openfda_label_response};
use super::targets::{enrich_indications, enrich_targets};
use super::{
    DRUG_SECTION_ALL, DRUG_SECTION_APPROVALS, DRUG_SECTION_CIVIC, DRUG_SECTION_INDICATIONS,
    DRUG_SECTION_INTERACTIONS, DRUG_SECTION_LABEL, DRUG_SECTION_NAMES, DRUG_SECTION_REGULATORY,
    DRUG_SECTION_SAFETY, DRUG_SECTION_SHORTAGE, DRUG_SECTION_TARGETS, Drug, DrugApproval,
    DrugRegion, DrugSearchFilters, OPTIONAL_SAFETY_TIMEOUT, build_ema_identity, build_who_identity,
    direct_drug_lookup,
};

#[derive(Debug, Clone, Copy, Default)]
struct DrugSections {
    include_label: bool,
    include_regulatory: bool,
    include_safety: bool,
    include_shortage: bool,
    include_targets: bool,
    include_indications: bool,
    include_interactions: bool,
    include_civic: bool,
    include_approvals: bool,
    requested_all: bool,
    requested_safety: bool,
    requested_shortage: bool,
}

#[cfg(test)]
fn parse_sections(sections: &[String]) -> Result<DrugSections, BioMcpError> {
    parse_sections_for_name("", sections)
}

fn parse_sections_for_name(name: &str, sections: &[String]) -> Result<DrugSections, BioMcpError> {
    let mut out = DrugSections::default();
    let mut include_all = false;
    let mut any_section = false;

    for raw in sections {
        let section = raw.trim().to_ascii_lowercase();
        if section.is_empty() {
            continue;
        }
        if section == "--json" || section == "-j" {
            continue;
        }
        any_section = true;
        match section.as_str() {
            DRUG_SECTION_LABEL => {
                out.include_label = true;
            }
            DRUG_SECTION_REGULATORY => out.include_regulatory = true,
            DRUG_SECTION_SAFETY => {
                out.include_safety = true;
                out.requested_safety = true;
            }
            DRUG_SECTION_SHORTAGE => {
                out.include_shortage = true;
                out.requested_shortage = true;
            }
            DRUG_SECTION_TARGETS => out.include_targets = true,
            DRUG_SECTION_INDICATIONS => out.include_indications = true,
            DRUG_SECTION_INTERACTIONS => out.include_interactions = true,
            DRUG_SECTION_CIVIC => out.include_civic = true,
            DRUG_SECTION_APPROVALS => out.include_approvals = true,
            DRUG_SECTION_ALL => {
                include_all = true;
                out.requested_all = true;
            }
            _ => {
                return Err(BioMcpError::InvalidArgument(unknown_drug_section_message(
                    name, sections, raw, &section,
                )));
            }
        }
    }

    if include_all {
        out.include_label = true;
        out.include_regulatory = true;
        out.include_safety = true;
        out.include_shortage = true;
        out.include_targets = true;
        out.include_indications = true;
        out.include_interactions = true;
        out.include_civic = true;
    } else if !any_section {
        out.include_targets = true;
    }

    Ok(out)
}

fn unknown_drug_section_message(
    name: &str,
    sections: &[String],
    raw_section: &str,
    normalized_section: &str,
) -> String {
    let available = DRUG_SECTION_NAMES.join(", ");
    let tail = sections
        .iter()
        .skip_while(|section| section.as_str() != raw_section)
        .skip(1)
        .collect::<Vec<_>>();
    let section_start = tail
        .iter()
        .position(|section| {
            DRUG_SECTION_NAMES.contains(&section.trim().to_ascii_lowercase().as_str())
        })
        .unwrap_or(tail.len());
    let name_tail = tail
        .iter()
        .take(section_start)
        .map(|section| section.trim())
        .filter(|section| !section.is_empty());
    let suggestion_name = std::iter::once(name.trim())
        .chain(std::iter::once(raw_section.trim()))
        .chain(name_tail)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let remaining_sections = tail
        .iter()
        .skip(section_start)
        .map(|section| section.trim())
        .filter(|section| !section.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let next = if remaining_sections.is_empty() {
        format!("biomcp get drug --name \"{suggestion_name}\"")
    } else {
        format!("biomcp get drug --name \"{suggestion_name}\" {remaining_sections}")
    };
    format!(
        "Unknown section \"{normalized_section}\" for drug. Available: {available}. If \"{raw_section}\" is part of a multi-word drug name, use `{next}`."
    )
}

fn is_section_only_requested(sections: &[String]) -> bool {
    !sections
        .iter()
        .any(|section| section.trim().eq_ignore_ascii_case(DRUG_SECTION_ALL))
        && sections.iter().any(|section| !section.trim().is_empty())
}

async fn fetch_civic_therapy_context(name: &str) -> (Option<CivicContext>, SectionOutcome) {
    let name = name.trim();
    if name.is_empty() {
        return (
            Some(CivicContext::default()),
            SectionOutcome::empty("CIViC"),
        );
    }

    let civic_fut = async {
        let client = CivicClient::new()?;
        client.by_therapy(name, 10).await
    };

    match tokio::time::timeout(OPTIONAL_SAFETY_TIMEOUT, civic_fut).await {
        Ok(Ok(context)) => {
            let outcome = if context.evidence_items.is_empty() && context.assertions.is_empty() {
                SectionOutcome::empty("CIViC")
            } else {
                SectionOutcome::data("CIViC")
            };
            (Some(context), outcome)
        }
        Ok(Err(_)) | Err(_) => (
            None,
            SectionOutcome::unavailable("CIViC drug evidence is temporarily unavailable."),
        ),
    }
}

fn apply_approvals_result(drug: &mut Drug, result: Result<Vec<DrugApproval>, BioMcpError>) {
    match result {
        Ok(rows) => {
            let outcome = if rows.is_empty() {
                SectionOutcome::empty("OpenFDA Drugs@FDA")
            } else {
                SectionOutcome::data("OpenFDA Drugs@FDA")
            };
            drug.approvals = Some(rows);
            drug.section_outcomes.complete("approvals", outcome);
        }
        Err(_) => {
            drug.approvals = Some(Vec::new());
            drug.section_outcomes.complete(
                "approvals",
                SectionOutcome::unavailable("Drugs@FDA approvals are unavailable."),
            );
        }
    }
}

async fn add_approvals_section(drug: &mut Drug) {
    let name = drug.name.trim();
    if name.is_empty() {
        drug.approvals = Some(Vec::new());
        drug.section_outcomes.complete(
            "approvals",
            SectionOutcome::unavailable("Drugs@FDA approvals are unavailable."),
        );
        return;
    }

    let escaped = OpenFdaClient::escape_query_value(name);
    let query = if name.chars().any(|c| c.is_whitespace()) {
        format!(
            "openfda.generic_name:\"{escaped}\" OR openfda.brand_name:\"{escaped}\" OR products.brand_name:\"{escaped}\""
        )
    } else {
        format!(
            "openfda.generic_name:*{escaped}* OR openfda.brand_name:*{escaped}* OR products.brand_name:*{escaped}*"
        )
    };

    let approvals_fut = async {
        let client = OpenFdaClient::new()?;
        client.drugsfda_search(&query, 8, 0).await
    };

    let result = match tokio::time::timeout(OPTIONAL_SAFETY_TIMEOUT, approvals_fut).await {
        Ok(Ok(resp)) => Ok(resp.map(map_drugsfda_approvals).unwrap_or_default()),
        Ok(Err(err)) => Err(err),
        Err(_) => Err(BioMcpError::SourceUnavailable {
            source_name: "OpenFDA Drugs@FDA".to_string(),
            reason: "request timed out".to_string(),
            suggestion: "Retry the request.".to_string(),
        }),
    };
    apply_approvals_result(drug, result);
}

pub(super) struct ResolvedDrugBase {
    pub(super) drug: Drug,
    pub(super) label_response: Option<serde_json::Value>,
    pub(super) label_attempt_failed: bool,
    trial_alias_candidates: Vec<TrialAlias>,
}

enum SparseDrugDiscoverRescue {
    Canonical(String),
    AliasFallback,
    None,
}

fn normalized_discover_drug_label(value: &str) -> String {
    value.trim().trim_matches('.').to_ascii_lowercase()
}

async fn discover_sparse_drug_rescue(name: &str) -> SparseDrugDiscoverRescue {
    let Ok(result) = crate::entities::discover::resolve_query(
        name,
        crate::entities::discover::DiscoverMode::AliasFallback,
    )
    .await
    else {
        return SparseDrugDiscoverRescue::None;
    };

    let Some(top) = result.concepts.first() else {
        return SparseDrugDiscoverRescue::None;
    };

    let has_drug_signal = result
        .concepts
        .iter()
        .any(|concept| concept.primary_type == crate::entities::discover::DiscoverType::Drug);
    if !has_drug_signal {
        return SparseDrugDiscoverRescue::None;
    }

    if top.primary_type == crate::entities::discover::DiscoverType::Drug
        && top.match_tier == crate::entities::discover::MatchTier::Exact
        && top.confidence == crate::entities::discover::DiscoverConfidence::CanonicalId
    {
        let top_label = normalized_discover_drug_label(&top.label);
        let competing_exact_drug = result.concepts.iter().any(|concept| {
            concept.primary_type == crate::entities::discover::DiscoverType::Drug
                && concept.match_tier == crate::entities::discover::MatchTier::Exact
                && concept.confidence == crate::entities::discover::DiscoverConfidence::CanonicalId
                && normalized_discover_drug_label(&concept.label) != top_label
        });
        if !top_label.is_empty() && !competing_exact_drug {
            return SparseDrugDiscoverRescue::Canonical(top.label.clone());
        }
    }

    SparseDrugDiscoverRescue::AliasFallback
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrialAliasSource {
    Requested,
    Canonical,
    OpenFdaBrand,
    DrugBankSynonym,
}

impl TrialAliasSource {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Canonical => "canonical",
            Self::OpenFdaBrand => "openfda_brand",
            Self::DrugBankSynonym => "drugbank_synonym",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TrialAlias {
    pub(crate) label: String,
    pub(crate) source: TrialAliasSource,
}

#[derive(Clone)]
struct TrialAliasResolution {
    canonical_name: String,
    aliases: Vec<TrialAlias>,
}

struct TrialAliasLookup {
    canonical_name: String,
    candidates: Vec<TrialAlias>,
}

static TRIAL_ALIAS_CACHE: OnceLock<Mutex<HashMap<String, TrialAliasResolution>>> = OnceLock::new();

fn trial_alias_cache() -> &'static Mutex<HashMap<String, TrialAliasResolution>> {
    TRIAL_ALIAS_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn trial_alias_cache_key(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

fn is_investigational_code(alias: &str) -> bool {
    static CODE_RE: OnceLock<Regex> = OnceLock::new();
    CODE_RE
        .get_or_init(|| {
            Regex::new(r"(?i)^[a-z]+[ -]\d{2}[a-z0-9-]*$")
                .expect("valid investigational code regex")
        })
        .is_match(alias)
}

fn has_free_base_descriptor(alias: &str) -> bool {
    static FREE_BASE_RE: OnceLock<Regex> = OnceLock::new();
    FREE_BASE_RE
        .get_or_init(|| {
            Regex::new(r"(?i)\bfree(?:\s+|-+)base\b").expect("valid free-base descriptor regex")
        })
        .is_match(alias)
}

fn is_simple_trial_name(alias: &str) -> bool {
    alias.chars().count() <= 64
        && alias.split_whitespace().count() <= 4
        && alias
            .chars()
            .all(|ch| ch.is_alphanumeric() || ch.is_whitespace() || matches!(ch, '\'' | '-'))
}

fn eligible_drugbank_trial_alias(alias: &str) -> bool {
    !has_free_base_descriptor(alias)
        && (is_investigational_code(alias) || is_simple_trial_name(alias))
}

fn push_trial_alias(
    aliases: &mut Vec<TrialAlias>,
    seen: &mut HashSet<String>,
    alias: &str,
    source: TrialAliasSource,
) {
    let alias = alias.trim();
    if alias.is_empty() {
        return;
    }
    if seen.insert(alias.to_ascii_lowercase()) {
        aliases.push(TrialAlias {
            label: alias.to_string(),
            source,
        });
    }
}

fn build_trial_aliases(
    requested_name: &str,
    canonical_name: Option<&str>,
    candidates: &[TrialAlias],
) -> Vec<TrialAlias> {
    let mut aliases = Vec::new();
    let mut seen = HashSet::new();

    push_trial_alias(
        &mut aliases,
        &mut seen,
        requested_name,
        TrialAliasSource::Requested,
    );
    if let Some(canonical_name) = canonical_name {
        push_trial_alias(
            &mut aliases,
            &mut seen,
            canonical_name,
            TrialAliasSource::Canonical,
        );
    }

    let mut provider_aliases = 0;
    for source in [
        TrialAliasSource::OpenFdaBrand,
        TrialAliasSource::DrugBankSynonym,
    ] {
        let mut source_candidates = candidates
            .iter()
            .filter(|candidate| candidate.source == source)
            .filter(|candidate| {
                source == TrialAliasSource::OpenFdaBrand
                    || eligible_drugbank_trial_alias(candidate.label.trim())
            })
            .collect::<Vec<_>>();
        source_candidates.sort_by(|left, right| {
            left.label
                .trim()
                .to_ascii_lowercase()
                .cmp(&right.label.trim().to_ascii_lowercase())
                .then_with(|| left.label.trim().cmp(right.label.trim()))
        });
        for candidate in source_candidates {
            if provider_aliases >= 3 {
                break;
            }
            let previous_len = aliases.len();
            push_trial_alias(&mut aliases, &mut seen, &candidate.label, candidate.source);
            provider_aliases += usize::from(aliases.len() > previous_len);
        }
    }

    aliases
}

fn trial_alias_candidates_from_hits(hits: &[&MyChemHit]) -> Vec<TrialAlias> {
    let mut candidates = Vec::new();
    for hit in hits {
        if let Some(openfda) = &hit.openfda {
            candidates.extend(
                openfda
                    .brand_name
                    .clone()
                    .into_vec()
                    .into_iter()
                    .map(|label| TrialAlias {
                        label,
                        source: TrialAliasSource::OpenFdaBrand,
                    }),
            );
        }
        if let Some(drugbank) = &hit.drugbank {
            candidates.extend(drugbank.synonyms.iter().cloned().map(|label| TrialAlias {
                label,
                source: TrialAliasSource::DrugBankSynonym,
            }));
        }
    }
    candidates
}

fn trial_alias_resolution_from_lookup_result(
    requested_name: &str,
    result: Result<TrialAliasLookup, BioMcpError>,
) -> (TrialAliasResolution, bool) {
    match result {
        Ok(resolved) => (
            TrialAliasResolution {
                canonical_name: resolved.canonical_name.clone(),
                aliases: build_trial_aliases(
                    requested_name,
                    Some(&resolved.canonical_name),
                    &resolved.candidates,
                ),
            },
            true,
        ),
        Err(BioMcpError::NotFound { .. }) => (
            TrialAliasResolution {
                canonical_name: requested_name.to_string(),
                aliases: vec![TrialAlias {
                    label: requested_name.to_string(),
                    source: TrialAliasSource::Requested,
                }],
            },
            true,
        ),
        Err(err) => {
            warn!(
                drug = %requested_name,
                "Drug alias lookup unavailable for trial search: {err}"
            );
            (
                TrialAliasResolution {
                    canonical_name: requested_name.to_string(),
                    aliases: vec![TrialAlias {
                        label: requested_name.to_string(),
                        source: TrialAliasSource::Requested,
                    }],
                },
                false,
            )
        }
    }
}

async fn resolve_trial_alias_resolution(name: &str) -> Result<TrialAliasResolution, BioMcpError> {
    let requested_name = name.trim();
    if requested_name.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Trial intervention alias expansion requires a non-empty drug name".into(),
        ));
    }

    let cache_key = trial_alias_cache_key(requested_name);
    if let Ok(cache) = trial_alias_cache().lock()
        && let Some(cached) = cache.get(&cache_key)
    {
        let mut resolution = cached.clone();
        if let Some(requested_alias) = resolution.aliases.first_mut() {
            requested_alias.label = requested_name.to_string();
        }
        return Ok(resolution);
    }

    let lookup = resolve_drug_base(requested_name, false, false)
        .await
        .map(|resolved| TrialAliasLookup {
            canonical_name: resolved.drug.name,
            candidates: resolved.trial_alias_candidates,
        });
    let (resolution, cacheable) = trial_alias_resolution_from_lookup_result(requested_name, lookup);

    if cacheable && let Ok(mut cache) = trial_alias_cache().lock() {
        cache.insert(cache_key, resolution.clone());
    }

    Ok(resolution)
}

pub(crate) async fn resolve_trial_aliases(name: &str) -> Result<Vec<String>, BioMcpError> {
    Ok(resolve_trial_alias_resolution(name)
        .await?
        .aliases
        .into_iter()
        .map(|alias| alias.label)
        .collect())
}

pub(crate) async fn resolve_trial_aliases_with_sources(
    name: &str,
) -> Result<Vec<TrialAlias>, BioMcpError> {
    Ok(resolve_trial_alias_resolution(name).await?.aliases)
}

pub(crate) async fn resolve_trial_canonical_name(name: &str) -> Result<String, BioMcpError> {
    Ok(resolve_trial_alias_resolution(name).await?.canonical_name)
}

pub(super) async fn resolve_drug_base(
    name: &str,
    fetch_label_response: bool,
    label_required: bool,
) -> Result<ResolvedDrugBase, BioMcpError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Drug name is required. Example: biomcp get drug pembrolizumab".into(),
        ));
    }
    if name.len() > 256 {
        return Err(BioMcpError::InvalidArgument(
            "Drug name is too long.".into(),
        ));
    }

    let original_not_found = || BioMcpError::NotFound {
        entity: "drug".into(),
        id: name.to_string(),
        suggestion: format!("Try searching: biomcp search drug -q \"{name}\""),
    };

    let mut lookup_name = name.to_string();
    let mut resp = direct_drug_lookup(name).await?;

    if resp.hits.is_empty() {
        let fallback_filters = DrugSearchFilters {
            query: Some(name.to_string()),
            ..Default::default()
        };
        let fallback_name = search_page(&fallback_filters, 2, 0)
            .await
            .ok()
            .and_then(|page| {
                if page.results.len() != 1 {
                    return None;
                }
                let candidate = page.results[0].name.trim();
                if candidate.is_empty() || candidate.eq_ignore_ascii_case(name) {
                    None
                } else {
                    Some(candidate.to_string())
                }
            });

        if let Some(candidate) = fallback_name {
            if let Ok(fallback_resp) = direct_drug_lookup(&candidate).await
                && !fallback_resp.hits.is_empty()
            {
                lookup_name = candidate;
                resp = fallback_resp;
            } else {
                return Err(original_not_found());
            }
        } else {
            return Err(original_not_found());
        }
    }

    let mut selected = transform::drug::select_hits_for_name(&resp.hits, &lookup_name);
    let mut drug = transform::drug::merge_mychem_hits(&selected, &lookup_name);
    let needs_canonical_fallback =
        drug.drugbank_id.is_none() && drug.chembl_id.is_none() && drug.unii.is_none();
    if needs_canonical_fallback
        && let Ok(client) = OpenFdaClient::new()
        && let Ok(Some(label_response)) = client.label_search(name).await
        && let Some(candidate) =
            search_results_from_openfda_label_response(&label_response, name, 1)
                .into_iter()
                .next()
        && !candidate.name.eq_ignore_ascii_case(name)
        && let Ok(fallback_resp) = direct_drug_lookup(&candidate.name).await
        && !fallback_resp.hits.is_empty()
    {
        lookup_name = candidate.name;
        resp = fallback_resp;
        selected = transform::drug::select_hits_for_name(&resp.hits, &lookup_name);
        drug = transform::drug::merge_mychem_hits(&selected, &lookup_name);
    }

    if drug.drugbank_id.is_none() && drug.chembl_id.is_none() && drug.unii.is_none() {
        match discover_sparse_drug_rescue(name).await {
            SparseDrugDiscoverRescue::Canonical(candidate) => {
                if let Ok(fallback_resp) = direct_drug_lookup(&candidate).await
                    && !fallback_resp.hits.is_empty()
                {
                    lookup_name = candidate;
                    resp = fallback_resp;
                    selected = transform::drug::select_hits_for_name(&resp.hits, &lookup_name);
                    drug = transform::drug::merge_mychem_hits(&selected, &lookup_name);
                }
            }
            SparseDrugDiscoverRescue::AliasFallback => return Err(original_not_found()),
            SparseDrugDiscoverRescue::None => {}
        }
    }

    let mut label_response_opt: Option<serde_json::Value> = None;
    let mut label_attempt_failed = false;
    if fetch_label_response {
        match OpenFdaClient::new() {
            Ok(client) => match client.label_search(&drug.name).await {
                Ok(v) => label_response_opt = v,
                Err(err) => {
                    if label_required {
                        return Err(err);
                    }
                    label_attempt_failed = true;
                }
            },
            Err(err) => {
                if label_required {
                    return Err(err);
                }
                label_attempt_failed = true;
            }
        }
    }

    if let Some(label_response) = label_response_opt.as_ref() {
        apply_openfda_metadata(&mut drug, label_response);
        drug.label_set_id = extract_label_set_id(label_response);
    }

    let trial_alias_candidates = trial_alias_candidates_from_hits(&selected);
    Ok(ResolvedDrugBase {
        drug,
        label_response: label_response_opt,
        label_attempt_failed,
        trial_alias_candidates,
    })
}

async fn populate_common_sections(
    requested_name: &str,
    drug: &mut Drug,
    label_response: Option<&serde_json::Value>,
    section_flags: &DrugSections,
    raw_label: bool,
) -> Result<(), BioMcpError> {
    let (civic_context, civic_outcome) =
        if section_flags.include_targets || section_flags.include_civic {
            fetch_civic_therapy_context(&drug.name).await
        } else {
            (
                None,
                SectionOutcome::unavailable("CIViC drug evidence is temporarily unavailable."),
            )
        };

    drug.label = if section_flags.include_label {
        label_response.and_then(|response| extract_inline_label(response, raw_label))
    } else {
        None
    };

    if section_flags.include_interactions {
        let report = super::interaction_report_from_base(
            requested_name.to_string(),
            drug.clone(),
            label_response.cloned(),
            super::interactions::DEFAULT_INTERACTION_LIMIT,
            0,
        )
        .await?;
        super::apply_interaction_report(drug, &report);
    } else {
        drug.interactions.clear();
        drug.interaction_text = None;
        drug.interaction_pagination = None;
        drug.interaction_bundle_freshness = None;
    }

    if section_flags.include_targets {
        let outcome = enrich_targets(drug, civic_context.as_ref()).await;
        drug.section_outcomes.complete("targets", outcome);
    } else {
        drug.variant_targets.clear();
    }

    if section_flags.include_indications {
        let outcome = enrich_indications(drug).await;
        drug.section_outcomes.complete("indications", outcome);
    }

    if section_flags.include_civic {
        drug.civic = Some(civic_context.unwrap_or_default());
        drug.section_outcomes.complete("civic", civic_outcome);
    } else {
        drug.civic = None;
    }
    Ok(())
}

async fn populate_top_adverse_event_preview(drug: &mut Drug) -> bool {
    match tokio::time::timeout(
        OPTIONAL_SAFETY_TIMEOUT,
        fetch_top_adverse_events(&drug.name),
    )
    .await
    {
        Ok(Ok((events, faers_query))) => {
            drug.top_adverse_events = events;
            drug.faers_query = faers_query;
            false
        }
        Ok(Err(_)) | Err(_) => true,
    }
}

async fn populate_us_regional_sections(
    drug: &mut Drug,
    label_response: Option<&serde_json::Value>,
    section_flags: &DrugSections,
) -> Result<(), BioMcpError> {
    if section_flags.include_shortage {
        drug.shortage = Some(fetch_shortage_entries(&drug.name).await?);
    } else {
        drug.shortage = None;
    }

    if section_flags.include_regulatory || section_flags.include_approvals {
        add_approvals_section(drug).await;
    } else {
        drug.approvals = None;
    }

    drug.us_safety_warnings = if section_flags.include_safety {
        label_response.and_then(extract_label_warnings_text)
    } else {
        None
    };

    Ok(())
}

async fn populate_ema_sections(
    drug: &mut Drug,
    requested_name: &str,
    section_flags: &DrugSections,
) -> Result<bool, BioMcpError> {
    if !section_flags.include_regulatory
        && !section_flags.include_safety
        && !section_flags.include_shortage
    {
        drug.ema_regulatory = None;
        drug.ema_safety = None;
        drug.ema_shortage = None;
        return Ok(false);
    }

    let safety_only = section_flags.include_safety
        && !section_flags.include_regulatory
        && !section_flags.include_shortage;
    let client = match EmaClient::ready(EmaSyncMode::Auto).await {
        Ok(client) => client,
        Err(_) if safety_only => {
            drug.ema_safety = None;
            return Ok(true);
        }
        Err(err) => return Err(err),
    };
    let identity = build_ema_identity(requested_name, drug);
    let anchor = match client.resolve_anchor(&identity) {
        Ok(anchor) => anchor,
        Err(_) if safety_only => {
            drug.ema_safety = None;
            return Ok(true);
        }
        Err(err) => return Err(err),
    };

    drug.ema_regulatory = if section_flags.include_regulatory {
        Some(client.regulatory(&anchor)?)
    } else {
        None
    };
    let mut safety_failed = false;
    drug.ema_safety = if section_flags.include_safety {
        match client.safety(&anchor) {
            Ok(safety) => Some(safety),
            Err(_) if safety_only => {
                safety_failed = true;
                None
            }
            Err(err) => return Err(err),
        }
    } else {
        None
    };
    drug.ema_shortage = if section_flags.include_shortage {
        Some(client.shortages(&anchor)?)
    } else {
        None
    };

    Ok(safety_failed)
}

async fn populate_who_sections(
    drug: &mut Drug,
    requested_name: &str,
    section_flags: &DrugSections,
) -> Result<(), BioMcpError> {
    if !section_flags.include_regulatory {
        drug.who_prequalification = None;
        return Ok(());
    }

    let client = WhoPqClient::ready(WhoPqSyncMode::Auto).await?;
    let identity = build_who_identity(requested_name, drug);
    drug.who_prequalification = Some(client.regulatory(&identity, WhoProductTypeFilter::Both)?);
    Ok(())
}

fn validate_region_usage(
    section_flags: &DrugSections,
    region: DrugRegion,
    region_explicit: bool,
) -> Result<(), BioMcpError> {
    if !region_explicit {
        return Ok(());
    }

    if section_flags.include_approvals {
        return Err(BioMcpError::InvalidArgument(
            "--region is not supported with approvals. Use regulatory for the regional regulatory view.".into(),
        ));
    }

    if !(section_flags.include_regulatory
        || section_flags.include_safety
        || section_flags.include_shortage)
    {
        return Err(BioMcpError::InvalidArgument(
            "--region can only be used with regulatory, safety, shortage, or all.".into(),
        ));
    }

    if matches!(region, DrugRegion::Who)
        && (section_flags.requested_safety || section_flags.requested_shortage)
        && !section_flags.requested_all
    {
        return Err(BioMcpError::InvalidArgument(
            "WHO regional data currently supports regulatory only; use --region us|eu for safety or shortage, or use --region who with regulatory/all.".into(),
        ));
    }

    Ok(())
}

fn validate_raw_usage(section_flags: &DrugSections, raw_label: bool) -> Result<(), BioMcpError> {
    if raw_label && !section_flags.include_label {
        return Err(BioMcpError::InvalidArgument(
            "--raw can only be used with label or all.".into(),
        ));
    }
    Ok(())
}

pub fn get_with_region(
    name: &str,
    sections: &[String],
    region: DrugRegion,
    region_explicit: bool,
    raw_label: bool,
) -> impl std::future::Future<Output = Result<Drug, BioMcpError>> + Send {
    let name = name.to_string();
    let sections = sections.to_vec();
    async move { get_with_region_owned(name, sections, region, region_explicit, raw_label).await }
}

async fn get_with_region_owned(
    name: String,
    sections: Vec<String>,
    region: DrugRegion,
    region_explicit: bool,
    raw_label: bool,
) -> Result<Drug, BioMcpError> {
    let section_flags = parse_sections_for_name(&name, &sections)?;
    validate_region_usage(&section_flags, region, region_explicit)?;
    validate_raw_usage(&section_flags, raw_label)?;

    let section_only = is_section_only_requested(&sections);
    let fetch_label_response = !section_only
        || section_flags.include_label
        || section_flags.include_interactions
        || (region.includes_us() && section_flags.include_safety);

    let mut resolved =
        resolve_drug_base(&name, fetch_label_response, section_flags.include_label).await?;
    populate_common_sections(
        &name,
        &mut resolved.drug,
        resolved.label_response.as_ref(),
        &section_flags,
        raw_label,
    )
    .await?;

    let faers_failed = if region.includes_us() && (!section_only || section_flags.include_safety) {
        Some(populate_top_adverse_event_preview(&mut resolved.drug).await)
    } else {
        resolved.drug.top_adverse_events.clear();
        resolved.drug.faers_query = None;
        None
    };

    if region.includes_us() {
        populate_us_regional_sections(
            &mut resolved.drug,
            resolved.label_response.as_ref(),
            &section_flags,
        )
        .await?;
    } else {
        resolved.drug.shortage = None;
        resolved.drug.approvals = None;
        resolved.drug.us_safety_warnings = None;
    }

    let ema_safety_failed = if region.includes_eu() {
        populate_ema_sections(&mut resolved.drug, &name, &section_flags).await?
    } else {
        resolved.drug.ema_regulatory = None;
        resolved.drug.ema_safety = None;
        resolved.drug.ema_shortage = None;
        false
    };

    if region.includes_who() {
        populate_who_sections(&mut resolved.drug, &name, &section_flags).await?;
    } else {
        resolved.drug.who_prequalification = None;
    }

    if section_flags.include_safety && (region.includes_us() || region.includes_eu()) {
        let mut successful_sources = Vec::new();
        let mut contributors = Vec::new();
        let mut failed = false;
        if region.includes_us() {
            if faers_failed == Some(true) {
                failed = true;
            } else {
                successful_sources.push("OpenFDA FAERS");
                if !resolved.drug.top_adverse_events.is_empty() {
                    contributors.push("OpenFDA FAERS");
                }
            }
            if resolved.label_attempt_failed {
                failed = true;
            } else {
                successful_sources.push("OpenFDA label");
                if resolved
                    .drug
                    .us_safety_warnings
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                {
                    contributors.push("OpenFDA label");
                }
            }
        }
        if region.includes_eu() {
            if ema_safety_failed {
                failed = true;
            } else {
                successful_sources.push("EMA");
            }
            if resolved.drug.ema_safety.as_ref().is_some_and(|safety| {
                !safety.dhpcs.is_empty()
                    || !safety.referrals.is_empty()
                    || !safety.psusas.is_empty()
            }) {
                contributors.push("EMA");
            }
        }
        let outcome = if failed {
            if contributors.is_empty() {
                SectionOutcome::unavailable("Drug safety evidence is temporarily unavailable.")
            } else {
                SectionOutcome::degraded(
                    contributors,
                    "Drug safety evidence is incomplete because a source was unavailable.",
                )
            }
        } else if contributors.is_empty() {
            SectionOutcome::empty_sources(successful_sources)
        } else {
            SectionOutcome::data_sources(contributors)
        };
        resolved.drug.section_outcomes.complete("safety", outcome);
    }

    Ok(resolved.drug)
}

pub fn get(
    name: &str,
    sections: &[String],
) -> impl std::future::Future<Output = Result<Drug, BioMcpError>> + Send {
    get_with_region(name, sections, DrugRegion::Us, false, false)
}

#[cfg(test)]
mod tests;
