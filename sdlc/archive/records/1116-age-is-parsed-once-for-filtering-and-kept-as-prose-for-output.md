---
flow: build
priority: 4
---

# Parse each trial age bound once and retain the provider notation

## Outcome

One canonical `TrialAge` owns a bound's provider text and parsed number/unit. ClinicalTrials.gov filtering and public trial output consume that same parsed value, so they cannot interpret one notation differently. NCI detail conversion emits the same public structure. Existing human-readable age ranges stay unchanged except that malformed bounds no longer exclude trials accidentally.

## Public representation and serde contract

Replace `TrialEligibility.minimum_age` and `maximum_age` from `Option<String>` with `Option<TrialAge>`. `TrialAge` is an object-only public type with exactly three required members:

```json
{"number":6.0,"unit":"months","original":"6 Months"}
```

- `number` is a non-negative finite JSON number or `null`.
- `unit` is one of `years`, `months`, `weeks`, `days`, `hours`, `minutes`, or `null`.
- `original` is a nonblank string: the provider string after outer trimming, with internal whitespace, spelling, punctuation, and case otherwise unchanged.
- `number` and `unit` are either both non-null or both null. Exactly one null is invalid. Unknown members, a legacy string in place of the object, an unknown unit, a blank `original`, a negative/non-finite number, or a one-null pair fails public deserialization.
- A both-null pair is valid and retains a recognized no-limit sentinel or malformed nonblank provider notation. Public deserialization does not try to reinterpret `original`; normal provider construction is the canonical parser's responsibility.
- `TrialAge` serialization always emits all three members, including explicit nulls, and rejects an invalid in-memory value before emitting JSON. Object deserialize/serialize round-trip preserves the exact values.
- Keep the members behind validated constructors/accessors so ordinary production code cannot construct a negative, non-finite, one-null, or blank-original value and defer discovery until serialization.
- On `TrialEligibility`, absent or blank provider bounds become `None` and are omitted by `skip_serializing_if`. A missing public field and an explicit public `null` both deserialize to `None` and reserialize as omission. `N/A` is not omission: it emits `{"number":null,"unit":null,"original":"N/A"}`.

This intentionally corrects the opt-in JSON eligibility schema from strings to objects before 0.9 stable. It does not add a second raw/parsed public pair. Top-level `age_range: Option<String>` remains and is formatted from the canonical values; the `eligibility` key remains section-gated. Markdown remains byte-for-byte stable for valid values and no-limit sentinels, including `2 Years to 18 Years`, `18 Years to Any age`, and `Any age to 6 Months`.

## CTGov wire boundary and exact grammar

ClinicalTrials.gov documents `minimumAge` and `maximumAge` as `NormalizedTime`. The receipted metadata says the fields may be missing for `N/A`; the provider's official field statistics reviewed 2026-09-04 show title-cased singular/plural Years, Months, Weeks, Days, Hours, and Minutes. That six-unit provider vocabulary is distinct from the existing BioMCP filter's four comparable units.

Keep `CtGovStudy` provider-shaped. Its age fields become `Option<CtGovAgeWire>`, where the internal wrapper owns both the original wire string and its one canonical `Option<TrialAge>`. Custom string-only wire serde must:

- deserialize a JSON string to the wrapper and run the canonical parse once;
- serialize the wrapper back to that same provider string, never a `TrialAge` object;
- map a missing or explicit-null wire field to `None` and preserve the current `null` serialization of `None`; and
- retain a blank wire string for wire round-trip while exposing no public bound and no comparable value.

The public `TrialAge` deserializer must not accept CTGov's string form. Filtering reads the wrapper's already-parsed `TrialAge`; CTGov transformation clones that value into `TrialEligibility` and derives `age_range` from it without reparsing `original`.

The canonical provider-string grammar is deliberately stricter than today's accidental `f64`/token behavior:

