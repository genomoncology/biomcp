//! Pure structural citation-evidence extraction for ticket 1145.

use roxmltree::{Node, NodeType};

use super::super::collapse_whitespace;
use super::find_child;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct JatsCitationTargetIds {
    pub(crate) doi: Option<String>,
    pub(crate) pmid: Option<String>,
    pub(crate) pmcid: Option<String>,
}

/// One recovered passage with its structural locator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct JatsCitationPassage {
    pub(crate) text: String,
    pub(crate) section_path: Vec<String>,
    pub(crate) paragraph: usize,
    pub(crate) marker: String,
}

/// The pure structural outcome of extracting citation evidence from JATS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum JatsCitationExtraction {
    /// XML parsed, but the document is not an article with a body.
    ParsedUnusable,
    /// Body present, but the cited reference could not be resolved exactly.
    ReferenceUnresolved,
    /// Reference resolved, but no unambiguous marker reached an eligible paragraph.
    MarkerUnlinked { ref_id: String },
    /// Reference resolved with passages in document order.
    Linked {
        ref_id: String,
        passages: Vec<JatsCitationPassage>,
    },
}

const EXCLUDED_PARAGRAPH_SCOPES: [&str; 8] = [
    "ref-list",
    "table-wrap",
    "table",
    "fig",
    "caption",
    "fn-group",
    "supplementary-material",
    "boxed-text",
];

