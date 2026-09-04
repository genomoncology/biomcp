---
flow: build
priority: 9
---

# Distinguish phenotype similarity from direct support

## Goal

Phenotype search does not present ontology similarity as proof that a disease has the requested phenotype. On 2026-09-04, `biomcp --no-cache search phenotype HP:0000256 --limit 5` ranked isolated microcephaly first for a macrocephaly query. Monarch's direct disease associations contained microcephaly and did not contain macrocephaly. BioMCP still recommended the first row as the next disease to open. The reproduction and provider evidence appear in `sdlc/issues/2026-09-04-phenotype-similarity-ranks-an-opposite-phenotype-without-warning.md`.

## Desired functionality

Phenotype search identifies the resolved query terms and distinguishes a similarity candidate from a disease with direct source support for those terms. Human-readable, JSON, and MCP output do not describe an unsupported similarity candidate as a phenotype match. Suggested follow-up commands retain that distinction.

## Success criteria

- The fixed macrocephaly response does not recommend isolated microcephaly as an unqualified match.
- Every result states whether the source directly associates the disease with a submitted or resolved phenotype term.
- Human-readable, JSON, and MCP output include the submitted or resolved HPO identifiers and labels.
- Free-text macrocephaly search follows the same contract after term resolution.
- Provider failure during direct-support lookup remains visible and does not become a negative association.
- A fixed provider fixture proves the behavior without a live request.

## Boundaries

This ticket qualifies semantic-similarity results with public association evidence. It does not diagnose a patient, invent antonym rules for phenotype terms, change Monarch's similarity score, or rank a complete patient phenotype profile.
