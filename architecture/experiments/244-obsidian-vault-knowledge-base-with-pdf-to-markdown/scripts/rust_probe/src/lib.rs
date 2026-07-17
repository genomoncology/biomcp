use std::fs;
use std::path::Path;
use std::time::Instant;

use anyhow::{Context, Result};
use regex::Regex;
use roxmltree::{Document, Node, NodeType};
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PdfEngine {
    Unpdf,
    PdfOxide,
}

#[derive(Serialize)]
pub struct ProbeReport {
    pub kind: &'static str,
    pub engine: Option<&'static str>,
    pub input: String,
    pub output: String,
    pub elapsed_ms: u128,
    pub success: bool,
    pub error: Option<String>,
    pub metrics: Value,
}

pub fn run_jats_file(input: &Path, output: &Path, started: Instant) -> Result<ProbeReport> {
    let xml = fs::read_to_string(input).with_context(|| format!("read {}", input.display()))?;
    let result = extract_jats_markdown(&xml);

    match result {
        Ok((markdown, metrics)) => {
            write_output(output, &markdown)?;
            Ok(ProbeReport {
                kind: "jats",
                engine: Some("roxmltree-minimal"),
                input: input.display().to_string(),
                output: output.display().to_string(),
                elapsed_ms: started.elapsed().as_millis(),
                success: true,
                error: None,
                metrics,
            })
        }
        Err(error) => Ok(ProbeReport {
            kind: "jats",
            engine: Some("roxmltree-minimal"),
            input: input.display().to_string(),
            output: output.display().to_string(),
            elapsed_ms: started.elapsed().as_millis(),
            success: false,
            error: Some(error.to_string()),
            metrics: json!({}),
        }),
    }
}

pub fn run_html_file(
    input: &Path,
    base_url: &str,
    output: &Path,
    started: Instant,
) -> Result<ProbeReport> {
    let html = fs::read_to_string(input).with_context(|| format!("read {}", input.display()))?;
    let (markdown, metrics) = extract_html_markdown(&html, base_url)?;
    write_output(output, &markdown)?;

    Ok(ProbeReport {
        kind: "html",
        engine: Some("readability-rust+html2md"),
        input: input.display().to_string(),
        output: output.display().to_string(),
        elapsed_ms: started.elapsed().as_millis(),
        success: true,
        error: None,
        metrics,
    })
}

pub fn run_pdf_file(
    engine: PdfEngine,
    input: &Path,
    output: &Path,
    page_limit: u32,
    started: Instant,
) -> Result<ProbeReport> {
    let result = extract_pdf_markdown(input, engine, page_limit);

    match result {
        Ok((markdown, mut metrics, engine_name)) => {
            write_output(output, &markdown)?;
            let quality = score_pdf(&metrics);
            metrics["quality"] = json!(quality);
            Ok(ProbeReport {
                kind: "pdf",
                engine: Some(engine_name),
                input: input.display().to_string(),
                output: output.display().to_string(),
                elapsed_ms: started.elapsed().as_millis(),
                success: true,
                error: None,
                metrics,
            })
        }
        Err(error) => Ok(ProbeReport {
            kind: "pdf",
            engine: Some(match engine {
                PdfEngine::Unpdf => "unpdf",
                PdfEngine::PdfOxide => "pdf_oxide",
            }),
            input: input.display().to_string(),
            output: output.display().to_string(),
            elapsed_ms: started.elapsed().as_millis(),
            success: false,
            error: Some(error.to_string()),
            metrics: json!({ "page_limit": page_limit }),
        }),
    }
}

