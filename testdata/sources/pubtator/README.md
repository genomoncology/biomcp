# PubTator fixtures

## `export_22663011.json`

Captured from `https://www.ncbi.nlm.nih.gov/research/pubtator3-api/publications/export/biocjson?pmids=22663011` on 2026-07-30. This is a provider-faithful capture: it retains the Gene, Disease, Chemical, Species, and Variant annotation shapes returned for the article, including numeric and string `normalized_id` values. Do not minimize away returned annotation kinds; the Tier 3 parser contract depends on these provider shapes.

## `export_30738221.json`

Captured from `https://www.ncbi.nlm.nih.gov/research/pubtator3-api/publications/export/biocjson?pmids=30738221` on 2026-10-05 for ticket 1296. Provider-faithful capture of the KRAS G12C non-small-cell lung cancer article: NCBI Gene identifiers, `MESH:D002289` and `MESH:D009369` disease identifiers, tmVar composite variant identifiers with `rsid`, `rsids`, and `hgvs` infons, and document-global `locations` offsets. The entity identifier spec page and the annotation aggregation tests depend on these provider shapes.

## `export_37887282.json`

Captured from `https://www.ncbi.nlm.nih.gov/research/pubtator3-api/publications/export/biocjson?pmids=37887282` on 2026-10-08 for ticket 2034. Provider-faithful capture of an article whose every human KRAS mention spells the gene "K-Ras" or "K-RAS" (NCBI Gene 3845) while its variant rows carry `gene_id`, `hgvs`, and `rsid` infons. The entity identifier spec page and the annotation aggregation tests use it to pin the official-symbol fallback that keeps spelled-gene rows on the gene-plus-protein form.
