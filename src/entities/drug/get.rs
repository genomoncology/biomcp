//! Drug retrieval workflows, section parsing, and region validation.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use regex::Regex;
use tracing::warn;

use crate::error::BioMcpError;
use crate::sources::civic::{CivicClient, CivicContext};
use crate::sources::ema::{EmaClient, EmaSyncMode};
use crate::sources::openfda::OpenFdaClient;
use crate::sources::who_pq::{WhoPqClient, WhoPqSyncMode, WhoProductTypeFilter};
use crate::transform;

use super::label::{
    extract_inline_label, extract_interaction_text_from_label, extract_label_set_id,
    extract_label_warnings_text,
};
use super::metadata::{
    apply_openfda_metadata, fetch_shortage_entries, fetch_top_adverse_events,
    map_drugsfda_approvals,
};
use super::search::{search_page, search_results_from_openfda_label_response};
use super::targets::{enrich_indications, enrich_targets};
use super::{
    DRUG_SECTION_ALL, DRUG_SECTION_APPROVALS, DRUG_SECTION_CIVIC, DRUG_SECTION_INDICATIONS,
    DRUG_SECTION_INTERACTIONS, DRUG_SECTION_LABEL, DRUG_SECTION_NAMES, DRUG_SECTION_REGULATORY,
    DRUG_SECTION_SAFETY, DRUG_SECTION_SHORTAGE, DRUG_SECTION_TARGETS, Drug, DrugRegion,
    DrugSearchFilters, OPTIONAL_SAFETY_TIMEOUT, build_ema_identity, build_who_identity,
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

fn parse_sections(sections: &[String]) -> Result<DrugSections, BioMcpError> {
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
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown section \"{section}\" for drug. Available: {}",
                    DRUG_SECTION_NAMES.join(", ")
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

fn is_section_only_requested(sections: &[String]) -> bool {
    !sections
        .iter()
        .any(|section| section.trim().eq_ignore_ascii_case(DRUG_SECTION_ALL))
        && sections.iter().any(|section| !section.trim().is_empty())
}

async fn fetch_civic_therapy_context(name: &str) -> Option<CivicContext> {
    let name = name.trim();
    if name.is_empty() {
        return Some(CivicContext::default());
    }

    let civic_fut = async {
        let client = CivicClient::new()?;
        client.by_therapy(name, 10).await
    };

    match tokio::time::timeout(OPTIONAL_SAFETY_TIMEOUT, civic_fut).await {
        Ok(Ok(context)) => Some(context),
        Ok(Err(err)) => {
            warn!(drug = %name, "CIViC unavailable for drug section: {err}");
            None
        }
        Err(_) => {
            warn!(
                drug = %name,
                timeout_secs = OPTIONAL_SAFETY_TIMEOUT.as_secs(),
                "CIViC drug section timed out"
            );
            None
        }
    }
}

async fn add_approvals_section(drug: &mut Drug) {
    let name = drug.name.trim();
    if name.is_empty() {
        drug.approvals = Some(Vec::new());
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

    match tokio::time::timeout(OPTIONAL_SAFETY_TIMEOUT, approvals_fut).await {
        Ok(Ok(resp)) => {
            let approvals = resp.map(map_drugsfda_approvals).unwrap_or_default();
            drug.approvals = Some(approvals);
        }
        Ok(Err(err)) => {
            warn!(drug = %drug.name, "OpenFDA Drugs@FDA unavailable: {err}");
            drug.approvals = Some(Vec::new());
        }
        Err(_) => {
            warn!(
                drug = %drug.name,
                timeout_secs = OPTIONAL_SAFETY_TIMEOUT.as_secs(),
                "OpenFDA Drugs@FDA section timed out"
            );
            drug.approvals = Some(Vec::new());
        }
    }
}

struct ResolvedDrugBase {
    drug: Drug,
    label_response: Option<serde_json::Value>,
}

#[derive(Clone)]
struct TrialAliasResolution {
    canonical_name: String,
    aliases: Vec<String>,
}

static TRIAL_ALIAS_CACHE: OnceLock<Mutex<HashMap<String, TrialAliasResolution>>> = OnceLock::new();

fn trial_alias_cache() -> &'static Mutex<HashMap<String, TrialAliasResolution>> {
    TRIAL_ALIAS_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn trial_alias_cache_key(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

fn looks_like_trial_formulation_variant(alias: &str) -> bool {
    static STRENGTH_RE: OnceLock<Regex> = OnceLock::new();
    static FORMULATION_RE: OnceLock<Regex> = OnceLock::new();

    let strength_re = STRENGTH_RE.get_or_init(|| {
        Regex::new(r"(?i)\b\d+(?:\.\d+)?\s*(?:mg|g|mcg|μg|ug|ml)(?:\s*/\s*(?:ml|l))?\b")
            .expect("valid strength regex")
    });
    if strength_re.is_match(alias) {
        return true;
    }

    FORMULATION_RE
        .get_or_init(|| {
            Regex::new(r"(?i)\b(tablet|capsule|injection|solution|suspension)\b")
                .expect("valid formulation regex")
        })
        .is_match(alias)
}

fn push_trial_alias(
    aliases: &mut Vec<String>,
    seen: &mut HashSet<String>,
    alias: &str,
    filter_formulation_variant: bool,
) {
    let alias = alias.trim();
    if alias.is_empty() {
        return;
    }
    if filter_formulation_variant && looks_like_trial_formulation_variant(alias) {
        return;
    }

    let key = alias.to_ascii_lowercase();
    if seen.insert(key) {
        aliases.push(alias.to_string());
    }
}

fn build_trial_aliases(
    requested_name: &str,
    canonical_name: Option<&str>,
    brand_names: &[String],
) -> Vec<String> {
    let mut aliases = Vec::new();
    let mut seen = HashSet::new();

    push_trial_alias(&mut aliases, &mut seen, requested_name, false);
    if let Some(canonical_name) = canonical_name {
        push_trial_alias(&mut aliases, &mut seen, canonical_name, false);
    }
    for brand_name in brand_names {
        push_trial_alias(&mut aliases, &mut seen, brand_name, true);
    }

    aliases
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
        return Ok(cached.clone());
    }

    let (resolution, cacheable) = match resolve_drug_base(requested_name, false, false).await {
        Ok(resolved) => (
            TrialAliasResolution {
                canonical_name: resolved.drug.name.clone(),
                aliases: build_trial_aliases(
                    requested_name,
                    Some(&resolved.drug.name),
                    &resolved.drug.brand_names,
                ),
            },
            true,
        ),
        Err(BioMcpError::NotFound { .. }) => (
            TrialAliasResolution {
                canonical_name: requested_name.to_string(),
                aliases: vec![requested_name.to_string()],
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
                    aliases: vec![requested_name.to_string()],
                },
                false,
            )
        }
    };

    if cacheable && let Ok(mut cache) = trial_alias_cache().lock() {
        cache.insert(cache_key, resolution.clone());
    }

    Ok(resolution)
}

pub(crate) async fn resolve_trial_aliases(name: &str) -> Result<Vec<String>, BioMcpError> {
    Ok(resolve_trial_alias_resolution(name).await?.aliases)
}

pub(crate) async fn resolve_trial_canonical_name(name: &str) -> Result<String, BioMcpError> {
    Ok(resolve_trial_alias_resolution(name).await?.canonical_name)
}

async fn resolve_drug_base(
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

    let mut label_response_opt: Option<serde_json::Value> = None;
    if fetch_label_response {
        match OpenFdaClient::new() {
            Ok(client) => match client.label_search(&drug.name).await {
                Ok(v) => label_response_opt = v,
                Err(err) => {
                    if label_required {
                        return Err(err);
                    }
                }
            },
            Err(err) => {
                if label_required {
                    return Err(err);
                }
            }
        }
    }

    if let Some(label_response) = label_response_opt.as_ref() {
        apply_openfda_metadata(&mut drug, label_response);
        drug.label_set_id = extract_label_set_id(label_response);
    }

    Ok(ResolvedDrugBase {
        drug,
        label_response: label_response_opt,
    })
}

async fn populate_common_sections(
    drug: &mut Drug,
    label_response: Option<&serde_json::Value>,
    section_flags: &DrugSections,
    raw_label: bool,
) {
    let civic_context = if section_flags.include_targets || section_flags.include_civic {
        fetch_civic_therapy_context(&drug.name).await
    } else {
        None
    };

    drug.label = if section_flags.include_label {
        label_response.and_then(|response| extract_inline_label(response, raw_label))
    } else {
        None
    };

    if section_flags.include_interactions {
        drug.interaction_text = label_response.and_then(extract_interaction_text_from_label);
    } else {
        drug.interactions.clear();
        drug.interaction_text = None;
    }

    if section_flags.include_targets {
        enrich_targets(drug, civic_context.as_ref()).await;
    } else {
        drug.variant_targets.clear();
    }

    if section_flags.include_indications {
        enrich_indications(drug).await;
    }

    if section_flags.include_civic {
        drug.civic = Some(civic_context.unwrap_or_default());
    } else {
        drug.civic = None;
    }
}

async fn populate_top_adverse_event_preview(drug: &mut Drug) {
    match tokio::time::timeout(
        OPTIONAL_SAFETY_TIMEOUT,
        fetch_top_adverse_events(&drug.name),
    )
    .await
    {
        Ok(Ok((events, faers_query))) => {
            drug.top_adverse_events = events;
            drug.faers_query = faers_query;
        }
        Ok(Err(err)) => {
            warn!(
                drug = %drug.name,
                "OpenFDA adverse-event preview unavailable: {err}"
            );
        }
        Err(_) => {
            warn!(
                drug = %drug.name,
                timeout_secs = OPTIONAL_SAFETY_TIMEOUT.as_secs(),
                "OpenFDA adverse-event preview timed out"
            );
        }
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
) -> Result<(), BioMcpError> {
    if !section_flags.include_regulatory
        && !section_flags.include_safety
        && !section_flags.include_shortage
    {
        drug.ema_regulatory = None;
        drug.ema_safety = None;
        drug.ema_shortage = None;
        return Ok(());
    }

    let client = EmaClient::ready(EmaSyncMode::Auto).await?;
    let identity = build_ema_identity(requested_name, drug);
    let anchor = client.resolve_anchor(&identity)?;

    drug.ema_regulatory = if section_flags.include_regulatory {
        Some(client.regulatory(&anchor)?)
    } else {
        None
    };
    drug.ema_safety = if section_flags.include_safety {
        Some(client.safety(&anchor)?)
    } else {
        None
    };
    drug.ema_shortage = if section_flags.include_shortage {
        Some(client.shortages(&anchor)?)
    } else {
        None
    };

    Ok(())
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

pub async fn get_with_region(
    name: &str,
    sections: &[String],
    region: DrugRegion,
    region_explicit: bool,
    raw_label: bool,
) -> Result<Drug, BioMcpError> {
    let section_flags = parse_sections(sections)?;
    validate_region_usage(&section_flags, region, region_explicit)?;
    validate_raw_usage(&section_flags, raw_label)?;

    let section_only = is_section_only_requested(sections);
    let fetch_label_response = !section_only
        || section_flags.include_label
        || section_flags.include_interactions
        || (region.includes_us() && section_flags.include_safety);

    let mut resolved =
        resolve_drug_base(name, fetch_label_response, section_flags.include_label).await?;
    populate_common_sections(
        &mut resolved.drug,
        resolved.label_response.as_ref(),
        &section_flags,
        raw_label,
    )
    .await;

    if region.includes_us() && (!section_only || section_flags.include_safety) {
        populate_top_adverse_event_preview(&mut resolved.drug).await;
    } else {
        resolved.drug.top_adverse_events.clear();
        resolved.drug.faers_query = None;
    }

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

    if region.includes_eu() {
        populate_ema_sections(&mut resolved.drug, name, &section_flags).await?;
    } else {
        resolved.drug.ema_regulatory = None;
        resolved.drug.ema_safety = None;
        resolved.drug.ema_shortage = None;
    }

    if region.includes_who() {
        populate_who_sections(&mut resolved.drug, name, &section_flags).await?;
    } else {
        resolved.drug.who_prequalification = None;
    }

    Ok(resolved.drug)
}

pub async fn get(name: &str, sections: &[String]) -> Result<Drug, BioMcpError> {
    get_with_region(name, sections, DrugRegion::Us, false, false).await
}

#[cfg(test)]
mod tests;
