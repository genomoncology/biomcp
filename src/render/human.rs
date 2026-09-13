use unicode_normalization::UnicodeNormalization;

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

/// Unicode 16.0 `Default_Ignorable_Code_Point` scalar ranges, inclusive.
/// Generated from DerivedCoreProperties.txt (16.0.0, dated 2024-05-31):
/// every line whose property list includes Default_Ignorable_Code_Point.
const UNICODE_16_0_DEFAULT_IGNORABLE_RANGES: &[(u32, u32)] = &[
    (0x00AD, 0x00AD),
    (0x034F, 0x034F),
    (0x061C, 0x061C),
    (0x115F, 0x1160),
    (0x17B4, 0x17B5),
    (0x180B, 0x180D),
    (0x180E, 0x180E),
    (0x180F, 0x180F),
    (0x200B, 0x200F),
    (0x202A, 0x202E),
    (0x2060, 0x2064),
    (0x2065, 0x2065),
    (0x2066, 0x206F),
    (0x3164, 0x3164),
    (0xFE00, 0xFE0F),
    (0xFEFF, 0xFEFF),
    (0xFFA0, 0xFFA0),
    (0xFFF0, 0xFFF8),
    (0x1BCA0, 0x1BCA3),
    (0x1D173, 0x1D17A),
    (0xE0000, 0xE0000),
    (0xE0001, 0xE0001),
    (0xE0002, 0xE001F),
    (0xE0020, 0xE007F),
    (0xE0080, 0xE00FF),
    (0xE0100, 0xE01EF),
    (0xE01F0, 0xE0FFF),
];

/// Unicode 16.0 `General_Category=Format` (Cf) scalar ranges, inclusive.
/// Generated from DerivedGeneralCategory.txt (16.0.0, dated 2024-04-30):
/// every line whose extracted category is exactly Cf.
const UNICODE_16_0_FORMAT_RANGES: &[(u32, u32)] = &[
    (0x00AD, 0x00AD),
    (0x0600, 0x0605),
    (0x061C, 0x061C),
    (0x06DD, 0x06DD),
    (0x070F, 0x070F),
    (0x0890, 0x0891),
    (0x08E2, 0x08E2),
    (0x180E, 0x180E),
    (0x200B, 0x200F),
    (0x202A, 0x202E),
    (0x2060, 0x2064),
    (0x2066, 0x206F),
    (0xFEFF, 0xFEFF),
    (0xFFF9, 0xFFFB),
    (0x110BD, 0x110BD),
    (0x110CD, 0x110CD),
    (0x13430, 0x1343F),
    (0x1BCA0, 0x1BCA3),
    (0x1D173, 0x1D17A),
    (0xE0001, 0xE0001),
    (0xE0020, 0xE007F),
];

