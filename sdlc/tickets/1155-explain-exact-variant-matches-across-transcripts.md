---
flow: build
priority: 8
---

# Explain exact variant matches across transcripts

## Goal

An exact variant search explains why a result matched when the displayed transcript uses different HGVS descriptions. On 2026-09-04, `biomcp search variant -g HSD17B4 --hgvsp H540R --limit 10` matched `p.His540Arg` but displayed `NM_000414.3`, `c.1544A>G`, and `p.His515Arg`. The provider record contains the matching `NM_001199291.2`, `c.1619A>G`, and `p.His540Arg` annotation. BioMCP flattened the alternate annotations and lost their transcript relationship. The reproduction and source analysis appear in `sdlc/issues/feature-explain-exact-variant-matches-with-paired-transcript-annotations.md` at commit `84f2343f`.

## Desired functionality

Exact-search output preserves each transcript, coding HGVS, and protein HGVS combination supplied by the source. The output identifies the combination that satisfied the request and the combination selected for display. Human-readable output explains an alternate-transcript match when those combinations differ.

## Success criteria

- The fixed HSD17B4 example identifies `NM_001199291.2`, `c.1619A>G`, and `p.His540Arg` as one matching annotation.
- The same result identifies `NM_000414.3`, `c.1544A>G`, and `p.His515Arg` as the displayed annotation.
- Human-readable output explains the difference without implying that one transcript renumbered the other.
- JSON and MCP output preserve the paired annotations and identify their roles.
- Broad searches remain compact when the matched and displayed annotations agree.
- A fixed provider fixture proves the behavior without a live request.

## Boundaries

This ticket explains exact matches with the transcript data that the provider supplies. It does not choose a clinically preferred transcript, normalize HGVS independently, implement MANE policy, or change variant classification.
