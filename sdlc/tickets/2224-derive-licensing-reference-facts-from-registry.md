---
priority: 3
---

# 2224 — Derive licensing reference facts from the registry

Status: OPEN.

Milestone: 0.9.4

Depends on: 2217

## Outcome

Each source's licensing facts in the reference agree with the registry, so a correction cannot leave a stale table or detail paragraph. Keep the registry as the source of truth and preserve distinct source terms and caveats.

- Evidence:2217 changed the Reactome registry and summary table, and twelve existing licensing-contract tests passed, while its detailed reference paragraph still said the old incompatible terms. Fresh review fixed the paragraph. Existing checks cover inventory/tier/presence but miss this drift; no licensing generator was found in the current tools/scripts inventory.
- Changes: `docs/reference/source-licensing.md` `tests/test_source_licensing_docs_contract.py` `tools/check-source-registry.py` `CHANGELOG.md`. Inspect existing docs tooling first and declare any replacement exact path before editing. Prefer deriving the summary table and detailed registry-owned facts with existing tooling, without a new framework or hardcoded license assertions.
- Proof: demonstrate one registry/reference drift that current checks miss, then make the existing contract detect it and verify all source entries. Generate registry-owned facts where practical and preserve hand-written contextual guidance. Run existing affected documentation/licensing checks; no new legal interpretation or source availability changes.

PM ruling: approved at priority3 in0.9.4, after2217; one source of truth is preferred when existing docs tooling can support it.

## Progress

- 2026-10-18 started
