# 2068 — Make drug brand lists name only the drug's own brands

Status: OPEN.

Milestone: 0.9.3

## Outcome

A drug card's brand list names only products whose label names that drug as its sole active ingredient, and keeps a brand name the user typed when lookup resolved it.

## Evidence

Filed 2026-10-13 from the post-release install and product test of v0.9.2; rechecked against current main on 2026-10-14.

- `get drug trastuzumab` lists Enhertu, which is trastuzumab deruxtecan, as a trastuzumab brand. `get drug Tarceva` lists the chemical name "[6,7-Bis-(2-methoxy-ethoxy)…]-amine" as a brand and omits Tarceva.
- The causes: `merge_mychem_hits` copies DrugBank synonyms into `brand_names`, then sorts and cuts them to three; `apply_openfda_metadata` pools brands from every openFDA result, and the openFDA search matches Enhertu's generic name; the brand rescue swaps the lookup name and merges under the new name, so the typed brand is lost.
- This ticket was split on 2026-10-14. The error messages moved to 2087, and the PubTator warning, tokenless ORCID and FDA orphan health row moved to 2088.
- PM decision 2026-10-14: brands come only from openFDA labels whose generic name matches the card's ingredient, never from DrugBank synonyms. When the brand rescue resolved a brand the user typed, that brand is listed.

- Starts from: `merge_mychem_hits` (`src/transform/drug.rs:1085-1112`, sort and cut at 1182 and 1187); `apply_openfda_metadata` (`src/entities/drug/metadata.rs:167-197`) with `extract_openfda_values` (`src/entities/drug/label.rs:551-569`) and `choose_label_response` (`src/entities/drug/label.rs:1137-1160`); the brand-rescue swap in `src/entities/drug/get.rs:852` and the merge at 871.
- Keeps: 0.9.2's drug and label resolution rules (2031, 2039, 2043).
- Changes: claims `src/transform/drug.rs` `src/entities/drug/metadata.rs` `src/entities/drug/metadata/tests.rs` `src/entities/drug/label.rs` `src/entities/drug/get.rs` `src/transform/drug/name_resolution_tests.rs` `testdata/sources/openfda/**` `testdata/sources/mychem/**` `testdata/sources/capture-receipts.json` `CHANGELOG.md`.
  - Brands come only from openFDA results whose `generic_name` matches the card's ingredient; DrugBank synonyms stop feeding brands.
  - When the rescue resolved a typed brand, that brand is listed.
  - One Fixes changelog bullet.
- Proof: recorded-reply tests for `get drug trastuzumab` (no Enhertu) and `get drug Tarceva` (Tarceva listed, no chemical name); both fail on current main; one live run of each, recorded in the review. Routine suite.
- Defers: the error messages (2087); the PubTator, ORCID and FDA fixes (2088).
- Order: after 2085, which also edits `src/entities/drug/get.rs` and `testdata/sources/openfda/**`.
