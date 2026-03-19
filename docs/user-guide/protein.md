# Protein

Use protein commands to query UniProt accessions and expand into domains, interactions, complexes, and structure IDs.

## Search proteins

```bash
biomcp search protein -q kinase --limit 5
```

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

## Helper command

```bash
biomcp protein structures P15056
```

## JSON mode

```bash
biomcp --json get protein P15056 all
```

## Related guides

- [Gene](gene.md)
- [Pathway](pathway.md)
