# 2052 — Fix the drug regressions and untested rules found before the 0.9.2 tag

Status: OPEN.

Milestone: 0.9.2

## Outcome

`biomcp get drug` answers a common single-ingredient drug, its brand, and its spelled-out name with that drug and a label for that drug or that brand. Each rule 2050 added has a test that fails without it, and the test suite never reads the machine's own cache.

## Evidence

Filed 2026-10-11 from the independent review of main at `b039ac3e7` (candidate `91dbbeaec`), run on the build host with the release binary and `--no-cache`, compared with the v0.9.1 release binary.

- `get drug 5-fluorouracil` is named "5-fluorouracil" and prints "No openFDA SPL label record matched this drug." v0.9.1 named it fluorouracil with label f0b60247, and `get drug fluorouracil` still gets that label. The label lookup searches by the card name.
- `get drug Motrin` is named "ibuprofen and diphenhydramine citrate" with label 10ec6386 (an Ibuprofen PM store brand). openFDA holds Motrin IB labels for ibuprofen alone. v0.9.1 named it ibuprofen.
- `get drug acetaminophen` and `get drug Tylenol` refuse with "could not map … to a single drug". `search drug -q acetaminophen` lists acetaminophen first, and openFDA's Tylenol labels are mostly acetaminophen alone. v0.9.1 answered both with a wrong combination card. The refusal comes from the discover rescue giving up (`src/entities/drug/get.rs:815`).
- `get drug "Extra Strength Tylenol"` names acetaminophen with label 7f445de3, a store children's fever reducer, though openFDA holds TYLENOL Extra Strength labels.
- `get drug aspirin` is named "aspirin 81 mg", a product row's strength, not the ingredient.
- Combination brand cards carry one ingredient's identifiers: "Headache Relief" shows acetaminophen's DB00316 and approval date and lists the chemical synonym "4'-hydroxyacetanilide" as a brand name; "advil pm" shows ibuprofen's DB01050.
- Every variant card prints a new blank line after "Transcript:" (`templates/variant.md.j2:13` prints the note line when there is no note).
- Mutations on the build host, each reverted:
  - Removing the brand-tier generic-name check (`generic_name_fits_card`, `src/entities/drug/label.rs:975-979`) passes all 344 drug tests. 2050's Changes asked for this check.
  - Setting the CLI alias-fallback timeout (`src/cli/shared.rs:449`) to an hour passes all 175 refusal, slow and alias tests. The slow-source test calls `drug::get` and skips the CLI. A refusal can spend two full time limits: the entity rescue gives up at 2 s and is not remembered, then the CLI fallback starts a fresh lookup with its own 2 s. That matches the 5 s refusals for 5-FU and Ara-C.
  - Forcing the retired-endpoint flag to false at `src/entities/variant/gwas.rs:517` passes all 38 GWAS tests. `is_retired_endpoint_error` (`src/sources/gwas.rs:335`) compares reason text with `RETIRED_ENDPOINT_REASON`, so one fact lives in two places.
- Tests still read the home cache. `src/test_support.rs:29` says every fixture env copy routes through the shared type, but about 27 hand-written env guards remain, including `disease/enrichment/tests.rs:5`, `variant/get/tests.rs:9` and `trial/get/tests.rs:482`. With a 191 MB home cache, `ticket_589_disease_base_enrichment_failures_are_unavailable_without_credit` hit the 240 s timeout, and two GWAS tests ran past 120 s. With `BIOMCP_CACHE_DIR` set to an empty folder they pass in 0.03 s. The cause is the fallback to `dirs::cache_dir()` at `src/cache/config.rs:139`.
- `src/entities/disease/resolution.rs:581` still ranks "unlabelled" with `hit_label(...).is_none()`, while `rerank_disease_search_hits` now uses `hit_is_provider_named`, so the two rankers disagree. The `mondo.label` branch in `holder_display_label` (`:777`) can no longer run.
- `src/entities/drug/label.rs:838-840` matches the literal `" and hyaluronidase"`, an ingredient name in production code that `generic_name_fits_card` and `get.rs:291` now depend on.

- Starts from: ticket 2050 and its landing `1cdc15b6f`.
- Keeps: every 2050 fix and its tests; the 31 cancer brands and ingredients that resolve with their own label (Tarceva, Zejula, Lonsurf, Keytruda, Herceptin, Rybrevant, Tagrisso, Gleevec, the multi-word brands, Phesgo and the rest of the recorded sweep); the honest 5-FU, Ara-C, mf, "Pain Relief" and terfenadine answers; no lists of product, brand, drug or gene names.
- Changes: look up the label by the resolved ingredient as well as the card name; let a brand whose labels mostly name one ingredient resolve to it, and an ingredient query take the plain ingredient's name and label; name an ingredient card without a product row's strength; keep a combination card from showing one ingredient's identifiers as the whole product's; print no blank line for an absent note; give both refusal rounds one shared deadline or remember the timeout; carry "retired" as a typed error, not reason text; make test builds default the cache to a temporary folder and correct the `test_support.rs` comment; use `hit_is_provider_named` in both rankers and delete the dead branch; derive the hyaluronidase pairing from the record's structure.
- Proof: recorded-reply tests for 5-fluorouracil, Motrin, acetaminophen, Tylenol and aspirin that fail on `91dbbeaec`; a test where an exact-brand label carries an unrelated generic name; a CLI test that bounds the slow refusal under about 3 s; a test from `add_gwas_section` through rendered output for a retired endpoint; the disease and GWAS scopes pass in seconds with a large home cache present; a recorded live run of this ticket's drugs and the 2050 sweep, compared with v0.9.1.
- Defers: Tarceva's generic label, since openFDA holds no TARCEVA brand label; the article card splitting one gene across spellings (K-Ras and K-RAS).
