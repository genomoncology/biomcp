//! Search-all follow-up link and command generation.

use serde_json::Value;

use crate::render::markdown::shell_quote_arg as quote_arg;

use super::format::{first_gettable_variant_id, is_civic_variant_id};
use super::plan::PreparedInput;
use super::{SearchAllLink, SectionKind};

const EXPAND_LIMIT: usize = 20;

pub(super) fn build_links(
    kind: SectionKind,
    input: &PreparedInput,
    results: &[Value],
    _search_self: &str,
) -> Vec<SearchAllLink> {
    let mut links = Vec::new();

    // get.top: inspect the top result in detail (teaches the `get` verb)
    if let Some(cmd) = top_get_command(kind, input, results) {
        links.push(SearchAllLink {
            rel: "get.top".to_string(),
            title: format!("Inspect top {}", kind.entity()),
            command: cmd,
        });
    }

    // Cross-entity links: use `search` commands (not thin wrappers like
    // `disease trials`) so users can append filters like -s, -p, etc.
    match kind {
        SectionKind::Gene => {
            if let Some(gene) = first_string(results, &["symbol"]).or(input.gene_anchor()) {
                links.push(SearchAllLink {
                    rel: "cross.trials".to_string(),
                    title: "Gene-linked trials".to_string(),
                    command: format!("biomcp search trial --biomarker {}", quote_arg(gene)),
                });
            }
        }
        SectionKind::Disease => {
            // Prefer name over ID because ClinicalTrials.gov
            // doesn't understand MONDO IDs.
            if let Some(disease) = first_string(results, &["name"]).or(input.disease.as_deref()) {
                links.push(SearchAllLink {
                    rel: "cross.trials".to_string(),
                    title: "Disease-linked trials".to_string(),
                    command: format!("biomcp search trial -c {}", quote_arg(disease)),
                });
            }
        }
        SectionKind::Drug => {
            // Prefer user's input drug name for AE cross-links: FAERS indexes by
            // generic/brand name, not salt forms. DrugBank canonical names (e.g.
            // "dabrafenib mesylate") return far fewer FAERS reports than the generic
            // name ("dabrafenib": 4K+ vs 15 reports).
            if let Some(drug) = input
                .drug
                .as_deref()
                .or_else(|| first_string(results, &["name"]))
            {
                links.push(SearchAllLink {
                    rel: "cross.adverse-events".to_string(),
                    title: "Top adverse events".to_string(),
                    command: format!("biomcp search adverse-event -d {}", quote_arg(drug)),
                });
            }
        }
        _ => {}
    }

    // Filter hints: show useful unused filters for this entity
    links.extend(filter_hints(kind, input));

    links
}

