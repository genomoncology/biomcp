//! JATS full-text extraction and markdown rendering helpers.
use std::collections::HashSet;

use roxmltree::{Node, NodeType};

use crate::entities::article::ArticleFulltextQuality;
use crate::xml::{ARTICLE_XML_NODE_LIMIT, parse_external_xml};

use super::{
    ArticleDocumentCoverage, ArticleDocumentUnusable, ClassifiedArticleDocument,
    collapse_whitespace,
};

mod refs;
mod supplements;
mod tables;
use self::refs::render_references;
pub(crate) use self::supplements::extract_jats_supplement_links;
use self::tables::{convert_complex_table, convert_regular_table};

pub(crate) fn classify_jats_document(
    xml: &str,
) -> Result<ClassifiedArticleDocument, ArticleDocumentUnusable> {
    let doc = parse_external_xml(xml, ARTICLE_XML_NODE_LIMIT)
        .map_err(|_| ArticleDocumentUnusable::Malformed)?;
    let root = doc.root_element();
    if !root.has_tag_name("article") {
        return Err(ArticleDocumentUnusable::Unsupported);
    }
    let abstract_text = find_child(root, "front")
        .and_then(|front| {
            front
                .descendants()
                .find(|node| node.has_tag_name("abstract"))
        })
        .map(inline_text)
        .filter(|text| !text.is_empty());
    let coverage = if find_child(root, "body").is_some_and(body_has_meaningful_content) {
        ArticleDocumentCoverage::FullText
    } else if abstract_text.is_some() {
        ArticleDocumentCoverage::AbstractOnly
    } else {
        ArticleDocumentCoverage::MetadataOnly
    };
    let quality = quality_from_root(root, coverage);
    let markdown = render_jats_root(root);
    if coverage == ArticleDocumentCoverage::FullText && markdown.is_none() {
        return Err(ArticleDocumentUnusable::Conversion);
    }
    Ok(ClassifiedArticleDocument {
        coverage,
        markdown,
        abstract_text,
        quality,
    })
}

fn quality_from_root(
    root: Node<'_, '_>,
    coverage: ArticleDocumentCoverage,
) -> ArticleFulltextQuality {
    ArticleFulltextQuality {
        has_sections: root
            .descendants()
            .any(|node| node.is_element() && node.has_tag_name("sec")),
        has_tables: root.descendants().any(|node| {
            node.is_element() && matches!(node.tag_name().name(), "table" | "table-wrap")
        }),
        has_references: root
            .descendants()
            .any(|node| node.is_element() && node.has_tag_name("ref-list")),
        has_fulltext_signal: coverage == ArticleDocumentCoverage::FullText,
        has_entity_annotations: false,
    }
}

fn body_has_meaningful_content(body: Node<'_, '_>) -> bool {
    body.descendants().any(|node| {
        if !node.is_element()
            || !matches!(
                node.tag_name().name(),
                "p" | "list-item" | "td" | "th" | "caption" | "disp-quote" | "preformat"
            )
            || (node.has_tag_name("caption")
                && !node.ancestors().any(|ancestor| {
                    ancestor.has_tag_name("fig") || ancestor.has_tag_name("table-wrap")
                }))
            || node
                .ancestors()
                .skip(1)
                .take_while(|ancestor| *ancestor != body)
                .filter(|ancestor| ancestor.is_element())
                .any(|ancestor| {
                    !matches!(
                        ancestor.tag_name().name(),
                        "sec"
                            | "fig"
                            | "caption"
                            | "table-wrap"
                            | "table"
                            | "thead"
                            | "tbody"
                            | "tfoot"
                            | "tr"
                            | "td"
                            | "th"
                            | "list"
                            | "list-item"
                            | "disp-quote"
                    )
                })
        {
            return false;
        }
        !inline_text(node).is_empty()
    })
}

fn render_jats_root(root: Node<'_, '_>) -> Option<String> {
    let mut blocks = Vec::new();
    let mut state = RenderState::default();
    convert_front(root, &mut blocks);
    convert_body(root, &mut blocks, &mut state);
    convert_floats_group(root, &mut blocks, &mut state);
    if let Some(references) = render_references(root) {
        blocks.push(references);
    }

    let rendered = join_blocks(blocks);
    (!rendered.is_empty()).then_some(rendered)
}

