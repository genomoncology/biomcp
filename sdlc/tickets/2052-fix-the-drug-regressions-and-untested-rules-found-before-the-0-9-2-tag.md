# 2052 — Fix the drug regressions and untested rules found before the 0.9.2 tag

Status: OPEN.

Milestone: 0.9.2

Reviews: revision 2f2ed9d, accept

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

## Evidence (implementation, 2026-10-12)

Branch `tickets/2052-work`, head below. Lander: Status stays OPEN for your
review. Every gate ran on the build host against the pushed head:

- `make lint` green (fmt, clippy `-D warnings`, deny, quality ratchet,
  test-wait ratchet: zero new `watchdog:` markers — the stalled-source
  fixture parks on a read that never answers instead of a timer).
- `make test` green (4205 Rust + full offline pytest contract lane). One
  residual: the three `rmcp_client_contract` stdio tests are load-flaky on
  the build host right after `prepare-test` builds (each passes standalone
  in 11.6 s and in an idle full run; main's margin is similar but thinner
  here). CI's own run of the same lane on this head is green.
- `make spec` green (mustmatch contracts).
- CI green on every named commit below (canonical gates, repository
  contracts, full features, stress, windows, release-panic).

### Rules landed (src)

- Label lookup follows the resolved ingredient (the card's own name), the
  request, and the record's own spelling of a reordered multi-word brand
  (`lookup_label_response` + `brand_spellings_for_request`): "Extra
  Strength Tylenol" reaches the TYLENOL Extra Strength record (103d109d)
  through the record's "Tylenol Extra Strength" row, where the request's
  phrase order matched nothing.
- A brand whose labels mostly name one ingredient resolves to it
  (`openfda_label_identity_candidate` rewrite in get.rs): votes read as
  ingredient word sets, owner rows vote first, one shared vote wins
  (Motrin → ibuprofen; Headache Relief → the acetaminophen-aspirin-
  caffeine line, stabilizing the card name against MyChem row churn), and
  exactly one plain single-ingredient vote among combination votes is the
  "mostly" rule. Several plain ingredients (Sleep Aid), several
  combinations (Cold and Flu) or unrelated singles (Pain Relief) still
  refuse.
- An ingredient query takes the plain ingredient's name and label: records
  whose own established name equals the query select themselves over
  records that only carry the query on another product's row
  (`hit_names_itself` + `SharedNameVote`), combination votes abstain while
  a plain vote exists, and with no plain vote the combination votes
  compare as word sets. Secondary database titles (GtoPdb/ChEBI
  "5-fluorouracil") never name the card; 5-fluorouracil → fluorouracil
  with label f0b60247.
- A card name never carries a product row's strength
  (`strip_trailing_strength`): aspirin is aspirin; "aspirin 81 mg" still
  resolves to the aspirin card.
- Combination cards show product-level identifiers or none
  (`merge_mychem_hits` clears drugbank/chembl/unii, approval dates, the
  synonym brands and drugbank interactions when the card name names a
  combination): Headache Relief and advil pm keep their combination cards
  with no DB00316/DB01050, no one ingredient's approval date and no
  chemical synonym listed as a brand.
- The hyaluronidase pairing derives from the record's structure
  (`plain_paired_ingredient_name(name, ingredients)`): the plain half is
  the half the record lists as an established ingredient while the paired
  half only extends a listed multi-word stem. No ingredient name appears
  in production code; Qlex's "berahyaluronidase alfa-pmph" resolves to
  pembrolizumab (the old literal refused it), while Opdualag and Lonsurf
  stay whole (both halves listed / the half is a truncated salt).
- Both refusal rounds share one budget: the entity rescue remembers its
  start when it times out and the CLI alias fallback spends only the
  remaining time (`remember_refusal_discover_start` +
  `refusal_discover_remaining_budget`). A stalled-source drug refusal
  settles in ~2 s, not ~4.
- "Retired" is typed (`BioMcpError::SourceEndpointRetired`): one sentence
  lives in `RETIRED_ENDPOINT_REASON`, classification is a type match, and
  the rendered GWAS section degrades permanently with no retry command.
- Test builds default the cache to a per-process folder under the OS temp
  dir (`default_cache_root` under cfg(test)); the test_support.rs comment
  now tells the truth about the hand-written guards that remain.
- `hit_is_provider_named` decides the unlabelled tie-break in both
  disease rankers; the dead `mondo.label` branch in
  `holder_display_label` is gone.
- Variant cards print no blank line for an absent protein-numbering note
  (templates/variant.md.j2 moves the newline inside the conditional).

### Red-on-main probes (probe branch `probe-2052`, main at 1268b7e11, branch deleted after recording)

14 of 18 probed tests fail on main: the cache-default test, the disease
tie-break test, the combination-identifier test, aspirin, Motrin,
5-fluorouracil, acetaminophen, Tylenol, Qlex, the updated Headache Relief
pin, the updated Lonsurf pin, the updated agreeing-pairings pin, the
variant template test, and the stalled-drug-budget test (4.07 s on main,
two budgets). The four that pass on main pin existing rules and fail only
under their recorded mutations: the exact-brand/unrelated-generic pin
(2050's generic-fits rule), the retired rendered-output pin (main already
degrades permanently), the CLI alias bound (main already runs 2 s), and
the Cold and Flu refusal guard (a keep).

### Mutations (each applied, run, reverted)

- Removing the brand-tier generic-fits check (`generic_name_fits_card`)
  fails `an_exact_brand_label_with_an_unrelated_generic_never_wins_the_
  brand_tier` (the unrelated record takes rank 4).
- Setting the CLI alias-fallback timeout to one hour fails
  `a_slow_alias_fallback_round_gives_up_within_the_refusal_budget`
  (123.9 s run).
- Forcing the retired flag false at the degrade site fails
  `a_retired_endpoint_degrades_the_rendered_gwas_section_with_no_retry`
  (a Retry line renders).

### Live sweep (release binary, `--no-cache`, 2026-10-12)

Every 2050 keep re-verified with its own label and unchanged from main's
binary where main still resolves it: Tarceva (erlotinib hydrochloride,
ab6f3cb3 — main and v0.9.1 both churned to the salt card), Zejula
(b7f675e2), Lonsurf (f5beed22, combination card), Keytruda (9333c79b),
Herceptin (492dbdb2), Rybrevant (1466c070), Tagrisso (5e81b4a7), Gleevec
(211ef2da), Herceptin Hylecta (ebf30894), Rituxan Hycela (3e5b7e82),
Tecentriq Hybreza (a617b089), Phesgo (27dd5e6b), Darzalex plain
(a4d0efe9) and Faspro (4bb241af), Avastin (939b5d1f), Rituxan (b172773b),
pertuzumab (17f85d17), trastuzumab (492dbdb2), imatinib (0291eca5).
Honest answers hold: Pain Relief, Cold and Flu, Sleep Aid and terfenadine
(empty label) refuse/answer as on main; 5-FU and Ara-C refuse on main and
here alike (MyChem no longer carries those abbreviations in any record
name — live churn since the 2050 sweep); mf/MF still refuse as ambiguous
abbreviations; the Gleevec mutation set keeps the imatinib mesylate card;
variant cards carry no blank line after "Transcript:".

New cases, this binary vs the v0.9.1 binary:

| Query | v0.9.1 | 0.9.2-dev (this branch) |
| --- | --- | --- |
| 5-fluorouracil | "5-fluorouracil", no label | fluorouracil card, label f0b60247 |
| Motrin | ibuprofen and diphenhydramine citrate, 10ec6386 | ibuprofen card, Motrin IB label 52c650cf |
| acetaminophen | hydrocodone…combination, 995a6cc7 | acetaminophen card, plain label 7f445de3 |
| Tylenol | combination label 1022767d | acetaminophen card, plain TYLENOL-family label 997b9551 |
| Extra Strength Tylenol | combination e97383af | acetaminophen card, TYLENOL Extra Strength 103d109d |
| aspirin | dental-kit combination 58d78805 | aspirin card, plain aspirin label a0a19cd1 |
| aspirin 81 mg | aspirin (7ce4807e) | aspirin (ac94cce3) |
| Headache Relief | "tension headache relief" + DB00316 | combination card, no ingredient ids |
| advil pm | combination + DB01050 | combination card, no ingredient ids |
| Qlex | refuses | pembrolizumab card, 9333c79b |

Sweep timings: every card settles in 0.7–2.7 s (`--no-cache`); refusals
2.1–2.7 s (one shared discover budget); v0.9.1's debug binary ran the same
list in 0.8–19.2 s.

### Comment sweep

Per Ian's mid-lane direction, code comments in `src/`, `templates/`,
the test trees and the gate scripts under `tools/` no longer name ticket
numbers: every comment that leaned on one now states the rule or fact
alone. The first pass (572 automated rewrites plus hand edits) was
single-line and left 31 references split across continuation lines plus
a handful of single-line leftovers; the review caught them, and the
finished sweep joins comment blocks before checking, removes the
split parentheticals, and repairs the one sentence the first pass
corrupted (the `plain_paired_ingredient_name` doc comment). Verified
with the continuation-line-aware check: zero ticket references remain in
comment scope. The round-2 review found the earlier check still matched only
"ticket"-worded and parenthetical shapes, leaving 29 bare mentions ("the
2040 trial tests", "pre-1304 cache", "2035 #6", "And 2047:", "the 2017
refusal"); all 29 are rewritten to state the rule or fact alone,
including the second mangled rewrite remnant the review caught
(label_choice.rs "And 2047:") and the two remaining gate-script files
(tools/with-build-identity, tools/update-rust-source-size-inventory).
The two python files that keep numbers in comments narrating their own
changelog fixtures are `tests/test_changelog_coverage_check.py` (e.g.
"Land 1306: fold the 1302 review carryovers" names 1302 in its text) and
`tests/test_docs_changelog_refresh.py` (e.g. "2040 landed after 2038"):
the numbers are the fixture content the gate reasons about, like a PMID,
not a justification leaned on. Assertion strings, python docstrings and
the sdlc record keep their references as before.

The verification command (run at the repo root, single-line comment
form; triage excludes dates, PMIDs/PMCIDs, CVCL/HP ids, DOIs, versions,
genomic and protein coordinates, and the two fixture files above):

```
$ grep -rn -E "^\s*(//|#).*\b(1[0-9]{3}|20[0-9]{2})\b" src/ templates/ tests/ tools/ \
    --include="*.rs" --include="*.py" --include="*.j2" \
    --include="check-*" --include="update-*" --include="with-build-identity" \
  | grep -vE "PMID|PMCID|CVCL|HP:|10\.[0-9]{4}|DOI|[0-9]{4}-[0-9]{2}|windows-20|CC BY|CTR Pv2|default_value|7157|1956|residues|Ser at|rows|P38398|position|Apr 20|test_changelog_coverage_check|test_docs_changelog_refresh"
(no output — zero ticket references remain in comment scope)
```

The same check run over joined comment blocks (continuation-line aware,
so a reference split across lines cannot hide) also returns no hits; the
only NNNN-shaped strings it surfaces are domain identifiers (ClinVar
55588, NM_007294, NC_000010.11 coordinates, the 0291eca5 label set id, a
4096-byte bound, a CI run id). fmt, clippy and the targeted suites
(527 Rust tests across the drug, NCI, cell-line, protein-change, WHO-PQ,
GWAS and annotation lanes) stayed green through the sweep.

### Residual risks

- The build host's `make test` occasionally flakes the three
  `rmcp_client_contract` stdio tests right after a build (60 s inner
  bound); standalone and idle runs pass, and CI is green on this head.
- openFDA/MyChem churn moved some recorded answers since the ticket was
  filed (Tarceva's card is the salt on every binary; 5-FU/Ara-C now
  refuse on main too). The keeps were verified against main's binary, not
  the 2050 recording.
- Tarceva's generic (salt) label remains deferred as filed.