pub fn extract_html_markdown(html: &str, base_url: &str) -> Result<(String, Value)> {
    let mut parser = readability_rust::Readability::new_with_base_uri(html, base_url, None)
        .context("initialize readability parser")?;

    let (article_title, content_html, extraction_mode, extraction_error) = match parser.parse() {
        Some(article) => {
            let title = article.title.clone();
            if let Some(content) = article.content.filter(|value| !value.trim().is_empty()) {
                (title, content, "readability", None)
            } else {
                (
                    title,
                    html.to_string(),
                    "raw-html-fallback",
                    Some("readability returned no content".to_string()),
                )
            }
        }
        None => (
            None,
            html.to_string(),
            "raw-html-fallback",
            Some("readability returned no article".to_string()),
        ),
    };

    let markdown = html2md::parse_html(&content_html);
    let text = if markdown.trim().is_empty() {
        content_html.as_str()
    } else {
        markdown.as_str()
    };
    let mut metrics = markdown_metrics(text);
    metrics["base_url"] = json!(base_url);
    metrics["article_title"] = json!(article_title);
    metrics["extraction_mode"] = json!(extraction_mode);
    metrics["extraction_error"] = json!(extraction_error);
    metrics["quality_score"] = json!(score_html(&metrics));
    Ok((markdown, metrics))
}

pub fn extract_pdf_markdown(
    input: &Path,
    engine: PdfEngine,
    page_limit: u32,
) -> Result<(String, Value, &'static str)> {
    match engine {
        PdfEngine::Unpdf => run_unpdf(input, page_limit),
        PdfEngine::PdfOxide => run_pdf_oxide(input, page_limit),
    }
}

fn run_unpdf(input: &Path, page_limit: u32) -> Result<(String, Value, &'static str)> {
    let doc = unpdf::parse_file(input).context("unpdf parse_file")?;
    let page_count = doc.page_count();
    let page_end = page_count.min(page_limit);
    let options = unpdf::render::RenderOptions::default()
        .with_heading_analysis()
        .with_page_range(1..=page_end.max(1));
    let markdown = unpdf::render::to_markdown(&doc, &options).context("unpdf to_markdown")?;
    let mut metrics = markdown_metrics(&markdown);
    metrics["page_count"] = json!(page_count);
    metrics["pages_processed"] = json!(page_end);
    metrics["page_limit"] = json!(page_limit);
    Ok((markdown, metrics, "unpdf"))
}

fn run_pdf_oxide(input: &Path, page_limit: u32) -> Result<(String, Value, &'static str)> {
    let mut doc = pdf_oxide::PdfDocument::open(input).context("pdf_oxide open")?;
    let page_count = doc.page_count().context("pdf_oxide page_count")?;
    let options = pdf_oxide::converters::ConversionOptions::default();
    let pages_to_process = page_count.min(page_limit as usize);
    let mut parts = Vec::new();
    for page in 0..pages_to_process {
        let page_markdown = doc
            .to_markdown(page, &options)
            .with_context(|| format!("pdf_oxide to_markdown page {page}"))?;
        if !page_markdown.trim().is_empty() {
            parts.push(format!("<!-- page {} -->\n\n{}", page + 1, page_markdown));
        }
    }
    let markdown = parts.join("\n\n");
    let mut metrics = markdown_metrics(&markdown);
    metrics["page_count"] = json!(page_count);
    metrics["pages_processed"] = json!(pages_to_process);
    metrics["page_limit"] = json!(page_limit);
    Ok((markdown, metrics, "pdf_oxide"))
}

pub fn write_output(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content).with_context(|| format!("write {}", path.display()))
}

