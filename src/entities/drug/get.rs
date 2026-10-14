//! Drug retrieval workflows, section parsing, and region validation.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use regex::Regex;
use tracing::debug as warn;

use crate::entities::section_outcome::SectionOutcome;
use crate::error::BioMcpError;
use crate::sources::civic::{CivicClient, CivicContext};
use crate::sources::ema::{EmaClient, EmaSyncMode};
use crate::sources::mychem::{MyChemHit, MyChemQueryResponse};
use crate::sources::openfda::OpenFdaClient;
use crate::sources::who_pq::{WhoPqClient, WhoPqSyncMode, WhoProductTypeFilter};
use crate::transform;

use super::label::{
    ELEMENTS_SEARCH_OVERSIZE_NOTE, LABEL_UNAVAILABLE_MESSAGE, LabelLookup, NO_LABEL_TEXT_NOTE,
    NO_SPL_RECORD_NOTE, extract_inline_label, extract_label_boxed_warning, extract_label_set_id,
    extract_label_warnings_text, lookup_label_response,
};
use super::metadata::{
    apply_openfda_metadata, fetch_shortage_entries, map_drugsfda_approvals, orphan_aliases,
    populate_top_adverse_event_preview,
};
use super::search::search_page;
use super::targets::{enrich_indications, enrich_targets};
use super::{
    DRUG_SECTION_ALL, DRUG_SECTION_APPROVALS, DRUG_SECTION_CELL_LINES, DRUG_SECTION_CIVIC,
    DRUG_SECTION_INDICATIONS, DRUG_SECTION_INTERACTIONS, DRUG_SECTION_LABEL, DRUG_SECTION_NAMES,
    DRUG_SECTION_REGULATORY, DRUG_SECTION_SAFETY, DRUG_SECTION_SHORTAGE, DRUG_SECTION_TARGETS,
    Drug, DrugApproval, DrugRegion, DrugSearchFilters, OPTIONAL_SAFETY_TIMEOUT, build_ema_identity,
    build_who_identity, direct_drug_lookup,
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
    include_cell_lines: bool,
    requested_all: bool,
    requested_safety: bool,
    requested_shortage: bool,
    allow_partial_interactions: bool,
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
            DRUG_SECTION_CELL_LINES => out.include_cell_lines = true,
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
    out.allow_partial_interactions = out.include_interactions;

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
    pub(super) label_elements_oversize: bool,
    trial_alias_candidates: Vec<TrialAlias>,
    selected_hits: Vec<MyChemHit>,
}

#[derive(Clone)]
enum SparseDrugDiscoverRescue {
    Canonical(String),
    AliasFallback,
    None,
}

/// The identity search's answer for one drug name: the candidate the
/// page's records agree on, no answer when the page casts no deciding
/// vote, or the disagreement itself — several distinct ingredients —
/// which the name-miss caller refuses as an ambiguous drug name while
/// the card-keeping callers treat it as no answer.
enum LabelIdentityAnswer {
    Candidate(String),
    NoAnswer,
    Ambiguous(Vec<AmbiguousIngredient>),
}

/// One refusal candidate: a distinct single ingredient the counted label
/// records carry, and how many records carry it.
struct AmbiguousIngredient {
    records: usize,
    name: String,
}

/// One counted label record's identity contribution: the single
/// ingredients its own fields list. Owner records and page records sit in
/// separate lists, so the vote mode decides which records count.
struct LabelRecordVote {
    ingredients: Vec<String>,
}

async fn openfda_label_identity_candidate(name: &str) -> LabelIdentityAnswer {
    let Ok(client) = OpenFdaClient::new() else {
        return LabelIdentityAnswer::NoAnswer;
    };
    let Some(response) = client.label_search(name).await.ok().flatten() else {
        return LabelIdentityAnswer::NoAnswer;
    };
    let Some(results) = response.get("results").and_then(|value| value.as_array()) else {
        return LabelIdentityAnswer::NoAnswer;
    };
    label_identity_answer(name, results)
}

