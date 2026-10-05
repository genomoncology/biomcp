//! PubTator annotation aggregation for article detail views.

use std::collections::HashMap;

use crate::entities::article::{AnnotationCount, AnnotationPosition, ArticleAnnotations};
use crate::sources::pubtator::PubTatorDocument;

/// Longest identifier or namespace value carried through to entity rows.
const MAX_IDENTIFIER_BYTES: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnnotationKind {
    Gene,
    Disease,
    Chemical,
    Mutation,
}

fn annotation_kind(kind: &str) -> Option<AnnotationKind> {
    let k = kind.trim().to_ascii_lowercase();
    if k.is_empty() {
        return None;
    }
    if k.contains("gene") {
        return Some(AnnotationKind::Gene);
    }
    if k.contains("disease") {
        return Some(AnnotationKind::Disease);
    }
    if k.contains("chemical") || k.contains("drug") {
        return Some(AnnotationKind::Chemical);
    }
    if k.contains("mutation") || k.contains("variant") {
        return Some(AnnotationKind::Mutation);
    }
    None
}

/// A PubTator3 identifier paired with the namespace that names its registry.
/// The namespace spelling matches the identifier form BioMCP can type back in.
type AnnotationIdentity = (&'static str, String);

fn clean_identifier(value: &str) -> Option<&str> {
    let value = value.trim();
    (!value.is_empty() && value != "-" && value.len() <= MAX_IDENTIFIER_BYTES).then_some(value)
}

fn registry_identity(identifier: &str) -> Option<AnnotationIdentity> {
    let identifier = clean_identifier(identifier)?;
    let upper = identifier.to_ascii_uppercase();
    if let Some(rest) = upper.strip_prefix("MESH:") {
        (rest.len() <= MAX_IDENTIFIER_BYTES).then(|| ("MESH", identifier.to_string()))
    } else if let Some(rest) = upper.strip_prefix("OMIM:") {
        (rest.len() <= MAX_IDENTIFIER_BYTES).then(|| ("OMIM", identifier.to_string()))
    } else {
        None
    }
}

/// Read the identifier PubTator3 gives for one annotation. Variant mentions
/// carry a tmVar composite in `identifier`, so their typed-back identifier is
/// the rsID when one exists and the HGVS expression otherwise.
fn annotation_identity(
    kind: AnnotationKind,
    infons: &crate::sources::pubtator::PubTatorAnnotationInfons,
) -> Option<AnnotationIdentity> {
    match kind {
        AnnotationKind::Gene => clean_identifier(infons.identifier.as_deref()?)
            .map(|identifier| ("NCBIGene", identifier.to_string())),
        AnnotationKind::Disease | AnnotationKind::Chemical => {
            registry_identity(infons.identifier.as_deref()?)
        }
        AnnotationKind::Mutation => {
            if let Some(rsid) = infons.rsid.as_deref().and_then(clean_identifier) {
                return Some(("rsID", rsid.to_string()));
            }
            if let Some(rsid) = infons
                .rsids
                .as_ref()
                .and_then(|rsids| rsids.first())
                .and_then(|rsid| clean_identifier(rsid))
            {
                return Some(("rsID", rsid.to_string()));
            }
            if let Some(hgvs) = infons.hgvs.as_deref().and_then(clean_identifier) {
                return Some(("HGVS", hgvs.to_string()));
            }
            None
        }
    }
}

/// Same mention text with different identifiers stays a separate row, so the
/// aggregation key is the pair, not the lowercased text alone.
type AnnotationKey = (String, Option<String>);

struct AnnotationTally {
    text: String,
    count: u32,
    first_seen_order: usize,
    namespace: Option<&'static str>,
    identifier: Option<String>,
    positions: Vec<AnnotationPosition>,
}

fn push_annotation_count(
    map: &mut HashMap<AnnotationKey, AnnotationTally>,
    text: &str,
    identity: Option<AnnotationIdentity>,
    locations: &[crate::sources::pubtator::PubTatorLocation],
    include_positions: bool,
    order: usize,
) {
    let t = text.trim();
    if t.is_empty() || t.len() > 128 {
        return;
    }
    let key = (
        t.to_ascii_lowercase(),
        identity.as_ref().map(|(_, identifier)| identifier.clone()),
    );
    let entry = map.entry(key).or_insert_with(|| AnnotationTally {
        text: t.to_string(),
        count: 0,
        first_seen_order: order,
        namespace: identity.as_ref().map(|(namespace, _)| *namespace),
        identifier: identity.as_ref().map(|(_, identifier)| identifier.clone()),
        positions: Vec::new(),
    });
    entry.count += 1;
    if include_positions {
        entry
            .positions
            .extend(locations.iter().map(|location| AnnotationPosition {
                offset: location.offset,
                length: location.length,
            }));
    }
}

fn finalize_counts(map: HashMap<AnnotationKey, AnnotationTally>) -> Vec<AnnotationCount> {
    let mut out = map
        .into_values()
        .map(|tally| {
            (
                AnnotationCount {
                    text: tally.text,
                    count: tally.count,
                    namespace: tally.namespace.map(str::to_string),
                    identifier: tally.identifier,
                    positions: tally.positions,
                },
                tally.first_seen_order,
            )
        })
        .collect::<Vec<_>>();
    out.sort_by(|(a, a_order), (b, b_order)| {
        b.count.cmp(&a.count).then_with(|| a_order.cmp(b_order))
    });
    out.truncate(8);
    out.into_iter().map(|(row, _)| row).collect()
}

pub fn extract_annotations(
    doc: &PubTatorDocument,
    include_positions: bool,
) -> Option<ArticleAnnotations> {
    let mut genes: HashMap<AnnotationKey, AnnotationTally> = HashMap::new();
    let mut diseases: HashMap<AnnotationKey, AnnotationTally> = HashMap::new();
    let mut chemicals: HashMap<AnnotationKey, AnnotationTally> = HashMap::new();
    let mut mutations: HashMap<AnnotationKey, AnnotationTally> = HashMap::new();
    let mut next_order = 0usize;

    for passage in &doc.passages {
        for ann in &passage.annotations {
            let Some(text) = ann.text.as_deref() else {
                continue;
            };
            let Some(infons) = ann.infons.as_ref() else {
                continue;
            };
            let Some(kind) = infons.kind.as_deref().and_then(annotation_kind) else {
                continue;
            };
            let identity = annotation_identity(kind, infons);

            match kind {
                AnnotationKind::Gene => push_annotation_count(
                    &mut genes,
                    text,
                    identity,
                    &ann.locations,
                    include_positions,
                    next_order,
                ),
                AnnotationKind::Disease => push_annotation_count(
                    &mut diseases,
                    text,
                    identity,
                    &ann.locations,
                    include_positions,
                    next_order,
                ),
                AnnotationKind::Chemical => push_annotation_count(
                    &mut chemicals,
                    text,
                    identity,
                    &ann.locations,
                    include_positions,
                    next_order,
                ),
                AnnotationKind::Mutation => push_annotation_count(
                    &mut mutations,
                    text,
                    identity,
                    &ann.locations,
                    include_positions,
                    next_order,
                ),
            }
            next_order += 1;
        }
    }

    let annotations = ArticleAnnotations {
        genes: finalize_counts(genes),
        diseases: finalize_counts(diseases),
        chemicals: finalize_counts(chemicals),
        mutations: finalize_counts(mutations),
    };

    if annotations.genes.is_empty()
        && annotations.diseases.is_empty()
        && annotations.chemicals.is_empty()
        && annotations.mutations.is_empty()
    {
        None
    } else {
        Some(annotations)
    }
}

#[cfg(test)]
mod tests;