| Input class | Canonical result | Filter comparison |
| --- | --- | --- |
| `0`, `18`, `0.5`, `18.25` | finite number; omitted unit becomes `years` | comparable |
| one or more Unicode whitespace characters around/between tokens | accepted; outer whitespace removed only from `original` | comparable when unit is compatible |
| `Year`/`Years`, `Month`/`Months`, `Week`/`Weeks`, `Day`/`Days`, any ASCII case | finite number plus canonical plural lowercase unit | years: amount; months: `/12`; weeks: `/52`; days: `/365` |
| `Hour`/`Hours`, `Minute`/`Minutes`, any ASCII case | finite number plus canonical plural lowercase unit | deliberately non-comparable and therefore fail-open |
| `N/A`, any ASCII case | null number/unit; exact trimmed spelling retained | fail-open/no limit |
| blank | no `TrialAge` | fail-open/no limit |
| `+18`, `-1`, `.5`, `5.`, exponent notation, `NaN`, any infinity spelling, or numeric overflow | null number/unit; original retained | fail-open |
| punctuation on number/unit (`18, Years`, `18 Years,`) or any third/trailing token (`18 Years old`) | null number/unit; original retained | fail-open |
| unknown/missing unit after a non-unitless multi-token value | null number/unit; original retained | fail-open |

The accepted numeric token is exactly ASCII `[0-9]+(?:\.[0-9]+)?`, followed by either no token or exactly one recognized unit token. Comparisons are inclusive. Zero is valid. Hours/minutes are structurally parsed and visible in output but do not gain new filter semantics in this ticket.

This is an intentional selection correction. Current valid integer/decimal, unitless, case-insensitive singular/plural forms for years/months/weeks/days keep their results. Current accidental parsing of signed, non-finite, exponent, punctuation, and trailing-token forms is removed; each becomes malformed and fails open. In particular, malformed minimum or maximum bounds no longer reject a trial through `NaN` or another non-clinical comparison. Provider prose is retained even when it cannot filter.

## NCI construction and section boundary

`from_nci_trial` remains the owning conversion seam. It builds top-level `age_range` and a `TrialEligibility` from `eligibility.structured.sex`, `min_age`, and `max_age` only when at least one normalized sex or retained bound exists; otherwise `eligibility` is `None`, including when `eligibility`/`structured` is missing or non-object and supplies no usable member. A malformed nonblank age is a retained bound and therefore keeps the object present. `format_sex` continues to map `FEMALE`/`MALE`/`ALL` to `Female`/`Male`/`All`. `entities::trial::get` remains the section-selection seam: after conversion it clears `trial.eligibility` unless `eligibility`/`all` was requested, exactly as the CTGov branch already does. Clearing structured eligibility never clears top-level `age_range`.

NCI age strings use the same canonical parser once. Missing `eligibility`, missing/non-object `structured`, or null/bool/array/object/blank age members yield no bound. A JSON number retains current `json_get_string` compatibility by becoming its finite string form and the unitless-years canonical value. A malformed nonblank string is retained with null parsed members and remains visible in prose range output.

Only an NCI **maximum** whose trimmed text equals `999 Years` case-insensitively is the provider's no-upper-bound sentinel. Requested JSON retains it as `{"number":null,"unit":null,"original":"999 Years"}`; top-level and eligibility Markdown render `Any age` for that side. `999 Years` in an NCI minimum or either CTGov bound is an ordinary parsed years value. Other malformed maximum strings remain prose, not no-limit sentinels.

The receipted NCI fixture's `*_age_number`, `*_age_unit`, and `*_age_in_years` fields are assertions about fixture consistency only. Runtime conversion does not substitute or validate them, so missing/null/malformed auxiliary members cannot erase the provider's `min_age`/`max_age` text. NCI `--age` remains unsupported and is rejected by `entities::trial::search::validate_trial_search` before either the NCI CTS or MyDisease provider client performs a request.

## Current evidence

Reviewed 2026-09-04 at `5a3c5656` (`0.9.0-dev.6`; `e6650bc0` advanced by one ticket-only commit during review):

