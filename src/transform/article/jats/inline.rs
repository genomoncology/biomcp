//! Inline text, styling, links, and word boundaries for JATS rendering.

use roxmltree::{Node, NodeType};

use super::super::collapse_whitespace;

pub(super) fn inline_text(node: Node<'_, '_>) -> String {
    let mut buffer = InlineBuffer::default();
    match node.node_type() {
        NodeType::Text => buffer.text.push_str(node.text().unwrap_or_default()),
        _ => {
            for child in node.children() {
                append_inline_node(child, &mut buffer);
            }
        }
    }
    collapse_whitespace(&buffer.text)
}

/// Inline walk state: the rendered text plus whether the next word
/// character must attach to it. Word-joining styling elements (small caps,
/// styled content, monospace, footnote markers) set the flag at their
/// boundaries so fragments of one word stay one word instead of taking the
/// separating space distinct words get.
#[derive(Default)]
struct InlineBuffer {
    text: String,
    attach_next: bool,
}

impl InlineBuffer {
    pub(super) fn push_word_separated(&mut self, text: &str) {
        if self.attach_next {
            self.attach_next = false;
            self.text.push_str(text);
            return;
        }
        push_word_separated(&mut self.text, text);
    }
}

fn append_inline_node(node: Node<'_, '_>, buffer: &mut InlineBuffer) {
    match node.node_type() {
        NodeType::Root => {
            for child in node.children() {
                append_inline_node(child, buffer);
            }
        }
        NodeType::Element => {
            match node.tag_name().name() {
                "italic" => return append_wrapped_inline(node, "*", buffer),
                "bold" => return append_wrapped_inline(node, "**", buffer),
                "sup" => return append_wrapped_inline(node, "^", buffer),
                "sub" => return append_wrapped_inline(node, "~", buffer),
                "xref" => return append_xref(node, buffer),
                "ext-link" => return append_ext_link(node, buffer),
                "sc" | "styled-content" | "monospace" | "fn" => {
                    return append_word_joining_inline(node, buffer);
                }
                _ => {}
            }

            for child in node.children() {
                append_inline_node(child, buffer);
            }
        }
        NodeType::Text => append_inline_text(node.text().unwrap_or_default(), buffer),
        _ => {}
    }
}

/// Inline styling that wraps fragments of one word — small caps
/// (`T<sc>able</sc>`), styled content, monospace, and footnote markers
/// (`PD<sc>L1</sc>`). Both boundaries attach: the renderer must not split
/// these words the way it separates `surname`/`given-names` runs.
fn append_word_joining_inline(node: Node<'_, '_>, buffer: &mut InlineBuffer) {
    buffer.attach_next = true;
    for child in node.children() {
        append_inline_node(child, buffer);
    }
    buffer.attach_next |= buffer.text.ends_with(char::is_alphanumeric);
}

fn append_inline_text(text: &str, buffer: &mut InlineBuffer) {
    if (buffer.text.ends_with('^') || buffer.text.ends_with('~'))
        && let Some(rest) = text.strip_prefix('.')
        && rest.chars().next().is_some_and(char::is_alphanumeric)
    {
        buffer.attach_next = false;
        buffer.text.push_str(". ");
        buffer.text.push_str(rest);
        return;
    }
    buffer.push_word_separated(text);
}

/// Joins inline content at word boundaries: when the pending output and the new
/// text both touch the boundary with word characters, the rendered words need a
/// separating space even though the source kept the elements adjacent (for
/// example `<surname>Jemal</surname><given-names>A</given-names>`). Markup
/// markers such as `*`, `^`, `~`, and `[` keep attached runs attached, and the
/// word-joining styling elements route through `InlineBuffer` so fragments of
/// one word never take this space.
fn push_word_separated(out: &mut String, text: &str) {
    if text.starts_with(char::is_alphanumeric) && out.ends_with(char::is_alphanumeric) {
        out.push(' ');
    }
    out.push_str(text);
}

fn append_wrapped_inline(node: Node<'_, '_>, marker: &str, buffer: &mut InlineBuffer) {
    let text = inline_text(node);
    if text.is_empty() {
        return;
    }
    buffer.attach_next = false;
    buffer.text.push_str(marker);
    buffer.text.push_str(&text);
    buffer.text.push_str(marker);
}

fn append_xref(node: Node<'_, '_>, buffer: &mut InlineBuffer) {
    let text = inline_text(node);
    if text.is_empty() {
        return;
    }
    match node.attribute("ref-type") {
        Some("bibr") => {
            buffer.text.push('[');
            buffer.text.push_str(&text);
            buffer.text.push(']');
        }
        Some("fig") | Some("table") if xref_has_source_parentheses(node, &buffer.text) => {
            buffer.text.push_str(&text);
        }
        Some("fig") | Some("table") => {
            buffer.text.push('(');
            buffer.text.push_str(&text);
            buffer.text.push(')');
        }
        _ => buffer.push_word_separated(&text),
    }
}

fn xref_has_source_parentheses(node: Node<'_, '_>, out: &str) -> bool {
    out.ends_with('(')
        && node
            .next_sibling()
            .and_then(|sibling| sibling.text())
            .is_some_and(|text| text.starts_with(')'))
}

fn append_ext_link(node: Node<'_, '_>, buffer: &mut InlineBuffer) {
    const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

    let text = inline_text(node);
    let url = node
        .attribute((XLINK_NS, "href"))
        .or_else(|| node.attribute("href"))
        .map(str::trim)
        .filter(|value| !value.is_empty());

    match (text.is_empty(), url) {
        (false, Some(url)) => {
            buffer.text.push('[');
            buffer.text.push_str(&text);
            buffer.text.push_str("](");
            buffer.text.push_str(url);
            buffer.text.push(')');
        }
        (false, None) => buffer.push_word_separated(&text),
        (true, Some(url)) => buffer.push_word_separated(url),
        (true, None) => {}
    }
}

