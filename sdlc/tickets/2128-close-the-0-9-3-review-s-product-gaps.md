# 2128 — Close the 0.9.3 review's product gaps

Status: OPEN.

Milestone: 0.9.3

## Outcome

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
- Changes: claims the files named above, their tests, `docs/reference/configuration.md` and `CHANGELOG.md`.
- Proof: one recorded-reply or unit test per item, each failing on current main; routine suite.
- Order: shares `get.rs` with 2113 and `structure.rs` with nothing open; rebase after 2113.