/// A secondary identity resolution through openFDA's field-scoped label
/// search: the label records the page
/// holds vote on the drug they name. A record whose own brand equals the
/// query owns it — only owner rows vote when any exist — and
/// a vote reads as the plain ingredient the record's own ingredient list
/// supports, so an under-the-skin pairing still resolves to the plain
/// ingredient it names (Darzalex to daratumumab, Qlex to pembrolizumab)
/// while a true two-drug combination stays whole (Opdualag).
/// Votes compare as ingredient word sets, so one product's
/// spelling variants count together ("acetaminophen, aspirin, caffeine"
/// and "acetaminophen, aspirin and caffeine" are one vote).
///
/// The page resolves the query when its records agree: one shared vote
/// names the drug whether it is a plain ingredient or a combination
/// product line, and short of that, exactly one plain single-ingredient
/// vote among combination votes is the brand whose labels mostly name one
/// ingredient — Motrin's labels name ibuprofen. Anything
/// else — several plain ingredients ("Sleep Aid"), several combinations
/// ("Cold and Flu"), unrelated singles ("Pain Relief") — names no one
/// drug: the name-miss caller refuses it as an ambiguous drug name whose
/// candidates are the ingredients the counted records carry, while the
/// card-keeping callers keep the card they already hold. Some label
/// records carry no openFDA identity block at all, so this path can miss
/// (a brand MyChem itself carries resolves through the MyChem record
/// instead).
fn label_identity_answer(query: &str, results: &[serde_json::Value]) -> LabelIdentityAnswer {
    use std::collections::BTreeSet;

    if results.is_empty() {
        return LabelIdentityAnswer::NoAnswer;
    }
    let query = query.trim().to_ascii_lowercase();
    // One vote per record: the plain ingredient the record's own identity
    // supports, so counting votes counts records.
    let mut owner_votes: Vec<String> = Vec::new();
    let mut page_votes: Vec<String> = Vec::new();
    let mut owner_records: Vec<LabelRecordVote> = Vec::new();
    let mut page_records: Vec<LabelRecordVote> = Vec::new();
    for result in results {
        let Some(raw) = super::label::label_result_generic_names(result)
            .into_iter()
            .next()
            .map(|generic| generic.trim().to_ascii_lowercase())
            .filter(|generic| !generic.is_empty())
        else {
            continue;
        };
        let ingredients: Vec<String> = super::label::label_result_ingredient_names(result)
            .into_iter()
            .map(|ingredient| ingredient.trim().to_ascii_lowercase())
            .filter(|ingredient| !ingredient.is_empty())
            .collect();
        let plain = super::label::plain_paired_ingredient_name(&raw, &ingredients)
            .trim()
            .to_ascii_lowercase();
        if plain.is_empty() || plain == query {
            continue;
        }
        let owns_brand = super::label::extract_openfda_values_from_result(result, "brand_name")
            .iter()
            .any(|brand| brand.trim().to_ascii_lowercase() == query);
        let vote = LabelRecordVote { ingredients };
        if owns_brand {
            owner_votes.push(plain.clone());
            owner_records.push(vote);
        } else {
            page_records.push(vote);
        }
        page_votes.push(plain);
    }
    // Only owner rows vote when any exist, and the refusal's candidates
    // come from the same counted records that voted.
    let (votes, counted): (&[String], &[LabelRecordVote]) = if owner_votes.is_empty() {
        (&page_votes, &page_records)
    } else {
        (&owner_votes, &owner_records)
    };
    if votes.is_empty() {
        return LabelIdentityAnswer::NoAnswer;
    }
    let vote_words = |vote: &str| -> BTreeSet<String> { super::label::ingredient_word_set(vote) };
    let mut distinct_sets: Vec<BTreeSet<String>> = Vec::new();
    for vote in votes {
        let words = vote_words(vote);
        if !distinct_sets.contains(&words) {
            distinct_sets.push(words);
        }
    }
    // One shared vote: the majority spelling of the one ingredient set —
    // a plain ingredient or a combination product line the records agree
    // on ("Headache Relief" is the acetaminophen-aspirin-caffeine line).
    if distinct_sets.len() == 1 {
        let mut best = votes[0].clone();
        let mut best_count = 0;
        for vote in votes {
            let count = votes.iter().filter(|seen| seen == &vote).count();
            if count > best_count {
                best_count = count;
                best = vote.clone();
            }
        }
        return LabelIdentityAnswer::Candidate(best);
    }
    // Short of agreement, exactly one plain single-ingredient vote among
    // the combination votes is the brand whose labels mostly name one
    // ingredient.
    let mut plain_singles: Vec<&String> = Vec::new();
    for vote in votes {
        if !vote.contains(" and ") && !vote.contains(',') {
            plain_singles.push(vote);
        }
    }
    let mut plain_sets: Vec<BTreeSet<String>> = Vec::new();
    for vote in &plain_singles {
        let words = vote_words(vote);
        if !plain_sets.contains(&words) {
            plain_sets.push(words);
        }
    }
    if plain_sets.len() == 1 {
        return LabelIdentityAnswer::Candidate((*plain_singles[0]).clone());
    }
    // Anything else names no one drug: several plain ingredients ("Sleep
    // Aid"), several combinations ("Cold and Flu"), or unrelated singles
    // ("Pain Relief") — several drug products carry the name, and the
    // refusal lists the single ingredients those products' own records
    // carry.
    LabelIdentityAnswer::Ambiguous(ambiguous_ingredient_candidates(counted))
}

