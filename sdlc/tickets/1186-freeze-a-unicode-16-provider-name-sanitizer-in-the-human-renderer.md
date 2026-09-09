---
flow: build
priority: 4
deps: []
---

# Freeze a Unicode-16 provider-name sanitizer in the human renderer

## Goal

Extend the established `src/render/human.rs` sanitizer with
`sanitize_provider_inline` beside `sanitize_inline`; do not build a second
provider-local escape policy. Provider display text passes through the
operation exactly once at the Markdown leaf. Ticket 1142 is the first
consumer, applying it to ORCID provider display fields.

## Algorithm

`sanitize_provider_inline` performs, in order:

1. NFC normalization with direct dependency `unicode-normalization = "0.1.25"`
   (already locked at that version; the package path count does not change).
2. Replace each U+2028 LINE SEPARATOR or U+2029 PARAGRAPH SEPARATOR with one
   ASCII space. Replace each maximal run in the Unicode 16.0
   `Default_Ignorable_Code_Point` property **or** General_Category `Cf` with one
   visible U+FFFD, using checked-in scalar-range match tables named with that
   Unicode version; this includes bidi controls, soft hyphen, variation/tag
   selectors, joiners, and zero-width spaces.
3. Before a remaining scalar whose Unicode canonical combining class is
   nonzero, insert U+25CC DOTTED CIRCLE when no retained non-space scalar has
   occurred since the start or last whitespace. Combining marks following a
   base remain attached; NFC-composable sequences are already composed.
4. Pass the result through existing `sanitize_inline` for ANSI/C0/C1 handling.
   Then preserve every non-ASCII scalar plus ASCII letters, digits, and spaces,
   while encoding every other ASCII graphic as decimal HTML
   `&#<codepoint>;` with no leading zeroes.

## Fixtures

Exact sanitizer controls are:

```text
provider input:  "e\u{0301}" | "\u{0301}A" | "A\u{2028}B\u{2029}C"
Markdown output: "é" | "◌́A" | "A B C"

provider input:  "A\u{202E}B\u{200B}\u{200D}C 👩\u{200D}🔬 <x&`$()>"
Markdown output: "A�B�C 👩�🔬 &#60;x&#38;&#96;&#36;&#40;&#41;&#62;"
```

The second output's leading mark includes an inserted U+25CC. Fixture
assertions compare UTF-8 bytes for isolated/attached combining marks,
consecutive line separators, every table boundary, bidi
isolates/overrides, zero-width/joiner/variation/tag characters, and NFC
composition.

## Dependency

Add `unicode-normalization = "0.1.25"` as a direct dependency. It is already
locked at that version, so the lock resolution and the package path count do
not change.

## Acceptance

`sanitize_provider_inline` exists in `src/render/human.rs` beside
`sanitize_inline`. Shared human-renderer tables freeze the Unicode-16
predicate boundaries and every sanitizer byte example above without changing
JSON values: both control fixtures byte-for-byte, plus predicate boundary tests
for `Default_Ignorable_Code_Point`, General_Category `Cf`, and canonical
combining class behavior. No other render behavior changes. `make lint`,
`make test`, and `make spec` pass, and the package path count stays exactly
1,300.

## Boundaries

No ORCID knowledge. No CLI or MCP surface change. JSON values are untouched;
only Markdown leaf output changes.