#[derive(Default)]
struct RenderState {
    rendered_float_ids: HashSet<String>,
}

fn should_skip_rendered_float(node: Node<'_, '_>, state: &RenderState) -> bool {
    node.attribute("id")
        .is_some_and(|id| state.rendered_float_ids.contains(id))
}

fn remember_float_id(node: Node<'_, '_>, state: &mut RenderState) {
    if let Some(id) = node.attribute("id").filter(|id| !id.is_empty()) {
        state.rendered_float_ids.insert(id.to_string());
    }
}

fn convert_front(root: Node<'_, '_>, blocks: &mut Vec<String>) {
    let Some(front) = find_child(root, "front") else {
        return;
    };

    if let Some(title) = front
        .descendants()
        .find(|node| node.is_element() && node.has_tag_name("article-title"))
        .map(inline_text)
        .filter(|value| !value.is_empty())
    {
        blocks.push(format!("# {title}"));
    }

    if let Some(abstract_node) = front
        .descendants()
        .find(|node| node.is_element() && node.has_tag_name("abstract"))
    {
        blocks.push("## Abstract".into());
        let mut state = RenderState::default();
        append_content_blocks(abstract_node, 2, blocks, &mut state);
    }
}

fn convert_body(root: Node<'_, '_>, blocks: &mut Vec<String>, state: &mut RenderState) {
    let Some(body) = find_child(root, "body") else {
        return;
    };
    append_content_blocks(body, 2, blocks, state);
}

fn convert_floats_group(root: Node<'_, '_>, blocks: &mut Vec<String>, state: &mut RenderState) {
    let Some(floats_group) = find_child(root, "floats-group") else {
        return;
    };

    for child in floats_group.children().filter(|child| child.is_element()) {
        match child.tag_name().name() {
            "fig" => {
                if should_skip_rendered_float(child, state) {
                    continue;
                }
                if let Some(figure) = convert_figure(child) {
                    remember_float_id(child, state);
                    blocks.push(figure);
                }
            }
            "table-wrap" => {
                if should_skip_rendered_float(child, state) {
                    continue;
                }
                let table_blocks = convert_table_wrap(child);
                if !table_blocks.is_empty() {
                    remember_float_id(child, state);
                    blocks.extend(table_blocks);
                }
            }
            "supplementary-material" => {
                if let Some(supplement) = convert_supplementary_material(child) {
                    blocks.push(supplement);
                }
            }
            _ => {}
        }
    }
}

fn append_content_blocks(
    node: Node<'_, '_>,
    heading_level: usize,
    blocks: &mut Vec<String>,
    state: &mut RenderState,
) {
    for child in node.children().filter(|child| child.is_element()) {
        match child.tag_name().name() {
            "title" | "label" => {}
            "p" => {
                if let Some(paragraph) = convert_paragraph(child) {
                    blocks.push(paragraph);
                }
            }
            "sec" => convert_section(child, heading_level, blocks, state),
            "fig" => {
                if let Some(figure) = convert_figure(child) {
                    remember_float_id(child, state);
                    blocks.push(figure);
                }
            }
            "table-wrap" => {
                let table_blocks = convert_table_wrap(child);
                if !table_blocks.is_empty() {
                    remember_float_id(child, state);
                    blocks.extend(table_blocks);
                }
            }
            "supplementary-material" => {
                if let Some(supplement) = convert_supplementary_material(child) {
                    blocks.push(supplement);
                }
            }
            "list" => {
                if let Some(list) = convert_list(child) {
                    blocks.push(list);
                }
            }
            "disp-quote" => {
                let text = inline_text(child);
                if !text.is_empty() {
                    blocks.push(format!("> {text}"));
                }
            }
            "preformat" => {
                let text = inline_text(child);
                if !text.is_empty() {
                    blocks.push(format!("```text\n{text}\n```"));
                }
            }
            _ => {}
        }
    }
}

fn convert_section(
    section: Node<'_, '_>,
    heading_level: usize,
    blocks: &mut Vec<String>,
    state: &mut RenderState,
) {
    if let Some(title) = find_child(section, "title")
        .map(inline_text)
        .filter(|value| !value.is_empty())
    {
        let level = heading_level.clamp(2, 6);
        blocks.push(format!("{} {}", "#".repeat(level), title));
    }
    append_content_blocks(section, heading_level + 1, blocks, state);
}