/// The refusal's candidates: the distinct single ingredients the counted
/// records carry — the structural ingredient list each record's own
/// fields name — one per ingredient, counted by records, spelled as most
/// records spell it, sorted by record count and then by name, capped at
/// ten. Spelling variants count as one ingredient, the same way votes
/// compare as ingredient word sets.
fn ambiguous_ingredient_candidates(records: &[LabelRecordVote]) -> Vec<AmbiguousIngredient> {
    use std::collections::{BTreeSet, HashMap};

    let mut grouped: Vec<(BTreeSet<String>, HashMap<String, usize>)> = Vec::new();
    for record in records {
        // One record counts once per ingredient, never once per mention.
        let mut counted = BTreeSet::new();
        for ingredient in &record.ingredients {
            let words = super::label::ingredient_word_set(ingredient);
            if words.is_empty() || !counted.insert(words.clone()) {
                continue;
            }
            match grouped.iter_mut().find(|(set, _)| *set == words) {
                Some((_, spellings)) => {
                    *spellings.entry(ingredient.clone()).or_insert(0) += 1;
                }
                None => grouped.push((words, HashMap::from([(ingredient.clone(), 1)]))),
            }
        }
    }
    let mut ranked: Vec<AmbiguousIngredient> = grouped
        .into_iter()
        .map(|(_, spellings)| {
            let records = spellings.values().sum();
            let name = spellings
                .into_iter()
                .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
                .map(|(name, _)| name)
                .unwrap_or_default();
            AmbiguousIngredient { records, name }
        })
        .collect();
    ranked.sort_by(|left, right| {
        right
            .records
            .cmp(&left.records)
            .then_with(|| left.name.cmp(&right.name))
    });
    ranked.truncate(10);
    ranked
}

/// The ambiguous-name refusal, the disease refusal's shape
/// (`src/entities/disease/resolution.rs`) applied to a drug name several
/// products carry: `InvalidArgument`, a `Candidates:` list, exit 2. The
/// Try line names the ingredient form, the one spelling that resolves.
fn ambiguous_drug_name_error(name: &str, candidates: &[AmbiguousIngredient]) -> BioMcpError {
    let lines = candidates
        .iter()
        .map(|candidate| format!("- {}\n", candidate.name))
        .collect::<String>();
    BioMcpError::InvalidArgument(format!(
        "Ambiguous drug name '{name}': several drug products carry this name.\n\
Candidates:\n{lines}\
Try `biomcp get drug \"<ingredient>\"`"
    ))
}

/// A MyChem response for `name` whose own names match `name`: `None` when
/// only other drugs' records matched the full-text query.
async fn named_drug_response(name: &str) -> Option<MyChemQueryResponse> {
    let resp = direct_drug_lookup(name).await.ok()?;
    (!transform::drug::select_hits_for_name(&resp.hits, name).is_empty()).then_some(resp)
}

/// The honest no-match refusal: name what the MyChem text
/// query matched, so a wrong card is never silently substituted.
fn name_miss_not_found(name: &str, hits: &[MyChemHit]) -> BioMcpError {
    let mut seen = HashSet::new();
    let candidates = hits
        .iter()
        .filter_map(transform::drug::from_mychem_search_hit)
        .map(|row| row.name)
        .filter(|candidate| seen.insert(candidate.clone()))
        .take(3)
        .collect::<Vec<_>>();
    let matched_note = if candidates.is_empty() {
        String::new()
    } else {
        format!(
            " The MyChem text search matched other drugs: {}.",
            candidates.join(", ")
        )
    };
    BioMcpError::NotFound {
        entity: "drug".into(),
        id: name.to_string(),
        suggestion: format!(
            "No drug card matches \"{name}\" by name, brand, or synonym.{matched_note} Try searching: biomcp search drug -q \"{name}\""
        ),
    }
}

fn normalized_discover_drug_label(value: &str) -> String {
    value.trim().trim_matches('.').to_ascii_lowercase()
}

