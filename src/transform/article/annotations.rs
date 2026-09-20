//! PubTator annotation aggregation for article detail views.

use std::collections::HashMap;

use crate::entities::article::{AnnotationCount, ArticleAnnotations};
use crate::sources::pubtator::{PubTatorDetail, PubTatorDocument};

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

fn push_annotation_count(
    map: &mut HashMap<String, (String, u32, usize)>,
    text: &str,
    order: usize,
) {
    let t = text.trim();
    if t.is_empty() || t.len() > 128 {
        return;
    }
    let key = t.to_ascii_lowercase();
    let entry = map.entry(key).or_insert_with(|| (t.to_string(), 0, order));
    entry.1 += 1;
}

fn finalize_counts(map: HashMap<String, (String, u32, usize)>) -> Vec<AnnotationCount> {
    let mut out = map
        .into_values()
        .map(|(text, count, first_seen_order)| (AnnotationCount { text, count }, first_seen_order))
        .collect::<Vec<_>>();
    out.sort_by(|(a, a_order), (b, b_order)| {
        b.count.cmp(&a.count).then_with(|| a_order.cmp(b_order))
    });
    out.truncate(8);
    out.into_iter().map(|(row, _)| row).collect()
}

pub fn extract_detail_annotations(detail: &PubTatorDetail) -> Option<ArticleAnnotations> {
    match detail {
        PubTatorDetail::Adopted(response) => {
            let record = response.provider_record()?;
            let passages = record.passages();
            let mut accumulator = AnnotationAccumulator::default();
            for index in 0..passages.len() {
                let Some(passage) = passages.get(index) else {
                    continue;
                };
                for annotation in passage.annotations().iter() {
                    if let Some(kind) = annotation.infons().annotation_type() {
                        accumulator.push(annotation.text(), kind);
                    }
                }
            }
            accumulator.finish()
        }
        PubTatorDetail::Legacy { document, .. } => retained_extract_annotations(document),
    }
}

pub(crate) fn retained_extract_annotations(doc: &PubTatorDocument) -> Option<ArticleAnnotations> {
    aggregate_annotations(doc.passages.iter().flat_map(|passage| {
        passage.annotations.iter().filter_map(|annotation| {
            annotation
                .text
                .as_deref()
                .zip(annotation.infons.as_ref()?.kind.as_deref())
        })
    }))
}

type AnnotationMap = HashMap<String, (String, u32, usize)>;

#[derive(Default)]
struct AnnotationAccumulator {
    genes: AnnotationMap,
    diseases: AnnotationMap,
    chemicals: AnnotationMap,
    mutations: AnnotationMap,
    next_order: usize,
}

impl AnnotationAccumulator {
    fn push(&mut self, text: &str, kind: &str) {
        let Some(kind) = annotation_kind(kind) else {
            return;
        };
        let map = match kind {
            AnnotationKind::Gene => &mut self.genes,
            AnnotationKind::Disease => &mut self.diseases,
            AnnotationKind::Chemical => &mut self.chemicals,
            AnnotationKind::Mutation => &mut self.mutations,
        };
        push_annotation_count(map, text, self.next_order);
        self.next_order += 1;
    }

    fn finish(self) -> Option<ArticleAnnotations> {
        let annotations = ArticleAnnotations {
            genes: finalize_counts(self.genes),
            diseases: finalize_counts(self.diseases),
            chemicals: finalize_counts(self.chemicals),
            mutations: finalize_counts(self.mutations),
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
}

fn aggregate_annotations<'a>(
    annotations: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Option<ArticleAnnotations> {
    let mut accumulator = AnnotationAccumulator::default();
    for (text, kind) in annotations {
        accumulator.push(text, kind);
    }
    accumulator.finish()
}

#[cfg(test)]
mod tests;
