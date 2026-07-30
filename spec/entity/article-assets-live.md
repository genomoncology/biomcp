# Live Article Supplement Assets

This operator-run contract checks that article-document supplement discovery
still works against real NCBI/PMC representations. It is intentionally outside
the routine fixture lane because upstream document availability and access
policy can change independently of BioMCP releases.

## PMID 20516115 linked supplements

This article names a PDF and a workbook in NCBI JATS and PMC HTML. Each named
file must remain visible as its own provider-labelled coverage result and as a
retrievable asset with a stable BioMCP handle. A generic package miss is not
sufficient.

```bash
../../tools/biomcp-ci --json get article 20516115 assets | jq '
def acceptable_coverage:
  (.provider.source | type == "string" and length > 0) and
  (.source_document | type == "string" and length > 0) and
  (.outcome == "retrievable");
def retrievable_asset:
  (.handle | startswith("biomcp get article 20516115 asset ")) and
  (.size_bytes > 0) and (.sha256 | test("^[0-9a-f]{64}$"));
(.pmid == "20516115") and
any(.coverage[]?; (.filename | endswith("Supplementary_Methods__Figures__Tables.pdf")) and acceptable_coverage) and
any(.coverage[]?; (.filename | endswith("Supplementary_Tables.xls")) and acceptable_coverage) and
any(.assets[]?; (.filename | endswith("Supplementary_Methods__Figures__Tables.pdf")) and retrievable_asset) and
any(.assets[]?; (.filename | endswith("Supplementary_Tables.xls")) and retrievable_asset)
' | mustmatch 'true'
```
