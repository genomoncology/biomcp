# Article

Use article commands for literature retrieval by disease, gene, drug, and identifier.

## Typical article workflow

1. search a topic,
2. choose an identifier,
3. retrieve default summary,
4. request full text or annotations only when needed.

## Search articles

By gene and disease:

```bash
biomcp search article -g BRAF -d melanoma --limit 5
```

By keyword:

```bash
biomcp search article -k "immunotherapy resistance" --limit 5
```

By date:

```bash
biomcp search article -g BRAF --since 2024-01-01 --limit 5
```

Exclude preprints when supported by source metadata:

```bash
biomcp search article -g BRAF --since 2024-01-01 --no-preprints --limit 5
```

### Multi-source federation

Article search fans out to PubTator3 and Europe PMC in parallel by default.
Results are deduplicated by PMID when both backends return the same paper.
Output is grouped by source; PubTator rows include a score column.

Use `--source <all|pubtator|europepmc>` to select one backend or keep the default federated search.

To search a single backend:

```bash
biomcp search article -g BRAF --source pubtator --limit 5
biomcp search article -g BRAF --source europepmc --limit 5
```

## Get an article

Supported IDs are PMID (digits only), PMCID (e.g., PMC9984800), and DOI
(e.g., 10.1056/NEJMoa1203421). Publisher PIIs (e.g., `S1535610826000103`) are not
indexed by PubMed or Europe PMC and cannot be resolved.

```bash
biomcp get article 22663011
```

When `S2_API_KEY` is set, default article output also includes an optional
Semantic Scholar section with TLDR text, influence counts, and open-access PDF
metadata when that paper resolves in Semantic Scholar. `search article` still
fans out only to PubTator3 and Europe PMC; Semantic Scholar is enrichment and
navigation, not federated search.

## Request specific sections

Full text section:

```bash
biomcp get article 22663011 fulltext
```

Annotation section:

```bash
biomcp get article 22663011 annotations
```

Semantic Scholar TLDR section:

```bash
biomcp get article 22663011 tldr
```

## Helper commands

```bash
biomcp article entities 22663011   # extract annotated entities via PubTator
biomcp article citations 22663011 --limit 3         # Semantic Scholar citation graph
biomcp article references 22663011 --limit 3        # Semantic Scholar reference graph
biomcp article recommendations 22663011 --limit 3   # Semantic Scholar related papers
```

These Semantic Scholar helper commands require `S2_API_KEY`. Without the key,
ordinary `get article` still works, but the explicit helper commands return an
API-key-required error. Citations usually work broadly; references and
recommendations can be sparse or empty for paywalled papers because of
publisher elision in the Semantic Scholar graph.

## Caching behavior

Downloaded content is stored in the BioMCP cache directory.
This avoids repeated large payload downloads during iterative workflows.

## JSON mode

```bash
biomcp --json get article 22663011
```

JSON article responses include `_meta.next_commands`, so article workflows can
promote the next likely pivots without scraping markdown. JSON-capable article
follow-ups preserve the same next-step guidance shape.

## Practical tips

- Start with narrow `--limit` values.
- Add a disease term when gene-only search is too broad.
- Use section requests to avoid oversized responses.
- Use `biomcp get article <id> tldr` when you want only the optional Semantic Scholar section.

## Related guides

- [Gene](gene.md)
- [Trial](trial.md)
- [How to find articles](../how-to/find-articles.md)
