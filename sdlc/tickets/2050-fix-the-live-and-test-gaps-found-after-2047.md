# 2050 — Fix the live and test gaps found after 2047

Status: OPEN.

Milestone: 0.9.2

## Outcome

Every drug, disease, trial and variant answer that 0.9.2 changed is right on live sources for the cases below, a refusal never waits on a slow outside service, and each 2039 and 2040 rule has a test that fails when the rule is removed and runs the same on any machine. The fixes stay general rules with no lists of product, brand or gene names.

## Evidence

Filed 2026-10-11 from the independent review of main at `75fabdb20`: a live sweep of a release build on the build host with the cache off, a side-by-side timing against the released v0.9.1 binary, and a code read with mutation checks of the 2039, 2040 and 2047 landings.

The live sweep confirms the named fixes: the brand and plain-ingredient labels for about 50 drugs (Keytruda, Herceptin, Rybrevant, Darzalex, pertuzumab, biosimilars and the Qlex and Faspro forms), 39 oncology brands, the Pain Relief, Cold and Flu and Sleep Aid refusals, the TP53 and BRCA1 numbering answers, the GWAS 410 handling and the "Lung Cancers" link.

Live defects on main:

- `get drug "Headache Relief"` returns acetaminophen DB00316 with the label of a three-ingredient product (set `a876daa7`, acetaminophen, aspirin, caffeine) under the card name "tension headache relief". `get drug Advil` is named "advil pm" though it shows the plain Advil label `1f01c10a`. `get drug "Nighttime Sleep Aid"` is named "diphenhydramine hydrochloride zinc acetate". The cause is one loose rule: `src/transform/drug.rs:446-449` lets a product name that starts or ends with the request count as a match, `hit_canonical_names` (`:197`) counts NDC nonproprietary names as names the record owns, and `merge_mychem_hits` takes that name at `:705`. The brand tier at `src/entities/drug/label.rs:900` takes a brand-equal label without checking that its generic name fits the card.
- Herceptin Hylecta, Rituxan Hycela and Tecentriq Hybreza print "No openFDA SPL label record matched this drug." openFDA holds `ebf30894`, `3e5b7e82` and `a617b089`. `sparse_element_identity_names` (`label.rs:854-880`) takes only the first word as the brand, and `element_leading_brand_names` (`label.rs:576`) reads the same line with a different rule.
- `get drug 5-FU` and `get drug Ara-C` refuse honestly in about 4 and 6 seconds, against about 2 for v0.9.1, which answered them wrongly (camphor, menthol; n4-palmitoyl cytarabine). A trace shows one EBI OLS4 search taking 4.7 seconds; the same search called directly takes 5.4 to 5.9 seconds. The rescue added in 2037 waits for it with no time limit.
- `get disease mf` returns mycosis fungoides, `search trial -c mf --source nci` returns 233 results, and `search trial -c mf` prints no note, while "MF" refuses or notes. `src/entities/disease/resolution.rs:727-731` counts only all-capital words as abbreviations.
- Phesgo still shows `DrugBank ID: -` and `Safety: unavailable`, and sparse cards print blank lines for empty fields (four on Herceptin Hylecta). The "Lung Cancers" disease card is titled `# MONDO:0021117` with no name. The GWAS line cuts its reason mid-word ("v2 endpoint i"), repeats it, and offers a retry command that cannot work against a retired endpoint.
- `src/sources/gwas.rs:116` now reports HTTP 404 as "retired upstream (HTTP 410)"; before, a 404 meant no associations for the rsID.

Test gaps:

- Replacing `(candidates.len() == 1).then(...)` at `src/entities/drug/get.rs:299` with `candidates.first().cloned()` passes all 336 drug tests. The Sleep Aid test refuses only because its fixture has no MyChem reply for the first candidate.
- The 2040 tests in `src/entities/trial/search/ctgov/abbreviation_note_tests.rs:67-69` read the machine's own cache. On the build host, with a 183 MB cache, each takes 216 seconds and two disease tests hit the 240-second limit; with a scratch cache they take 0.03 seconds. `TrialSearchEnvRestore` (`src/entities/trial/test_support.rs:14`) copies `CtGovFixtureEnv` (line 43), and `src/sources/gwas/tests/retired_endpoint.rs:23` writes a third copy, each deciding on its own what to isolate. The 2040 build record calls the cache slowdown "a host finding worth its own ticket"; this item is that ticket.
- The discover memo at `src/entities/discover.rs:362` is process-wide, has no size limit and leaves the source address out of its key, so a long-running MCP server keeps every answer and two test fixtures with the same query share one.
- A doc comment left after `cached_discover_rescue` was deleted now sits on `TrialAliasSource` (`get.rs:382-386`) and says the rescue runs at one site, which is false (line 788).
- `label_result_rank` (`label.rs:887-925`) writes the plain-ingredient and qualified-form checks twice. The short-token rule is written at `resolution.rs:807` and `ambiguity.rs:168`, and `AbbreviationAmbiguity.clinical_reading` (`ambiguity.rs:147`) is read only by a test.

- Starts from: main `75fabdb20`.
- Keeps: every fixed answer in the live sweep above; the shared-name refusal by ingredient count; the exact-brand-else-one-ingredient label rule; the hyaluronidase-only pairing split; the honest 5-FU and Ara-C refusals; the "MF" note and NCI refusal; no lists of product, brand or gene names.
- Changes: count a product name as the record's own only when it equals the request, and check a brand-tier label's generic name against the card; read the brand from a sparse record with one reader that keeps multi-word brands; give the refusal rescue a time limit of about 2 seconds and refuse when it runs out; treat a short abbreviation the same in any letter case, or record why not in the ticket and make the lowercase answer say so; fill or drop empty card fields and name the disease card; keep 404 as no associations, cut no reason mid-word, and offer no retry for a retired endpoint; add the first-candidate fixture to the Sleep Aid test; give tests one shared environment helper that always sets a scratch cache and cache mode off; bound the memo per request or by size with the address in its key; delete the stray comment; share the duplicated rules and drop the unused field.
- Proof: for each live defect, a recorded-reply test that fails on `75fabdb20`; the `get.rs:299` mutation caught by a test; the 2040 tests run in under a second with a large cache in place; a test that the refusal returns within the time limit when OLS4 is slow; a recorded live sweep of the cases above and the earlier sweep's names on a build-host binary of the landing tree.
- Defers: nothing.