pub fn extract_jats_markdown(xml: &str) -> Result<(String, Value)> {
    let sanitized = strip_doctype_declaration(xml);
    let doc = Document::parse(&sanitized).context("parse JATS XML")?;
    let article = doc
        .descendants()
        .find(|node| node.is_element() && node.has_tag_name("article"))
        .context("no <article> element")?;

    let mut blocks = Vec::new();
    let title = article
        .descendants()
        .find(|node| node.is_element() && node.has_tag_name("article-title"))
        .map(inline_text)
        .filter(|value| !value.is_empty());
    if let Some(title) = &title {
        blocks.push(format!("# {title}"));
    }

    if let Some(abstract_node) = article
        .descendants()
        .find(|node| node.is_element() && node.has_tag_name("abstract"))
    {
        blocks.push("## Abstract".to_string());
        append_content_blocks(abstract_node, 2, &mut blocks);
    }

    if let Some(body) = article
        .children()
        .find(|node| node.is_element() && node.has_tag_name("body"))
    {
        append_content_blocks(body, 2, &mut blocks);
    }

    if let Some(references) = render_references(article) {
        blocks.push(references);
    }

    let markdown = join_blocks(blocks);
    let mut metrics = markdown_metrics(&markdown);
    metrics["article_title"] = json!(title);
    metrics["section_count_xml"] = json!(count_desc(article, "sec"));
    metrics["paragraph_count_xml"] = json!(count_desc(article, "p"));
    metrics["figure_count_xml"] = json!(count_desc(article, "fig"));
    metrics["table_wrap_count_xml"] = json!(count_desc(article, "table-wrap"));
    metrics["reference_count_xml"] = json!(count_desc(article, "ref"));
    metrics["has_abstract"] = json!(count_desc(article, "abstract") > 0);
    metrics["quality_score"] = json!(score_jats(&metrics));
    Ok((markdown, metrics))
}

fn append_content_blocks(node: Node<'_, '_>, heading_level: usize, blocks: &mut Vec<String>) {
    for child in node.children().filter(|child| child.is_element()) {
        match child.tag_name().name() {
            "title" | "label" => {}
            "p" => {
                let text = inline_text(child);
                if !text.is_empty() {
                    blocks.push(text);
                }
            }
            "sec" => convert_section(child, heading_level, blocks),
            "fig" => {
                if let Some(figure) = convert_figure(child) {
                    blocks.push(figure);
                }
            }
            "table-wrap" => blocks.extend(convert_table_wrap(child)),
            "list" => {
                if let Some(list) = convert_list(child) {
                    blocks.push(list);
                }
            }
            _ => {}
        }
    }
}

fn convert_section(section: Node<'_, '_>, heading_level: usize, blocks: &mut Vec<String>) {
    if let Some(title) = find_child(section, "title")
        .map(inline_text)
        .filter(|value| !value.is_empty())
    {
        let level = heading_level.clamp(2, 6);
        blocks.push(format!("{} {}", "#".repeat(level), title));
    }
    append_content_blocks(section, heading_level + 1, blocks);
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
    }
    blocks
}

fn convert_regular_table(table: Node<'_, '_>) -> Option<String> {
    let mut rows = Vec::new();
    for row in table
        .descendants()
        .filter(|node| node.is_element() && node.has_tag_name("tr"))
    {
        let mut cells = Vec::new();
        for cell in row.children().filter(|child| child.is_element()) {
            if !matches!(cell.tag_name().name(), "th" | "td") {
                continue;
            }
            if cell.attribute("rowspan").is_some() || cell.attribute("colspan").is_some() {
                return None;
            }
            cells.push(inline_text(cell).replace('|', "\\|"));
        }
        if !cells.is_empty() && cells.iter().any(|cell| !cell.trim().is_empty()) {
            rows.push(cells);
        }
    }

    let first = rows.first()?;
    let width = first.len();
    if width == 0 || rows.iter().any(|row| row.len() != width) {
        return None;
    }
    let mut lines = Vec::new();
    lines.push(format!("| {} |", first.join(" | ")));
    lines.push(format!("| {} |", vec!["---"; width].join(" | ")));
    for row in rows.iter().skip(1) {
        lines.push(format!("| {} |", row.join(" | ")));
    }
    Some(lines.join("\n"))
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
        let text = inline_text(item);
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

fn caption_text(node: Node<'_, '_>) -> Option<String> {
    let text = inline_text(node);
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn render_references(article: Node<'_, '_>) -> Option<String> {
    let refs = article
        .descendants()
        .filter(|node| node.is_element() && node.has_tag_name("ref"))
        .collect::<Vec<_>>();
    if refs.is_empty() {
        return None;
    }
    let items = refs
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let text = inline_text(*node);
            if text.is_empty() {
                format!("{}.", index + 1)
            } else {
                format!("{}. {text}", index + 1)
            }
        })
        .collect::<Vec<_>>();
    Some(format!("## References\n\n{}", items.join("\n")))
}

