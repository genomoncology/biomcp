---
base: 609590bf98093f1fdc129727917a06c46c477205
head: 4097519700935e1c1c3e870501dc715ca91160c2
---
`biomcp search all --counts-only` advertises a counts-first orientation mode, but the implementation still executes full per-section fetches and the JSON surface returns the full `results` payload. This is both a correctness defect (the JSON response does not match the semantic promise of `--counts-only`) and a performance defect (the hot path pays full fan-out cost even when the caller explicitly asked for a lightweight summary). Agent callers using `--counts-only` for orientation get inflated token volume with no benefit.

Imported from March ticket 206. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/206-fix-search-all-counts-only-json-contract-and-performance
