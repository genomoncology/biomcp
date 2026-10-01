---
flow: build
priority: 10
deps: [2001]
---

# Prepare primary MyGene consumer adoption

Status: design metadata prepared October 1, 2026; fresh independent design review pending. Owner: BioMCP migration coordinator. Preparation author: delegated Codex agent. The root coordinator assigned this bounded preparation while final maintenance hosted verification runs. Ticket 2007 records the BioMCP preparation lane. Existing BioData ticket 0182 remains the owning product scope.

Serve the accepted shared-source migration and usable gene lookup outcome. Use accepted BioData MyGene identity for primary get/search while retaining product acquisition, CLI/MCP envelopes, alias behavior, filtering, ordering, counts and source-only enrichment. Keep HGNC publisher authority distinct and preserve accepted strict Ensembl rejection with explicit compatibility loss.

The [design](../planning/mygene-consumer-2007/design.md) inventories current files and contracts, reconciles the [61 donor selectors](../planning/mygene-consumer-2007/selectors.tsv), records [static receipts](../planning/mygene-consumer-2007/static-evidence.json), and defines finite future offline proofs and timing receipts. Prior evidence comes from accepted BioData 0182 library, 0193 test adoption, 0198 numeric HGNC and 0213 Ensembl decisions. No new experiment is needed to prepare this metadata.

Preparation permits only ticket/design/record metadata and its routine commit/push. It does not authorize implementation, fixtures, runtime, compilation, providers, private data, hosted dispatch, or landing. Root verified maintenance hosted run [36849645376](https://github.com/genomoncology/biodata/actions/runs/36849645376) COMPLETED SUCCESS at exact `11904b5658d8208d4d6b8bb79a3bbf49a6954344` against `c9938b99bd091ab4bf6da8b909ed239826e5ab6d`. Root landed final BioData 0203 metadata on accepted main `59fde6246c1a07ac115d6fc9f16471354a725bff`; library source is unchanged from 18eb6e9. Root owns scoped cleanup. Implementation requires fresh design acceptance and root assignment against that accepted baseline. No additional human approval or release-owner handoff is introduced.

Preparation acceptance requires exact-base inspection, receipt digest matches, 44 retained / 10 rewritten / 1 omitted / 6 live outside the gate, correct integration-target selection, finite cross-surface errors, exact dependency boundaries, timing plans without invented measurements, and fresh read-only review of the pushed commit. Ian can overturn the compatibility defaults or source scope. Root can correct the implementation sequence within accepted 0182.

October1 correction: root added the missing selection-loader scope and mixed-target proof after fresh findings. The actual pinned BioData guard is685 seconds; fresh corrected review is pending. [Record](../records/2007-correct-selection-loader-design.md).
