use super::*;

#[test]
fn quote_arg_wraps_whitespace_and_escapes_quotes() {
    assert_eq!(quote_arg("BRAF"), "BRAF");
    assert_eq!(quote_arg("BRAF V600E"), "\"BRAF V600E\"");
    assert_eq!(quote_arg("BRAF \"V600E\""), "\"BRAF \\\"V600E\\\"\"");
}

#[test]
fn markdown_cells_keep_layout_rules_while_removing_terminal_controls() {
    assert_eq!(
        markdown_cell("Alpha\u{7}Beta | γ\nDelta\tEpsilon"),
        "Alpha Beta \\| γ Delta Epsilon"
    );
    assert_eq!(markdown_cell("\u{1b}[31m\u{202e}"), "-");
}

#[test]
fn discover_try_line_quotes_shell_sensitive_queries() {
    assert_eq!(
        discover_try_line("ERBB1\"alias", "resolve abbreviations and synonyms"),
        "Try: biomcp discover \"ERBB1\\\"alias\"   - resolve abbreviations and synonyms"
    );
    assert_eq!(
        discover_try_line("BRAF $(touch marker)", "resolve abbreviations and synonyms"),
        "Try: biomcp discover \"BRAF \\$(touch marker)\"   - resolve abbreviations and synonyms"
    );
    assert_eq!(
        discover_try_line("BRAF V600E", "resolve abbreviations and synonyms"),
        "Try: biomcp discover \"BRAF V600E\"   - resolve abbreviations and synonyms"
    );
}
