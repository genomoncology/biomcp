# Discover

Use `biomcp discover` to resolve free-text biomedical phrases into the right
BioMCP follow-up commands. It is primarily a single-entity resolver for aliases,
brands, symptoms, and close concept names. Run it when you know the phrase but
do not yet know whether the next step should be `get gene`, `search disease`,
`search pathway`, or another typed command.

Use `search all` after you already have typed slots such as `--gene`,
`--disease`, `--drug`, `--variant`, or `--keyword`. `discover` resolves free
text into concepts first; `search all` fans out from the typed slots you
already trust. Relational or multi-entity questions may redirect to
`biomcp search all --keyword "<query>"` instead of surfacing weak collocation
matches as if they were a good discover answer.

## Examples

```bash
biomcp discover BRCA1
biomcp discover dabigatran
biomcp discover ERBB1
biomcp discover Keytruda
biomcp discover "chest pain"
biomcp discover "developmental delay"
biomcp --json discover diabetes
```

## What it does

- Queries OLS4 for structured ontology-backed matches.
- Adds optional UMLS crosswalks when `UMLS_API_KEY` is set.
- Adds MedlinePlus plain-language context for disease and symptom queries.
- Suggests `biomcp search phenotype "HP:..."` first when symptom concepts
  resolve to HPO-backed IDs.
- Keeps the existing discover-specific routed flows for symptom-of-disease,
  HPO symptom, treatment, gene+disease, and unambiguous gene-plus-topic prompts.
- Relational or multi-entity questions may redirect to
  `biomcp search all --keyword "<query>"` through `notes` and
  `_meta.next_commands` when only weak single-entity residue remains.
- If no entities resolve, suggests `biomcp search article -k "<query>" --type review --limit 5`.
- If only low-confidence concepts resolve, adds a broader-results article-search hint.
- Returns suggested BioMCP follow-up commands without auto-executing them.

## Output

Markdown groups concepts by type, shows notes, and lists suggested commands.

For symptom-first queries such as `biomcp discover "developmental delay"`,
discover can surface `HP:` identifiers directly in the concept list and suggest
`biomcp search phenotype "HP:..."` as the next command. Disease-specific
queries like `biomcp discover "symptoms of Marfan syndrome"` still route to
`biomcp get disease ... phenotypes`. Treatment prompts, gene+disease
orientation, and unambiguous gene-plus-topic follow-ups stay supported as
existing discover exceptions.

JSON preserves the same concepts, keeps the same guidance in `notes`, and adds:

- `_meta.next_commands`
- `_meta.section_sources`
- `_meta.discovery_sources`
- `_meta.evidence_urls`

`notes` is the user-visible guidance channel in both markdown and JSON.
`_meta.next_commands` remains the machine-actionable command list, including the
relational redirect to `biomcp search all --keyword "<query>"` when discover
declines to answer a multi-entity question directly.

## Notes

- OLS4 is required; if it fails, `discover` fails.
- UMLS is optional. Without `UMLS_API_KEY`, discover still works and reports
  that clinical crosswalk enrichment is unavailable.
- MedlinePlus is supplemental and only shown for disease or symptom flows.
- Queries are sent to third-party biomedical APIs. Do not send PHI or other
  patient-identifying text.

## Related guides

- [Search All Workflow](../how-to/search-all-workflow.md)
- [Disease](disease.md)
- [Gene](gene.md)
