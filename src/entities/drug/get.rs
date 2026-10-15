//! Drug retrieval workflows, section parsing, and region validation.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use regex::Regex;

use crate::entities::section_outcome::SectionOutcome;
use crate::error::BioMcpError;
use crate::sources::civic::{CivicClient, CivicContext};
use crate::sources::ema::{EmaClient, EmaSyncMode};
use crate::sources::mychem::{MyChemHit, MyChemQueryResponse};
use crate::sources::openfda::OpenFdaClient;
use crate::sources::who_pq::{WhoPqClient, WhoPqSyncMode, WhoProductTypeFilter};
use crate::transform;
use tracing::warn;

use super::label::{
    ELEMENTS_SEARCH_OVERSIZE_NOTE, LABEL_UNAVAILABLE_MESSAGE, LabelLookup, NO_LABEL_TEXT_NOTE,
    NO_SPL_RECORD_NOTE, extract_inline_label, extract_label_boxed_warning, extract_label_set_id,
    extract_label_warnings_text, lookup_label_response,
};
use super::metadata::{
    apply_openfda_metadata, apply_top_adverse_event_preview, fetch_shortage_entries,
    fetch_top_adverse_event_preview, map_drugsfda_approvals, orphan_aliases,
};
use super::targets::{TargetSources, enrich_indications, enrich_targets, fetch_target_sources};
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
async fn fetch_civic_therapy_context(name: &str) -> CivicFetch {
    let name = name.trim();
    if name.is_empty() {
        return CivicFetch {
            context: Some(CivicContext::default()),
            outcome: SectionOutcome::empty("CIViC"),
        };
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
            CivicFetch {
                context: Some(context),
                outcome,
            }
        }
        Ok(Err(_)) | Err(_) => CivicFetch {
            context: None,
            outcome: SectionOutcome::unavailable("CIViC drug evidence is temporarily unavailable."),
        },
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

mod resolver;
pub(super) use resolver::*;

mod trial_alias;
#[cfg(test)]
use trial_alias::*;
pub(crate) use trial_alias::{
    TrialAlias, TrialAliasSource, resolve_trial_aliases, resolve_trial_aliases_with_sources,
    resolve_trial_canonical_name,
};
use trial_alias::{optional_lookup, trial_alias_candidates_from_hits};

/// The CIViC therapy fetch's answer: the context the target merge consumes
/// and the outcome the civic section completes with. A fetch that never ran
/// carries no context and the unavailable outcome, the pair the sequential
/// card produced for a skipped fetch.
struct CivicFetch {
    context: Option<CivicContext>,
    outcome: SectionOutcome,
}

/// The label fetch's answer once the drug is named: the chosen response,
/// or the failure flags the label section's outcome needs. A request that
/// fetches no label carries none of them.
struct LabelFetchOutcome {
    response: Option<serde_json::Value>,
    attempt_failed: bool,
    elements_oversize: bool,
}

/// The card's label lookup once the drug is named: the chosen response, or
/// the failure flags the label section's outcome needs. A failed label
/// fetch degrades the label section to an unavailable outcome instead of
/// failing the card; sparse-metadata misses resolve through the guarded
/// identity-field fallback in `lookup_label_response`. The label must
/// match the brand or plain ingredient the user asked for, so the
/// request's own name travels with the card's canonical name and the
/// record's own spelling of the requested brand.
async fn fetch_label_lookup(
    requested_name: &str,
    drug: &Drug,
    record_brand_spellings: &[String],
) -> LabelFetchOutcome {
    let client = match OpenFdaClient::new() {
        Ok(client) => client,
        Err(_) => {
            return LabelFetchOutcome {
                response: None,
                attempt_failed: true,
                elements_oversize: false,
            };
        }
    };
    match lookup_label_response(&client, requested_name, &drug.name, record_brand_spellings).await {
        Ok(LabelLookup::Response(response)) => LabelFetchOutcome {
            response: Some(response),
            attempt_failed: false,
            elements_oversize: false,
        },
        Ok(LabelLookup::NoSplRecord) => LabelFetchOutcome::without_response(),
        Ok(LabelLookup::ElementsSearchTooLarge) => LabelFetchOutcome {
            response: None,
            attempt_failed: false,
            elements_oversize: true,
        },
        Err(_) => LabelFetchOutcome {
            response: None,
            attempt_failed: true,
            elements_oversize: false,
        },
    }
}

/// Resolve a drug name to its base card through the resolver module's
/// custody flow; the card's four parallel source fetches (the openFDA
/// label, the CIViC context, the target sources and the adverse-event
/// preview) start from the named card. The 0.9 label-identity voting and
/// the ambiguous-shared-name refusal are ticket 2097's port into the
/// resolver, not this merge.
async fn resolve_drug_identity(name: &str) -> Result<ResolvedDrugBase, BioMcpError> {
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
    resolve_drug_base(name, false, false).await
}

async fn populate_common_sections(
    requested_name: &str,
    drug: &mut Drug,
    label: &LabelFetchOutcome,
    section_flags: &DrugSections,
    raw_label: bool,
    civic: CivicFetch,
    target_sources: Option<TargetSources>,
) -> Result<(), BioMcpError> {
    let label_response = label.response.as_ref();
    drug.label = if section_flags.include_label {
        label_response.and_then(|response| extract_inline_label(response, raw_label))
    } else {
        None
    };

    if section_flags.include_label {
        let outcome = if label.attempt_failed {
            SectionOutcome::unavailable(LABEL_UNAVAILABLE_MESSAGE)
        } else if drug.label.is_some() {
            SectionOutcome::data("OpenFDA label")
        } else if label_response.is_some() {
            SectionOutcome::empty_with_reason("OpenFDA label", NO_LABEL_TEXT_NOTE)
        } else if label.elements_oversize {
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
            label.attempt_failed,
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

    // The target sources were fetched under the same section flag when the
    // card was named, so a present fetch is the section running and an
    // absent one is the section skipped; the merge still applies the CIViC
    // context after both answers are in.
    match target_sources {
        Some(sources) => {
            let outcome = enrich_targets(drug, civic.context.as_ref(), sources).await;
            drug.section_outcomes.complete("targets", outcome);
        }
        None => drug.variant_targets.clear(),
    }

    if section_flags.include_indications {
        let outcome = enrich_indications(drug).await;
        drug.section_outcomes.complete("indications", outcome);
    }

    if section_flags.include_civic {
        let CivicFetch { context, outcome } = civic;
        drug.civic = Some(context.unwrap_or_default());
        drug.section_outcomes.complete("civic", outcome);
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

    let mut resolved = resolve_drug_identity(&name).await?;

    // Once the drug is named, the card's four independent source fetches —
    // the openFDA label, the CIViC context, the target sources and the
    // adverse-event preview — start together, so the card waits for the
    // slowest of them instead of their sum. None of the four needs
    // another's answer: CIViC and the preview take the card's name, the
    // target lookup takes its ChEMBL id, and the label's metadata
    // application changes neither. Each fetch keeps its own guard, and a
    // fetch whose guard is off is never started.
    let label_fut = async {
        if !fetch_label_response {
            return LabelFetchOutcome::without_response();
        }
        fetch_label_lookup(&name, &resolved.drug, &resolved.record_brand_spellings).await
    };
    let civic_fut = async {
        if !(section_flags.include_targets || section_flags.include_civic) {
            return CivicFetch {
                context: None,
                outcome: SectionOutcome::unavailable(
                    "CIViC drug evidence is temporarily unavailable.",
                ),
            };
        }
        fetch_civic_therapy_context(&resolved.drug.name).await
    };
    let targets_fut = async {
        if !section_flags.include_targets {
            return None;
        }
        Some(fetch_target_sources(resolved.drug.chembl_id.as_deref()).await)
    };
    let adverse_event_fut = async {
        if !(region.includes_us() && (!section_only || section_flags.include_safety)) {
            return None;
        }
        Some(fetch_top_adverse_event_preview(&resolved.drug.name).await)
    };
    let (label_lookup, civic, target_sources, adverse_event_fetch) =
        tokio::join!(label_fut, civic_fut, targets_fut, adverse_event_fut);

    if let Some(label_response) = label_lookup.response.as_ref() {
        apply_openfda_metadata(&mut resolved.drug, label_response);
        resolved.drug.label_set_id = extract_label_set_id(label_response);
    }

    populate_common_sections(
        &name,
        &mut resolved.drug,
        &label_lookup,
        &section_flags,
        raw_label,
        civic,
        target_sources,
    )
    .await?;

    let faers_failed = match adverse_event_fetch {
        Some(result) => Some(apply_top_adverse_event_preview(&mut resolved.drug, result)),
        None => {
            resolved.drug.top_adverse_events.clear();
            resolved.drug.faers_query = None;
            None
        }
    };

    if region.includes_us() {
        populate_us_regional_sections(
            &name,
            &mut resolved.drug,
            &resolved.selected_hits,
            label_lookup.response.as_ref(),
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
            if label_lookup.attempt_failed {
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

#[cfg(test)]
pub(super) mod consumer_tests;
