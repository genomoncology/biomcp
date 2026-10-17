---
priority: 2
---

# 2182 — Report no Europe PMC full text without a source failure

Status: OPEN.

Milestone: 0.9.3

## Outcome

When Europe PMC has no full text for a PubMed ID, the full-text attempt reports that it found no full text, promptly, and the section never says a source failed.

- Evidence: found while checking research's PDF report on 2026-10-17 with the candidate. `biomcp get article 22663011 fulltext` and `biomcp get article 10615958 fulltext` record the Europe PMC MED XML attempt as `source_unavailable` and say "Full text is unavailable because one or more consulted sources failed", after about fifteen seconds of retries. A direct GET of `https://www.ebi.ac.uk/europepmc/webservices/rest/MED/22663011/fullTextXML` returns 404 in under half a second, and `decode_full_text_xml` in `src/sources/europepmc.rs` already maps 404 to no full text, so the client is not reaching that branch.
- Starts from: the request the MED attempt actually sends and the status it gets; find why it retries instead of reading 404.
- Keeps: a real Europe PMC outage still reports unavailable; retries stay for transient failures.
- Changes: claims `src/sources/europepmc.rs` `src/entities/article/fulltext.rs`, their tests, `CHANGELOG.md`.
- Proof: a recorded 404 for the MED attempt yields an attempt outcome of no full text with no retries and no source-failure message; on current main the same case reports `source_unavailable`. A recorded 503 still reports unavailable.

Coordinator outcome before code: Distinguish an ordinary absent Europe PMC MED full-text response from a genuine transient source failure, preserving outage retries and unavailable reporting. Before product edits, reproduce on current main through the build job and report the exact command/output, proposed implementation/test files and the outside-in failing proof. Root records that evidence and scope before implementation. Developers write no ticket or status records.

## Progress

- 2026-10-17 started
