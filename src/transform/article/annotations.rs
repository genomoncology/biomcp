//! PubTator annotation aggregation for article detail views.

use std::collections::{HashMap, HashSet};

use crate::entities::article::{AnnotationCount, AnnotationPosition, ArticleAnnotations};
use crate::entities::variant::{
    VariantIdFormat, VariantInputKind, VariantShorthand, classify_variant_input,
};
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
/// the rsID while that rsID names one allele in this document and the
/// gene-qualified HGVS expression otherwise.
fn annotation_identity(
    kind: AnnotationKind,
    infons: &crate::sources::pubtator::PubTatorAnnotationInfons,
    mutation_context: &MutationContext,
) -> Option<AnnotationIdentity> {
    match kind {
        AnnotationKind::Gene => clean_identifier(infons.identifier.as_deref()?)
            .map(|identifier| ("NCBIGene", identifier.to_string())),
        AnnotationKind::Disease | AnnotationKind::Chemical => {
            registry_identity(infons.identifier.as_deref()?)
        }
        AnnotationKind::Mutation => mutation_identity(infons, mutation_context),
    }
}

fn first_rsid(infons: &crate::sources::pubtator::PubTatorAnnotationInfons) -> Option<&str> {
    infons
        .rsid
        .as_deref()
        .and_then(clean_identifier)
        .or_else(|| {
            infons
                .rsids
                .as_ref()
                .and_then(|rsids| rsids.first())
                .and_then(|rsid| clean_identifier(rsid))
        })
}

fn is_protein_hgvs(hgvs: &str) -> bool {
    hgvs.trim().to_ascii_lowercase().starts_with("p.")
}

/// One rsID legitimately names several alleles (KRAS G12A, G12D and G12V all
/// carry rs121913529), so a mutation row keeps the rsID identity only while
/// the document's own annotations show that rsID naming one change. Otherwise
/// the row carries the allele-specific form: the gene symbol the document's
/// gene annotations give plus the HGVS protein change, which `get variant`
/// accepts as exact input. A coding HGVS names the allele alone. A protein
/// change without a usable gene symbol has no typed-back form, so the row
/// carries no identifier and keeps its mention-text command.
fn mutation_identity(
    infons: &crate::sources::pubtator::PubTatorAnnotationInfons,
    mutation_context: &MutationContext,
) -> Option<AnnotationIdentity> {
    if let Some(rsid) = first_rsid(infons) {
        if !mutation_context
            .multi_allele_rsids
            .contains(&rsid.to_ascii_lowercase())
        {
            return Some(("rsID", rsid.to_string()));
        }
        let hgvs = infons.hgvs.as_deref().and_then(clean_identifier)?;
        if !is_protein_hgvs(hgvs) {
            return Some(("HGVS", hgvs.to_string()));
        }
        return mutation_context
            .gene_qualified_change(infons, hgvs)
            .map(|form| ("HGVS", form));
    }
    infons
        .hgvs
        .as_deref()
        .and_then(clean_identifier)
        .map(|hgvs| ("HGVS", hgvs.to_string()))
}

/// Document-wide facts that disambiguate mutation identities: the gene symbol
/// each NCBI Gene id carries, and the rsIDs whose mentions span more than one
/// distinct protein change.
struct MutationContext {
    gene_symbols: HashMap<u64, String>,
    multi_allele_rsids: HashSet<String>,
}

impl MutationContext {
    fn collect(doc: &PubTatorDocument) -> Self {
        let mut gene_tallies: HashMap<u64, HashMap<String, (u32, usize)>> = HashMap::new();
        let mut rsid_changes: HashMap<String, HashSet<String>> = HashMap::new();
        let mut order = 0usize;

        for passage in &doc.passages {
            for ann in &passage.annotations {
                if let (Some(text), Some(infons)) = (ann.text.as_deref(), ann.infons.as_ref())
                    && let Some(kind) = infons.kind.as_deref().and_then(annotation_kind)
                {
                    match kind {
                        AnnotationKind::Gene => {
                            let text = text.trim();
                            if let Some(identifier) = infons.identifier.as_deref().map(str::trim)
                                && !text.is_empty()
                                && text.len() <= 128
                                && let Ok(gene_id) = identifier.parse::<u64>()
                            {
                                let tally = gene_tallies.entry(gene_id).or_default();
                                tally
                                    .entry(text.to_string())
                                    .and_modify(|(count, _)| *count += 1)
                                    .or_insert((1, order));
                            }
                        }
                        AnnotationKind::Mutation => {
                            if let Some(rsid) = first_rsid(infons)
                                && let Some(hgvs) =
                                    infons.hgvs.as_deref().and_then(clean_identifier)
                            {
                                rsid_changes
                                    .entry(rsid.to_ascii_lowercase())
                                    .or_default()
                                    .insert(distinct_change_key(hgvs));
                            }
                        }
                        AnnotationKind::Disease | AnnotationKind::Chemical => {}
                    }
                }
                order += 1;
            }
        }

        let gene_symbols = gene_tallies
            .into_iter()
            .map(|(gene_id, texts)| {
                // Most-mentioned text wins; ties break to the earliest mention
                // and then to the lexicographically smaller text.
                let (symbol, _) = texts
                    .into_iter()
                    .min_by(
                        |(left_text, (left_count, left_order)),
                         (right_text, (right_count, right_order))| {
                            right_count
                                .cmp(left_count)
                                .then_with(|| left_order.cmp(right_order))
                                .then_with(|| left_text.cmp(right_text))
                        },
                    )
                    .expect("tally entries are non-empty");
                (gene_id, symbol)
            })
            .collect();
        let multi_allele_rsids = rsid_changes
            .into_iter()
            .filter_map(|(rsid, changes)| (changes.len() > 1).then_some(rsid))
            .collect();

        Self {
            gene_symbols,
            multi_allele_rsids,
        }
    }

    fn gene_qualified_change(
        &self,
        infons: &crate::sources::pubtator::PubTatorAnnotationInfons,
        hgvs: &str,
    ) -> Option<String> {
        let gene_id = infons.gene_id.or_else(|| {
            infons
                .gene_ids
                .as_ref()
                .and_then(|ids| ids.first().copied())
        })?;
        let symbol = self.gene_symbols.get(&gene_id)?;
        let form = format!("{symbol} {hgvs}");
        matches!(
            classify_variant_input(&form),
            VariantInputKind::Exact(VariantIdFormat::GeneProteinChange { .. })
        )
        .then_some(form)
    }
}

/// Distinct-allele comparison key for the protein changes sharing one rsID.
fn distinct_change_key(hgvs: &str) -> String {
    match classify_variant_input(hgvs) {
        VariantInputKind::Shorthand(VariantShorthand::ProteinChangeOnly { change }) => change,
        _ => hgvs.trim().to_string(),
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
    let mutation_context = MutationContext::collect(doc);
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
            let identity = annotation_identity(kind, infons, &mutation_context);

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
