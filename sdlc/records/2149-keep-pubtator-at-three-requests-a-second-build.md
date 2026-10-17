# 2149 build record — keep PubTator at three requests a second

## Report being fixed

The research team reported 2026-10-16 that BioMCP paces PubTator3 at ten
requests a second when `NCBI_API_KEY` is set: `pubtator_min_interval`
returned 100 ms keyed and 334 ms unkeyed, the `from_env` comment claimed the
key enables a higher PubTator budget, and the test pinned the wrong pair.

## Provider documentation check

PubTator3's own API documentation (checked 2026-10-16 at
<https://www.ncbi.nlm.nih.gov/research/pubtator3/>, API Documentation
section) says: "In order not to overload the PubTator3 server, we ask that
users post no more than three requests per second." The page names no API
key and no keyed rate, so the published cap is three requests a second for
every caller and 334 ms is the matching interval. The NCBI key's
ten-a-second budget belongs to E-utilities alone; the keyed 100 ms
E-utilities pin is untouched. The citation and the reproduction command and
output are recorded in the ticket's Evidence and Reproduction sections.

## Change

- `src/sources/rate_limit.rs`: `pubtator_min_interval` returns 334 ms with
  and without the key (the parameter stays so the test pins both key
  states); the `from_env` comment now says the key raises the E-utilities
  budget only.
- The rate-limit test is renamed
  `pubtator_interval_stays_at_three_per_second_with_or_without_the_key` and
  pins 334 ms both ways; the keyed E-utilities test still pins 100 ms.
- Docs that claimed the key improves PubTator were corrected:
  `docs/getting-started/api-keys.md`, `docs/reference/configuration.md`,
  `docs/reference/data-sources.md` (row plus its pinned string in
  `tests/test_source_licensing_docs_contract.py`),
  `docs/reference/sources.json` (pubtator3 `provider_access` and `notes`,
  `reviewed_on` 2026-10-16), and `docs/blog/biomcp-pubmed-articles.md`.
- `CHANGELOG.md` carries the Fixes bullet. No new fixtures, so no boundary
  pin or capture-receipt change.

## Red on main

Job `1792206495882-2160230` (the Linux gate host) at `db3d2d3f6` = origin/main
`b138f6097` + ticket citation + corrected test, no product change:

- `pubtator_interval_stays_at_three_per_second_with_or_without_the_key`
  FAILED at `src/sources/rate_limit.rs:626` — `left: 100ms, right: 334ms`:
  main paces a keyed PubTator client at ten requests a second.
- `pubmed_eutils_interval_uses_key_aware_values` passed — the keyed
  E-utilities budget holds.
- CI run `38997576589` at that same push failed the same test, confirming
  the red state in a second environment.

## Green evidence

| Check | Where | Tip | Verdict |
| --- | --- | --- | --- |
| Targeted rate-limit tests | the Linux gate host, job `1792207540003-2215218` | `15a8d90be` | 3/3 passed |
| `make test` (routine offline gate) | the Linux gate host, job `1792208694524-2227816` | `15a8d90be` | passed: Rust 4,301/4,301 (33 skipped), Python 1,159 passed (3 skipped), mkdocs strict |
| `make lint` | the macOS gate host, job `1792212424683-50166` | `15a8d90be` | passed |
| CI | GitHub | `15a8d90be`, run `38998714628` | green in every executed lane |
| `make test` after the first rebase (onto `f9a11e81e`) | the Linux gate host, job `1792214612828-2798971` | `94486c1d4` | passed: Rust 4,304/4,304 (33 skipped), Python 1,160 passed (3 skipped) |
| `make lint` after the first rebase | the macOS gate host, job `1792218223317-62798` | `94486c1d4` | passed |
| CI after the first rebase | GitHub | `94486c1d4`, run `39006084119` | green in every executed lane |
| `make lint` after the second rebase (onto `159c68610`) | the macOS gate host, job `1792226203155-22615` | `1f2f440e5` | passed |
| Targeted rate-limit tests after the second rebase | the Linux gate host, job `1792225023866-3632567` | `1f2f440e5` | 3/3 passed |
| CI after the second rebase | GitHub | `1f2f440e5`, run `39016731916` | every executed lane green except `rust-tests`, which fails only the inherited main defect below |

The branch was rebased twice because tickets 2138, 2148, and then 2131
landed on main while this branch was in review gates; the rate-limit change
is identical across all three tips.

## Inherited main failure (blocker for the green-CI landing bar)

`entities::article::variant_search::tests::no_entity_annotation_run_names_the_miss_instead_of_complete_empty`
fails on main itself after ticket 2131 landed:

- CI on main, run `39012801091` (the `Land 2131` push) — failed, that test.
- CI on main, run `39015559033` (at `159c68610`, the alias-query-plan fix
  that followed) — failed, that test ("a no-entity miss is not an outage").
- CI on this branch, run `39016731916` at `1f2f440e5` — every executed lane
  green except `rust-tests`, which fails only that test.
- The branch's own `make test` at `94486c1d4` (the base before 2131) passed
  the full suite, and the branch's targeted run at `1f2f440e5` passes every
  rate-limit test, so the failure is not this change's; these runners carry
  no NCBI key, and the unkeyed PubTator interval was 334 ms before and
  after.

Until the 2131 lane repairs that test on main, no rebased tip of this
branch can post a green CI run; after it is repaired, rebase and rerun the
routine gate at the accept tip to close the landing bar.

## Findings

- The API-keys guide, configuration reference, and data-source registry rows
  still claim the NCBI key improves PMC OA and NCBI ID Converter quota; the
  rate limiter keeps both at 334 ms with or without the key. Pre-existing
  beyond this ticket's PubTator scope; the rows now scope their claim so
  PubTator3 is no longer listed.
- The `sources.json` pubtator3 entry keeps `bioMcp_auth: optional_env` with
  `env_var: NCBI_API_KEY`, because the registry's auth shape is a
  licensing-registry decision; its prose now states the key does not lift
  PubTator3's rate.
- `cargo nextest` still emits the pre-existing `unused import: std::fs`
  warning for `src/cache/manager.rs` (noted in the 2146 record; not touched
  here).
