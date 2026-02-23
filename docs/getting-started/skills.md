# Skills (Embedded Use Cases)

BioMCP bundles reusable workflow documents as embedded skills.

Skills are available to both CLI users and MCP clients via resources.

## List skills

```bash
biomcp skill
biomcp skill list
```

## Open a specific skill

By number:

```bash
biomcp skill 01
```

By slug:

```bash
biomcp skill variant-to-treatment
```

## Install into an agent directory

```bash
biomcp skill install ~/.claude
```

Force replace existing files:

```bash
biomcp skill install ~/.claude --force
```

When no directory is provided, BioMCP attempts supported agent-directory detection.

## Skill topics included

| # | Slug | Focus |
|---|------|-------|
| 01 | `variant-to-treatment` | Variant to treatment/evidence workflow |
| 02 | `drug-investigation` | Drug mechanism, safety, alternatives |
| 03 | `trial-searching` | Trial discovery + patient matching |
| 04 | `rare-disease` | Rare disease evidence and trial strategy |
| 05 | `drug-shortages` | Shortage monitoring and alternatives |
| 06 | `advanced-therapies` | CAR-T/checkpoint therapy workflows |
| 07 | `hereditary-cancer` | Hereditary syndrome workflows |
| 08 | `resistance` | Resistance and next-line options |
| 09 | `gene-function-lookup` | Gene-centric function and context lookup |
| 10 | `gene-set-analysis` | Enrichment + pathway + interaction synthesis |
| 11 | `literature-synthesis` | Evidence synthesis with cross-entity checks |
| 12 | `pharmacogenomics` | PGx gene-drug interactions and dosing |
| 13 | `phenotype-triage` | Symptom-first rare disease workup |
| 14 | `protein-pathway` | Protein structure and pathway deep dive |

## Relationship to MCP resources

The same use cases are readable through `resources/list` and `resources/read` with
`biomcp://skill/<slug>` URIs.
