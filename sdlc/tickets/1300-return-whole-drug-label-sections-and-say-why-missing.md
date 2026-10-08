# 1300 — return whole drug label sections and say why a label is missing

Filed 2026-10-05 by the BioMCP 0.9 lead, from the KB lead's EGFR-wiki message of 2026-10-04 (asks 1 and 3), reproduced 2026-10-05.

Status: OPEN.

## Build status

- Built on branch `tickets/1300-drug-label-sections`, sha `149cee54b`, 2026-10-07. Two review cycles. Cycle 1: FIX, one P1 (raw label+safety dedup compared whole label warnings against capped safety warnings, printing over-cap warnings twice). Cycle 2: ACCEPT — the dedup compares the Markdown projection on both sides, pinned by an over-cap test that fails on the old code; verified against the branch's own gefitinib capture.
- The wrong-drug trap is closed structurally: a guarded full-text fallback fires only on a narrow no-match, an identity word-sequence filter rejects records that merely mention the requested drug (the amivantamab and itraconazole captures pin it), fetch errors stay errors, and Ok(None) means openFDA answered with no record of this drug. Four honest label states with retry; whole sections in JSON with the cap moved to the Markdown view; the required-label abort became a settling unavailable outcome per the ticket's Changes item 2.
- Deferred: SPL section-name mapping across versions; safety and interactions keep their own caps; the fallback is get-path only; a record past the full-text limit of 100 reports no SPL match.

## Outcome

## Outcome

`get drug NAME label -j` returns whole label sections in JSON, and when a drug has no label it says why in the section outcome, in line with the four section states. An agent never sees a silent `label: null`.

## Evidence

- Starts from: Reproduced on main a877443f on 2026-10-05. `get drug osimertinib label -j` returns `label: null` with no section outcome at all; the KB lead saw the same for mobocertinib. `get drug gefitinib label -j` returns a label object. Direct openFDA calls show why: `drug/label.json?search=openfda.generic_name:"osimertinib"` returns "No matches found", and neither `openfda.brand_name:"TAGRISSO"` nor `products.brand_name:"TAGRISSO"` match — osimertinib's current SPL label carries no populated `openfda.generic_name` or `brand_name`, while a full-text `"TAGRISSO"` query finds it. BioMCP's label lookup queries only those three narrow fields (`src/entities/drug/get.rs:233, 237` through the openFDA client), so sparse-metadata labels are silently missing.
- Keeps: The label shapes that work today, the DailyMed full-label URL, and every drug that already resolves.
- Changes: (1) Widen the label query with a fallback: when the field-scoped query returns no match, fall back to a full-text search for the name, taking the top match. (2) A requested label section that comes back empty carries a section outcome that says why: no SPL record matched, no label for this match, or fetch failed — in line with the four section states. (3) Label sections are capped by `LABEL_MAX_CHARS = 2000` (`src/entities/drug/label.rs:10`, `truncate_with_note` at :44) — the ~2,138 characters the KB lead measured is 2,000 plus the truncation note. JSON carries the whole section; Markdown may keep a short form that says how to get the rest, mirroring ticket 1294's abstract rule.
- Proof: A spec table from recorded openFDA responses: osimertinib (sparse metadata) resolves to a label; a drug with no SPL record at all states which outcome and why; gefitinib's JSON carries whole sections with no truncation note; Markdown keeps its short form with the pointer. Guard, validated 2026-10-05: openFDA full-text "osimertinib" first returns AMIVANTAMAB's label (it mentions osimertinib), so the fallback must match the record's own identity — active ingredient or the SPL product's generic/brand fields — and never return another drug's label. A wrong-drug row is a failing case in the spec table.
- Defers: Mapping label section names across SPL versions.

## Build status (fix round, 2026-10-07)

- Fix round on branch `tickets/1300-drug-label-sections`, commits
  2fd55e62c and 48d976598 (finding 8's six items), after one timeout
  revival with a checkpoint (nothing lost).
- Code review: ACCEPT 2026-10-07 on the fix round (fresh reviewer,
  tip 48d976598). All six fixes verified at code level: quoted
  pointer; view-cut gate (the coordinator's directive reading,
  honestly pinned); oversize settles empty-with-reason through the
  WithSourceContext-aware body-limit match; fetch errors stay errors
  with a genuinely biting test; the identity guard matches within one
  identity run with the leading-active-name rule for SPL elements
  (all fixture traces checked); empty_with_reason bounded and safe
  with label_note gone from Drug. Inventory repins measured exact;
  MAX_PACKAGE_FILES 1_402 measured on the branch (merge-time
  re-measure is the coordinator's). Two report-only P2s, recorded not
  folded: the pointer does not print in the summary-fallback branch
  (direction-safe; a cut section still shows its own truncation note
  with the full-label URL), and two inventory reason strings still
  cite the removed label_note field (refresh on next touch).

## Second-fix review

- Built on the rebased branch through c6966f982, 2026-10-07 (the
  rebase unions 326 receipts in main's formatting; the 1-space
  re-indent is gone).
- Code re-review (second-fix round): ACCEPT 2026-10-07 with an
  independent exhaustive hand-trace of every element in all three
  captures plus the Propulsid record — zero inactive-ingredient leaks;
  every named rejection and match holds; the Datroway histidine leak
  (unnamed by the review, real under the old rule) is rejected and
  pinned; the elements-field fallback returns the Propulsid label with
  terfenadine honestly empty and the oversize settle surviving;
  MAX_PACKAGE_FILES 1_404 measured. No findings. Merge-time recount
  expected (finding 12 rule).
