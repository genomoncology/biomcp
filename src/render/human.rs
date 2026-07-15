#[derive(Clone, Copy)]
enum LayoutPolicy {
    Document,
    Inline,
}

pub(crate) fn sanitize_document(value: &str) -> String {
    sanitize(value, LayoutPolicy::Document)
}

pub(crate) fn sanitize_inline(value: &str) -> String {
    sanitize(value, LayoutPolicy::Inline)
}

fn sanitize(value: &str, policy: LayoutPolicy) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    let mut replacing_controls = false;

    while let Some(ch) = chars.next() {
        match ch {
            '\u{1b}' => match chars.peek().copied() {
                Some('[') => {
                    chars.next();
                    consume_csi(&mut chars);
                }
                Some(']') => {
                    chars.next();
                    consume_control_string(&mut chars, true);
                }
                Some('P' | 'X' | '^' | '_') => {
                    chars.next();
                    consume_control_string(&mut chars, false);
                }
                Some(next) if ('0'..='~').contains(&next) => {
                    chars.next();
                }
                _ => replace_control(&mut output, &mut replacing_controls),
            },
            '\u{9b}' => consume_csi(&mut chars),
            '\u{9d}' => consume_control_string(&mut chars, true),
            '\u{90}' | '\u{98}' | '\u{9e}' | '\u{9f}' => {
                consume_control_string(&mut chars, false);
            }
            '\u{061c}'
            | '\u{200e}'
            | '\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2066}'..='\u{2069}' => {}
            '\n' | '\t' if matches!(policy, LayoutPolicy::Document) => {
                output.push(ch);
                replacing_controls = false;
            }
            '\0'..='\u{1f}' | '\u{7f}'..='\u{9f}' => {
                replace_control(&mut output, &mut replacing_controls);
            }
            _ => {
                output.push(ch);
                replacing_controls = false;
            }
        }
    }

    output
}

fn replace_control(output: &mut String, replacing_controls: &mut bool) {
    if !*replacing_controls && !output.ends_with(' ') {
        output.push(' ');
    }
    *replacing_controls = true;
}

fn consume_csi(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    for ch in chars.by_ref() {
        if ('@'..='~').contains(&ch) {
            break;
        }
    }
}

fn consume_control_string(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    bell_terminates: bool,
) {
    while let Some(ch) = chars.next() {
        if ch == '\u{9c}' || (bell_terminates && ch == '\u{7}') {
            break;
        }
        if ch == '\u{1b}' && matches!(chars.peek(), Some('\\')) {
            chars.next();
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{sanitize_document, sanitize_inline};

    #[test]
    fn sanitizes_terminal_sequence_families() {
        let cases = [
            ("CSI", "A\u{1b}[31mB\u{1b}[0mC", "ABC"),
            ("8-bit CSI", "A\u{9b}31mB", "AB"),
            (
                "OSC BEL",
                "A\u{1b}]8;;https://bad\u{7}label\u{1b}]8;;\u{7}B",
                "AlabelB",
            ),
            (
                "OSC ST",
                "A\u{9d}8;;https://bad\u{1b}\\label\u{9d}8;;\u{9c}B",
                "AlabelB",
            ),
            ("DCS", "A\u{1b}Ppayload\u{1b}\\B", "AB"),
            ("SOS", "A\u{98}payload\u{9c}B", "AB"),
            ("PM", "A\u{9e}payload\u{9c}B", "AB"),
            ("APC", "A\u{9f}payload\u{9c}B", "AB"),
            ("two-byte ESC", "A\u{1b}7B", "AB"),
            ("unterminated CSI", "A\u{1b}[31", "A"),
            ("unterminated OSC", "A\u{1b}]8;;https://bad", "A"),
        ];

        for (name, input, expected) in cases {
            assert_eq!(sanitize_inline(input), expected, "{name}");
        }
    }

    #[test]
    fn applies_inline_and_document_control_policies() {
        let input = "A\0\u{1}\u{7}B\rC\nD\tE\u{7f}\u{80}F\u{1b}";
        assert_eq!(sanitize_inline(input), "A B C D E F ");
        assert_eq!(sanitize_document(input), "A B C\nD\tE F ");
    }

    #[test]
    fn removes_bidi_controls_and_preserves_biomedical_unicode() {
        let input = "BRAF\u{061c}\u{200e}\u{202e}\u{2067} α-synuclein e\u{301} 👩\u{200d}🔬";
        let expected = "BRAF α-synuclein e\u{301} 👩\u{200d}🔬";
        assert_eq!(sanitize_inline(input), expected);
        assert_eq!(sanitize_document(input), expected);
    }

    #[test]
    fn is_idempotent_without_collapsing_authored_spaces() {
        for sanitize in [sanitize_inline as fn(&str) -> String, sanitize_document] {
            let once = sanitize("A  B \0 C");
            assert_eq!(once, "A  B  C");
            assert_eq!(sanitize(&once), once);
        }
    }
}