fn in_ranges(ranges: &[(u32, u32)], scalar: char) -> bool {
    let point = scalar as u32;
    ranges
        .binary_search_by(|&(low, high)| {
            if point < low {
                std::cmp::Ordering::Greater
            } else if point > high {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

fn is_unicode_16_0_default_ignorable(scalar: char) -> bool {
    in_ranges(UNICODE_16_0_DEFAULT_IGNORABLE_RANGES, scalar)
}

fn is_unicode_16_0_format(scalar: char) -> bool {
    in_ranges(UNICODE_16_0_FORMAT_RANGES, scalar)
}

/// Provider display text passes through this operation exactly once at the
/// Markdown leaf; ticket 1186 freezes the policy. No second escape surface.
// dead-code reason: Ticket 1186 freezes the Unicode-16 provider sanitizer; ticket 1142 is the first consumer.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn sanitize_provider_inline(value: &str) -> String {
    // 1. NFC normalization.
    let normalized: String = value.chars().nfc().collect();
    // 2. Line/paragraph separators become one space; each maximal
    //    Default_Ignorable_Code_Point or Cf run becomes one U+FFFD.
    let mut replaced = String::with_capacity(normalized.len());
    let mut pending_ignorable = false;
    for scalar in normalized.chars() {
        if scalar == '\u{2028}' || scalar == '\u{2029}' {
            replaced.push(' ');
            pending_ignorable = false;
        } else if is_unicode_16_0_default_ignorable(scalar) || is_unicode_16_0_format(scalar) {
            if !pending_ignorable {
                replaced.push('\u{FFFD}');
            }
            pending_ignorable = true;
        } else {
            replaced.push(scalar);
            pending_ignorable = false;
        }
    }
    // 3. Insert U+25CC before an orphan combining mark: one with no retained
    //    non-space scalar since the start or the last whitespace.
    let mut dotted = String::with_capacity(replaced.len());
    let mut has_base = false;
    for scalar in replaced.chars() {
        if scalar.is_whitespace() {
            dotted.push(scalar);
            has_base = false;
        } else if unicode_normalization::char::canonical_combining_class(scalar) != 0 {
            if !has_base {
                dotted.push('\u{25CC}');
            }
            dotted.push(scalar);
            has_base = true;
        } else {
            dotted.push(scalar);
            has_base = true;
        }
    }
    // 4. Existing ANSI/C0/C1 handling, then decimal HTML encoding for every
    //    ASCII scalar that is not a letter, digit, or space.
    encode_non_literal_ascii(&sanitize_inline(&dotted))
}

fn encode_non_literal_ascii(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for scalar in value.chars() {
        if !scalar.is_ascii() || scalar.is_ascii_alphanumeric() || scalar == ' ' {
            output.push(scalar);
        } else {
            output.push_str(&format!("&#{};", scalar as u32));
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{sanitize_document, sanitize_inline};

    use super::{encode_non_literal_ascii, sanitize_provider_inline};
    use unicode_normalization::char::canonical_combining_class;

    #[test]
    fn provider_sanitizer_control_fixtures_are_byte_exact() {
        let cases = [
            ("nfc composition", "e\u{0301}", "\u{00E9}"),
            (
                "orphan mark gets dotted circle",
                "\u{0301}A",
                "\u{25CC}\u{0301}A",
            ),
            (
                "line and paragraph separators",
                "A\u{2028}B\u{2029}C",
                "A B C",
            ),
            (
                "ignorable runs, separators, and ascii graphics",
                "A\u{202E}B\u{200B}\u{200D}C \u{1F469}\u{200D}\u{1F52C} <x&`$()>",
                "A\u{FFFD}B\u{FFFD}C \u{1F469}\u{FFFD}\u{1F52C} &#60;x&#38;&#96;&#36;&#40;&#41;&#62;",
            ),
        ];
        for (name, input, expected) in cases {
            let produced = sanitize_provider_inline(input);
            assert_eq!(produced.as_bytes(), expected.as_bytes(), "{name}");
        }
    }

    #[test]
    fn provider_sanitizer_collapses_maximal_ignorable_runs() {
        assert_eq!(
            sanitize_provider_inline("A\u{200B}\u{200D}\u{FEFF}\u{00AD}B"),
            "A\u{FFFD}B"
        );
        assert_eq!(
            sanitize_provider_inline("A\u{200B}B\u{200D}C"),
            "A\u{FFFD}B\u{FFFD}C"
        );
        assert_eq!(sanitize_provider_inline("A\u{2028}\u{2028}B"), "A  B");
        for isolated in [
            '\u{200E}', '\u{200F}', '\u{202A}', '\u{202E}', '\u{2066}', '\u{2069}',
        ] {
            assert_eq!(
                sanitize_provider_inline(&format!("A{isolated}B")),
                "A\u{FFFD}B"
            );
        }
        for character in ['\u{FE0E}', '\u{FE0F}', '\u{E0041}', '\u{E007F}'] {
            assert_eq!(
                sanitize_provider_inline(&format!("A{character}B")),
                "A\u{FFFD}B"
            );
        }
    }

    #[test]
    fn provider_sanitizer_inserts_dotted_circle_only_before_orphan_marks() {
        // A digit base has no canonical composition with any combining
        // mark, so the mark stays attached instead of composing.
        assert_eq!(sanitize_provider_inline("7\u{0301}"), "7\u{0301}");
        assert_eq!(sanitize_provider_inline(" \u{0301}"), " \u{25CC}\u{0301}");
        assert_eq!(
            sanitize_provider_inline("\u{0301}\u{0302}"),
            "\u{25CC}\u{0301}\u{0302}"
        );
        assert_eq!(
            sanitize_provider_inline("A\u{200B} \u{0301}"),
            "A\u{FFFD} \u{25CC}\u{0301}"
        );
        assert_eq!(canonical_combining_class('\u{093E}'), 0);
        assert_eq!(sanitize_provider_inline(" \u{093E}B"), " \u{093E}B");
    }

    #[test]
    fn provider_sanitizer_counts_a_replacement_as_retained_base() {
        // U+200B is replaced by U+FFFD, and the replacement is itself a
        // retained non-space scalar, so a following combining mark attaches
        // to the replacement and gets no dotted circle. This freezes that
        // step-3 interpretation; the implementation is unchanged.
        assert_eq!(
            sanitize_provider_inline("A\u{200B}\u{0301}"),
            "A\u{FFFD}\u{0301}"
        );
    }

    #[test]
    fn provider_sanitizer_encodes_every_non_literal_ascii_graphic() {
        for code in 0x21u32..=0x7e {
            let character = char::from_u32(code).unwrap();
            if character.is_ascii_alphanumeric() {
                continue;
            }
            let expected = format!("A&#{code};B");
            assert_eq!(
                sanitize_provider_inline(&format!("A{character}B")),
                expected,
                "U+{code:04X}"
            );
        }
        assert_eq!(
            sanitize_provider_inline("A B \u{00E9} \u{4E00}"),
            "A B \u{00E9} \u{4E00}"
        );
        assert_eq!(encode_non_literal_ascii("aZ09 \u{FFFD}"), "aZ09 \u{FFFD}");
    }

    #[test]
    fn provider_sanitizer_matches_unicode_16_boundaries() {
        // (scalar, is_replaced) pairs at every Unicode 16.0 DI/Cf range edge.
        const BOUNDARY_CASES: &[(u32, bool)] = &[
            (0x00AC, false),
            (0x00AD, true),
            (0x00AE, false),
            (0x034E, false),
            (0x034F, true),
            (0x0350, false),
            (0x061B, false),
            (0x061C, true),
            (0x061D, false),
            (0x115E, false),
            (0x115F, true),
            (0x1160, true),
            (0x1161, false),
            (0x17B3, false),
            (0x17B4, true),
            (0x17B5, true),
            (0x17B6, false),
            (0x180A, false),
            (0x180B, true),
            (0x180D, true),
            (0x180E, true),
            (0x180F, true),
            (0x1810, false),
            (0x200A, false),
            (0x200B, true),
            (0x200F, true),
            (0x2010, false),
            (0x2029, false),
            (0x202A, true),
            (0x202E, true),
            (0x202F, false),
            (0x205F, false),
            (0x2060, true),
            (0x2064, true),
            (0x2065, true),
            (0x2066, true),
            (0x206F, true),
            (0x2070, false),
            (0x3163, false),
            (0x3164, true),
            (0x3165, false),
            (0xFDFF, false),
            (0xFE00, true),
            (0xFE0F, true),
            (0xFE10, false),
            (0xFEFE, false),
            (0xFEFF, true),
            (0xFF00, false),
            (0xFF9F, false),
            (0xFFA0, true),
            (0xFFA1, false),
            (0xFFEF, false),
            (0xFFF0, true),
            (0xFFF8, true),
            (0xFFF9, true),
            (0x1BC9F, false),
            (0x1BCA0, true),
            (0x1BCA3, true),
            (0x1BCA4, false),
            (0x1D172, false),
            (0x1D173, true),
            (0x1D17A, true),
            (0x1D17B, false),
            (0xDFFFF, false),
            (0xE0000, true),
            (0xE0001, true),
            (0xE0002, true),
            (0xE001F, true),
            (0xE0020, true),
            (0xE007F, true),
            (0xE0080, true),
            (0xE00FF, true),
            (0xE0100, true),
            (0xE01EF, true),
            (0xE01F0, true),
            (0xE0FFF, true),
            (0xE1000, false),
            (0x05FF, false),
            (0x0600, true),
            (0x0605, true),
            (0x0606, false),
            (0x06DC, false),
            (0x06DD, true),
            (0x06DE, false),
            (0x070E, false),
            (0x070F, true),
            (0x0710, false),
            (0x088F, false),
            (0x0890, true),
            (0x0891, true),
            (0x0892, false),
            (0x08E1, false),
            (0x08E2, true),
            (0x08E3, false),
            (0xFFFB, true),
            (0xFFFC, false),
            (0x110BC, false),
            (0x110BD, true),
            (0x110BE, false),
            (0x110CC, false),
            (0x110CD, true),
            (0x110CE, false),
            (0x1342F, false),
            (0x13430, true),
            (0x1343F, true),
            (0x13440, false),
        ];
        for &(scalar, is_replaced) in BOUNDARY_CASES {
            let character = char::from_u32(scalar).expect("valid scalar");
            // A digit base has no canonical composition with any mark.
            let produced = sanitize_provider_inline(&format!("7{character}7"));
            if scalar == 0x2028 || scalar == 0x2029 {
                assert_eq!(produced, "7 7", "U+{scalar:04X} is a separator");
            } else if is_replaced {
                assert_eq!(produced, "7\u{FFFD}7", "U+{scalar:04X} must collapse");
            } else {
                assert_eq!(
                    produced,
                    format!("7{character}7"),
                    "U+{scalar:04X} must survive"
                );
            }
        }
    }

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
