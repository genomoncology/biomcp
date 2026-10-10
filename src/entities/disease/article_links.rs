//! The article disease-row link rule (ticket 2047). A PubTator3 disease
//! row carries a MeSH or OMIM identifier with the concept name PubTator3
//! assigns it; several MONDO records crosswalk one broad descriptor, and a
//! bare identifier opens whichever card sorts first. A row's follow-up get
//! may name a hit's ontology ID only when exactly one crosswalk hit holds
//! the concept name exactly, so every other row keeps the v0.9.1 search
//! command instead of a confidently wrong get.

use crate::error::BioMcpError;
use crate::transform;

/// Fold a final-word plural conservatively: `Neoplasms` to `neoplasm`,
/// `Adenomas` to `adenoma`. Words whose final `s` is part of the spelling
/// (`adenosis`, `serous`) do not fold.
fn singular_final_word(value: &str) -> String {
    let trimmed = value.trim_end();
    let Some((head, last)) = trimmed.rsplit_once(' ') else {
        return fold_one_word(trimmed).to_string();
    };
    if head.is_empty() {
        return fold_one_word(trimmed).to_string();
    }
    format!("{head} {}", fold_one_word(last))
}

fn fold_one_word(word: &str) -> &str {
    let foldable = word.len() > 3
        && word.ends_with('s')
        && !word.ends_with("ss")
        && !word.ends_with("us")
        && !word.ends_with("is");
    foldable.then(|| &word[..word.len() - 1]).unwrap_or(word)
}

/// An article disease row's concept name holds a hit exactly when a name,
/// label, or synonym is the same phrase (case-insensitive, punctuation and
/// `carcinoma` normalized, final-word plurals folded).
fn crosswalk_hit_holds_name(name: &str, hit: &crate::sources::mydisease::MyDiseaseHit) -> bool {
    let name = super::resolution::normalize_disease_text(name);
    let name_forms = [name.clone(), singular_final_word(&name)];
    let mut labels = vec![transform::disease::name_from_mydisease_hit(hit)];
    if let Some(value) = hit.mondo.as_ref().and_then(|v| v.get("label")) {
        super::resolution::collect_json_strings(value, &mut labels);
    }
    if let Some(value) = hit.mondo.as_ref().and_then(|v| v.get("synonym")) {
        super::resolution::collect_json_strings(value, &mut labels);
    }
    if let Some(value) = hit
        .disease_ontology
        .as_ref()
        .and_then(|v| v.get("synonyms"))
    {
        super::resolution::collect_json_strings(value, &mut labels);
    }
    labels.into_iter().any(|label| {
        let label = super::resolution::normalize_disease_text(&label);
        let label_forms = [label, singular_final_word(&label)];
        name_forms.iter().any(|name| label_forms.contains(name))
    })
}

/// The one crosswalk hit that holds the concept name PubTator3 assigns a
/// row's MeSH or OMIM identifier (ticket 2047). A link may name a hit's
/// ontology ID only when exactly one hit holds that name: several MONDO
/// records crosswalk one broad MeSH descriptor (`MESH:D008175` "Lung
/// Neoplasms" hits both `lung neoplasm` and `lung benign neoplasm`), and a
/// bare identifier opens whichever card sorts first — the wrong card.
fn exact_named_crosswalk_hit<'a>(
    name: &str,
    hits: &'a [crate::sources::mydisease::MyDiseaseHit],
) -> Option<&'a crate::sources::mydisease::MyDiseaseHit> {
    let mut holders = hits
        .iter()
        .filter(|hit| crosswalk_hit_holds_name(name, hit));
    let first = holders.next()?;
    holders.next().is_none().then_some(first)
}

/// The follow-up `get disease` command for an article disease row, or None
/// when the row must keep the v0.9.1 search command (ticket 2047). The row's
/// MeSH or OMIM identifier resolves through the MyDisease crosswalk, and the
/// command names the crosswalk hit that holds the identifier's concept name
/// exactly; no name, an unreadable crosswalk, or several holders means no
/// verified card exists, so the row falls back to its text search instead of
/// a confidently wrong get. OMIM rows keep their `OMIM:`-prefixed form so the
/// crosswalk sees the same input a user would type.
pub(crate) async fn article_disease_row_get_command(
    namespace: &str,
    identifier: &str,
    concept_name: Option<&str>,
) -> Option<String> {
    let concept_name = concept_name
        .map(str::trim)
        .filter(|name| !name.is_empty())?;
    let identifier = identifier.trim();
    if identifier.is_empty() {
        return None;
    }
    let kind = match namespace.trim().to_ascii_uppercase().as_str() {
        "MESH" => super::resolution::DiseaseXrefKind::Mesh,
        "OMIM" => super::resolution::DiseaseXrefKind::Omim,
        _ => return None,
    };
    let value = identifier
        .split_once(':')
        .map(|(_, rest)| rest)
        .unwrap_or(identifier)
        .trim();
    if value.is_empty() {
        return None;
    }
    let client = crate::sources::mydisease::MyDiseaseClient::new().ok()?;
    let response = match client
        .lookup_disease_by_xref(kind.source_key(), value, 5)
        .await
    {
        Ok(response) => response,
        Err(error) => {
            tracing::debug!(
                identifier = identifier,
                "article disease crosswalk unavailable; keeping the search command: {error}"
            );
            return None;
        }
    };
    let hit = exact_named_crosswalk_hit(concept_name, &response.hits)?;
    Some(format!("biomcp get disease {}", hit.id))
}
