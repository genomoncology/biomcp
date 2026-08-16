# Phenotype

Use phenotype commands to rank disease matches from HPO IDs or symptom phrases
via the Monarch Initiative similarity search.

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
search runs. A query may contain at most 10 unique HPO terms. Use `--limit` and
`--offset` within Monarch's first 50 ranked matches; `offset + limit` cannot
exceed 50. When that provider window is exhausted, BioMCP reports possible
truncation and asks you to refine the terms instead of emitting an unusable
continuation.

## Get records

Phenotype is search-only. There is no `get phenotype` subcommand.

## Request sections

Phenotype search rows do not expose extra section names. Use `search disease`
or `get disease <id> phenotypes` when you want a normalized disease follow-up.

## Helper commands

Phenotype is search-only. Start with `search phenotype` for HPO term sets or
symptom phrases, then switch to disease commands once you have the right
normalized concept. If you want to inspect candidate HPO terms first, run
`biomcp discover "<symptom text>"` and use the suggested `HP:` IDs.
Markdown phenotype search results add a `See also:` block that reuses the
top-ranked disease match, for example `biomcp get disease "Dravet syndrome"
genes phenotypes`. JSON phenotype search responses expose the same disease
follow-up plus `biomcp list phenotype` in `_meta.next_commands`.

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
