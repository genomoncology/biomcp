# BioMCP jq Examples

These patterns show common extraction workflows for BioMCP JSON output.

```bash
biomcp --json get gene BRAF | jq -r '.symbol'
biomcp --json get variant rs113488022 all | jq -r '.gene'
biomcp --json search article -g BRAF -d melanoma --limit 5 | jq -r '.results[].pmid'
biomcp --json search trial -c melanoma --status recruiting --limit 5 | jq -r '.results[].nct_id'
biomcp --json search drug --target BRAF --limit 5 | jq -r '.results[].name'
biomcp --json search variant -g TP53 --limit 20 | jq '.results[] | select(.significance == "pathogenic")'
biomcp --json get trial NCT02576665 eligibility | jq -r '.eligibility.inclusion'
biomcp --json search article -g EGFR --limit 20 | jq '.count'
biomcp --json enrich BRAF,KRAS,NRAS --limit 3 | jq -r '.results[] | "\(.source) \(.native) \(.name)"'
biomcp --json get disease melanoma pathways | jq -r '.pathways[]?.id'
```
