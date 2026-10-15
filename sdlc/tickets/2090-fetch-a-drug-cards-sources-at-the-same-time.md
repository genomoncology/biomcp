# 2090 — Fetch a drug card's sources at the same time

Status: OPEN.

Milestone: 0.9.3

Depends on: 2085

Reviews: revision a8abda6, accept

## Outcome

Once MyChem has named the drug, `biomcp get drug NAME` fetches the openFDA label, the CIViC context, the Open Targets data and the openFDA adverse-event preview at the same time, so the card waits for the slowest of them instead of their sum, and its output is byte-identical.

## Evidence

- A run-time investigation on the build host (2026-10-14) timed every network call of `get drug imatinib` through a logging local proxy. A warm run makes four calls one after the other, none cacheable: the openFDA label search (0.9 to 1.3 s), CIViC (about 0.2 s), Open Targets (about 0.2 s) and the openFDA adverse-event count (0.4 to 0.8 s). They are about 1.75 s of a 1.7 to 2.2 s run. The 2069 baseline saw the same command take 1.4 to 6.6 s, which the investigation could not reproduce; the likely cause is a slow openFDA reply or the retry middleware's back-off, so stacking two openFDA calls makes a slow moment count twice.
- None of the four depends on another's answer: CIViC and the adverse-event preview take `drug.name`, the target lookup takes `drug.chembl_id`, and `apply_openfda_metadata` changes neither.

- Starts from: `resolve_drug_base` in `src/entities/drug/get.rs`, which awaits `lookup_label_response` after MyChem selection; `populate_common_sections`, which awaits `fetch_civic_therapy_context` and later `enrich_targets` (`src/entities/drug/targets.rs`); `get_with_region_owned`, which awaits `populate_top_adverse_event_preview` (`src/entities/drug/metadata.rs`) after `populate_common_sections`; the `BIOMCP_OPENFDA_BASE`, `BIOMCP_CIVIC_BASE` and `BIOMCP_OPENTARGETS_BASE` overrides.
- Keeps: every section's content and outcome text; the 8-second `OPTIONAL_SAFETY_TIMEOUT` on CIViC and the adverse-event preview; the label's role in identity fallback and brand rescue before MyChem resolves; the order fields are merged into the card; which sections run for each `sections` and `region` argument.
- Changes: claims `src/entities/drug/get.rs` `src/entities/drug/metadata.rs` `src/entities/drug/targets.rs` `src/entities/drug/get/tests.rs` `CHANGELOG.md`.
  - Start the label lookup, the CIViC fetch, the target fetch and the adverse-event fetch as futures once the drug is named, join them, then apply their results in today's order. The target merge still uses the CIViC context after both finish.
  - Keep each fetch's existing guard (section flags, region, section-only requests); a skipped fetch is not started.
  - One Fixes changelog bullet: drug cards wait for their slowest source, not the sum.
- Proof: routine suite. A new test in `src/entities/drug/get/tests.rs` points the three base overrides at a local server that serves the recorded replies after a 1 s delay per request: the card completes in under 2 s (fails on current main at about 4 s), and its JSON and Markdown are identical to the output on the same replies with no delay. The existing drug card tests pass unchanged. On the build host, the reviewer runs `for i in $(seq 10); do BIOMCP_CACHE_MODE=infinite /usr/bin/time -f %e biomcp get drug imatinib | md5sum; done` on main and on the branch and records the medians and identical checksums.
- Defers: caching openFDA replies (2091).
- Order: runs alongside the build and test work. It edits `src/entities/drug/get.rs`, which 2085 and 2068 also edit. PM decision 2026-10-14: 2085 lands first, so 2090 rebases onto 2085 and lands before 2068 and 2091.

- Code review: ACCEPT 2026-10-15 by Coordinator Review

  Basis: the timed card test settles 1.05 s on the branch against the 2 s bound with the four ticket-named replies held, JSON and Markdown asserted identical to the undelayed run, guards keep never starting a fetch whose section flag or shape is off, and the baseline pins cover the grown modules; full-suite verification runs in the end-of-0.9.3 pass.
