---
flow: quickfix
priority: 5
---
# Quickfix: discover code-label canary credential-gates UMLS (green make verify without UMLS_API_KEY)

`spec/surface/discover.md::Normalize-to-Codes Playbook Uses Live Discover Code Labels` asserts that `biomcp --json discover "type 2 diabetes mellitus"` returns `SNOMEDCT` and `ICD10CM` source labels. Those labels only appear when `UMLS_API_KEY` is set; without it, `discover` gracefully degrades to the MONDO result and prints `UMLS enrichment unavailable (set UMLS_API_KEY)`. So `make verify` is red in any environment without the key. This canary was added alongside the normalize-to-codes example (#450). It is not a feature defect — it's a live-source/credential contract mismatch, exactly the failure mode the team's testing strategy forbids ("specs must fail for product regressions, not stale credentials"). Targets the 0.8.25 line (0.8.24 is already live). Tracked issue: `452-live-discover-code-label-canary-requires-umls`.

Completed under March on 2026-06-26, as March ticket 454. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/454-quickfix-discover-code-label-canary-credential-gates-umls-green-make-verify-without-umls-api-key
