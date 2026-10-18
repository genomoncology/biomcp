---
priority: 2
---

# 2196 — Keep disabled sources silent and named

Status: COMPLETE.

Milestone: 0.9.3

Reviews: revision d4c2c60, accept

Reviews: revision a578b61, accept

Reviews: revision d704ab3, accept

## Outcome

A disabled source is never contacted, including by `biomcp health`, and every place it would have appeared names the setting instead of a failure.

- Evidence: release review of 7025e117 with all sources disabled: `biomcp health` still contacts MyGene, MyVariant, MyChem, Monarch, HPO, MyDisease, HPA and the FDA orphan site, then lists them as never called. Disabled ClinVar, gnomAD and CIViC sections say "request failed", "temporarily unavailable" or "Retry". CIViC and ClinVar data carried inside MyVariant still appear, contrary to the changelog's "its carried fields drop". Disabled article sources show `"enabled": true, "status": "degraded"`, `complete: false` and WARN lines. The MCP override error blames the environment variable when the flag set it. The `commercial-restricted` group omits Cancer Genome Interpreter, whose recorded terms restrict commercial use, because it lacks the commercial terms field; check DisGeNET and AlphaGenome the same way.
- Cause: health builds its own client (`src/cli/health/runner.rs` near 344), matches rows by display name, and the FDA orphan probe uses a raw client.
- Changes: `tests/test_quality_ratchet_contract.py`, `CHANGELOG.md`, `docs/reference/configuration.md`, `docs/reference/source-licensing.md`, `docs/reference/sources.json`, `src/cli/health/catalog.rs`, `src/cli/health/runner.rs`, `src/cli/health/tests/runner.rs`, `src/entities/adverse_event.rs`, `src/entities/article/mod.rs`, `src/entities/article/search.rs`, `src/entities/article/search/status.rs`, `src/entities/cell_line.rs`, `src/entities/diagnostic/mod.rs`, `src/entities/disease/mod.rs`, `src/entities/drug/mod.rs`, `src/entities/gene.rs`, `src/entities/pathway.rs`, `src/entities/pgx.rs`, `src/entities/protein.rs`, `src/entities/section_outcome.rs`, `src/entities/source_state_registry.rs`, `src/entities/variant/clinvar.rs`, `src/entities/variant/get.rs`, `src/entities/variant/mod.rs`, `src/entities/variant/structure.rs`, `src/error.rs`, `src/render/markdown/mod.rs`, `src/render/markdown/sections.rs`, `src/render/markdown/variant.rs`, `src/render/provenance.rs`, `src/sources/disable.rs`, `src/sources/myvariant.rs`, `src/sources/myvariant/carried.rs`, `src/transform/article/federation.rs`, `src/transform/gene.rs`, `src/transform/pathway.rs`, `src/transform/protein.rs`, `tests/disable_sources_cli.rs`, `tests/test_source_licensing_docs_contract.py`. route health probes through the disabled-source check by source id; report disabled sections and article sources as disabled with the setting's name; drop carried fields of disabled sources; name the actual origin of an override; fix the terms records so the group matches its documented rule.
- Proof: traced-network outside-in cases: health with sources disabled makes no call to them; each named section reports disabled; carried fields absent; group membership matches the recorded terms. RED job2196/1792323075017-50525 reproduced health/section/group/origin failures. GREEN job2196/1792323566617-32155 passed20 outside-in CLI/MCP,9 policy,8 section and12 licensing cases; final unchanged-function relocation passed full lint and quality in2196/1792323847304-80949. Additive sections keep eligible evidence; centrally disabled means all registered providers are off. Official DisGeNET and hosted AlphaGenome terms support recorded groups; CGI retains existing restricted classification, its linked page refused retrieval. Combined gate b01ef9cb44853b1de322264632978fa75eec955e found one stale health-descriptor mutation (1280 other Python cases passed). Repair d704ab3 preserved checker enforcement and passed valid/missing/orphan cases plus mechanical checks in2196/1792324991220-78899; fresh repair review accepted.

## Progress

- 2026-10-18 started
