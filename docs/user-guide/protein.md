# Protein

Use protein commands to query UniProt accessions and expand into domains, interactions, complexes, and structure IDs.

## Search proteins

```bash
biomcp search protein -q kinase --limit 5
```

Search is human-only and reviewed-only by default. Species and review scope are
independent: add `--all-species` without changing review quality, or add
`--include-unreviewed` without changing the human scope. The two flags may be
combined. `--reviewed` remains an explicit spelling of the default and conflicts
with `--include-unreviewed`.

## Get protein records

```bash
biomcp get protein P15056
```

## Request protein sections

Domains:

```bash
biomcp get protein P15056 domains
```

Interactions:

```bash
biomcp get protein P15056 interactions
```

Complexes:

```bash
biomcp get protein P15056 complexes
```

Complexes render as a narrow summary table first, then one bounded member-preview bullet
per complex so long names and large memberships stay readable in a terminal.

```text
## Complexes

| ID | Name | Members | Curation |
|---|---|---:|---|
| CPX-13454 | BRAF:DELE1 stress-response complex | 2 | predicted |
- `CPX-13454` members (2): DELE1, BRAF
```

Structures:

```bash
biomcp get protein P15056 structures
```

## Helper commands

```bash
biomcp protein structures P15056
```

## JSON mode

```bash
biomcp --json get protein P15056 all
```

## Optional-section outcomes

JSON and MCP protein records include outcomes for `domains`, `interactions`,
`complexes`, and `structures`. Unrequested keys are `not_requested`; a requested
empty vector is a confirmed zero only when its outcome is `empty`. Provider
failure is `unavailable` and is also reported in Markdown.

## Practical tips

- Use a UniProt accession when you need the most stable exact lookup.
- Follow JSON `pagination.next_offset` while `has_more` is true; it is omitted at the end.
- Request only the section you need first, especially for `interactions` and `complexes`.
- Use `protein structures` when the next step is a structure handoff rather than a full protein card.

## Related guides

- [Gene](gene.md)
- [Pathway](pathway.md)
