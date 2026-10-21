# Dated gene and keyword PubTator replies

These responses were captured from the official PubTator3 API on 2026-10-21. `autocomplete.json` answers `/entity/autocomplete/?query=CHEK2&limit=10`. Both search responses use `/search/` with `page=1&size=25`. `ungrouped.json` answers `text=@GENE_CHEK2 G167R OR Gly167Arg` and reports 17,471 hits across 699 pages. `grouped.json` answers `text=@GENE_CHEK2 AND (G167R OR Gly167Arg)` and reports 15 hits on one page. The response bodies preserve provider metadata. Authentication parameters are omitted from these request descriptions.

The CLI regression uses these replies to distinguish precise provider work from an accidentally broad gene search. Its companion source row and held later-page reply are explicit local fixture overlays.
