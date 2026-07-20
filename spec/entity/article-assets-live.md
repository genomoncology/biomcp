# Live Article Supplement Assets

This operator-run contract checks that article-document supplement discovery
still works against real NCBI/PMC representations. It is intentionally outside
the routine fixture lane because upstream document availability and access
policy can change independently of BioMCP releases.

## PMID 20516115 linked supplements

This article names a PDF and a workbook in NCBI JATS and PMC HTML. Each named
file must remain visible as its own provider-labelled coverage result: either a
stable BioMCP handle retrieves it, or a specific typed outcome explains why it
cannot be retrieved. A generic package miss is not sufficient.

```bash
../../tools/biomcp-ci --json get article 20516115 assets | jq '
def acceptable:
  (.provider.source | type == "string" and length > 0) and
  (.source_document | type == "string" and length > 0) and
  ((.outcome == "retrievable" and (.handle | startswith("biomcp get article 20516115 asset "))) or
   (.outcome == "healthy_absent") or
   (.outcome == "access_or_licence_denied") or
   (.outcome == "unsupported_origin") or
   (.outcome == "source_unavailable"));
([.coverage[]? | select(.filename | endswith("Supplementary_Methods__Figures__Tables.pdf"))] | length == 1 and all(acceptable)) and
([.coverage[]? | select(.filename | endswith("Supplementary_Tables.xls"))] | length == 1 and all(acceptable))
' | mustmatch 'true'
```
