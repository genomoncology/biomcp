# 2353 — Finish dated gene PubTator searches

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision a7f05235d379720964d68c0fff495d036a7a7940, accept

Landed: 4b22f7b

## Outcome

The study's dated FANCL article search avoids persistent unnecessary PubTator work within the existing source guard and provider limits. Diagnose the exact single-gene request, pagination and date handling; this command has no keyword OR expression. Preserve typed identity, date meaning, source pacing and other-source results. Genuine upstream failures remain honestly source-local. Do not raise the guard or hide incomplete work.

- Changes: `src/sources/pubtator.rs`, `src/entities/article/backends.rs`, the narrow year-filter request projection in `src/sources/literature_retry.rs`, `tests/test_article_pubtator_dates.py`, `tests/feature-links.json` and `CHANGELOG.md`.
- Proof: reproduce current-main command, then an outside-in recorded-provider case completes its dated gene leg without historical-page churn; a genuine timeout retains other-source results and actual attempted-request diagnostics.

## Evidence

- PM m3492 authorizes filing all confirmed uncovered 0.9.4 defects from the 27 research observations. This is research c002/call-0007, TSV physical line11, distinct from 2346's CHEK2 OR precedence cause.
- Current-main M5 j4335 on f7165ac3b6e2c25367f31f25393343313564b5e7 ran `biomcp --json --no-cache search article -g FANCL --date-from 2026-09-01 --limit 50 --max-per-source 50 --debug-plan`. It exited0 after14.458s with five other/partial rows, complete false, and PubTator failure_class=timeout/retry_attempted=true. The exact command has no OR keyword, so no Boolean-fix equivalence is claimed. Yellow ticket-evidence/2350/followup-probe retains stdout/stderr and recipe.
- Coordinator restatement: remove the reproduced persistent cause within existing limits, after bounded mechanical diagnosis; preserve the already honest degraded result and all identity safeguards. 2336 retry coverage alone does not settle this persistent current-main reproduction.

- Bounded provider diagnosis j4359 compared unfiltered relevance, explicit date sort and native year filters; no silent sort change was adopted. j4362 measured typed FANCL count1508,61pages,25rows/page and29 year buckets whose counts exactly sum1508, including24 hits in2026. Current client scanned historical relevance pages that the local day filter discarded.
- Outside-in RED j4368 failed three cases; j4372 showed unnecessary unfiltered page2. The bounded fix uses native year filters only after validating unique numeric positive-count buckets with sum exactly equal to the provider count. Open bounds derive from actual provider years, retaining future years; exact day filters and relevance remain. Partial/missing/malformed facets and nonzero raw provider offsets keep existing honest behavior. No arbitrary year range, guard increase or pacing change.
- Ordering control j4392 caught initial-page backup overriding successful native order; the final code uses native success ordering/scores/dedup and preserves backup rows only on failure. Failed/truncated totals remain unknown; exact day-filtered total is stated only after successful stream exhaustion. The existing actual-request diagnostic projects only validated year-filter fields, never arbitrary JSON or authentication.
- Final affected j4398 passed17 cases, formatting, Ruff and all-target Clippy. Controls cover future2028/open bounds, date-to/multi-year/proven empty, nine incomplete/malformed facet fallbacks, native-order ownership and genuine filtered timeout with preserved other results. Earlier formatting/diagnostic failures remain historical. No full routine duplicate ran.
- Exact study command live confirmation is j4379, not the initially mistyped j4377: source f7165ac plus tested2353 patch completed exit0 in8.194s, complete true/five rows/no PubTator degradation. Earlier j4335 took14.458s with incomplete five rows and timeout. These are actual live observations, not an isolated causal timing benchmark. Final branch9db29a40e1145859479514c472d602ed538d5176 includes the accepted earlier batch and changes only SDLC after the final implementation proof. Yellow ticket-evidence/2353/dated-pubtator-proof.tar.gz SHA2561e22c2806ccfbb9df239b2b454903f07e1b975b290446b555c2b7ea5e48d5913 retains commands, source/patch bindings and raw proofs.

- Fresh independent reviewer accepts a7f05235d379720964d68c0fff495d036a7a7940 after complete facet/bounds/order/pagination/failure/diagnostic and affected-caller review. Its independent exact-source M5 j4401 passed all17 built-CLI affected cases. No fixes. One combined landing gate remains required.
