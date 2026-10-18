---
priority: 3
---

# 2224 — Derive licensing reference facts from the registry

Status: OPEN.

Milestone: 0.9.4

Depends on: 2217

Reviews: revision c892df7, accept

Reviews: revision 61e8d2b, accept

Reviews: revision 61e8d2b, reject

## Outcome

Each source's licensing facts in the reference agree with the registry, so a correction cannot leave a stale table or detail paragraph. Keep the registry as the source of truth and preserve distinct source terms and caveats.

- Evidence:2217 changed the Reactome registry and summary table, and twelve existing licensing-contract tests passed, while its detailed reference paragraph still said the old incompatible terms. Fresh review fixed the paragraph. Existing checks cover inventory/tier/presence but miss this drift; no licensing generator was found in the current tools/scripts inventory.
- Changes: `docs/reference/source-licensing.md` `tests/test_source_licensing_docs_contract.py` `tools/render-source-licensing.py` `docs/reference/sources.json` `CHANGELOG.md`. Inspect existing docs tooling first and declare any replacement exact path before editing. Prefer deriving the summary table and detailed registry-owned facts with existing tooling, without a new framework or hardcoded license assertions.
- Proof: demonstrate one registry/reference drift that current checks miss, then make the existing contract detect it and verify all source entries. Generate registry-owned facts where practical and preserve hand-written contextual guidance. Run existing affected documentation/licensing checks; no new legal interpretation or source availability changes.

PM ruling: approved at priority3 in0.9.4, after2217; one source of truth is preferred when existing docs tooling can support it.

## Progress

- 2026-10-18 started

Approved pre-code plan: a small standard-library renderer owns marked summary, commercial-group membership and per-source fact blocks, including Notes, from the registry. Preserve generic guidance and unique contextual caveats; move approved unique existing caveats verbatim into the existing registry notes field while retaining registry-only context. Do not change license/terms/tier/auth/access decisions. Existing tier/heading checks remain; a check mode rejects stale/missing/duplicate blocks and write mode regenerates. Extend the existing licensing contract to check rendering, then demonstrate current checks miss registry-only license/reuse/Notes drift while the new contract detects it and regeneration repairs it. The unrelated Rust source/health audit tool stays untouched. Genuine contradictory terms require a report, not new interpretation.