/// Return contextual filter hints — filters the user *could* add to narrow or
/// pivot the search. Each hint teaches a different capability of the entity
/// search rather than repeating the query the user already ran.
fn filter_hints(kind: SectionKind, input: &PreparedInput) -> Vec<SearchAllLink> {
    let mut hints = Vec::new();

    match kind {
        SectionKind::Variant => {
            let variant_base = variant_base_args(input);
            let significance_command = if variant_base.is_empty() {
                "biomcp search variant --significance pathogenic".to_string()
            } else {
                format!("biomcp search variant {variant_base} --significance pathogenic")
            };
            if input.disease.is_none() {
                hints.push(SearchAllLink {
                    rel: "filter.hint".to_string(),
                    title: "Filter by significance".to_string(),
                    command: significance_command,
                });
            }
            // Population frequency is always useful context
            let rarity_command = if variant_base.is_empty() {
                "biomcp search variant --max-frequency 0.01".to_string()
            } else {
                format!("biomcp search variant {variant_base} --max-frequency 0.01")
            };
            hints.push(SearchAllLink {
                rel: "filter.hint".to_string(),
                title: "Rare variants only".to_string(),
                command: rarity_command,
            });
        }
        SectionKind::Trial => {
            if input.since.is_none() {
                hints.push(SearchAllLink {
                    rel: "filter.hint".to_string(),
                    title: "Recruiting only".to_string(),
                    command: format!(
                        "{} -s recruiting",
                        canonical_search_command(kind, input, input.limit)
                    ),
                });
            }
            hints.push(SearchAllLink {
                rel: "filter.hint".to_string(),
                title: "Phase 3 trials".to_string(),
                command: format!(
                    "{} -p phase3",
                    canonical_search_command(kind, input, input.limit)
                ),
            });
        }
        SectionKind::Article => {
            hints.push(SearchAllLink {
                rel: "filter.hint".to_string(),
                title: "Clinical trials only".to_string(),
                command: format!(
                    "{} --type research-article",
                    canonical_search_command(kind, input, input.limit)
                ),
            });
            hints.push(SearchAllLink {
                rel: "filter.hint".to_string(),
                title: "Reviews & meta-analyses".to_string(),
                command: format!(
                    "{} --type review",
                    canonical_search_command(kind, input, input.limit)
                ),
            });
        }
        SectionKind::Drug => {}
        SectionKind::AdverseEvent => {
            if let Some(drug) = input.drug.as_deref() {
                hints.push(SearchAllLink {
                    rel: "filter.hint".to_string(),
                    title: "Top reactions by frequency".to_string(),
                    command: adverse_event_count_command(input),
                });
                hints.push(SearchAllLink {
                    rel: "filter.hint".to_string(),
                    title: "Serious reports only".to_string(),
                    command: format!(
                        "biomcp search adverse-event --drug {} --serious",
                        quote_arg(drug)
                    ),
                });
            }
        }
        SectionKind::Gene
        | SectionKind::Disease
        | SectionKind::Pathway
        | SectionKind::Pgx
        | SectionKind::Gwas => {}
    }

    hints
}

/// Build the base args for a variant search command from the current input.
fn variant_base_args(input: &PreparedInput) -> String {
    let mut args = Vec::new();
    if let Some(gene) = input.gene_anchor() {
        args.push(format!("--gene {}", quote_arg(gene)));
    }
    if let Some(condition) = input.disease.as_deref() {
        args.push(format!("--condition {}", quote_arg(condition)));
    }
    if let Some(therapy) = input.drug.as_deref() {
        args.push(format!("--therapy {}", quote_arg(therapy)));
    }
    args.join(" ")
}

pub(super) fn top_get_command(
    kind: SectionKind,
    input: &PreparedInput,
    results: &[Value],
) -> Option<String> {
    match kind {
        SectionKind::Gene => first_string(results, &["symbol"])
            .or(input.gene_anchor())
            .map(|id| format!("biomcp get gene {}", quote_arg(id))),
        SectionKind::Variant => first_gettable_variant_id(results)
            .or_else(|| {
                input
                    .variant
                    .as_deref()
                    .filter(|id| !is_civic_variant_id(id))
                    .map(str::to_string)
            })
            .map(|id| format!("biomcp get variant {}", quote_arg(&id))),
        SectionKind::Disease => first_string(results, &["id", "name"])
            .or(input.disease.as_deref())
            .map(|id| format!("biomcp get disease {}", quote_arg(id))),
        SectionKind::Drug => input
            .drug
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .map(str::to_string)
            .or_else(|| {
                crate::render::markdown::preferred_drug_name(
                    results
                        .iter()
                        .filter_map(|row| row.as_object())
                        .filter_map(|obj| obj.get("name"))
                        .filter_map(Value::as_str),
                    input.drug.as_deref(),
                )
            })
            .or_else(|| first_string(results, &["name"]).map(str::to_string))
            .map(|id| format!("biomcp get drug {}", quote_arg(&id))),
        SectionKind::Pathway => {
            first_string(results, &["id"]).map(|id| format!("biomcp get pathway {}", quote_arg(id)))
        }
        SectionKind::Trial
        | SectionKind::Article
        | SectionKind::Pgx
        | SectionKind::Gwas
        | SectionKind::AdverseEvent => None,
    }
}

