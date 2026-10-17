# LitVar2 fixtures

## `autocomplete_cdkn2a_a57v.json`

Captured from `https://www.ncbi.nlm.nih.gov/research/litvar2-api/variant/autocomplete/?query=CDKN2A%20A57V` on 2026-10-17 for ticket 2131. Provider-faithful capture of the one variant entry the query returns: `_id` `litvar@rs372266620##`, `gene` `["CDKN2A"]`, `name`/`hgvs` `p.A57V`, and `pmids_count` 39. The identity-match and merge tests depend on this provider shape.

## `autocomplete_rsid_rs113488022.json`

Captured from `https://www.ncbi.nlm.nih.gov/research/litvar2-api/variant/autocomplete/?query=rs113488022` on 2026-10-17 for ticket 2131. Provider-faithful capture of the rsID query's answer: the same `_id` shape as above with `rsid` `rs113488022`, `gene` `["BRAF"]`, `name` `p.V600E`, `pmids_count` 32227, and a `match` field naming the rsID hit. The rsID-path tests depend on this shape.

## `publications_rs372266620.json`

Captured from `https://www.ncbi.nlm.nih.gov/research/litvar2-api/variant/get/litvar%40rs372266620%23%23/publications` on 2026-10-17 for ticket 2131. Provider-faithful capture of the CDKN2A p.A57V publication list: 39 `pmids`, 34 `pmcids`, and `pmids_count` 39. Note the provider sends `pmcids` as one unkeyed list, so BioMCP consumes only `pmids`.

## `publications_rs113488022.json`

Minimized capture from `https://www.ncbi.nlm.nih.gov/research/litvar2-api/variant/get/litvar%40rs113488022%23%23/publications` on 2026-10-17 for ticket 2131. The full reply carries 32,227 `pmids` and 26,421 `pmcids` (641,714 bytes); the committed bytes keep the first 100 of each list and the provider's `pmids_count`, and the receipt records the original full-response SHA-256. The fixture pins the large-list shape without committing the full payload.

## `autocomplete_samhd1_i448t_20261017.json`

Captured from `https://www.ncbi.nlm.nih.gov/research/litvar2-api/variant/autocomplete/?query=SAMHD1%20I448T` on 2026-10-17 for ticket 2140. Provider-faithful capture of the one variant entry the query returns: `_id` `litvar@rs774964432##`, `gene` `["SAMHD1"]`, `name`/`hgvs` `p.I448T`, `pmids_count` 5. The genomic-alias tie tests depend on this shape.

## `publications_rs774964432_20261017.json`

Captured from `https://www.ncbi.nlm.nih.gov/research/litvar2-api/variant/get/litvar%40rs774964432%23%23/publications` on 2026-10-17 for ticket 2140. The five papers LitVar2 links to rs774964432 (27397505, 28229507, 34480199, 32371413, 30275001) with their PMCIDs.
