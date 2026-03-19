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
