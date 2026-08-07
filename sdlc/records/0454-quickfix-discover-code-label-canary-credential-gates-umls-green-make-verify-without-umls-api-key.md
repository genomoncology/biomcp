---
base: 7c1b746ff091782f6ead63172066580fdd7b9eef
head: 1cf027d02d61a8ed5d7af328b13f896273a0aae8
---
`spec/surface/discover.md::Normalize-to-Codes Playbook Uses Live Discover Code Labels` asserts that `biomcp --json discover "type 2 diabetes mellitus"` returns `SNOMEDCT` and `ICD10CM` source labels. Those labels only appear when `UMLS_API_KEY` is set; without it, `discover` gracefully degrades to the MONDO result and prints `UMLS enrichment unavailable (set UMLS_API_KEY)`. So `make verify` is red in any environment without the key. This canary was added alongside the normalize-to-codes example (#450). It is not a feature defect — it's a live-source/credential contract mismatch, exactly the failure mode the team's testing strategy forbids ("specs must fail for product regressions, not stale credentials"). Targets the 0.8.25 line (0.8.24 is already live). Tracked issue: `452-live-discover-code-label-canary-requires-umls`.

Imported from March ticket 454. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/454-quickfix-discover-code-label-canary-credential-gates-umls-green-make-verify-without-umls-api-key
