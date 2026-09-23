---
base: f11d4361
head: abade1b4
---

Made the FDA label boxed and legacy warnings reach the reader, from the
issue filed 2026-09-23.

`boxed_warning` is now a field of the label model (`DrugLabel` and the
`Drug` summary), always rendered when present: a leading `### Boxed
Warning` block in raw and summary modes and a `### FDA boxed warning`
subsection ahead of the ordinary warnings in the safety block. The
non-boxed warnings text falls back from `warnings_and_cautions` to the
legacy `warnings` field, so older-format labels report again. The raw-mode
emptiness early-return includes the boxed field, and the section-outcome
contributor check counts a boxed-only label as sourced, closing the exact
reported symptom: a fetched label whose only content was a boxed warning
rendered as "No data found".

Seven tests cover the three fixture shapes (both fields, legacy-only,
boxed-only), the fallback chain, summary-mode population, and render order
in raw, summary, and safety modes; the pre-existing pinned unit survives.
The safety-path boxed text truncates at the same 2,000-character cap as
the inline path, via the constant hoisted to module scope. The
source-size ratchet records authorized increases for the four touched
files above their baselines (`tools/rust-source-size-inventory.json`,
ticket 1232).

Evidence: yellow gate at abade1b4 — `make lint` OK, `make spec` OK;
`make test` green, with the first run failing one unrelated GenCC
subprocess-lease test that passes three-for-three in isolation and matches
the documented load-flake class (`entities::gene::gencc`, not touched by
this branch). Rust suite 3,776/3,776 on the passing run. Two intermediate
gate failures were fixed on the branch: an out-of-scope constant
reference in 88cdca4b, and rustfmt plus the ratchet baselines in
abade1b4.

Residual: a boxed-only label still renders the section fallback line
under "### FDA label warnings" while the boxed block above it carries
content; the design accepted this as honest provenance for the genuinely
empty warnings field.
