---
priority: 2
---

# 2128 — Close the 0.9.3 review's product gaps

Status: COMPLETE.

Milestone: 0.9.3

Depends on: 2156

Reviews: revision 4fceba1, accept

Reviews: revision 77e48fa, accept

Reviews: revision 78e627f, accept

Reviews: revision 5bf26a9, accept

Landed: 5bf26a9 (fast-forward: Accepted source landed inside carrier 27e59b197 with its passing shared gate; record this carried ticket separately without rerunning unchanged code.)

## Outcome

Coordinator implementation brief: close exactly the nine recorded caller-visible gaps while preserving their existing clinical, identity and cache-policy decisions. The affected regressions were authored first. Draft production edits began before the remote baseline finished; those edits were parked until the baseline returned, so reproduction is not claimed to have preceded that drafting.

Current-main `a1d96019d` with only regression tests overlaid failed all nine focused Rust assertions in job `1792242442747-76827`, exit 100. They cover unresolved hotspot presence, absent cache-age notes, unrelated brand candidates, duplicate disease holders, path-bearing health errors, invalid protein guidance and generic-brand ordering. Installer job `1792242677104-86365` exited 1: installation succeeded but the existing-binary marker was absent for a path containing spaces. The earlier accidental project-wheel build was cancelled with exit 143 and proves nothing. The paging changelog correction needs no runtime test.

The smaller product gaps the 0.9.3 review found in landed tickets are closed, each with a test that fails on current main.

## Evidence

Found by the 0.9.3 adversarial review on 2026-10-16.

1. 2114: after a reference mismatch, `cancerhotspots(gene, hotspot_change)` still prints position counts for the rejected position. 2114 also built `check_reference_residue` in `src/entities/variant/structure.rs` instead of reusing `requested_reference_residue` from `get.rs`, against its decision; the two treat a position past the protein's end differently.
2. 2091: day-cached GWAS and openFDA answers turn off the stale-serve note (`src/cache/manager.rs`, `if !day_cache_request_active()`), so an answer up to a day old prints with no age; under `BIOMCP_CACHE_MODE=infinite` they never expire, which the configuration docs do not say.
3. 2085: when no label record owns the exact brand, `label_identity_answer` (`src/entities/drug/get.rs:415-420`) falls back to every record on the page and refuses with products that do not carry the name, against the decision.
4. 2066: the leading clinical-reading candidate is not de-duplicated against the holders (`src/entities/disease/resolution.rs:802-808`).
5. 2065: cache error rows print `err.to_string()` (`src/cli/health/local.rs:255-270,431`), which can carry a local path into MCP replies.
6. 2087: the residue check covers only the gene-plus-change form; `BRAF 600Z` and a bare `Z999Q` pass.
7. 2068: openFDA `brand_name` values that are generic names list as brands; the list keeps the first five in openFDA order.
8. 2084: its changelog bullet implies a page crossing the 25-row boundary now matches, which the PM decision said it may not.
9. 2064: `install.sh:372,374` run `$($installed_bin --version …)` unquoted.

- Decisions:
  - 1: mark hotspot counts inapplicable when the placement is unresolved; move the structure path onto the shared residue check.
  - 2: day-cached answers carry the same cache-age note as other cached serves; the docs state the infinite-mode exception.
  - 3: with no exact owner, the refusal lists no candidates and names the search instead.
  - 4: drop the reading's disease from the holders list when it leads.
  - 5: health error rows give the error kind, never a path.
  - 6: run the residue check on every protein-change form that names a residue.
  - 7: drop brand values equal to the generic name; keep the cap, ordered by exact brand match first.
  - 8: rewrite the bullet to say what the fix covers.
  - 9: quote the substitution.
- Changes: claims `src/entities/drug/test_support/label_choice.rs` `src/entities/drug/test_support/ingredient_resolution.rs` `src/entities/drug/test_support/brand_resolution.rs` `src/cli/gwas/dispatch.rs` `src/cli/stale_json_note_tests.rs` `src/entities/variant/structure.rs` `src/entities/variant/structure/tests.rs` `src/entities/variant/get.rs` `src/entities/variant/get/protein_change_tests.rs` `src/entities/variant/resolution.rs` `src/entities/variant/resolution/tests.rs` `src/cache/manager.rs` `src/cache/manager/tests/stale_marker_tests.rs` `src/entities/drug/get.rs` `src/entities/drug/get/tests.rs` `src/entities/drug/metadata.rs` `src/entities/drug/metadata/tests.rs` `src/entities/drug/label.rs` `src/entities/drug/label/tests/identity.rs` `src/entities/disease/resolution.rs` `src/entities/disease/resolution/ambiguity.rs` `src/entities/disease/resolution/tests.rs` `src/cli/health/local.rs` `src/cli/health/tests/local.rs` `install.sh` `docs/install.sh` `tests/test_public_installer_checksum.py` `docs/reference/configuration.md` `CHANGELOG.md`.
- Proof: reproduce each behavior on current main through its actual entry point: `biomcp variant structure`, day-cached `biomcp --json search gwas` and openFDA calls, `biomcp get drug`, `biomcp get disease MM`, MCP health, `biomcp get variant`, and a staged `install.sh` installation whose existing path contains spaces. Record the exact commands and replies before changes. Add the smallest recorded-reply regression for each behavior; review the corrected 2084 prose against its existing scope decision. Run affected regressions through the build job and the routine landing gate. No prose-only regression test is needed.
- Keeps: all nine recorded product decisions above; no new clinical or identity rule. Reuse the approved shared residue check. Escalate a need to change those decisions through the PM.
- Defers: unrelated clinical interpretation and broad fixture restructuring.

## Progress

- 2026-10-17 started
- 2026-10-17 next: Review accepted 78e627f01. Original batch failures 1792249841729-7187 and 1792250314261-50149 exposed stale shared-name expectations inconsistent with approved exact-owner decision3. Minimal assertion/fixture-key corrections preserve runtime logic and recorded responses. Complete drug family passed 180 cases without fail-fast in 1792250810209-75160; formatting passed and Clippy reused from 1792250665363-95005. Corrected six-ticket batch gate is running.
- 2026-10-17 next: Batch members: 2159, 2149, 2168, 2128, 2122 and 2171. Shared landing gate: job 1792250897357-93113, candidate ea4161cc814ad034634ffd249e67b475e77e38dc. Required preceding main check 1792249698624-96397 passed. Gate is running; each ticket keeps its individual review and landing commit.
- 2026-10-17 next: Accepted health-test duplication repair 5bf26a959 preserves warning classification and JSON/Markdown path-redaction checks; file now 691 lines. Raw full Rust job 1792251220333-62539 completed every case: 4308 passed, 25 GenCC failures, 34 skipped; all health checks passed. The raw command omitted the existing Make canonical temporary-directory setup, matching the known alias limitation; prior configured gate supplies GenCC evidence. Configured six-ticket gate is running again; no full-pass claim is made for the raw job.
- 2026-10-17 next: Batch members: 2159, 2149, 2168, 2128, 2122 and 2171. Shared configured landing gate PASSED: job 1792251400030-83132, candidate c264da2a525c2c6e5a424693d55dfab0f59fadf4, 240.3 seconds. This replaces the earlier failed batch attempts. Preceding full main check 1792249698624-96397 passed. Land each reviewed member through pm, then run one full check on final batch main.
