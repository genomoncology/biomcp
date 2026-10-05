# 1300 — return whole drug label sections and say why a label is missing

Filed 2026-10-05 by the BioMCP 0.9 lead, from the KB lead's EGFR-wiki message of 2026-10-04 (asks 1 and 3), reproduced 2026-10-05.

Status: OPEN.

## Outcome

`get drug NAME label -j` returns whole label sections in JSON, and when a drug has no label it says why in the section outcome, in line with the four section states. An agent never sees a silent `label: null`.

## Evidence

- Starts from: Reproduced on main a877443f on 2026-10-05. `get drug osimertinib label -j` returns `label: null` with no section outcome at all; the KB lead saw the same for mobocertinib. `get drug gefitinib label -j` returns a label object. Direct openFDA calls show why: `drug/label.json?search=openfda.generic_name:"osimertinib"` returns "No matches found", and neither `openfda.brand_name:"TAGRISSO"` nor `products.brand_name:"TAGRISSO"` match — osimertinib's current SPL label carries no populated `openfda.generic_name` or `brand_name`, while a full-text `"TAGRISSO"` query finds it. BioMCP's label lookup queries only those three narrow fields (`src/entities/drug/get.rs:233, 237` through the openFDA client), so sparse-metadata labels are silently missing.
- Keeps: The label shapes that work today, the DailyMed full-label URL, and every drug that already resolves.
- Changes: (1) Widen the label query with a fallback: when the field-scoped query returns no match, fall back to a full-text search for the name, taking the top match. (2) A requested label section that comes back empty carries a section outcome that says why: no SPL record matched, no label for this match, or fetch failed — in line with the four section states. (3) Label sections are capped by `LABEL_MAX_CHARS = 2000` (`src/entities/drug/label.rs:10`, `truncate_with_note` at :44) — the ~2,138 characters the KB lead measured is 2,000 plus the truncation note. JSON carries the whole section; Markdown may keep a short form that says how to get the rest, mirroring ticket 1294's abstract rule.
- Proof: A spec table from recorded openFDA responses: osimertinib (sparse metadata) resolves to a label; a drug with no SPL record at all states which outcome and why; gefitinib's JSON carries whole sections with no truncation note; Markdown keeps its short form with the pointer. Guard, validated 2026-10-05: openFDA full-text "osimertinib" first returns AMIVANTAMAB's label (it mentions osimertinib), so the fallback must match the record's own identity — active ingredient or the SPL product's generic/brand fields — and never return another drug's label. A wrong-drug row is a failing case in the spec table.
- Defers: Mapping label section names across SPL versions.
