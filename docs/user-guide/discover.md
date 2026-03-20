# Discover

`biomcp discover <query>` is the free-text entrypoint for concept resolution.
Use it when you know the biomedical phrase but do not yet know whether the next
step should be `get gene`, `search disease`, `search pathway`, or another typed
command.

## Examples

```bash
biomcp discover ERBB1
biomcp discover Keytruda
biomcp discover "chest pain"
biomcp --json discover diabetes
```

## What it does

- Queries OLS4 for structured ontology-backed matches.
- Adds optional UMLS crosswalks when `UMLS_API_KEY` is set.
- Adds MedlinePlus plain-language context for disease and symptom queries.
- Returns suggested BioMCP follow-up commands without auto-executing them.

## Output

Markdown groups concepts by type and shows suggested commands.

JSON preserves the same concepts and adds:

- `_meta.next_commands`
- `_meta.section_sources`
- `_meta.discovery_sources`
- `_meta.evidence_urls`

## Notes

- OLS4 is required; if it fails, `discover` fails.
- UMLS is optional. Without `UMLS_API_KEY`, discover still works and reports
  that clinical crosswalk enrichment is unavailable.
- MedlinePlus is best-effort and only shown for disease or symptom flows.
- Queries are sent to third-party biomedical APIs. Do not send PHI or other
  patient-identifying text.
