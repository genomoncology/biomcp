/// Escapes a user-provided value for Lucene-like query syntaxes.
///
/// This is intentionally conservative: all Lucene special characters are escaped
/// so user input cannot accidentally change query semantics.
pub(crate) fn escape_lucene_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' | '+' | '-' | '!' | '(' | ')' | '{' | '}' | '[' | ']' | '^' | '"' | '~' | '*'
            | '?' | ':' | '/' | '&' | '|' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::escape_lucene_value;

    #[test]
    fn escapes_lucene_special_characters() {
        let escaped = escape_lucene_value(r#"BRAF:V600E (class-1) "quoted"\path"#);
        assert_eq!(escaped, r#"BRAF\:V600E \(class\-1\) \"quoted\"\\path"#);
    }
}
