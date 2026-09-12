//! Reference-list rendering for JATS full-text extraction.

use roxmltree::{Node, NodeType};

use super::super::collapse_whitespace;
use super::find_child;

pub(super) fn render_references(root: Node<'_, '_>) -> Option<String> {
    let references = reference_nodes(root);
    if references.is_empty() {
        return None;
    }

    let items = references
        .into_iter()
        .enumerate()
        .map(|(index, ref_node)| render_reference(ref_node, index + 1))
        .collect::<Vec<_>>();

    Some(format!("## References\n\n{}", items.join("\n")))
}

fn reference_nodes<'a, 'input>(root: Node<'a, 'input>) -> Vec<Node<'a, 'input>> {
    root.descendants()
        .filter(|node| node.is_element() && node.has_tag_name("ref"))
        .filter(|node| {
            node.ancestors()
                .any(|ancestor| ancestor.is_element() && ancestor.has_tag_name("ref-list"))
        })
        .collect()
}

fn render_reference(ref_node: Node<'_, '_>, ordinal: usize) -> String {
    let mut citation = if let Some(mixed_citation) = find_child(ref_node, "mixed-citation") {
        render_mixed_citation(mixed_citation)
    } else if let Some(element_citation) = find_child(ref_node, "element-citation") {
        render_element_citation(element_citation)
    } else {
        fallback_reference_text(ref_node)
    };

    if let Some(label) = find_child(ref_node, "label")
        .map(reference_inline_text)
        .filter(|value| !value.is_empty())
        .filter(|value| value.parse::<usize>().is_err())
    {
        let marker = if label.starts_with('[') && label.ends_with(']') {
            label
        } else {
            format!("[{label}]")
        };
        citation = if citation.is_empty() {
            marker
        } else {
            format!("{marker} {citation}")
        };
    }

    if citation.is_empty() {
        format!("{ordinal}.")
    } else {
        format!("{ordinal}. {citation}")
    }
}

fn render_mixed_citation(node: Node<'_, '_>) -> String {
    reference_inline_text(node)
}

fn render_element_citation(node: Node<'_, '_>) -> String {
    let mut parts = Vec::new();

    if let Some(authors) = render_author_segment(node) {
        parts.push(authors);
    }

    if let Some(title) = find_child(node, "article-title")
        .or_else(|| find_child(node, "chapter-title"))
        .map(reference_inline_text)
        .filter(|value| !value.is_empty())
    {
        parts.push(title);
    }

    if let Some(source) = find_child(node, "source")
        .map(reference_inline_text)
        .filter(|value| !value.is_empty())
    {
        parts.push(source);
    }

    if let Some(bibliographic) = render_bibliographic_segment(node) {
        parts.push(bibliographic);
    }

    parts.extend(render_identifier_segment(node));
    parts.extend(render_extra_reference_links(node));

    let rendered = join_reference_parts(parts);
    if rendered.is_empty() {
        reference_inline_text(node)
    } else {
        rendered
    }
}

fn render_author_segment(node: Node<'_, '_>) -> Option<String> {
    let person_group = node
        .children()
        .find(|child| {
            child.is_element()
                && child.has_tag_name("person-group")
                && child
                    .attribute("person-group-type")
                    .is_some_and(|value| value.eq_ignore_ascii_case("author"))
        })
        .or_else(|| {
            node.children()
                .find(|child| child.is_element() && child.has_tag_name("person-group"))
        });

    if let Some(person_group) = person_group {
        let mut authors = Vec::new();
        for child in person_group.children().filter(|child| child.is_element()) {
            match child.tag_name().name() {
                "name" => {
                    let name = render_person_name(child);
                    if !name.is_empty() {
                        authors.push(name);
                    }
                }
                "string-name" | "collab" => {
                    let text = reference_inline_text(child);
                    if !text.is_empty() {
                        authors.push(text);
                    }
                }
                "etal" => authors.push("et al.".to_string()),
                _ => {}
            }
        }
        if !authors.is_empty() {
            return Some(authors.join(", "));
        }
    }

    let mut authors = Vec::new();
    for child in node.children().filter(|child| child.is_element()) {
        match child.tag_name().name() {
            "name" => {
                let name = render_person_name(child);
                if !name.is_empty() {
                    authors.push(name);
                }
            }
            "string-name" | "collab" => {
                let text = reference_inline_text(child);
                if !text.is_empty() {
                    authors.push(text);
                }
            }
            "etal" => authors.push("et al.".to_string()),
            _ => {}
        }
    }

    if authors.is_empty() {
        None
    } else {
        Some(authors.join(", "))
    }
}

