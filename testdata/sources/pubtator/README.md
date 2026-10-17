# PubTator fixtures

## `autocomplete_abcc8_r1436q.json`

Captured from `https://www.ncbi.nlm.nih.gov/research/pubtator3-api/entity/autocomplete/?query=ABCC8%20R1436Q` on 2026-10-17 for ticket 2138. Provider-faithful capture of the one variant entity a gene-and-protein autocomplete returns: the `_id` carries the entity id `@VARIANT_p.R1436Q_ABCC8_human` while the display `name` holds only the protein-change label `p.R1436Q` (the gene sits in the entity id and the description). The variant-entity route tests depend on this provider shape.

## `search_variant_abcc8_r1436q.json`

Captured from `https://www.ncbi.nlm.nih.gov/research/pubtator3-api/search/?text=%40VARIANT_p.R1436Q_ABCC8_human&page=1&size=25` on 2026-10-17 for ticket 2138. Provider-faithful capture of the seven papers PubTator annotates with the ABCC8 p.R1436Q variant entity. The variant-entity route tests return these papers when the annotation strategy searches by the entity id.

## `autocomplete_rsid_rs113488022.json`

Captured from `https://www.ncbi.nlm.nih.gov/research/pubtator3-api/entity/autocomplete/?query=rs113488022` on 2026-10-16 for ticket 2130. Provider-faithful capture of the one variant entity an rsID autocomplete returns: the `db_id` carries the normalized dbSNP id before `##` (`rs113488022##`) while the display name holds the protein-change label `p.M600E`. The rsID entity-resolution tests depend on this provider shape.

## `export_22663011.json`

Captured from `https://www.ncbi.nlm.nih.gov/research/pubtator3-api/publications/export/biocjson?pmids=22663011` on 2026-07-30. This is a provider-faithful capture: it retains the Gene, Disease, Chemical, Species, and Variant annotation shapes returned for the article, including numeric and string `normalized_id` values. Do not minimize away returned annotation kinds; the Tier 3 parser contract depends on these provider shapes.

## `export_30738221.json`

Captured from `https://www.ncbi.nlm.nih.gov/research/pubtator3-api/publications/export/biocjson?pmids=30738221` on 2026-10-05 for ticket 1296. Provider-faithful capture of the KRAS G12C non-small-cell lung cancer article: NCBI Gene identifiers, `MESH:D002289` and `MESH:D009369` disease identifiers, tmVar composite variant identifiers with `rsid`, `rsids`, and `hgvs` infons, and document-global `locations` offsets. The entity identifier spec page and the annotation aggregation tests depend on these provider shapes.

## `export_37887282.json`

Captured from `https://www.ncbi.nlm.nih.gov/research/pubtator3-api/publications/export/biocjson?pmids=37887282` on 2026-10-08 for ticket 2034. Provider-faithful capture of an article whose every human KRAS mention spells the gene "K-Ras" or "K-RAS" (NCBI Gene 3845) while its variant rows carry `gene_id`, `hgvs`, and `rsid` infons. The entity identifier spec page and the annotation aggregation tests use it to pin the official-symbol fallback that keeps spelled-gene rows on the gene-plus-protein form.
