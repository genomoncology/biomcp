---
flow: build
priority: 2
---

# 2149: Keep PubTator at three requests a second

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 238ed5f, accept

## Outcome

PubTator requests obey the documented three-per-second budget with or without an NCBI key; the keyed ten-per-second budget remains with E-utilities.

Depends on: 2156

- Starts from: `pubtator_min_interval`, its existing interval tests and the existing implementation branch.
- Keeps: keyed E-utilities and all other provider policies.
- Changes: claims `src/sources/rate_limit.rs` `docs/blog/biomcp-pubmed-articles.md` `docs/getting-started/api-keys.md` `docs/reference/configuration.md` `docs/reference/data-sources.md` `docs/reference/sources.json` `tests/test_source_licensing_docs_contract.py` `CHANGELOG.md`.
- Proof: verify the provider's current primary documentation and retain its citation. The existing interval cases show PubTator at 334 ms for both key states and E-utilities at 334/100 ms without/with the key; their corrected expectation fails on current main before implementation. Run affected checks and the routine merged-candidate gate through build jobs.
- Defers: other rate-limit policies.

PM decision: 2026-10-17. Reuse the existing narrow correction after validating its source citation and evidence. This precedes the added full-text annotation traffic.

## Existing branch evidence

## Evidence

Reported 2026-10-16 by the research team.

- `pubtator_min_interval` (`src/sources/rate_limit.rs:263-268`) returns 100 ms when `NCBI_API_KEY` is set and 334 ms otherwise; the comment at `:97` says the key enables a higher PubTator budget, and the test at `:624-626` pins it.
- PubTator3's API documentation asks callers to stay at three requests a second or fewer and names no higher keyed rate. The NCBI key raises the E-utilities limit only.
- Checked 2026-10-16 against PubTator3's own API documentation at
  <https://www.ncbi.nlm.nih.gov/research/pubtator3/> (the API Documentation
  section): "In order not to overload the PubTator3 server, we ask that users
  post no more than three requests per second." The page names no API key and
  no keyed rate, so the published cap is three requests a second for every
  caller and the matching BioMCP interval is 334 ms.

## Reproduction on main

A gate job on the Linux gate host (`1792206495882-2160230`) ran at `db3d2d3f6` —
origin/main `b138f6097` plus this ticket's citation and the corrected test,
no product change — with:

```
cargo nextest run --no-default-features -E 'test(sources::rate_limit::tests::pubtator_interval_stays_at_three_per_second_with_or_without_the_key) | test(sources::rate_limit::tests::pubmed_eutils_interval_uses_key_aware_values)'
```

- `pubtator_interval_stays_at_three_per_second_with_or_without_the_key`
  FAILED at `src/sources/rate_limit.rs:626`: `assertion \`left == right\` failed,
  left: 100ms, right: 334ms` — main paces a keyed PubTator client at ten
  requests a second.
- `pubmed_eutils_interval_uses_key_aware_values` passed — the keyed
  E-utilities budget of 100 ms is unchanged.

## Work

- Reproduce on main first, and confirm in PubTator3's current documentation that it grants no keyed rate. If it does, cite it in this ticket and close it.
- Otherwise make PubTator's interval 334 ms in both cases, and correct the comment and test.

## Acceptance

1. The rate-limit test pins 334 ms for PubTator with and without the key, and 100 ms for keyed E-utilities.
2. The changelog carries a bullet for this ticket.

## Takeover interpretation

Start from pushed `7f701bfac`. Keep its implementation and existing evidence. The outcome means the PubTator policy never receives the keyed E-utilities uplift. The eight files named in Changes are the implementation claim; this record only reconciles the approved contract. Existing interval tests are retained as a numeric boundary regression instead of adding duplicate network timing tests. Before further implementation, rerun the corrected PubTator expectation against main `078ce756d` through a build job; it must fail with 100 ms versus 334 ms, while the keyed E-utilities expectation remains green. Then run those same interval tests against this branch.

Primary source rechecked 2026-10-17: <https://www.ncbi.nlm.nih.gov/research/pubtator3/> embeds its API Documentation in <https://www.ncbi.nlm.nih.gov/research/pubtator3/main.b1489e87146c1c50.js>. It says users should post no more than three requests per second and grants no keyed rate. E-utilities separately documents its keyed uplift at <https://www.ncbi.nlm.nih.gov/books/NBK25497/>.