- `CtGovEligibilityModule` currently stores raw strings. `entities/trial/search/eligibility.rs::parse_age_years` parses them for filtering, while `transform/trial.rs::normalize_age` independently prepares `Trial.age_range` and string-valued `TrialEligibility` output.
- The current filter uses Rust `f64` parsing, strips nonalphabetic punctuation from the second token, ignores later tokens, and does not reject non-finite values. The comparison behavior described above therefore needs an explicit correction rather than a false no-selection-change promise.
- CTGov age filtering happens before `TrialSearchResult` projection. NCI rejects `--age`. Direct get, trial batch, typed/raw MCP get, and Markdown/JSON use the shared trial entity route; no route-specific age parser exists.
- Ticket 1132 has landed. The receipted unrestricted NCI fixture is `testdata/sources/nci_cts/get_nci_2023_04529_full_20260903.json`; it carries structured sex, `18 Years`, the maximum `999 Years` sentinel, and matching auxiliary number/unit/year fields. The older `search_melanoma.json` is `pending_verification` and is not new-contract provenance.
- Receipted CTGov captures prove years and months: `get_nct02576665_full_20260903.json` carries `18 Years`/`75 Years`, and `search_phelan_limit5_20260811.json` carries `3 Months`/`99 Years`. The deterministic spec fixture's `N/A` is synthetic and proves behavior, not provider provenance.
- The local archived ticket 0019 concerns source-integration architecture. The sibling BioData ADR/case reference is not a repository dependency; its complete number/unit/original outcome is restated here.
- There is one Cargo workspace package. `cargo package --list --allow-dirty --locked --offline` lists exactly 1,300 paths, and `tests/test_source_package_boundary.py` enforces `MAX_PACKAGE_FILES = 1_300`. The package has no free path slots. No age-specific crate is needed, and this ticket must add no packaged file.
- Relevant line counts were `transform/trial.rs` 686, its test file 670, `sources/clinicaltrials.rs` 599, `entities/trial/search/eligibility.rs` 412, `render/markdown/trial.rs` 266, and `entities/trial/search/ctgov.rs` exactly 1,000. The CLI 700-line and Rust-source 1,000-line quality ratchets require no allowance for this design.

## Test-first implementation plan

1. Add the canonical value/parser as a focused private section of the existing `src/entities/trial/mod.rs`, with a focused inline `#[cfg(test)]` age-test module, rather than adding a packaged path. Add table-driven red tests for every grammar row: integers/decimals, zero, singular/plural and mixed-case units, unitless input, all six provider units, outer/inter-token whitespace, signed/leading-or-trailing-dot/exponent forms, punctuation, trailing tokens, negative/non-finite/overflow tokens, `N/A`, blank, and unknown units. Assert exact objects, comparable years, and fail-open classifications.
2. In the same inline focused tests, pin public serde: exact object round-trip; explicit null members; rejected one-null pairs, negative/non-finite in-memory/public numbers, unknown units, blank originals, unknown members, and legacy strings; and `TrialEligibility` missing-field/explicit-null normalization to omitted output.
3. Add CTGov source-model tests proving a provider string deserializes once and serializes back as a string; object input is rejected; missing/null/blank wire cases behave exactly as above. Then replace `parse_age_years` and `normalize_age` consumers with the wire wrapper's canonical value. A provider-shaped `6 Months`/`N/A` study must prove `0.49` is excluded, `0.5` included, JSON exact, and Markdown unchanged.
4. Add focused named test cases to the existing `src/transform/trial/tests.rs` and `src/entities/trial/get/tests.rs` using the receipted NCI record; keep the transform test file below the 1,000-line Rust-source threshold. Assert auxiliary-field agreement as fixture evidence, sex plus both typed bounds when eligibility is requested, no eligibility key otherwise, `None` rather than an empty eligibility object when no normalized sex or retained bound exists, independent top-level range, max-only sentinel behavior, and missing/null/malformed primary and auxiliary cases. In the existing trial search test sidecars, add a validation test that points the NCI CTS and MyDisease bases at request-counting local servers, submits `--source nci --age`, and proves both counters remain zero.
5. Update the template to use the already-built common age range for both overview and eligibility display, so it does not independently inspect or reinterpret bound strings. Update `docs/user-guide/trial.md` with the exact corrected JSON, grammar, filter-compatible units, and fail-open rule.
6. Extend the existing under-limit `src/mcp/shell/typed_get_tests.rs` with focused get-route coverage rather than adding a packaged sidecar or growing inventoried `src/mcp/shell.rs`. Against one local CTGov fixture, invoke `BioMcpServer::get` with typed `TypedGet` and `BioMcpServer::biomcp` with raw `ShellCommand`, request JSON eligibility, parse each returned text content, and assert exact equality of both age objects with direct entity/CLI JSON. Also assert neither MCP result contains a legacy string bound. These are exact route assertions, not inherited-green claims.
7. After ticket 1160 lands, add the executable direct CLI/search/batch contract below and run the focused tests, `make lint`, `make test`, and `make spec`.

