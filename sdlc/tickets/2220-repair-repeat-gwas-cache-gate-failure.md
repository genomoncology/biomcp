---
priority: 1
---

# 2220 — Repair the repeat GWAS cache gate failure

Status: OPEN.

Milestone: 0.9.4

## Outcome

Explain and fix the repeat-search cache failure that stopped the unrelated Windows review landing. Preserve the outside-in claim that a same-day repeat search with an isolated cache returns the same rows without another provider request.

- Evidence: landing candidate2201 job1792353069557-23636 failed243.003s in `tests/test_gwas_day_cache_contract.py::test_a_repeat_gwas_search_inside_a_day_makes_no_request`: observed2 requests, expected1;1285 Python cases passed and10 skipped. Candidate source tree0f5c6dfed19faa01abaf17e826c04a8e794b3ba1. Main product3dc had previously passed its full check. Windows changes do not edit GWAS or cache production code.
- Changes: `tests/test_gwas_day_cache_contract.py` `CHANGELOG.md`. Begin with diagnosis and existing failing case; declare the actual source/helper paths before changing them if the cause requires it. No broad cache redesign or retries that mask a failure.
- Proof: use the saved failing gate log, reproduce on current main through a focused build-machine job with the same artifact/environment where possible, then run the existing affected test after the correction. Distinguish a product defect from a leaky or scheduling-dependent test. Reuse qualified artifacts rather than repeat a full suite during diagnosis; one reviewed repair and landing qualification close the hold.