fn inline_text(node: Node<'_, '_>) -> String {
    let mut out = String::new();
    append_inline_node(node, &mut out);
    collapse_whitespace(&out)
}

fn append_inline_node(node: Node<'_, '_>, out: &mut String) {
    match node.node_type() {
        NodeType::Root => {
            for child in node.children() {
                append_inline_node(child, out);
            }
        }
        NodeType::Element => match node.tag_name().name() {
            "italic" => append_wrapped_inline(node, "*", out),
            "bold" => append_wrapped_inline(node, "**", out),
            "sup" => append_wrapped_inline(node, "^", out),
            "sub" => append_wrapped_inline(node, "~", out),
            "xref" => {
                let text = inline_text_children(node);
                if !text.is_empty() {
                    out.push('[');
                    out.push_str(&text);
                    out.push(']');
                }
            }
            "ext-link" => append_ext_link(node, out),
            _ => {
                for child in node.children() {
                    append_inline_node(child, out);
                }
            }
        },
        NodeType::Text => out.push_str(node.text().unwrap_or_default()),
        _ => {}
    }
}

fn inline_text_children(node: Node<'_, '_>) -> String {
    let mut out = String::new();
    for child in node.children() {
        append_inline_node(child, &mut out);
    }
    collapse_whitespace(&out)
}

fn append_wrapped_inline(node: Node<'_, '_>, marker: &str, out: &mut String) {
    let text = inline_text_children(node);
    if text.is_empty() {
        return;
    }
    out.push_str(marker);
    out.push_str(&text);
    out.push_str(marker);
}

fn append_ext_link(node: Node<'_, '_>, out: &mut String) {
    const XLINK_NS: &str = "http://www.w3.org/1999/xlink";
    let text = inline_text_children(node);
    let url = node
        .attribute((XLINK_NS, "href"))
        .or_else(|| node.attribute("href"))
        .map(str::trim)
        .filter(|value| !value.is_empty());
    match (text.is_empty(), url) {
        (false, Some(url)) => out.push_str(&format!("[{text}]({url})")),
        (false, None) => out.push_str(&text),
        (true, Some(url)) => out.push_str(url),
        (true, None) => {}
    }
}

fn find_child<'a, 'input>(node: Node<'a, 'input>, name: &str) -> Option<Node<'a, 'input>> {
    node.children()
        .find(|child| child.is_element() && child.has_tag_name(name))
}

fn count_desc(node: Node<'_, '_>, tag: &str) -> usize {
    node.descendants()
        .filter(|child| child.is_element() && child.has_tag_name(tag))
        .count()
}