fn render_person_name(node: Node<'_, '_>) -> String {
    let surname = find_child(node, "surname")
        .map(reference_inline_text)
        .filter(|value| !value.is_empty());
    let given_names = find_child(node, "given-names")
        .map(reference_inline_text)
        .filter(|value| !value.is_empty());

    match (surname, given_names) {
        (Some(surname), Some(given_names)) => format!("{surname} {given_names}"),
        (Some(surname), None) => surname,
        (None, Some(given_names)) => given_names,
        (None, None) => reference_inline_text(node),
    }
}

fn render_bibliographic_segment(node: Node<'_, '_>) -> Option<String> {
    let year = find_child(node, "year")
        .map(reference_inline_text)
        .filter(|value| !value.is_empty());
    let volume = find_child(node, "volume")
        .map(reference_inline_text)
        .filter(|value| !value.is_empty());
    let issue = find_child(node, "issue")
        .map(reference_inline_text)
        .filter(|value| !value.is_empty());
    let comment = find_child(node, "comment")
        .map(reference_inline_text)
        .filter(|value| !value.is_empty());

    let pages = match (
        find_child(node, "fpage")
            .map(reference_inline_text)
            .filter(|value| !value.is_empty()),
        find_child(node, "lpage")
            .map(reference_inline_text)
            .filter(|value| !value.is_empty()),
    ) {
        (Some(fpage), Some(lpage)) if fpage == lpage => Some(fpage),
        (Some(fpage), Some(lpage)) => Some(format!("{fpage}-{lpage}")),
        (Some(fpage), None) => Some(fpage),
        (None, Some(lpage)) => Some(lpage),
        (None, None) => find_child(node, "elocation-id")
            .map(reference_inline_text)
            .filter(|value| !value.is_empty()),
    };

    let mut out = String::new();
    if let Some(year) = year {
        out.push_str(&year);
    }
    match (volume, issue, pages) {
        (Some(volume), issue, pages) => {
            if !out.is_empty() {
                out.push(';');
            }
            out.push_str(&volume);
            if let Some(issue) = issue {
                out.push('(');
                out.push_str(&issue);
                out.push(')');
            }
            if let Some(pages) = pages {
                out.push(':');
                out.push_str(&pages);
            }
        }
        (None, Some(issue), pages) => {
            if !out.is_empty() {
                out.push(';');
            }
            out.push('(');
            out.push_str(&issue);
            out.push(')');
            if let Some(pages) = pages {
                out.push(':');
                out.push_str(&pages);
            }
        }
        (None, None, Some(pages)) => {
            if !out.is_empty() {
                out.push(':');
            }
            out.push_str(&pages);
        }
        (None, None, None) => {}
    }

    if let Some(comment) = comment {
        if !out.is_empty() {
            out.push_str(". ");
        }
        out.push_str(&comment);
    }

    if out.is_empty() { None } else { Some(out) }
}

fn render_identifier_segment(node: Node<'_, '_>) -> Vec<String> {
    let mut identifiers = Vec::new();

    for child in node
        .children()
        .filter(|child| child.is_element() && child.has_tag_name("pub-id"))
    {
        let mut text = String::new();
        for grandchild in child.children() {
            append_reference_inline_node(grandchild, &mut text);
        }
        let text = collapse_whitespace(&text);
        if text.is_empty() {
            continue;
        }

        match child.attribute("pub-id-type") {
            Some(value) if value.eq_ignore_ascii_case("doi") => {
                let doi = normalize_doi(&text).unwrap_or(text);
                identifiers.push(format_doi_link(&doi));
            }
            Some(value) if value.eq_ignore_ascii_case("pmid") => {
                identifiers.push(format!("PMID: {text}"));
            }
            Some(value) if value.eq_ignore_ascii_case("pmcid") => {
                identifiers.push(format!("PMCID: {text}"));
            }
            _ => {}
        }
    }

    identifiers
}

fn render_extra_reference_links(node: Node<'_, '_>) -> Vec<String> {
    const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

    let mut links = Vec::new();

    for child in node
        .descendants()
        .filter(|child| child.is_element() && child.has_tag_name("ext-link"))
    {
        let text = reference_inline_text(child);
        let url = child
            .attribute((XLINK_NS, "href"))
            .or_else(|| child.attribute("href"))
            .map(str::trim)
            .filter(|value| !value.is_empty());

        if child
            .attribute("ext-link-type")
            .is_some_and(|value| value.eq_ignore_ascii_case("doi"))
            || url.and_then(normalize_doi).is_some()
        {
            continue;
        }

        let rendered = match (text.is_empty(), url) {
            (false, Some(url)) => format!("[{text}]({url})"),
            (false, None) => text,
            (true, Some(url)) => url.to_string(),
            (true, None) => String::new(),
        };

        if !rendered.is_empty() && !links.contains(&rendered) {
            links.push(rendered);
        }
    }

    links
}

fn fallback_reference_text(ref_node: Node<'_, '_>) -> String {
    reference_inline_text(ref_node)
}

