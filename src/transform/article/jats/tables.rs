//! JATS table conversion for ordinary grids and merged-cell source rows.

use roxmltree::Node;

use super::inline_text;
use crate::transform::article::collapse_whitespace;

pub(super) fn convert_regular_table(table: Node<'_, '_>) -> Option<String> {
    let mut rows = Vec::new();
    for row in table
        .descendants()
        .filter(|node| node.is_element() && node.has_tag_name("tr"))
    {
        let cells = row
            .children()
            .filter(|cell| cell.is_element() && matches!(cell.tag_name().name(), "th" | "td"))
            .map(|cell| {
                if cell.attribute("rowspan").is_some() || cell.attribute("colspan").is_some() {
                    None
                } else {
                    Some(normalize_cell(&inline_text(cell)))
                }
            })
            .collect::<Option<Vec<_>>>()?;
        if !cells.is_empty() && cells.iter().any(|cell| !cell.is_empty()) {
            rows.push(cells);
        }
    }

    let first = rows.first()?;
    let width = first.len();
    if width == 0 || rows.iter().any(|row| row.len() != width) {
        return None;
    }
    let mut lines = Vec::with_capacity(rows.len() + 1);
    lines.push(format!("| {} |", first.join(" | ")));
    lines.push(format!("| {} |", vec!["---"; width].join(" | ")));
    lines.extend(
        rows.iter()
            .skip(1)
            .map(|row| format!("| {} |", row.join(" | "))),
    );
    Some(lines.join("\n"))
}

fn complex_dimensions(table: Node<'_, '_>) -> Option<(usize, usize)> {
    let mut has_merged_cells = false;
    let mut row_count = 0;
    let mut max_cols = 0;
    for row in table
        .descendants()
        .filter(|node| node.is_element() && node.has_tag_name("tr"))
    {
        let mut cols = 0;
        for cell in row
            .children()
            .filter(|cell| cell.is_element() && matches!(cell.tag_name().name(), "th" | "td"))
        {
            has_merged_cells |=
                cell.attribute("rowspan").is_some() || cell.attribute("colspan").is_some();
            cols += cell
                .attribute("colspan")
                .and_then(|value| value.parse::<usize>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(1);
        }
        if cols > 0 {
            row_count += 1;
            max_cols = max_cols.max(cols);
        }
    }
    (has_merged_cells && row_count > 0 && max_cols > 0).then_some((row_count, max_cols))
}

pub(super) fn convert_complex_table(table: Node<'_, '_>) -> Option<String> {
    let (rows, cols) = complex_dimensions(table)?;
    let mut lines = vec![format!(
        "*[Complex table: {rows}×{cols}; merged-cell layout may be lossy. Raw source rows follow.]*"
    )];
    for (index, row) in table
        .descendants()
        .filter(|node| node.is_element() && node.has_tag_name("tr"))
        .enumerate()
    {
        let cells = row
            .children()
            .filter(|cell| cell.is_element() && matches!(cell.tag_name().name(), "th" | "td"))
            .map(|cell| {
                let mut text = normalize_cell(&inline_text(cell));
                for name in ["rowspan", "colspan"] {
                    if let Some(value) = cell.attribute(name) {
                        text.push_str(&format!(" [{name}={value}]"));
                    }
                }
                text
            })
            .collect::<Vec<_>>();
        if !cells.is_empty() {
            lines.push(format!("Row {}: {}", index + 1, cells.join(" | ")));
        }
    }
    Some(lines.join("\n"))
}

fn normalize_cell(value: &str) -> String {
    collapse_whitespace(value).replace('|', "\\|")
}
