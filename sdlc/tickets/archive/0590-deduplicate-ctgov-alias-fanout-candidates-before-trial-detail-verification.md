---
flow: build
priority: 6
---
# Deduplicate CTGov alias-fanout candidates before trial detail verification

CTGov intervention alias searches can fetch the same trial detail more than once. `search_page_with_ctgov_union` runs each alias worker concurrently through `fetch_ctgov_filtered_page`; each worker applies detail-backed post-filters **before** the union path deduplicates by NCT ID. The per-worker verifier is bounded to eight requests, but concurrent workers can each fetch the same NCT detail, so the same trial detail is requested once per alias and the aggregate bound is exceeded. Ticket 580 correctly reduced geo-plus-eligibility verification from two detail requests to one *within* each worker, but moving deduplication ahead of detail verification was outside its named per-filter reuse slice. Confirms open issue `580-alias-fanout-repeats-detail-verification` (performance / redundant provider I/O).

Completed under March on 2026-07-19, as March ticket 590. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/590-deduplicate-ctgov-alias-fanout-candidates-before-trial-detail-verification
