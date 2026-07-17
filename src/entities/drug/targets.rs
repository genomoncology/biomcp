//! Drug target-family, CIViC variant-target, and indication enrichment helpers.

use std::collections::{HashMap, HashSet};

use crate::entities::section_outcome::SectionOutcome;
use crate::sources::chembl::{ChemblClient, ChemblTarget};
use crate::sources::civic::CivicContext;
use crate::sources::opentargets::{OpenTargetsClient, OpenTargetsTarget};

use super::Drug;
use super::metadata::merge_unique_casefold;

pub(super) async fn enrich_targets(
    drug: &mut Drug,
    civic_context: Option<&CivicContext>,
) -> SectionOutcome {
    let mut chembl_rows = Vec::new();
    let mut opentargets_targets = Vec::new();
    let mut successful_sources = Vec::new();
    let mut contributors = Vec::new();
    let mut failed = false;
    if let Some(chembl_id) = drug.chembl_id.as_deref() {
        match ChemblClient::new() {
            Ok(client) => match client.drug_targets(chembl_id, 15).await {
                Ok(rows) => {
                    successful_sources.push("ChEMBL");
                    if rows
                        .iter()
                        .any(|row| !row.target.eq_ignore_ascii_case("Unknown target"))
                    {
                        contributors.push("ChEMBL");
                    }
                    let targets = rows
                        .iter()
                        .filter(|row| !row.target.eq_ignore_ascii_case("Unknown target"))
                        .map(|row| row.target.clone())
                        .collect::<Vec<_>>();
                    merge_unique_casefold(&mut drug.targets, targets);

                    let mechanisms = rows
                        .iter()
                        .filter(|row| !row.target.eq_ignore_ascii_case("Unknown target"))
                        .map(|row| {
                            row.mechanism
                                .clone()
                                .unwrap_or_else(|| format!("{} of {}", row.action, row.target))
                        })
                        .collect::<Vec<_>>();
                    merge_unique_casefold(&mut drug.mechanisms, mechanisms);
                    chembl_rows = rows;
                }
                Err(_) => failed = true,
            },
            Err(_) => failed = true,
        }

        match OpenTargetsClient::new() {
            Ok(client) => match client.drug_sections(chembl_id, 15).await {
                Ok(sections) => {
                    successful_sources.push("Open Targets");
                    if !sections.targets.is_empty() {
                        contributors.push("Open Targets");
                    }
                    let targets = sections
                        .targets
                        .iter()
                        .map(|t| t.approved_symbol.clone())
                        .collect::<Vec<_>>();
                    merge_unique_casefold(&mut drug.targets, targets);
                    opentargets_targets = sections.targets;
                }
                Err(_) => failed = true,
            },
            Err(_) => failed = true,
        }
    } else {
        successful_sources.extend(["ChEMBL", "Open Targets"]);
    }

    drug.targets.truncate(12);
    drug.variant_targets = civic_context
        .map(|context| extract_variant_targets_from_civic(context, &drug.targets))
        .unwrap_or_default();
    drug.variant_targets.truncate(12);
    drug.target_family = None;
    drug.target_family_name = None;
    let inferred_target_family = strict_target_family_label(&drug.targets);
    let inferred_target_family_name = inferred_target_family
        .as_ref()
        .and_then(|_| derive_target_family_name(&drug.targets, &opentargets_targets));

    if drug.targets.len() >= 2
        && let Some(target_chembl_id) = family_target_chembl_id(&chembl_rows, &drug.targets)
    {
        match ChemblClient::new() {
            Ok(client) => match client.target_summary(&target_chembl_id).await {
                Ok(summary) if summary.target_type.eq_ignore_ascii_case("PROTEIN FAMILY") => {
                    let _family_pref_name = summary.pref_name.trim();
                    drug.target_family = inferred_target_family.clone();
                    drug.target_family_name = inferred_target_family_name.clone();
                }
                Ok(_) => {}
                Err(_) => {
                    failed = true;
                    drug.target_family = inferred_target_family.clone();
                    drug.target_family_name = inferred_target_family_name.clone();
                }
            },
            Err(_) => {
                failed = true;
                drug.target_family = inferred_target_family.clone();
                drug.target_family_name = inferred_target_family_name.clone();
            }
        }
    }

    if !drug.mechanisms.is_empty() {
        drug.mechanism = drug.mechanisms.first().cloned();
    }
    drug.mechanisms.truncate(6);

    contributors.sort_unstable();
    contributors.dedup();
    successful_sources.sort_unstable();
    successful_sources.dedup();
    if failed {
        if contributors.is_empty() {
            SectionOutcome::unavailable("Drug target evidence is temporarily unavailable.")
        } else {
            SectionOutcome::degraded(
                contributors,
                "Drug target evidence is incomplete because a source was unavailable.",
            )
        }
    } else if contributors.is_empty() {
        SectionOutcome::empty_sources(successful_sources)
    } else {
        SectionOutcome::data_sources(contributors)
    }
}

