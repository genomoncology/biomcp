---
priority: 2
---

# 2182 — Report no Europe PMC full text without a source failure

Status: OPEN.

Milestone: 0.9.3

Reviews: revision ecdba16, accept

## Outcome

When Europe PMC has no full text for a PubMed ID, the full-text attempt reports that it found no full text, promptly, and the section never says a source failed.

- Evidence: found while checking research's PDF report on 2026-10-17 with the candidate. `biomcp get article 22663011 fulltext` and `biomcp get article 10615958 fulltext` record the Europe PMC MED XML attempt as `source_unavailable` and say "Full text is unavailable because one or more consulted sources failed", after about fifteen seconds of retries. A direct GET of `https://www.ebi.ac.uk/europepmc/webservices/rest/MED/22663011/fullTextXML` returns 404 in under half a second, and `decode_full_text_xml` in `src/sources/europepmc.rs` already maps 404 to no full text, so the client is not reaching that branch.
- Starts from: the request the MED attempt actually sends and the status it gets; find why it retries instead of reading 404.
- Keeps: a real Europe PMC outage still reports unavailable; retries stay for transient failures.
- Changes: claims `src/sources/europepmc.rs` `tests/article_asset_terminal.rs` `spec/fixtures/setup-article-fulltext-source-fixture.sh` `CHANGELOG.md`. Skip unsupported non-PMC XML requests while retaining the empty MED attempt; preserve supported PMC failure handling.
- Proof: a recorded 404 for the MED attempt yields an attempt outcome of no full text with no retries and no source-failure message; on current main the same case reports `source_unavailable`. A recorded 503 still reports unavailable.

Coordinator outcome before code: Distinguish an ordinary absent Europe PMC MED full-text response from a genuine transient source failure, preserving outage retries and unavailable reporting. Before product edits, reproduce on current main through the build job and report the exact command/output, proposed implementation/test files and the outside-in failing proof. Root records that evidence and scope before implementation. Developers write no ticket or status records.

## Progress

- 2026-10-17 started


## Coordinator reproduction and scope

Qualified CLI source `926b5786d400d3ec8b54c929701c1e46c62b5a95` differs from current main `90c00c371` only by ticket records. CLI SHA256: `e9992fadcb93e290a6c310827a14368444dcd1305784661c659c46311ed9083d`.

Outcome: healthy empty when no supported full-text resource exists; true outages retain retry/unavailability. Job 2182/1792273525777-75345 ran `biomcp --json get article 22663011 fulltext`: MED XML source_unavailable and source-failure message after about10s. Actual bare-ID route returned500; MED-prefixed undefined route404.

Official API supports fullTextXML only for PMC articles. Skip unsupported non-PMC XML requests, retain empty MED attempt/schema/order. Claims: src/sources/europepmc.rs, tests/article_asset_terminal.rs, existing XML route block in spec/fixtures/setup-article-fulltext-source-fixture.sh, changelog. No entity change unless reported.

Job 2182/1792273736408-87598 ran two real CLI regressions: PMC404/503 control passed; no-PMC failed with section outcome unavailable instead of empty after12.883s and MED XML source_unavailable. Authorize minimal supported-resource planner fix after this RED. Keep PMC404 exactlyone request and PMC503 retries/unavailability.

PM decision needed: confirm original MED404 proof correction to unsupported MED skip plus supported PMC404/503 controls. Implementation proceeds reversibly within approved outcome.

PM decision: 2026-10-17. Confirmed: skip unsupported MED route, retain empty attempt, PMC404 absence/no retry and PMC503 retries.

Coordinator verification: GREEN2182/1792273900673-4176 passed:3 CLI cases,2 existing planner cases, formatting and targeted Clippy with warnings denied. Local sourcec9c728c2eb46cd0cb95616ba66d68ef0a2b810f3 matches the applied tested patch. Existing PDF failure table remains green. The entity fulltext path was unchanged. Fresh review follows.
