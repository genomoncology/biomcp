//! Rank captured variant evidence without changing identity confirmation.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use regex::Regex;

use crate::entities::variant::{NormalizedVariantAliases, RequestedVariantIdentity};
use crate::transform::article::normalize_article_search_text;

use super::super::candidates::{ArticleCandidate, stable_article_identifier};
use super::super::identity_verification::ProviderLinkage;
use super::super::ranking::anchor_matches_text;

/// Finding routes from the same annotation origin count as one source.
fn source_family(source: &str) -> &str {
    match source {
        "pubtator" | "litvar" => "annotation",
        other => other,
    }
}

fn text_evidence(
    text: &str,
    requested: &RequestedVariantIdentity,
    aliases: &NormalizedVariantAliases,
) -> bool {
    let Some(gene) = requested.gene.as_deref() else {
        return false;
    };
    let text = normalize_article_search_text(text);
    if !anchor_matches_text(&text, &gene.to_ascii_lowercase()) {
        return false;
    }
    if aliases
        .coding_changes
        .iter()
        .chain(&aliases.genomic_ids)
        .chain(&aliases.rsids)
        .any(|alias| anchor_matches_text(&text, &alias.to_ascii_lowercase()))
    {
        return true;
    }
    // Normalize only spellings the existing variant parser supports. This is
    // lexical relevance; it supplies no identity observation or confirmation.
    static PROTEIN: OnceLock<Regex> = OnceLock::new();
    let protein = PROTEIN.get_or_init(|| {
        Regex::new(r"(?i)\b[a-z*]+[0-9]+(?:_[a-z*]*[0-9]+)?[a-z0-9*]*\b")
            .expect("valid protein spelling regex")
    });
    protein.find_iter(&text).any(|matched| {
        crate::entities::variant::normalize_protein_change(matched.as_str()).is_some_and(|observed| {
            aliases.protein_changes.iter().any(|alias| {
                crate::entities::variant::normalize_protein_change(alias).as_ref() == Some(&observed)
            })
        })
    })
}

fn validated_annotation(candidate: &ArticleCandidate) -> bool {
    candidate.identity.as_ref().is_some_and(|identity| {
        identity.status == "confirmed"
            && identity.contradictions.is_empty()
            && identity.observations.iter().any(|observation| {
                matches!(observation.provider_linkage, Some(ProviderLinkage::Pubtator { .. }))
            })
    })
}

fn evidence_key(
    candidate: &ArticleCandidate,
    requested: &RequestedVariantIdentity,
    aliases: &NormalizedVariantAliases,
) -> (bool, bool, bool, usize, f64) {
    let mut families = BTreeMap::<&str, usize>::new();
    for fact in &candidate.variant_provenance {
        families
            .entry(source_family(&fact.source))
            .and_modify(|position| *position = (*position).min(fact.native_position))
            .or_insert(fact.native_position);
    }
    let civic = candidate.variant_provenance.iter().any(|fact| {
        fact.source == "civic" && fact.route == "source_citation"
    });
    (
        text_evidence(&candidate.row.title, requested, aliases),
        text_evidence(&candidate.row.normalized_abstract, requested, aliases)
            || validated_annotation(candidate),
        civic,
        families.len(),
        families.values().map(|position| 1.0 / (60.0 + *position as f64)).sum(),
    )
}

pub(super) fn rank_candidates(
    candidates: &mut [ArticleCandidate],
    requested: &RequestedVariantIdentity,
    aliases: &NormalizedVariantAliases,
) {
    candidates.sort_by_cached_key(|candidate| {
        let (title, abstract_or_annotation, civic, families, reciprocal_rank) =
            evidence_key(candidate, requested, aliases);
        (
            std::cmp::Reverse((title, abstract_or_annotation, civic, families)),
            // Every term is positive and finite, so IEEE bit order agrees with
            // numeric order and gives a deterministic total comparison.
            std::cmp::Reverse(reciprocal_rank.to_bits()),
            stable_article_identifier(&candidate.row),
        )
    });
}
