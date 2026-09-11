fn truncate_utf8(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_owned();
    }

    let mut boundary = max_bytes;
    while boundary > 0 && !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    value[..boundary].trim_end().to_owned()
}

pub(crate) fn format_conditions(conditions: &[String]) -> String {
    const MAX_ITEMS: usize = 10;
    const MAX_BYTES: usize = 80;

    let cleaned = conditions
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    let joined = cleaned
        .iter()
        .take(MAX_ITEMS)
        .copied()
        .collect::<Vec<_>>()
        .join(", ");
    if cleaned.len() <= MAX_ITEMS && joined.len() <= MAX_BYTES {
        return joined;
    }

    let suffix = format!("… [abridged; {} conditions total]", cleaned.len());
    let prefix = truncate_utf8(&joined, MAX_BYTES.saturating_sub(suffix.len()));
    format!("{prefix}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::format_conditions;

    #[test]
    fn conditions_are_bounded_and_utf8_safe() {
        let conditions = (0..12)
            .map(|index| format!("Å-condition-{index}"))
            .collect::<Vec<_>>();
        let rendered = format_conditions(&conditions);
        assert!(rendered.contains("12 conditions total"));
        assert!(rendered.is_char_boundary(rendered.len()));
    }
}
