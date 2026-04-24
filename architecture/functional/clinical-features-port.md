# Disease Clinical Features Architecture

This document describes the shipped disease `clinical_features` section. It is
current-state architecture for the MedlinePlus-backed clinical-summary rows
available through BioMCP disease detail commands.

## Current Surface

`get disease <name_or_id> clinical_features` is an explicit opt-in disease
section. Default disease cards stay concise, and `all` excludes
`clinical_features` so broad disease lookups do not trigger a live
MedlinePlus search or add a clinical-summary table unexpectedly.

The section is separate from the HPO/Monarch phenotype section. HPO/Monarch
phenotypes remain separate because they model ontology-backed phenotype
associations, while `clinical_features` provides reviewed clinical-summary
features extracted from MedlinePlus health topics for configured diseases.

## Source Selection

MedlinePlus Search is the live source. BioMCP queries the NLM
`/ws/query?db=healthTopics` endpoint with reviewed disease-specific source
queries, then deduplicates topics by URL before feature extraction.

The shipped configuration intentionally covers reviewed configured diseases
rather than attempting open-ended clinical summarization. Each configured
disease records source queries, accepted MedlinePlus topics, reviewed feature
labels, evidence text, and optional HPO mapping metadata.

An embedded fallback keeps the section deterministic when live MedlinePlus
searches return no usable topic rows or are unavailable. The fallback uses the
same reviewed topic and feature configuration, so offline behavior preserves
the same source and provenance contract instead of inventing rows.

## Runtime Flow

1. `get disease` resolves the requested disease through the normal disease
   identity path.
2. Section parsing recognizes `clinical_features` only when the caller names
   it explicitly.
3. The disease enrichment path checks the reviewed disease configuration for
   the resolved disease and the original lookup text.
4. If a configuration matches, BioMCP queries MedlinePlus Search, deduplicates
   candidate topics, and falls back to embedded reviewed topics when needed.
5. Feature extraction emits only reviewed configured rows, attaches evidence
   and source URLs, and writes rows to the disease card.
6. Unsupported diseases leave the section empty rather than fabricating
   symptoms or labels from the disease name.

## Output Contract

Clinical-feature rows are source-native MedlinePlus evidence rows. JSON exposes
the full row contract:

- rank/order for stable display
- feature label or name
- source label `MedlinePlus`
- MedlinePlus topic URL and source-native identifier when available
- evidence text and evidence tier
- normalized HPO ID and HPO label when reviewed mapping exists
- HPO mapping method and confidence when available

Markdown renders stable display columns for rank, feature, HPO mapping,
confidence, evidence, and linked source only when the section is requested.
JSON preserves requested rows under `clinical_features` and includes evidence
URLs in the shared provenance metadata. When rows are present,
`_meta.section_sources` includes the `clinical_features` section with the
MedlinePlus source label.

## Failure Behavior

The section degrades by truthful omission:

- Unsupported diseases produce a requested-section empty state and do not
  fabricate clinical features.
- Live MedlinePlus failures can still use the embedded fallback for reviewed
  configured diseases.
- If neither live nor embedded reviewed topics produce rows, markdown explains
  that no configured clinical features are available.
- The `all` section set remains unchanged; callers must ask for
  `clinical_features` directly.

## Security and Boundary Notes

`clinical_features` is a read-only entity section. It does not add a local sync
command, mutable operator workflow, API-key requirement, or MCP filesystem
surface. The only live network dependency is the bounded MedlinePlus Search
request, and the committed fallback is embedded in the binary with the rest of
the disease clinical-feature fixtures.

## Verification

The shipped behavior is covered by focused rendering and source tests while the
bootstrap spec-v2 canaries stay limited to gene, variant, and article:

- Disease rendering and provenance tests cover the MedlinePlus section heading,
  evidence URLs, and `_meta.section_sources`.
- The MedlinePlus source tests cover bounded request parsing and invalid input
  handling for the live search client.