fn convert_paragraph(node: Node<'_, '_>) -> Option<String> {
    let text = inline_text(node);
    if text.is_empty() { None } else { Some(text) }
}

fn convert_figure(node: Node<'_, '_>) -> Option<String> {
    let label = find_child(node, "label").map(inline_text);
    let caption = find_child(node, "caption").and_then(caption_text);

    let mut parts = Vec::new();
    if let Some(label) = label.filter(|value| !value.is_empty()) {
        let suffix = if label.ends_with('.') { "" } else { "." };
        parts.push(format!("**{label}{suffix}**"));
    }
    if let Some(caption) = caption.filter(|value| !value.is_empty()) {
        parts.push(caption);
    }

    if parts.is_empty() {
        None
    } else {
        Some(format!("> {}", parts.join(" ")))
    }
}

fn convert_table_wrap(node: Node<'_, '_>) -> Vec<String> {
    let mut blocks = Vec::new();
    let label = find_child(node, "label").map(inline_text);
    let caption = find_child(node, "caption").and_then(caption_text);

    if label.is_some() || caption.is_some() {
        let mut parts = Vec::new();
        if let Some(label) = label.filter(|value| !value.is_empty()) {
            let suffix = if label.ends_with('.') { "" } else { "." };
            parts.push(format!("**{label}{suffix}**"));
        }
        if let Some(caption) = caption.filter(|value| !value.is_empty()) {
            parts.push(caption);
        }
        if !parts.is_empty() {
            blocks.push(parts.join(" "));
        }
    }

    let Some(table) = node
        .descendants()
        .find(|child| child.is_element() && child.has_tag_name("table"))
    else {
        return blocks;
    };

    if let Some(markdown) = convert_regular_table(table) {
        blocks.push(markdown);
    } else if let Some(markdown) = convert_complex_table(table) {
        blocks.push(markdown);
    }
    blocks
}

