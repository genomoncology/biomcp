# Phenotype

Use phenotype commands to resolve HPO IDs and labels, rank semantic-similarity
candidates through Monarch, and check the returned page for exact direct
phenotype associations. Similarity is not evidence that a disease has the
requested phenotype.

## Search phenotypes

By HPO identifiers (space- or comma-separated):

```bash
biomcp search phenotype "HP:0001250 HP:0001263"
biomcp search phenotype "HP:0001250,HP:0001263"
```

By one symptom phrase:

```bash
biomcp search phenotype "developmental delay"
```

By multiple symptom phrases (comma-separated):

```bash
biomcp search phenotype "seizure, developmental delay"
```

Multiple terms with limit:

```bash
biomcp search phenotype "HP:0001250 HP:0001263" --limit 20
```

The positional `terms` argument accepts:

- canonical HPO IDs, space- or comma-separated
- one symptom phrase
- multiple symptom phrases separated by commas

Free-text symptom phrases are resolved to HPO IDs before the Monarch similarity
search runs. Every phrase must resolve; BioMCP does not silently discard a
phrase with no HPO match. A query may contain at most 10 phrases and at most 10
unique resolved HPO terms. Output records the original phrase, normalized HPO
ID, and HPO label for every resolved term. Use `--limit` and
`--offset` within Monarch's first 50 ranked matches; `offset + limit` cannot
exceed 50. When that provider window is exhausted, BioMCP reports possible
truncation and asks you to refine the terms instead of emitting an unusable
continuation.

## Get records

Phenotype is search-only. There is no `get phenotype` subcommand.

## Request sections

Each candidate retains its semantic similarity score and reports one exact
direct-support state per resolved HPO term:

- `supported`: an exact, positive direct Monarch disease-to-phenotype row exists
- `not_supported`: a complete lookup contains no such row; this does not prove the disease lacks the phenotype
- `indeterminate`: truncation, missing fields, or inconsistent rows prevent a claim of absence
- `unavailable`: direct-support enrichment failed or exceeded its deadline; similarity results remain usable as candidates

Phenotype search rows do not expose extra section names. Use `search disease`
or `get disease <id> phenotypes` when you want a normalized disease follow-up.

## Helper commands

Phenotype is search-only. Start with `search phenotype` for HPO term sets or
symptom phrases, then switch to disease commands once you have the right
normalized concept. If you want to inspect candidate HPO terms first, run
`biomcp discover "<symptom text>"` and use the suggested `HP:` IDs.
Markdown and JSON suggest `biomcp get disease <MONDO_ID> phenotypes` only for
the first provider-ordered candidate that is `supported` for every resolved
term. If no row meets that rule, the disease follow-up is suppressed rather
than falling back to the first similarity result. Pagination remains the first
continuation when present, and `biomcp list phenotype` remains available.

## JSON mode

```bash
biomcp --json search phenotype "HP:0001250"
```

## Practical tips

- Use HPO IDs for precise lookups when you know the exact term.
- Use commas to separate multiple symptom phrases in one search.
- Combine multiple HPO IDs in a single query to retrieve a phenotype set.
- Prefer 2-5 high-confidence HPO IDs when you already know them.

## Related guides

- [Gene](gene.md)
- [Disease](disease.md)
- [GWAS](gwas.md)