## Executable contract

Extend the existing deterministic CTGov fixture with one clearly synthetic searchable/detail record having minimum `6 Months`, maximum `N/A`, and a unique condition. In `spec/entity/trial.md`:

- direct `--json get trial <id> eligibility` asserts the exact `6 Months` and `N/A` objects;
- overview and eligibility Markdown both say `Eligible Ages: 6 Months to Any age` with no new annotation;
- otherwise-identical searches at `--age 0.49` and `--age 0.5` omit and retain the record respectively; and
- `batch trial <id> --sections eligibility --json` contains age objects exactly equal to direct get.

The MCP exact-object assertions belong in focused Rust tests to avoid a second executable-page overlap. The synthetic fixture is not a captured provider response; do not modify capture receipts or claim provenance for it.

## Acceptance

- One CTGov wire parse produces the canonical value used by both filtering and output; `parse_age_years` and string-output normalization no longer exist as competing interpretations.
- Public and CTGov wire serde satisfy every exact rule above. No non-finite JSON number can be constructed or emitted.
- The table-driven grammar suite proves the intentional filter compatibility/tightening boundary, including all six provider units and every malformed class.
- NCI builds typed eligibility at transform, gates it at get, keeps top-level range independent, maps sex, preserves primary prose, treats only maximum `999 Years` as unlimited, ignores auxiliary fields at runtime, and still rejects `--age` before both provider requests.
- Direct CLI, batch, typed MCP, and raw MCP assert the same exact public objects. Markdown text, search rows, count precision, pagination, provider requests, and provenance otherwise remain unchanged.
- No dependency, feature, workspace package, capture, receipt, package-limit, or package-exclusion changes. Add zero packaged paths and keep the locked/offline package inventory at or below the enforced 1,300-file ceiling (expected: exactly 1,300). Put implementation and focused tests in the existing files named by the plan. Keep `transform/trial.rs` at or below 700 lines, every newly grown Rust source below 1,000 lines, do not grow `entities/trial/search/ctgov.rs`, and do not require a ratchet allowance.
- Focused tests and `make lint`, `make test`, and `make spec` pass.

## Exclusions

Do not add filter comparison for hours/minutes or NCI, change `--age` input validation, infer absent bounds, change valid-range wording, add search columns, change counts/pagination/request shapes/provenance, modify captures/receipts, or add a general units package. Do not add a second public raw-age field.

Do not edit `src/render/markdown/related/article_support.rs` or `src/render/markdown/related/tests/drug_variant_article_trial.rs`. Keeping typed bounds inside `TrialEligibility` avoids changing every `Trial` literal, including the concurrently modified related-renderer tests.

## Dependencies and integration boundary

Product dependencies: none. Ticket 1132 is landed.

Working-tree dependency: `spec/entity/trial.md` and `spec/fixtures/setup-ctgov-intervention-alias-spec-fixture.sh` contain uncommitted ticket-1160 hostile-title work and are also required by this executable contract. Do not implement 1116 in those files until 1160 lands or they are explicitly handed off. Then reread/rebase onto the landed files and add age coverage without replacing the hostile-title fixture or shell-safety assertions. This blocks implementation in the present dirty tree, not design re-review.

## Review

