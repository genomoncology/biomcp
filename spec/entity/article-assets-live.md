# Live Article Supplement Assets

This operator-run contract checks that article-document supplement discovery
still works against real NCBI/PMC representations. It is intentionally outside
the routine fixture lane because upstream document availability and access
policy can change independently of BioMCP releases.

## PMID 20516115 linked supplements

This article names a PDF and a workbook in NCBI JATS and PMC HTML. Each named
file must remain visible as its own provider-labelled coverage result. A real
binary is retrievable only when its received media type is not HTML/XHTML and
has a stable BioMCP handle; PMC's proof-of-work gate instead remains visible as
`pmc_proof_of_work` without publishing challenge bytes. A generic package miss
is not sufficient.

```bash
../../tools/biomcp-ci --json get article 20516115 assets | jq '
def named_coverage:
  (.provider.source | type == "string" and length > 0) and
  (.source_document | type == "string" and length > 0);
def retrievable_asset:
  (.handle | startswith("biomcp get article 20516115 asset ")) and
  (.size_bytes > 0) and
  (.sha256 | test("^[0-9a-f]{64}$")) and
  ((.media_type | type == "string") and
   ((ascii_downcase != "text/html") and (ascii_downcase != "application/xhtml+xml")));
def guarded_file($suffix):
  (any(.coverage[]?; (.filename | endswith($suffix)) and named_coverage and (.outcome == "pmc_proof_of_work")) and
   all(.assets[]?; (.filename | endswith($suffix)) | not)) or
  (any(.coverage[]?; (.filename | endswith($suffix)) and named_coverage and (.outcome == "retrievable")) and
   any(.assets[]?; (.filename | endswith($suffix)) and retrievable_asset));
(.pmid == "20516115") and
 guarded_file("Supplementary_Methods__Figures__Tables.pdf") and
 guarded_file("Supplementary_Tables.xls")
' | mustmatch 'true'
```
