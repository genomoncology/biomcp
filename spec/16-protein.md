# Protein Queries

Protein is a first-class entity backed by UniProt. These checks verify search and
get behavior using stable structural markers — headings, table columns, and metadata
keys — rather than volatile upstream record values.

| Section | Command focus | Why it matters |
|---|---|---|
| Positional search | `search protein BRAF` | Confirms positional arg accepted (no `-q` flag) |
| Table structure | `search protein BRAF` | Confirms stable result schema with Accession column |
| Detail card | `get protein P15056` | Confirms card sections: Gene, Function, `More:`, UniProt link, related hints |
| Complexes section | `get protein P15056 complexes` | Confirms terminal-friendly complex summary rows and bounded member previews |
| JSON metadata | `get protein P15056 --json` | Confirms `_meta.evidence_urls` and `_meta.next_commands` are present |
| Complexes JSON metadata | `--json get protein P15056 complexes` | Confirms section-scoped next commands do not repeat the current complexes command |

## Positional Search Query

Protein search accepts a positional gene/keyword argument without the `-q` flag.
The heading should echo the query token and the result table should be non-empty.

```bash
out="$(biomcp search protein BRAF --limit 3)"
echo "$out" | mustmatch like "# Proteins: BRAF"
echo "$out" | mustmatch like "Found "
```

## Search Table Structure

Search results expose a consistent table with an Accession column that callers use to
look up detail cards. The canonical BRAF protein (P15056) should appear in the top
results, and a usage hint should guide the next step.

```bash
out="$(biomcp search protein BRAF --limit 3)"
echo "$out" | mustmatch like "| Accession | Name | Gene | Species |"
echo "$out" | mustmatch like "P15056"
echo "$out" | mustmatch like "get protein <accession>"
```

## Getting Protein Details

`get protein` should return a card with the key identity and biology fields. We assert
on stable structural markers: Gene line, Function section heading, the `More:` helper,
the related-command block, and the UniProt evidence link. Literal protein names or
functional annotations are not asserted as they drift with upstream record updates.

```bash
out="$(biomcp get protein P15056)"
echo "$out" | mustmatch like "Accession: P15056"
echo "$out" | mustmatch like "Gene: BRAF"
echo "$out" | mustmatch like "## Function"
echo "$out" | mustmatch like "More:"
echo "$out" | mustmatch like "[UniProt]("
echo "$out" | mustmatch like "See also:"
```

## Protein Complexes Section

`get protein <accession> complexes` should render a compact summary table plus one
detail bullet per complex. We assert the repaired table header, the absence of the old
wide `Components` column, a stable detail-bullet shape, and the absence of a
self-referential complexes follow-up command.

```bash
out="$(biomcp get protein P15056 complexes)"
echo "$out" | mustmatch like "Accession: P15056"
echo "$out" | mustmatch like "## Complexes"
echo "$out" | mustmatch like "| ID | Name | Members | Curation |"
echo "$out" | mustmatch not like "| ID | Name | Components | Curation |"
echo "$out" | mustmatch '/- `CPX-[0-9]+` members \([0-9]+\): /'
echo "$out" | mustmatch not like "biomcp get protein P15056 complexes"
```

## JSON Metadata Contract

JSON output must carry `_meta.evidence_urls` and `_meta.next_commands`, and both must
be non-empty. The UniProt source label is stable because it is the only evidence URL
emitted for protein entities.

```bash
out="$(biomcp get protein P15056 --json)"
echo "$out" | mustmatch like '"_meta": {'
echo "$out" | mustmatch like '"evidence_urls": ['
echo "$out" | mustmatch like '"label": "UniProt"'
echo "$out" | mustmatch like '"next_commands": ['
echo "$out" | mustmatch like 'biomcp get protein P15056'
echo "$out" | mustmatch like 'biomcp get protein P15056 complexes'
```

## Complexes JSON Next Commands

Section-scoped protein JSON should keep `_meta.next_commands`, but it should not echo the
exact complexes command the user already ran.

```bash
out="$(biomcp --json get protein P15056 complexes)"
echo "$out" | mustmatch like '"next_commands": ['
echo "$out" | mustmatch like 'biomcp get protein P15056 structures'
echo "$out" | mustmatch not like 'biomcp get protein P15056 complexes'
```