- Evidence review (2026-09-04, `5a3c5656`): **ACCEPT for independent design review.**
- Independent design review (2026-09-04, `5a3c5656`): **REVISE.** Parsing grammar/selection semantics, wire versus public serde, NCI construction/gating, and exact MCP coverage were underspecified.
- Design remediation (2026-09-04, `5a3c5656`): **READY FOR RE-REVIEW.** The contract now deliberately tightens malformed filter inputs, distinguishes six provider units from four comparable units, specifies canonical/wire/public serde and finite validation, locates NCI transform/get ownership, requires exact typed/raw MCP assertions, and preserves packaging, line, provenance, and ticket-1160 integration boundaries.
- Independent design re-review (2026-09-04, `5a3c5656`): **ACCEPT after amendment.** The parsing table now makes the deliberate compatibility correction exhaustive; six provider units remain distinct from four filter-comparable units; public object serde and CTGov string-wire round trips are separated; NCI conversion, empty-object omission, section gating, sex, malformed prose, the maximum-only sentinel, auxiliary fixture evidence, and zero-request age rejection are explicit; and direct/batch/typed/raw outputs have exact proofs. Re-review found and corrected one blocking inventory error: 1,303 paths would violate the enforced 1,300-file ceiling, so implementation and tests must use named existing files and leave the expected inventory at 1,300. The dirty ticket-1160 overlap still blocks implementation of the two executable-contract files until it lands or is explicitly handed off; it does not block this design acceptance.
- Code review: pending.
- Implementation evidence (2026-09-04, `1492fb17`): **RED then GREEN.** The first focused compile after changing `TrialEligibility` to typed bounds failed in the existing renderer fixture with `expected TrialAge, found String`, proving the legacy public-string contract was active. After implementation, the focused trial entity/search/get lane passed 111 tests, trial transforms passed 34, CTGov parsing passed 8, and the exact typed/raw MCP age-object test passed. The NCI age-rejection test observed zero requests at both request-counting provider endpoints. Canonical range formatting moved into the existing trial entity module so `transform/trial.rs` remains 698 lines rather than exceeding its 700-line limit.
- Implementation gates (2026-09-04): **PASS.** `make lint`; `make test` (3,112 Rust passed/30 standard skips, 890 Python passed/3 skips, strict MkDocs); and `make spec` all passed. Locked/offline package inventory remains exactly 1,300 paths with no dependency, package, or packaged-path additions. The source-capture audit required receipted-record mutation for NCI edge cases and a tuple-form internal `NormalizedTimeWire`; both preserve the accepted wire behavior without changing captures or receipts.
- Independent code review (2026-09-04): **REJECT.** Production behavior appeared aligned, but the tests did not explicitly prove plain-decimal `f64` overflow, every singular/plural unit, all non-finite and invalid in-memory serde states, malformed-null round trips, wrong-typed values in each NCI bound, sentinel trimming/case, all NCI sex mappings, or the complete documented rejection grammar.
- Review remediation (2026-09-04): **READY FOR RE-REVIEW.** Existing tests now cover those exact matrices and exact public objects without production changes. Mutation-strength check: temporarily removing the canonical parser's finite-number guard made the new 400-digit plain-decimal case fail with `left: (Some(inf), Some("years"), Some(inf))` versus `(None, None, None)`; restoring the guard returned the focused suites to green. Focused age serde/parser, fail-open filtering, and NCI transform suites pass; capture audit, all-target no-default-feature clippy, strict MkDocs, and all 8 static specs pass.
- Independent code re-review (2026-09-04): **REJECT.** The implementation and remediated tests were satisfactory, but the user guide still omitted the exact Unicode-whitespace/original-preservation rule and did not explicitly name `NaN` plus positive and negative infinity spellings among malformed inputs.
- Documentation remediation (2026-09-04): **READY FOR RE-REVIEW.** The user guide now states the exact numeric/unit token separation and outer-versus-internal Unicode whitespace behavior, and explicitly lists `NaN` and positive/negative infinity spellings. This was a nondestructive documentation-only correction; product code, tests, fixtures, executable specs, captures, and receipts were unchanged.
- Final independent code re-review (2026-09-04): **ACCEPT with no findings**.
  Verified the exact numeric/unit and Unicode-whitespace grammar, outer-only
  `original` trimming, explicit non-finite rejection, all remediated parser,
  serde, filtering, NCI, CLI, batch, and MCP matrices, and unchanged packaging
  and source-size boundaries. Strict documentation and diff checks passed.

## Completed 2026-09-04

ClinicalTrials.gov age bounds now cross the public eligibility boundary as
typed `{number, unit, original}` objects produced by the same canonical parse
used for filtering. All six provider units remain visible; years, months,
weeks, and days are filter-comparable, while hours and minutes fail open. NCI
eligibility uses the same public objects, preserves malformed prose, maps sex,
and treats only a maximum `999 Years` as the no-upper-bound sentinel. Existing
top-level age-range and Markdown wording remain compatible.

Final primary-agent verification passed: `make lint`; `make test` (3,112 Rust
tests passed with 30 skipped, 890 Python tests passed with 3 skipped, and strict
documentation built); and `make spec` (all routine pages, 38 parallel-
isolation contracts, and 8 static specs passed). The locked package inventory
remained exactly 1,300 paths with no new dependency or packaged file.