pub(super) fn canonical_search_command(
    kind: SectionKind,
    input: &PreparedInput,
    limit: usize,
) -> String {
    let mut args: Vec<String> = vec!["biomcp".into(), "search".into(), kind.entity().into()];

    match kind {
        SectionKind::Gene => {
            push_opt(&mut args, "--query", input.gene_anchor());
        }
        SectionKind::Variant => {
            let gene = input.gene_anchor();
            let hgvsp = input
                .variant_context
                .as_ref()
                .and_then(|ctx| ctx.parsed_change.as_deref());
            let condition = input.disease.as_deref();
            let therapy = input.drug.as_deref();

            if gene.is_none()
                && hgvsp.is_none()
                && condition.is_none()
                && therapy.is_none()
                && let Some(raw_variant) = input.variant.as_deref()
            {
                // Keep the anchor in search.self even when we cannot parse gene/change.
                args.push(quote_arg(raw_variant));
            }

            push_opt(&mut args, "--gene", gene);
            push_opt(&mut args, "--hgvsp", hgvsp);
            push_opt(&mut args, "--condition", condition);
            push_opt(&mut args, "--therapy", therapy);
        }
        SectionKind::Disease => {
            push_opt(&mut args, "--query", input.disease.as_deref());
        }
        SectionKind::Drug => {
            push_opt(&mut args, "--query", input.drug_query());
            push_opt(&mut args, "--target", input.gene_anchor());
            push_opt(&mut args, "--indication", input.disease.as_deref());
        }
        SectionKind::Trial => {
            push_opt(&mut args, "--condition", input.trial_condition_query());
            push_opt(&mut args, "--intervention", input.drug.as_deref());
            push_opt(&mut args, "--biomarker", input.gene_anchor());
            push_opt_owned(&mut args, "--mutation", input.variant_trial_query());
            push_opt(&mut args, "--since", input.since.as_deref());
        }
        SectionKind::Article => {
            push_opt(&mut args, "--gene", input.gene_anchor());
            push_opt(&mut args, "--disease", input.article_disease_filter());
            push_opt(&mut args, "--drug", input.drug.as_deref());
            push_opt(&mut args, "--keyword", input.article_keyword_filter());
            push_opt(&mut args, "--since", input.since.as_deref());
        }
        SectionKind::Pathway => {
            push_opt(&mut args, "--query", input.gene_anchor());
        }
        SectionKind::Pgx => {
            push_opt(&mut args, "--gene", input.gene_anchor());
            push_opt(&mut args, "--drug", input.drug.as_deref());
        }
        SectionKind::Gwas => {
            push_opt(&mut args, "--gene", input.gene_anchor());
            push_opt(&mut args, "--trait", input.disease.as_deref());
        }
        SectionKind::AdverseEvent => {
            push_opt(&mut args, "--drug", input.drug.as_deref());
            push_opt(&mut args, "--since", input.since.as_deref());
        }
    }

    // Clamp to entity-specific maximums so generated commands are always runnable.
    let clamped = match kind {
        SectionKind::Pathway => limit.min(25),
        _ => limit,
    };
    args.push("--limit".into());
    args.push(clamped.to_string());
    args.join(" ")
}

fn adverse_event_count_command(input: &PreparedInput) -> String {
    let mut args = vec![
        "biomcp".to_string(),
        "search".to_string(),
        "adverse-event".to_string(),
    ];
    push_opt(&mut args, "--drug", input.drug.as_deref());
    push_opt(&mut args, "--since", input.since.as_deref());
    args.push("--count".to_string());
    args.push("patient.reaction.reactionmeddrapt".to_string());
    args.push("--limit".to_string());
    args.push(EXPAND_LIMIT.to_string());
    args.join(" ")
}

fn first_string<'a>(results: &'a [Value], fields: &[&str]) -> Option<&'a str> {
    let row = results.first()?.as_object()?;
    for field in fields {
        if let Some(value) = row
            .get(*field)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Some(value);
        }
    }
    None
}

fn push_opt(args: &mut Vec<String>, flag: &str, value: Option<&str>) {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return;
    };
    args.push(flag.to_string());
    args.push(quote_arg(value));
}

fn push_opt_owned(args: &mut Vec<String>, flag: &str, value: Option<String>) {
    let Some(value) = value else {
        return;
    };
    push_opt(args, flag, Some(value.as_str()));
}
