---
base: 4146574e
head: b64ca817
---

All optional gene enrichment checks now run in the routine offline gene page,
so `gene-live.md` and its live-registry entries are gone. The shared fixture
serves dated, receipted QuickGO, STRING, HPA, DGIdb, Open Targets, NIH
RePORTER, and MyGene responses, plus the bounded local GTR bundle. HTTP routes
validate the expected identifiers or request bodies and fail closed otherwise.

The executable page covers typed GO/interaction outcomes, HPA tissue and
subcellular rendering, combined EGFR druggability, bounded ERBB2 funding,
BRCA1 diagnostic output, and observed GET/POST requests. The separate NIH
live lane now retains only disease coverage instead of duplicating the routine
gene proof. Existing provider construction, decoder, orchestration, failure,
empty, and not-configured tests remain intact.

Verification passed: all eleven gene blocks and 74 fixture, receipt, registry,
and planning-contract tests. No source lines were added against the 150-line
ceiling.