/// The discover rescue runs against OLS4 with no deadline of its own, and a
/// slow round there must never hold a refusal hostage: the rescue gives up
/// after the refusal-path budget and the caller refuses without it
///.
const DISCOVER_RESCUE_TIMEOUT: std::time::Duration =
    crate::entities::discover::REFUSAL_DISCOVER_TIMEOUT;

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

/// The discover rescue inside its time budget: `None` when it ran out, so
/// the name-miss arm refuses without it and the sparse-card arm keeps the
/// card it already holds. A timeout remembers the deadline the round
/// started from, so the CLI alias-fallback round that
/// follows spends only what is left of the same budget instead of restarting
/// the clock, while the refusal text stays honest about what the sources
/// answered.
async fn discover_sparse_drug_rescue_bounded(name: &str) -> Option<SparseDrugDiscoverRescue> {
    let started_at = std::time::Instant::now();
    match tokio::time::timeout(DISCOVER_RESCUE_TIMEOUT, discover_sparse_drug_rescue(name)).await {
        Ok(result) => Some(result),
        Err(_elapsed) => {
            crate::entities::discover::remember_refusal_discover_start(name, started_at);
            None
        }
    }
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
    let cached_resolution = {
        // Scoped: the recovered guard must not live across the
        // resolution await below.
        let cache = crate::utils::sync::recover_poison(trial_alias_cache().lock());
        cache.get(&cache_key).cloned()
    };
    if let Some(mut resolution) = cached_resolution {
        if let Some(requested_alias) = resolution.aliases.first_mut() {
            requested_alias.label = requested_name.to_string();
        }
        return Ok(resolution);
    }

    let lookup = resolve_drug_base(requested_name, false)
        .await
        .map(|resolved| TrialAliasLookup {
            canonical_name: resolved.drug.name,
            candidates: resolved.trial_alias_candidates,
        });
    let (resolution, cacheable) = trial_alias_resolution_from_lookup_result(requested_name, lookup);

    if cacheable {
        let mut cache = crate::utils::sync::recover_poison(trial_alias_cache().lock());
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

        if let Some(candidate) = fallback_name
            && let Ok(fallback_resp) = direct_drug_lookup(&candidate).await
            && !fallback_resp.hits.is_empty()
        {
            lookup_name = candidate;
            resp = fallback_resp;
        } else {
            return Err(original_not_found());
        }
    }

    let mut selected = transform::drug::select_hits_for_name(&resp.hits, &lookup_name);
    if selected.is_empty() {
        // The MyChem full-text query matched only other drugs' records. The
        // query may still be a real brand whose identity openFDA knows, so
        // resolve it through openFDA's identity fields before refusing
        //.
        let identity_fallback = match openfda_label_identity_candidate(&lookup_name).await {
            LabelIdentityAnswer::Candidate(candidate) => named_drug_response(&candidate)
                .await
                .map(|candidate_resp| (candidate, candidate_resp)),
            // A page whose records disagree on the ingredient names no one
            // drug: several drug products carry this name, so refuse it as
            // an ambiguous name instead of falling through to not-found.
            LabelIdentityAnswer::Ambiguous(candidates) => {
                return Err(ambiguous_drug_name_error(&lookup_name, &candidates));
            }
            LabelIdentityAnswer::NoAnswer => None,
        };
        match identity_fallback {
            Some((candidate, candidate_resp)) => {
                lookup_name = candidate;
                resp = candidate_resp;
                selected = transform::drug::select_hits_for_name(&resp.hits, &lookup_name);
            }
            None => {
                // The query may still be a real brand that no current label
                // names: a withdrawn brand such as Tarceva holds no openFDA
                // label and lives in MyChem only on a record with no name,
                // so resolve it through the guarded discover rescue before
                // refusing. A rescue that runs out of time
                // refuses here rather than wait on the slow outside service
                //.
                let rescue = match discover_sparse_drug_rescue_bounded(name).await {
                    Some(SparseDrugDiscoverRescue::Canonical(candidate)) => {
                        named_drug_response(&candidate)
                            .await
                            .map(|candidate_resp| (candidate, candidate_resp))
                    }
                    _ => None,
                };
                match rescue {
                    Some((candidate, candidate_resp)) => {
                        lookup_name = candidate;
                        resp = candidate_resp;
                        selected = transform::drug::select_hits_for_name(&resp.hits, &lookup_name);
                    }
                    None => return Err(name_miss_not_found(name, &resp.hits)),
                }
            }
        }
    }
    let mut drug = transform::drug::merge_mychem_hits(&selected, &lookup_name);

    // A brand whose labels mostly name one ingredient resolves
    // to it. A product request that landed on a combination card — the
    // request names a brand, and the records it merged carry a combination
    // product row — asks the label page what the brand's records name: one
    // plain ingredient most records name resolves the card to that
    // ingredient (Motrin to ibuprofen), and one combination line the
    // records agree on stabilizes the card's own name (Headache Relief is
    // the acetaminophen-aspirin-caffeine line, not whichever product row
    // MyChem lists first). A request that itself names the combination, or
    // a page that names no one drug, keeps the card it already holds
    // (Opdualag, Phesgo, advil pm).
    if transform::drug::card_names_combination(&drug.name)
        && !drug.name.eq_ignore_ascii_case(lookup_name.trim())
        && let LabelIdentityAnswer::Candidate(candidate) =
            openfda_label_identity_candidate(&lookup_name).await
        && let Some(candidate_resp) = named_drug_response(&candidate).await
    {
        lookup_name = candidate;
        resp = candidate_resp;
        selected = transform::drug::select_hits_for_name(&resp.hits, &lookup_name);
        drug = transform::drug::merge_mychem_hits(&selected, &lookup_name);
    }

    let needs_canonical_fallback = !transform::drug::card_names_combination(&drug.name)
        && drug.drugbank_id.is_none()
        && drug.chembl_id.is_none()
        && drug.unii.is_none();
    if needs_canonical_fallback {
        // An ambiguous page keeps the sparse card the query already
        // named; the ambiguous refusal is the name-miss site's answer.
        let identity_fallback = match openfda_label_identity_candidate(&lookup_name).await {
            LabelIdentityAnswer::Candidate(candidate) => named_drug_response(&candidate)
                .await
                .map(|candidate_resp| (candidate, candidate_resp)),
            LabelIdentityAnswer::Ambiguous(_) | LabelIdentityAnswer::NoAnswer => None,
        };
        if let Some((candidate, candidate_resp)) = identity_fallback {
            lookup_name = candidate;
            resp = candidate_resp;
            selected = transform::drug::select_hits_for_name(&resp.hits, &lookup_name);
            drug = transform::drug::merge_mychem_hits(&selected, &lookup_name);
        }
    }

    // A combination card carries no single-ingredient identifier on
    // purpose: it is a complete product card, not a sparse
    // one, so the canonical rescue never runs for it and the discover
    // service cannot refuse a product MyChem itself named.
    if !transform::drug::card_names_combination(&drug.name)
        && drug.drugbank_id.is_none()
        && drug.chembl_id.is_none()
        && drug.unii.is_none()
    {
        // A timed-out rescue keeps the sparse card the query already named
        //: the card never waits on the slow outside service.
        if let Some(rescue) = discover_sparse_drug_rescue_bounded(name).await {
            match rescue {
                SparseDrugDiscoverRescue::Canonical(candidate) => {
                    // Adopt the canonical candidate only when its own MyChem
                    // record names it; a sparse card that matched the query by
                    // name stays put instead of becoming a name-only card
                    // rebuilt from another record's text hits.
                    if let Some(fallback_resp) = named_drug_response(&candidate).await {
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
    }

    // The brand as the resolved record itself spells it: MyChem carries the
    // labeler's own word order for a multi-word brand, and the label lookup
    // follows that spelling to reach records the request's phrase order
    // never matches.
    let record_brand_spellings = transform::drug::brand_spellings_for_request(&selected, name);

    let mut label_response_opt: Option<serde_json::Value> = None;
    let mut label_attempt_failed = false;
    let mut label_elements_oversize = false;
    if fetch_label_response {
        // A failed label fetch degrades the label section to an unavailable
        // outcome instead of failing the card; sparse-metadata misses resolve
        // through the guarded identity-field fallback in `lookup_label_response`.
        // The label must match the brand or plain ingredient the user asked
        // for, so the request's own name travels with the card's canonical
        // name and the record's own spelling of the requested brand
        //.
        match OpenFdaClient::new() {
            Ok(client) => {
                match lookup_label_response(&client, name, &drug.name, &record_brand_spellings)
                    .await
                {
                    Ok(LabelLookup::Response(response)) => label_response_opt = Some(response),
                    Ok(LabelLookup::NoSplRecord) => {}
                    Ok(LabelLookup::ElementsSearchTooLarge) => label_elements_oversize = true,
                    Err(_) => label_attempt_failed = true,
                }
            }
            Err(_) => label_attempt_failed = true,
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
        label_elements_oversize,
        trial_alias_candidates,
        selected_hits: selected.into_iter().cloned().collect(),
    })
}

async fn populate_common_sections(
    requested_name: &str,
    drug: &mut Drug,
    label_response: Option<&serde_json::Value>,
    label_attempt_failed: bool,
    label_elements_oversize: bool,
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

    if section_flags.include_label {
        let outcome = if label_attempt_failed {
            SectionOutcome::unavailable(LABEL_UNAVAILABLE_MESSAGE)
        } else if drug.label.is_some() {
            SectionOutcome::data("OpenFDA label")
        } else if label_response.is_some() {
            SectionOutcome::empty_with_reason("OpenFDA label", NO_LABEL_TEXT_NOTE)
        } else if label_elements_oversize {
            SectionOutcome::unavailable(ELEMENTS_SEARCH_OVERSIZE_NOTE)
        } else {
            SectionOutcome::empty_with_reason("OpenFDA label", NO_SPL_RECORD_NOTE)
        };
        drug.section_outcomes.complete("label", outcome);
    }

    if section_flags.include_interactions {
        super::interactions::populate_card_interactions(
            requested_name,
            drug,
            label_response,
            label_attempt_failed,
            section_flags.allow_partial_interactions,
        )
        .await?;
    } else {
        drug.interactions.clear();
        drug.interaction_text = None;
        drug.interaction_pagination = None;
        drug.interaction_bundle_freshness = None;
        drug.interaction_coverage_status = None;
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

    if section_flags.include_cell_lines {
        let section = super::cell_lines::load_cell_lines_section(&drug.name, requested_name).await;
        super::cell_lines::attach_cell_lines_section(drug, section);
    } else {
        drug.cell_lines = None;
    }
    Ok(())
}

async fn populate_us_regional_sections(
    requested_name: &str,
    drug: &mut Drug,
    selected_hits: &[MyChemHit],
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

    drug.fda_orphan_designations = if section_flags.include_regulatory {
        Some(Box::new(
            crate::sources::fda_orphan::fetch(orphan_aliases(requested_name, drug, selected_hits))
                .await?,
        ))
    } else {
        None
    };

    drug.us_safety_warnings = if section_flags.include_safety {
        label_response.and_then(extract_label_warnings_text)
    } else {
        None
    };
    drug.us_boxed_warning = if section_flags.include_safety {
        label_response.and_then(extract_label_boxed_warning)
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
        drug.who_note = None;
        return Ok(());
    }

    let ready = WhoPqClient::ready(WhoPqSyncMode::Auto).await;
    apply_who_sections_result(drug, requested_name, ready)
}

/// Apply the WHO bundle readiness to the card's WHO section. A readable
/// bundle fills the section; a bundle that cannot be read degrades the
/// section with a path-free note instead of failing the whole card, the
/// way the all-region search degrades:
/// a JSON or MCP caller still gets the rest of the card, the note names
/// the WHO gap, and the local data path stays terminal-only.
fn apply_who_sections_result(
    drug: &mut Drug,
    requested_name: &str,
    ready: Result<WhoPqClient, BioMcpError>,
) -> Result<(), BioMcpError> {
    let client = match ready {
        Ok(client) => client,
        Err(err) => {
            warn!(
                name = %drug.name,
                "WHO Prequalification data unavailable for drug card: {err}"
            );
            drug.who_prequalification = None;
            drug.who_note = Some(crate::entities::drug::who_pq_degradation_note(&err, "card"));
            return Ok(());
        }
    };
    let identity = build_who_identity(requested_name, drug);
    drug.who_note = None;
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

    let mut resolved = resolve_drug_base(&name, fetch_label_response).await?;
    populate_common_sections(
        &name,
        &mut resolved.drug,
        resolved.label_response.as_ref(),
        resolved.label_attempt_failed,
        resolved.label_elements_oversize,
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
            &name,
            &mut resolved.drug,
            &resolved.selected_hits,
            resolved.label_response.as_ref(),
            &section_flags,
        )
        .await?;
    } else {
        resolved.drug.shortage = None;
        resolved.drug.approvals = None;
        resolved.drug.fda_orphan_designations = None;
        resolved.drug.us_safety_warnings = None;
        resolved.drug.us_boxed_warning = None;
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
        resolved.drug.who_note = None;
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
                    || resolved
                        .drug
                        .us_boxed_warning
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
