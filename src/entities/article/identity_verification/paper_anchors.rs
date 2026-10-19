//! Literal transcript substitutions in prose, tied to typed provider spans.

use std::collections::BTreeSet;

use super::*;
use crate::entities::article::variant_search::{
    CanonicalEquivalence, CanonicalEquivalenceStatus, VariantArticleExecutionContext,
    canonical_observation,
};
use crate::entities::variant::CarNormalizationStatus;
use crate::sources::clingen_allele_registry::ClinGenAlleleRegistryClient;

const MAX_PAPER_ANCHORS: usize = 2;

struct PaperAnchor {
    literal: String,
    query: String,
    observation: VariantArticleIdentityObservation,
}

fn literal_anchors(
    requested: &RequestedVariantIdentity,
    expected_pmid: &str,
    response: &PubTatorExportResponse,
) -> Vec<PaperAnchor> {
    static ANCHOR_RE: OnceLock<Regex> = OnceLock::new();
    let pattern = ANCHOR_RE.get_or_init(|| {
        Regex::new(r"\b(NM_[0-9]+\.[0-9]+):(c\.[0-9]+[ACGT]\s*>\s*[ACGT])\b")
            .expect("valid literal transcript substitution regex")
    });
    let mut anchors = Vec::new();
    let mut seen = BTreeSet::new();
    for document in &response.documents {
        if !document_matches_pmid(document, expected_pmid) {
            continue;
        }
        for (index, passage) in document.passages.iter().enumerate() {
            // Flattened table cells cannot establish which transcript names a row.
            if passage
                .infons
                .as_ref()
                .and_then(|infons| infons.kind.as_deref())
                != Some("paragraph")
            {
                continue;
            }
            let (Some(text), Some(offset)) = (passage.text.as_deref(), passage.offset) else {
                continue;
            };
            for captures in pattern.captures_iter(text) {
                if !bounded(&captures[0], MAX_HGVS_BYTES) {
                    continue;
                }
                let coding_span = captures.get(2).expect("required coding capture");
                let coding = coding_span.as_str().split_whitespace().collect::<String>();
                let query = format!("{}:{coding}", &captures[1]);
                if seen.contains(&query)
                    || crate::entities::variant::validate_car_hgvs_input(&query).is_err()
                {
                    continue;
                }
                let Some(start) =
                    offset.checked_add(text[..coding_span.start()].chars().count() as u64)
                else {
                    continue;
                };
                let length = coding_span.as_str().chars().count() as u64;
                let Some(annotation_id) = passage.annotations.iter().find_map(|annotation| {
                    let (hgvs, _, _, id) = typed_variant(annotation)?;
                    (hgvs == coding
                        && annotation
                            .locations
                            .iter()
                            .any(|location| location.offset == start && location.length == length))
                    .then_some(id)
                }) else {
                    continue;
                };
                // Reuse the existing typed Gene and expected-PMID verifier. Literal
                // text supplies the transcript only; it never supplies gene linkage.
                let mut anchored = requested.clone();
                anchored.protein_change = None;
                anchored.coding_change = Some(coding);
                anchored.transcript = Some(captures[1].to_string());
                let linked = verify_pubtator_document(
                    &anchored,
                    expected_pmid,
                    response,
                    false,
                    Some(document),
                );
                let suffix = format!(":variant-passage:{}", index + 1);
                let Some(mut observation) = linked.observations.into_iter().find(|observation| {
                    observation.allele_annotation_id == annotation_id
                        && observation.locator.ends_with(&suffix)
                }) else {
                    continue;
                };
                observation
                    .locator
                    .push_str(&format!(":offset:{start}:length:{length}"));
                seen.insert(query.clone());
                anchors.push(PaperAnchor {
                    literal: captures[0].to_string(),
                    query,
                    observation,
                });
                // One extra anchor records that verification was bounded.
                if anchors.len() > MAX_PAPER_ANCHORS {
                    return anchors;
                }
            }
        }
    }
    anchors
}

pub(crate) async fn verify_paper_anchors(
    requested: &RequestedVariantIdentity,
    expected_pmid: &str,
    response: &PubTatorExportResponse,
    equivalence: &CanonicalEquivalence,
    execution: &VariantArticleExecutionContext,
    identity: &mut VariantArticleIdentity,
) -> bool {
    // A paper anchor cannot repair an ambiguous, incomplete, or unsupported
    // caller identity, or override an existing contradiction.
    let Some(caid) = equivalence.caid.as_deref().filter(|_| {
        identity.status == "unverified"
            && matches!(equivalence.status.as_str(), "single_identity" | "confirmed")
            && equivalence.complete
            && equivalence.exhaustive
            && !equivalence.observations.is_empty()
            && equivalence.observations.iter().all(|observation| {
                observation.status == CanonicalEquivalenceStatus::Resolved
                    && observation.provider_response_sha256.is_some()
            })
    }) else {
        return false;
    };
    let anchors = literal_anchors(requested, expected_pmid, response);
    let mut incomplete = anchors.len() > MAX_PAPER_ANCHORS;
    for anchor in anchors.into_iter().take(MAX_PAPER_ANCHORS) {
        execution.add_route_unit("identity_verification", "clingen_car");
        let Some(unit) = execution
            .begin_provider_unit("identity_verification", "clingen_car")
            .await
        else {
            incomplete = true;
            continue;
        };
        let result =
            match ClinGenAlleleRegistryClient::new_with_deadline(execution.deadline()).await {
                Ok(client) => client.normalize(&anchor.query).await,
                Err(error) => Err(error),
            };
        let item = match result {
            Ok(item) => item,
            Err(error) => {
                unit.record_error(&error);
                incomplete = true;
                continue;
            }
        };
        let status = match item.status {
            CarNormalizationStatus::Resolved | CarNormalizationStatus::NotFound => "ok",
            CarNormalizationStatus::Unavailable => "unavailable",
            CarNormalizationStatus::Invalid | CarNormalizationStatus::Indeterminate => "degraded",
        };
        incomplete |= !matches!(
            item.status,
            CarNormalizationStatus::Resolved | CarNormalizationStatus::NotFound
        );
        let confirms = item.status == CarNormalizationStatus::Resolved
            && item.exhaustive
            && item.caid.as_deref() == Some(caid)
            && item.provenance.response_sha256.is_some();
        unit.commit(status, 1, || {
            if confirms {
                let mut observation = anchor.observation;
                let Some(paper_linkage) = observation.provider_linkage.take() else {
                    return;
                };
                observation.observed_alias.clone_from(&anchor.literal);
                observation.provider_linkage = Some(ProviderLinkage::PaperTranscript {
                    literal_anchor: anchor.literal,
                    query: anchor.query.clone(),
                    caid: caid.to_string(),
                    paper_linkage: Box::new(paper_linkage),
                    requested_resolutions: equivalence.observations.clone(),
                    resolution: Box::new(canonical_observation(
                        "paper_transcript".into(),
                        anchor.query,
                        item,
                    )),
                });
                identity.observations.push(observation);
                identity.status = status_for(&identity.observations, &identity.contradictions);
                identity.basis = "paper_transcript_anchor";
                identity.reason = None;
            }
        });
    }
    identity.incomplete |= incomplete;
    incomplete
}