fn normalize_variant_target_label(profile_name: &str, gene_symbol: &str) -> Option<String> {
    let profile_name = profile_name.trim();
    let gene_symbol = gene_symbol.trim();
    if profile_name.is_empty() || gene_symbol.is_empty() {
        return None;
    }
    let profile_lower = profile_name.to_ascii_lowercase();
    let gene_lower = gene_symbol.to_ascii_lowercase();
    if profile_lower == gene_lower || !profile_lower.starts_with(&gene_lower) {
        return None;
    }
    let remainder = profile_name.get(gene_symbol.len()..)?.trim();
    if remainder.is_empty() {
        return None;
    }
    let remainder_flat = remainder.replace(' ', "");
    if gene_symbol.eq_ignore_ascii_case("EGFR") && remainder_flat.eq_ignore_ascii_case("VIII") {
        return Some("EGFRvIII".to_string());
    }
    Some(profile_name.to_string())
}

fn extract_variant_targets_from_civic(
    civic: &CivicContext,
    generic_targets: &[String],
) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for profile in civic
        .evidence_items
        .iter()
        .map(|row| row.molecular_profile.as_str())
        .chain(
            civic
                .assertions
                .iter()
                .map(|row| row.molecular_profile.as_str()),
        )
    {
        for target in generic_targets {
            let Some(normalized) = normalize_variant_target_label(profile, target) else {
                continue;
            };
            let key = normalized.to_ascii_lowercase();
            if seen.insert(key) {
                out.push(normalized);
            }
            break;
        }
    }
    out
}