fn convert_supplementary_material(node: Node<'_, '_>) -> Option<String> {
    const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

    let label = find_child(node, "label").map(inline_text);
    let caption = find_child(node, "caption").and_then(caption_text);
    let mut parts = Vec::new();

    if let Some(label) = label.filter(|value| !value.is_empty()) {
        let suffix = if label.ends_with('.') { "" } else { "." };
        parts.push(format!("**{label}{suffix}**"));
    }
    if let Some(caption) = caption.filter(|value| !value.is_empty()) {
        parts.push(caption);
    }

    let mut files = Vec::new();
    if let Some(href) = node
        .attribute((XLINK_NS, "href"))
        .or_else(|| node.attribute("href"))
    {
        push_unique_filename(&mut files, href);
    }
    for media in node
        .descendants()
        .filter(|child| child.is_element() && child.has_tag_name("media"))
    {
        if let Some(href) = media
            .attribute((XLINK_NS, "href"))
            .or_else(|| media.attribute("href"))
        {
            push_unique_filename(&mut files, href);
        }
    }
    if !files.is_empty() {
        parts.push(format!("File: {}", files.join(", ")));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

fn push_unique_filename(files: &mut Vec<String>, value: &str) {
    let filename = value.trim();
    if !filename.is_empty() && !files.iter().any(|existing| existing == filename) {
        files.push(filename.to_string());
    }
}

fn convert_list(node: Node<'_, '_>) -> Option<String> {
    let ordered = node
        .attribute("list-type")
        .is_some_and(|value| value.eq_ignore_ascii_case("order"));
    let mut items = Vec::new();

    for (index, item) in node
        .children()
        .filter(|child| child.is_element() && child.has_tag_name("list-item"))
        .enumerate()
    {
        let text = list_item_text(item);
        if text.is_empty() {
            continue;
        }
        if ordered {
            items.push(format!("{}. {text}", index + 1));
        } else {
            items.push(format!("- {text}"));
        }
    }

    if items.is_empty() {
        None
    } else {
        Some(items.join("\n"))
    }
}

fn list_item_text(node: Node<'_, '_>) -> String {
    let mut parts = Vec::new();
    for child in node.children().filter(|child| child.is_element()) {
        match child.tag_name().name() {
            "p" => {
                let text = inline_text(child);
                if !text.is_empty() {
                    parts.push(text);
                }
            }
            "list" => {
                if let Some(text) = convert_list(child) {
                    parts.push(text);
                }
            }
            _ => {
                let text = inline_text(child);
                if !text.is_empty() {
                    parts.push(text);
                }
            }
        }
    }

    if parts.is_empty() {
        inline_text(node)
    } else {
        parts.join(" ")
    }
}

fn caption_text(node: Node<'_, '_>) -> Option<String> {
    let mut parts = Vec::new();
    for child in node.children().filter(|child| child.is_element()) {
        match child.tag_name().name() {
            "title" | "p" => {
                let text = inline_text(child);
                if !text.is_empty() {
                    parts.push(text);
                }
            }
            "list" => {
                if let Some(list) = convert_list(child) {
                    parts.push(list);
                }
            }
            _ => {
                let text = inline_text(child);
                if !text.is_empty() {
                    parts.push(text);
                }
            }
        }
    }

    let joined = parts.join(" ");
    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

fn inline_text(node: Node<'_, '_>) -> String {
    let mut out = String::new();
    match node.node_type() {
        NodeType::Text => out.push_str(node.text().unwrap_or_default()),
        _ => {
            for child in node.children() {
                append_inline_node(child, &mut out);
            }
        }
    }
    collapse_whitespace(&out)
}

fn append_inline_node(node: Node<'_, '_>, out: &mut String) {
    match node.node_type() {
        NodeType::Root => {
            for child in node.children() {
                append_inline_node(child, out);
            }
        }
        NodeType::Element => {
            match node.tag_name().name() {
                "italic" => return append_wrapped_inline(node, "*", out),
                "bold" => return append_wrapped_inline(node, "**", out),
                "sup" => return append_wrapped_inline(node, "^", out),
                "sub" => return append_wrapped_inline(node, "~", out),
                "xref" => return append_xref(node, out),
                "ext-link" => return append_ext_link(node, out),
                _ => {}
            }

            for child in node.children() {
                append_inline_node(child, out);
            }
        }
        NodeType::Text => append_inline_text(node.text().unwrap_or_default(), out),
        _ => {}
    }
}

fn append_inline_text(text: &str, out: &mut String) {
    if (out.ends_with('^') || out.ends_with('~'))
        && let Some(rest) = text.strip_prefix('.')
        && rest.chars().next().is_some_and(char::is_alphanumeric)
    {
        out.push_str(". ");
        out.push_str(rest);
        return;
    }
    out.push_str(text);
}

fn append_wrapped_inline(node: Node<'_, '_>, marker: &str, out: &mut String) {
    let text = inline_text(node);
    if text.is_empty() {
        return;
    }
    out.push_str(marker);
    out.push_str(&text);
    out.push_str(marker);
}

fn append_xref(node: Node<'_, '_>, out: &mut String) {
    let text = inline_text(node);
    if text.is_empty() {
        return;
    }
    match node.attribute("ref-type") {
        Some("bibr") => {
            out.push('[');
            out.push_str(&text);
            out.push(']');
        }
        Some("fig") | Some("table") if xref_has_source_parentheses(node, out) => {
            out.push_str(&text);
        }
        Some("fig") | Some("table") => {
            out.push('(');
            out.push_str(&text);
            out.push(')');
        }
        _ => out.push_str(&text),
    }
}

fn xref_has_source_parentheses(node: Node<'_, '_>, out: &str) -> bool {
    out.ends_with('(')
        && node
            .next_sibling()
            .and_then(|sibling| sibling.text())
            .is_some_and(|text| text.starts_with(')'))
}

fn append_ext_link(node: Node<'_, '_>, out: &mut String) {
    const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

    let text = inline_text(node);
    let url = node
        .attribute((XLINK_NS, "href"))
        .or_else(|| node.attribute("href"))
        .map(str::trim)
        .filter(|value| !value.is_empty());

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

fn find_child<'a, 'input>(node: Node<'a, 'input>, name: &str) -> Option<Node<'a, 'input>> {
    node.children()
        .find(|child| child.is_element() && child.has_tag_name(name))
}

fn join_blocks(blocks: Vec<String>) -> String {
    blocks
        .into_iter()
        .map(|block| block.trim().to_string())
        .filter(|block| !block.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests;
