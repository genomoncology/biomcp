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
1,305 (1,300 plus ticket 1145's authorized five modules).

## Boundaries

No ORCID knowledge. No CLI or MCP surface change. JSON values are untouched;
only Markdown leaf output changes.

## Review Record

- Code review: ACCEPT 2026-09-13 (one commit, 119fdd0b). The mixed-table
  judgment was confirmed: pinning the crate by locked version while freezing
  the DI/Cf predicates in checked-in Unicode 16 tables is the ticket's own
  wording, and no fixture depends on the crate's Unicode 17 NFC differences.
- Remediation: three P2 fixes recorded. (a) Twelve negative boundary pairs
  added immediately after each single-point range's upper edge; (b) the
  step-3 interpretation that an inserted U+FFFD counts as a retained base is
  now frozen by a dedicated test ("A\u200B\u0301" yields "A\uFFFD\u0301", no
  dotted circle) with the implementation unchanged; (c) the stale package
  path count corrected to the enforced 1,305.

- Full gates (final): merged as PR #270. At a186340f: lint OK, Rust 3525/3525,
  spec OK; pytest failures exactly the eight documented contention tests.
  Code review ACCEPT with the Unicode 17 NFC / Unicode 16 predicate split
  confirmed as the ticket's intended reading; remediation added twelve
  boundary negatives and froze the replacement-character-as-base rule.
