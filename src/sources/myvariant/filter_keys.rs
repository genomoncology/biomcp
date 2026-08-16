pub(super) fn normalize_filter_key(value: &str) -> String {
    let mut out = String::new();
    let mut prev_sep = false;
    for ch in value.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_sep = false;
            continue;
        }
        if matches!(ch, ' ' | ',' | '-' | '_') && !prev_sep {
            out.push('_');
            prev_sep = true;
        } else if !matches!(ch, ' ' | ',' | '-' | '_') {
            out.push(ch);
            prev_sep = false;
        }
    }
    out.trim_matches('_').to_string()
}