fn reference_inline_text(node: Node<'_, '_>) -> String {
    let mut out = String::new();
    match node.node_type() {
        NodeType::Text => out.push_str(node.text().unwrap_or_default()),
        _ => {
            for child in node.children() {
                append_reference_inline_node(child, &mut out);
            }
        }
    }
    collapse_whitespace(&out)
}

fn append_reference_inline_node(node: Node<'_, '_>, out: &mut String) {
    match node.node_type() {
        NodeType::Root => {
            for child in node.children() {
                append_reference_inline_node(child, out);
            }
        }
        NodeType::Element => {
            match node.tag_name().name() {
                "label" => return,
                "italic" => return append_wrapped_reference_inline(node, "*", out),
                "bold" => return append_wrapped_reference_inline(node, "**", out),
                "sup" => return append_wrapped_reference_inline(node, "^", out),
                "sub" => return append_wrapped_reference_inline(node, "~", out),
                "ext-link" => return append_reference_ext_link(node, out),
                "pub-id" => return append_reference_pub_id(node, out),
                _ => {}
            }

            for child in node.children() {
                append_reference_inline_node(child, out);
            }
        }
        NodeType::Text => out.push_str(node.text().unwrap_or_default()),
        _ => {}
    }
}

fn append_wrapped_reference_inline(node: Node<'_, '_>, marker: &str, out: &mut String) {
    let text = reference_inline_text(node);
    if text.is_empty() {
        return;
    }
    out.push_str(marker);
    out.push_str(&text);
    out.push_str(marker);
}

fn append_reference_ext_link(node: Node<'_, '_>, out: &mut String) {
    const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

    let text = reference_inline_text(node);
    let url = node
        .attribute((XLINK_NS, "href"))
        .or_else(|| node.attribute("href"))
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if node
        .attribute("ext-link-type")
        .is_some_and(|value| value.eq_ignore_ascii_case("doi"))
        && let Some(doi) = url.and_then(normalize_doi).or_else(|| normalize_doi(&text))
    {
        out.push_str(&format_doi_link(&doi));
        return;
    }

    if let Some(doi) = url.and_then(normalize_doi) {
        out.push_str(&format_doi_link(&doi));
        return;
    }

    match (text.is_empty(), url) {
        (false, Some(url)) => {
            out.push('[');
            out.push_str(&text);
            out.push_str("](");
            out.push_str(url);
            out.push(')');
        }
        (false, None) => out.push_str(&text),
        (true, Some(url)) => out.push_str(url),
        (true, None) => {}
    }
}

fn append_reference_pub_id(node: Node<'_, '_>, out: &mut String) {
    let mut text = String::new();
    for child in node.children() {
        append_reference_inline_node(child, &mut text);
    }
    let text = collapse_whitespace(&text);
    if text.is_empty() {
        return;
    }

    match node.attribute("pub-id-type") {
        Some(value) if value.eq_ignore_ascii_case("doi") => {
            let doi = normalize_doi(&text).unwrap_or(text);
            out.push_str(&format_doi_link(&doi));
        }
        Some(value) if value.eq_ignore_ascii_case("pmid") => {
            out.push_str("PMID: ");
            out.push_str(&text);
        }
        Some(value) if value.eq_ignore_ascii_case("pmcid") => {
            out.push_str("PMCID: ");
            out.push_str(&text);
        }
        _ => out.push_str(&text),
    }
}

fn normalize_doi(value: &str) -> Option<String> {
    let trimmed = collapse_whitespace(value);
    if trimmed.is_empty() {
        return None;
    }

    let lower = trimmed.to_ascii_lowercase();
    for prefix in [
        "https://doi.org/",
        "http://doi.org/",
        "https://dx.doi.org/",
        "http://dx.doi.org/",
        "doi:",
    ] {
        if lower.starts_with(prefix) {
            return normalize_doi(&trimmed[prefix.len()..]);
        }
    }

    if lower.starts_with("10.") && trimmed.contains('/') {
        Some(trimmed)
    } else {
        None
    }
}

fn format_doi_link(doi: &str) -> String {
    format!("[{doi}](https://doi.org/{doi})")
}

fn join_reference_parts(parts: Vec<String>) -> String {
    let mut out = String::new();

    for part in parts.into_iter().filter(|part| !part.is_empty()) {
        if out.is_empty() {
            out.push_str(&part);
            continue;
        }

        if out.ends_with('.') || out.ends_with('?') || out.ends_with('!') {
            out.push(' ');
        } else {
            out.push_str(". ");
        }
        out.push_str(&part);
    }

    out
}

// --- Ticket 1145: pure structural citation-evidence extraction ---

/// Normalized typed identifiers of the resolved cited paper.
// --- Ticket 1145: pure structural citation-evidence extraction ---
pub(super) mod citation;

pub(crate) use citation::{
    JatsCitationExtraction, JatsCitationTargetIds, extract_citation_evidence,
};