pub(crate) fn extract_citation_evidence(
    xml: &str,
    target: &JatsCitationTargetIds,
) -> Result<JatsCitationExtraction, ()> {
    let doc =
        crate::xml::parse_external_xml(xml, crate::xml::ARTICLE_XML_NODE_LIMIT).map_err(|_| ())?;
    let root = doc.root_element();
    if !root.has_tag_name("article") {
        return Ok(JatsCitationExtraction::ParsedUnusable);
    }
    let Some(body) = find_child(root, "body") else {
        return Ok(JatsCitationExtraction::ParsedUnusable);
    };

    let Some(selected) = select_reference(root, target) else {
        return Ok(JatsCitationExtraction::ReferenceUnresolved);
    };

    let passages = collect_passages(body, &selected);
    if passages.is_empty() {
        return Ok(JatsCitationExtraction::MarkerUnlinked { ref_id: selected });
    }
    Ok(JatsCitationExtraction::Linked {
        ref_id: selected,
        passages,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum IdentifierClass {
    Doi,
    Pmid,
    Pmcid,
}

impl IdentifierClass {
    const fn precedence(&self) -> usize {
        match self {
            Self::Doi => 0,
            Self::Pmid => 1,
            Self::Pmcid => 2,
        }
    }
}

#[derive(Debug, Default, Clone)]
struct RefIdentifiers {
    dois: Vec<String>,
    pmids: Vec<String>,
    pmcids: Vec<String>,
}

impl RefIdentifiers {
    fn distinct(&self, class: &IdentifierClass) -> Vec<String> {
        let values = match class {
            IdentifierClass::Doi => &self.dois,
            IdentifierClass::Pmid => &self.pmids,
            IdentifierClass::Pmcid => &self.pmcids,
        };
        let mut seen = Vec::new();
        for value in values {
            if !seen.contains(value) {
                seen.push(value.clone());
            }
        }
        seen
    }
}

fn normalize_reference_doi(value: &str) -> Option<String> {
    let mut value = value.trim().to_ascii_lowercase();
    loop {
        let stripped = [
            "doi:",
            "https://doi.org/",
            "http://doi.org/",
            "https://dx.doi.org/",
            "http://dx.doi.org/",
        ]
        .into_iter()
        .find_map(|prefix| value.strip_prefix(prefix));
        match stripped {
            Some(rest) => value = rest.to_string(),
            None => break,
        }
    }
    let value = value.trim();
    if value.starts_with("10.") && value.contains('/') {
        Some(value.to_string())
    } else {
        None
    }
}

fn normalize_pmid(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let digits = trimmed
        .strip_prefix("pmid:")
        .or_else(|| trimmed.strip_prefix("PMID:"))
        .map(str::trim)
        .unwrap_or(trimmed);
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    digits
        .parse::<u64>()
        .ok()
        .map(|decimal| decimal.to_string())
}

fn normalize_pmcid(value: &str) -> Option<String> {
    let upper = value.trim().to_ascii_uppercase();
    let digits = upper
        .strip_prefix("PMCID:")
        .map(str::trim)
        .unwrap_or(&upper);
    let rest = digits.strip_prefix("PMC")?;
    if rest.is_empty() || !rest.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    rest.parse::<u64>()
        .ok()
        .map(|decimal| format!("PMC{decimal}"))
}

fn extract_ref_identifiers(ref_node: Node<'_, '_>) -> RefIdentifiers {
    let mut ids = RefIdentifiers::default();
    for node in ref_node.descendants() {
        if !node.is_element() {
            continue;
        }
        match node.tag_name().name() {
            "pub-id" => {
                let kind = node.attribute("pub-id-type").map(str::trim);
                match kind.map(|value| value.to_ascii_lowercase()).as_deref() {
                    Some("doi") => {
                        if let Some(doi) = normalize_reference_doi(&plain_inline_text(node)) {
                            ids.dois.push(doi);
                        }
                    }
                    Some("pmid") => {
                        if let Some(pmid) = normalize_pmid(&plain_inline_text(node)) {
                            ids.pmids.push(pmid);
                        }
                    }
                    Some("pmcid") => {
                        if let Some(pmcid) = normalize_pmcid(&plain_inline_text(node)) {
                            ids.pmcids.push(pmcid);
                        }
                    }
                    _ => {}
                }
            }
            "ext-link" => {
                let kind = node.attribute("ext-link-type").map(str::trim);
                if kind.is_some_and(|value| value.eq_ignore_ascii_case("doi"))
                    && let Some(doi) = normalize_reference_doi(&plain_inline_text(node))
                {
                    ids.dois.push(doi);
                }
            }
            _ => {}
        }
    }
    ids
}

fn target_for_class<'a>(
    target: &'a JatsCitationTargetIds,
    class: &IdentifierClass,
) -> Option<&'a str> {
    match class {
        IdentifierClass::Doi => target.doi.as_deref(),
        IdentifierClass::Pmid => target.pmid.as_deref(),
        IdentifierClass::Pmcid => target.pmcid.as_deref(),
    }
}

fn has_conflicting_higher_identifier(
    ids: &RefIdentifiers,
    target: &JatsCitationTargetIds,
    class: &IdentifierClass,
) -> bool {
    let higher = [
        IdentifierClass::Doi,
        IdentifierClass::Pmid,
        IdentifierClass::Pmcid,
    ]
    .into_iter()
    .filter(|candidate| candidate.precedence() < class.precedence());
    for higher_class in higher {
        let Some(want) = target_for_class(target, &higher_class) else {
            // Any valid identifier of a higher class conflicts when the
            // cited paper carries no identifier of that class.
            if !ids.distinct(&higher_class).is_empty() {
                return true;
            }
            continue;
        };
        if ids
            .distinct(&higher_class)
            .iter()
            .any(|value| value != want)
        {
            return true;
        }
    }
    false
}

fn ref_nodes<'a, 'input>(root: Node<'a, 'input>) -> Vec<Node<'a, 'input>> {
    root.descendants()
        .filter(|node| node.is_element() && node.has_tag_name("ref"))
        .filter(|node| {
            node.ancestors()
                .any(|ancestor| ancestor.is_element() && ancestor.has_tag_name("ref-list"))
        })
        .collect()
}

/// Resolve the selected reference ID, or `None` when the cited paper cannot
/// be resolved exactly (zero or multiple distinct matching `<ref>` elements,
/// or a missing/non-unique selected `id`).
fn select_reference(root: Node<'_, '_>, target: &JatsCitationTargetIds) -> Option<String> {
    let refs = ref_nodes(root);
    let scanned: Vec<(Node<'_, '_>, RefIdentifiers)> = refs
        .into_iter()
        .map(|node| (node, extract_ref_identifiers(node)))
        .collect();

    for class in [
        IdentifierClass::Doi,
        IdentifierClass::Pmid,
        IdentifierClass::Pmcid,
    ] {
        let Some(want) = target_for_class(target, &class) else {
            continue;
        };
        let mut matching: Vec<Node<'_, '_>> = Vec::new();
        for (node, ids) in &scanned {
            if has_conflicting_higher_identifier(ids, target, &class) {
                continue;
            }
            let distinct = ids.distinct(&class);
            if distinct.len() == 1 && distinct[0] == want {
                matching.push(*node);
            }
        }
        match matching.len() {
            0 => continue,
            1 => {
                let selected = matching[0];
                let trimmed_id = selected
                    .attribute("id")
                    .map(str::trim)
                    .filter(|id| !id.is_empty())?;
                if ref_id_is_unique_per_ref_list(root, trimmed_id) {
                    return Some(trimmed_id.to_string());
                }
                return None;
            }
            _ => return None,
        }
    }
    None
}

fn ref_id_is_unique_per_ref_list(root: Node<'_, '_>, selected_id: &str) -> bool {
    // Exactly one <ref> in the whole document may carry the selected ID:
    // a second reference with the same ID fails closed even when only one
    // of them matches the cited identifier.
    let mut matching = root.descendants().filter(|node| {
        node.is_element()
            && node.has_tag_name("ref")
            && node
                .ancestors()
                .any(|ancestor| ancestor.is_element() && ancestor.has_tag_name("ref-list"))
            && node
                .attribute("id")
                .map(str::trim)
                .is_some_and(|value| value == selected_id)
    });
    matching.next().is_some() && matching.next().is_none()
}

fn plain_inline_text(node: Node<'_, '_>) -> String {
    let mut out = String::new();
    append_plain_inline(node, &mut out);
    normalize_plain_text(&out)
}

fn append_plain_inline(node: Node<'_, '_>, out: &mut String) {
    match node.node_type() {
        NodeType::Text => out.push_str(node.text().unwrap_or_default()),
        NodeType::Element | NodeType::Root => {
            for child in node.children() {
                append_plain_inline(child, out);
            }
        }
        _ => {}
    }
}

fn normalize_plain_text(value: &str) -> String {
    crate::render::human::sanitize_inline(&collapse_whitespace(value))
}

fn paragraph_is_excluded(paragraph: Node<'_, '_>) -> bool {
    paragraph
        .ancestors()
        .take_while(|ancestor| !ancestor.has_tag_name("body"))
        .any(|ancestor| {
            ancestor.is_element() && EXCLUDED_PARAGRAPH_SCOPES.contains(&ancestor.tag_name().name())
        })
}

fn paragraph_ordinal(body: Node<'_, '_>, paragraph: Node<'_, '_>) -> Option<usize> {
    let mut ordinal = 0;
    for node in body.descendants() {
        if !node.is_element() || !node.has_tag_name("p") || paragraph_is_excluded(node) {
            continue;
        }
        ordinal += 1;
        if node.id() == paragraph.id() {
            return Some(ordinal);
        }
    }
    None
}

fn section_path(paragraph: Node<'_, '_>) -> Vec<String> {
    let mut titles = Vec::new();
    for ancestor in paragraph
        .ancestors()
        .take_while(|ancestor| !ancestor.has_tag_name("body"))
    {
        if !ancestor.is_element() || !ancestor.has_tag_name("sec") {
            continue;
        }
        if let Some(title) = ancestor
            .children()
            .find(|child| child.has_tag_name("title"))
        {
            let text = plain_inline_text(title);
            if !text.is_empty() {
                titles.push(text);
            }
        }
    }
    titles.reverse();
    titles
}

fn eligible_target_marker(node: Node<'_, '_>, selected_id: &str) -> bool {
    if !node.is_element() || !node.has_tag_name("xref") {
        return false;
    }
    if node
        .attribute("ref-type")
        .is_none_or(|value| value != "bibr")
    {
        return false;
    }
    let Some(rid) = node.attribute("rid") else {
        return false;
    };
    let mut tokens: Vec<String> = Vec::new();
    for token in rid.split_whitespace() {
        if !token.is_empty() && !tokens.iter().any(|existing| existing == token) {
            tokens.push(token.to_string());
        }
    }
    tokens.len() == 1 && tokens[0] == selected_id
}

fn render_paragraph_text(paragraph: Node<'_, '_>, selected_id: &str) -> (String, String, usize) {
    let mut out = String::new();
    let mut first_marker = String::new();
    let mut first_marker_raw = String::new();
    let mut first_marker_start = usize::MAX;
    append_paragraph_text(
        paragraph,
        selected_id,
        &mut out,
        &mut first_marker,
        &mut first_marker_raw,
        &mut first_marker_start,
    );
    let text = normalize_plain_text(&out);
    let start = if first_marker_start == usize::MAX {
        usize::MAX
    } else {
        // The marker start was recorded as a byte offset into the raw
        // accumulated text. Normalizing the raw prefix before the marker
        // separately yields the marker's own scalar position in the
        // normalized output; searching for the marker text could instead
        // land on an earlier unrelated occurrence and let the bounded slice
        // drop the marker entirely. A whitespace run between the prefix and
        // the marker collapses to exactly one space, and that joining space
        // exists when either side contributes whitespace.
        let raw_prefix = &out[..first_marker_start];
        let normalized_prefix = normalize_plain_text(raw_prefix);
        if normalized_prefix.is_empty() {
            0
        } else if raw_prefix.ends_with(char::is_whitespace)
            || first_marker_raw.starts_with(char::is_whitespace)
        {
            normalized_prefix.chars().count() + 1
        } else {
            normalized_prefix.chars().count()
        }
    };
    (text, normalize_plain_text(&first_marker), start)
}

fn append_paragraph_text(
    node: Node<'_, '_>,
    selected_id: &str,
    out: &mut String,
    first_marker: &mut String,
    first_marker_raw: &mut String,
    first_marker_start: &mut usize,
) {
    match node.node_type() {
        NodeType::Text => out.push_str(node.text().unwrap_or_default()),
        NodeType::Element => {
            if eligible_target_marker(node, selected_id) && *first_marker_start == usize::MAX {
                let marker = plain_inline_text(node);
                if !marker.is_empty() {
                    let mut raw = String::new();
                    append_plain_inline(node, &mut raw);
                    *first_marker = marker;
                    *first_marker_raw = raw;
                    *first_marker_start = out.len();
                }
            }
            for child in node.children() {
                append_paragraph_text(
                    child,
                    selected_id,
                    out,
                    first_marker,
                    first_marker_raw,
                    first_marker_start,
                );
            }
        }
        _ => {}
    }
}

const PASSAGE_SCALAR_LIMIT: usize = 1_200;
const PASSAGE_SLICE_SCALARS: usize = 1_198;

fn truncate_passage(text: &str, marker_start: Option<usize>) -> String {
    let scalars: Vec<char> = text.chars().collect();
    if scalars.len() <= PASSAGE_SCALAR_LIMIT {
        return text.to_string();
    }
    let Some(marker_start) = marker_start else {
        return text.to_string();
    };
    let anchor = marker_start.min(scalars.len().saturating_sub(1));
    let max_start = scalars.len() - PASSAGE_SLICE_SCALARS;
    let start = anchor.saturating_sub(599).min(max_start);
    let end = start + PASSAGE_SLICE_SCALARS;
    let mut slice: String = scalars[start..end].iter().collect();
    if start > 0 {
        slice.insert(0, '\u{2026}');
    }
    if end < scalars.len() {
        slice.push('\u{2026}');
    }
    slice
}

fn collect_passages(body: Node<'_, '_>, selected_id: &str) -> Vec<JatsCitationPassage> {
    struct PendingPassage {
        paragraph_id: roxmltree::NodeId,
        text: String,
        marker: String,
        marker_start: Option<usize>,
    }
    let mut pending: Vec<PendingPassage> = Vec::new();
    for node in body.descendants() {
        if !eligible_target_marker(node, selected_id) {
            continue;
        }
        // A marker inside an excluded scope (table cell, caption, ...) is
        // ineligible before paragraph recovery: the walk to the closest
        // ancestor <p> must not pass through an excluded scope.
        let mut paragraph = None;
        let mut excluded_path = false;
        for ancestor in node.ancestors() {
            if ancestor.is_element()
                && EXCLUDED_PARAGRAPH_SCOPES.contains(&ancestor.tag_name().name())
            {
                excluded_path = true;
                break;
            }
            if ancestor.is_element() && ancestor.has_tag_name("p") {
                paragraph = Some(ancestor);
                break;
            }
        }
        let Some(paragraph) = paragraph else {
            continue;
        };
        if excluded_path || paragraph_is_excluded(paragraph) {
            continue;
        }
        let (text, marker, marker_start) = render_paragraph_text(paragraph, selected_id);
        if marker.is_empty() {
            // An empty-text target marker is ineligible and supplies no
            // structural insertion point, but the paragraph may still be
            // recovered through a later nonblank marker.
            continue;
        }
        if pending
            .iter()
            .any(|item| item.paragraph_id == paragraph.id())
        {
            continue;
        }
        pending.push(PendingPassage {
            paragraph_id: paragraph.id(),
            text,
            marker,
            marker_start: Some(marker_start),
        });
    }

    let mut passages = Vec::new();
    for item in pending {
        if item.text.is_empty() {
            continue;
        }
        let paragraph_node = body.descendants().find(|node| {
            node.is_element() && node.has_tag_name("p") && node.id() == item.paragraph_id
        });
        let Some(paragraph_node) = paragraph_node else {
            continue;
        };
        let Some(ordinal) = paragraph_ordinal(body, paragraph_node) else {
            continue;
        };
        passages.push(JatsCitationPassage {
            text: truncate_passage(&item.text, item.marker_start),
            section_path: section_path(paragraph_node),
            paragraph: ordinal,
            marker: item.marker.clone(),
        });
        if passages.len() == 3 {
            break;
        }
    }
    passages
}