fn family_target_chembl_id(
    chembl_rows: &[ChemblTarget],
    displayed_targets: &[String],
) -> Option<String> {
    let displayed = displayed_targets
        .iter()
        .map(|target| target.to_ascii_lowercase())
        .collect::<HashSet<_>>();
    let target_ids = chembl_rows
        .iter()
        .filter(|row| {
            displayed.contains(&row.target.to_ascii_lowercase())
                || row
                    .mechanism
                    .as_deref()
                    .map(str::trim)
                    .is_some_and(|value| !value.is_empty())
        })
        .filter_map(|row| row.target_chembl_id.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<HashSet<_>>();
    if target_ids.len() == 1 {
        target_ids.into_iter().next()
    } else {
        None
    }
}

fn strict_target_family_label(targets: &[String]) -> Option<String> {
    let distinct = targets
        .iter()
        .map(|target| target.to_ascii_lowercase())
        .collect::<HashSet<_>>();
    if distinct.len() < 2 {
        return None;
    }

    let prefix_len = common_prefix_len_casefold(targets)?;
    if prefix_len < 2 {
        return None;
    }

    let prefix = &targets[0][..prefix_len];
    let all_numeric_suffixes = targets.iter().all(|target| {
        let Some(suffix) = target.get(prefix_len..) else {
            return false;
        };
        !suffix.is_empty() && suffix.chars().all(|ch| ch.is_ascii_digit())
    });
    if all_numeric_suffixes {
        Some(prefix.to_string())
    } else {
        None
    }
}

fn derive_target_family_name(
    displayed_targets: &[String],
    opentargets_targets: &[OpenTargetsTarget],
) -> Option<String> {
    let names_by_symbol = opentargets_targets
        .iter()
        .filter_map(|target| {
            let approved_name = target.approved_name.as_deref()?.trim();
            if approved_name.is_empty() {
                return None;
            }
            Some((
                target.approved_symbol.to_ascii_lowercase(),
                approved_name.to_string(),
            ))
        })
        .collect::<HashMap<_, _>>();
    let names = displayed_targets
        .iter()
        .map(|target| names_by_symbol.get(&target.to_ascii_lowercase()).cloned())
        .collect::<Option<Vec<_>>>()?;
    common_name_stem(&names)
}

fn common_name_stem(names: &[String]) -> Option<String> {
    if names.len() < 2 {
        return None;
    }

    let prefix_len = common_prefix_len_casefold(names)?;
    if prefix_len < 6 {
        return None;
    }

    let prefix = &names[0][..prefix_len];
    let boundary_aligned = prefix
        .chars()
        .last()
        .is_some_and(|ch| !ch.is_alphanumeric());
    let mut candidate = prefix.trim_end();
    if !boundary_aligned {
        candidate = trim_to_word_boundary(candidate);
    }
    candidate = candidate.trim_end_matches(|ch: char| ch.is_whitespace() || ",;:-/".contains(ch));
    if candidate.len() < 6 || !candidate.chars().any(|ch| ch.is_alphabetic()) {
        None
    } else {
        Some(candidate.to_string())
    }
}

fn trim_to_word_boundary(value: &str) -> &str {
    let mut end = value.len();
    while end > 0 {
        let Some(ch) = value[..end].chars().last() else {
            break;
        };
        if !ch.is_alphanumeric() && ch != ')' && ch != ']' {
            break;
        }
        end -= ch.len_utf8();
    }
    value[..end].trim_end()
}

fn common_prefix_len_casefold(values: &[String]) -> Option<usize> {
    let first = values.first()?;
    let mut prefix_len = first.len();
    for value in &values[1..] {
        let common_len = first
            .char_indices()
            .zip(value.chars())
            .take_while(|((_, left), right)| left.eq_ignore_ascii_case(right))
            .map(|((idx, left), _)| idx + left.len_utf8())
            .last()
            .unwrap_or(0);
        prefix_len = prefix_len.min(common_len);
    }
    Some(prefix_len)
}

pub(super) async fn enrich_indications(drug: &mut Drug) -> SectionOutcome {
    let Some(chembl_id) = drug.chembl_id.as_deref() else {
        return SectionOutcome::empty("Open Targets");
    };

    let outcome = match OpenTargetsClient::new() {
        Ok(client) => match client.drug_sections(chembl_id, 15).await {
            Ok(sections) => {
                let indications = sections
                    .indications
                    .into_iter()
                    .map(|i| {
                        match i
                            .max_clinical_stage
                            .as_deref()
                            .and_then(format_opentargets_clinical_stage)
                        {
                            Some(stage) => format!("{} ({stage})", i.disease_name),
                            None => i.disease_name,
                        }
                    })
                    .collect::<Vec<_>>();
                let outcome = if indications.is_empty() {
                    SectionOutcome::empty("Open Targets")
                } else {
                    SectionOutcome::data("Open Targets")
                };
                merge_unique_casefold(&mut drug.indications, indications);
                outcome
            }
            Err(_) => {
                SectionOutcome::unavailable("Drug indication evidence is temporarily unavailable.")
            }
        },
        Err(_) => {
            SectionOutcome::unavailable("Drug indication evidence is temporarily unavailable.")
        }
    };

    drug.indications.truncate(12);
    outcome
}

fn format_opentargets_clinical_stage(stage: &str) -> Option<String> {
    let normalized = stage.trim();
    if normalized.is_empty() {
        return None;
    }

    let normalized = normalized.to_ascii_uppercase();
    let label = match normalized.as_str() {
        "UNKNOWN" => return None,
        "APPROVAL" => "Approved".to_string(),
        "EARLY_PHASE_1" => "Early Phase 1".to_string(),
        "PHASE_1" => "Phase 1".to_string(),
        "PHASE_2" => "Phase 2".to_string(),
        "PHASE_3" => "Phase 3".to_string(),
        "PHASE_4" => "Phase 4".to_string(),
        "PHASE_1_2" => "Phase 1/2".to_string(),
        "PHASE_2_3" => "Phase 2/3".to_string(),
        other => other
            .replace('_', " ")
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                let Some(first) = chars.next() else {
                    return String::new();
                };
                let mut out = String::new();
                out.extend(first.to_uppercase());
                out.push_str(&chars.as_str().to_ascii_lowercase());
                out
            })
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
    };

    (!label.is_empty()).then_some(label)
}

#[cfg(test)]
mod tests;
