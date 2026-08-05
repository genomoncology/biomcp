# Live Article Supplement Assets

This operator-run contract checks that named supplement bytes remain available
from real NCBI/PMC representations. It is intentionally outside the routine
fixture lane because upstream document availability and access policy can change
independently of BioMCP releases.

## PMID 20516115 linked supplements

A real binary is retrievable only when its received media type is not
HTML/XHTML and has a stable BioMCP handle. A generic package miss is not
sufficient.

```bash
../../tools/biomcp-ci --json get article 20516115 assets | jq '
def retrievable_asset:
  (.handle | startswith("biomcp get article 20516115 asset ")) and
  (.size_bytes > 0) and
  (.sha256 | test("^[0-9a-f]{64}$")) and
  ((.media_type | type == "string") and
   ((ascii_downcase != "text/html") and (ascii_downcase != "application/xhtml+xml")));
def retrievable_file($suffix):
  any(.assets[]?; (.filename | endswith($suffix)) and retrievable_asset);
(.pmid == "20516115") and
 retrievable_file("Supplementary_Methods__Figures__Tables.pdf") and
 retrievable_file("Supplementary_Tables.xls")
' | mustmatch 'true'
```