fn join_blocks(blocks: Vec<String>) -> String {
    blocks
        .into_iter()
        .map(|block| block.trim().to_string())
        .filter(|block| !block.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn strip_doctype_declaration(xml: &str) -> String {
    let re = Regex::new(r#"(?is)<!DOCTYPE[^>]*>"#).expect("valid doctype regex");
    re.replace(xml, "").to_string()
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn markdown_metrics(markdown: &str) -> Value {
    let lower = markdown.to_ascii_lowercase();
    let lines = markdown.lines().collect::<Vec<_>>();
    let heading_count = lines
        .iter()
        .filter(|line| line.trim_start().starts_with('#'))
        .count();
    let table_row_count = lines
        .iter()
        .filter(|line| line.trim_start().starts_with('|') && line.contains('|'))
        .count();
    let image_count = markdown.matches("![").count();
    let link_count = markdown.matches("](").count();
    let word_count = markdown.split_whitespace().count();
    let nonempty_lines = lines.iter().filter(|line| !line.trim().is_empty()).count();
    let average_line_len = if nonempty_lines == 0 {
        0.0
    } else {
        lines.iter().map(|line| line.trim().len()).sum::<usize>() as f64 / nonempty_lines as f64
    };
    json!({
        "markdown_bytes": markdown.len(),
        "word_count": word_count,
        "heading_count": heading_count,
        "table_row_count": table_row_count,
        "image_ref_count": image_count,
        "link_count": link_count,
        "nonempty_line_count": nonempty_lines,
        "average_line_len": average_line_len,
        "has_reference_signal": lower.contains("references") || lower.contains("bibliography"),
        "has_figure_signal": lower.contains("figure") || lower.contains("fig."),
        "has_table_signal": lower.contains("table") || table_row_count > 0,
        "has_doi_signal": lower.contains("doi") || lower.contains("10."),
        "replacement_char_count": markdown.matches('\u{fffd}').count(),
    })
}

pub fn score_jats(metrics: &Value) -> u8 {
    let mut score = 1;
    if metrics["markdown_bytes"].as_u64().unwrap_or(0) > 10_000 {
        score += 1;
    }
    if metrics["section_count_xml"].as_u64().unwrap_or(0) > 3
        && metrics["heading_count"].as_u64().unwrap_or(0) > 3
    {
        score += 1;
    }
    if metrics["figure_count_xml"].as_u64().unwrap_or(0) > 0
        || metrics["table_wrap_count_xml"].as_u64().unwrap_or(0) > 0
    {
        score += 1;
    }
    if metrics["reference_count_xml"].as_u64().unwrap_or(0) > 5
        && metrics["has_reference_signal"].as_bool().unwrap_or(false)
    {
        score += 1;
    }
    score.min(5)
}

pub fn score_html(metrics: &Value) -> u8 {
    let mut score = 1;
    if metrics["word_count"].as_u64().unwrap_or(0) > 500 {
        score += 1;
    }
    if metrics["heading_count"].as_u64().unwrap_or(0) > 0 {
        score += 1;
    }
    if metrics["link_count"].as_u64().unwrap_or(0) > 2 {
        score += 1;
    }
    if metrics["extraction_mode"].as_str() == Some("readability")
        && metrics["markdown_bytes"].as_u64().unwrap_or(0) > 2_000
    {
        score += 1;
    }
    score.min(5)
}

pub fn score_pdf(metrics: &Value) -> Value {
    let heading = match metrics["heading_count"].as_u64().unwrap_or(0) {
        5.. => 5,
        2..=4 => 4,
        1 => 2,
        _ => 1,
    };
    let table = if metrics["table_row_count"].as_u64().unwrap_or(0) >= 4 {
        5
    } else if metrics["has_table_signal"].as_bool().unwrap_or(false) {
        3
    } else {
        1
    };
    let figure = if metrics["image_ref_count"].as_u64().unwrap_or(0) > 0 {
        5
    } else if metrics["has_figure_signal"].as_bool().unwrap_or(false) {
        3
    } else {
        1
    };
    let references = if metrics["has_reference_signal"].as_bool().unwrap_or(false)
        && metrics["has_doi_signal"].as_bool().unwrap_or(false)
    {
        5
    } else if metrics["has_reference_signal"].as_bool().unwrap_or(false) {
        3
    } else {
        1
    };
    let readability = if metrics["word_count"].as_u64().unwrap_or(0) > 1_500
        && metrics["replacement_char_count"].as_u64().unwrap_or(0) == 0
    {
        5
    } else if metrics["word_count"].as_u64().unwrap_or(0) > 500 {
        3
    } else {
        1
    };
    let overall = ((heading + table + figure + references + readability) as f64 / 5.0).round();
    json!({
        "heading_detection": heading,
        "table_preservation": table,
        "figure_handling": figure,
        "reference_extraction": references,
        "overall_readability": readability,
        "overall_score": overall as u8,
    })
}
