# Search All: gene=BRAF disease=melanoma

## Genes (1)
|Symbol|Name|Entrez|
|---|---|---|
|BRAF|B-Raf proto-oncogene, serine/threonine kinase|673|

- `biomcp get gene BRAF`
- `biomcp search trial --biomarker BRAF`

## Variants (42)
|ID|Gene|Protein|Significance|
|---|---|---|---|
|chr7:g.140449165A>C|BRAF|p.Asp638Glu|Pathogenic|
|chr7:g.140476832A>G|BRAF|p.Leu525Pro|Pathogenic|
|chr7:g.140534497T>G|BRAF|p.Gln139Pro|Uncertain significance|

- `biomcp get variant "chr7:g.140449165A>C"`
- `biomcp search variant --gene BRAF --condition melanoma --max-frequency 0.01`

## Diseases (90)
|ID|Name|Synonyms|
|---|---|---|
|MONDO:0005105|melanoma|malignant melanoma, melanoma (disease) (and 2 more)|
|MONDO:0005012|skin melanoma|cutaneous (skin) melanoma, cutaneous melanoma (and 8 more)|
|MONDO:0006486|uveal melanoma|intraocular melanoma, melanoma (disease) of uvea (and 5 more)|

- `biomcp get disease MONDO:0005105`
- `biomcp search trial -c melanoma`

## Drugs (2)
|Name|Target|Mechanism|
|---|---|---|
|dabrafenib|BRAF|Protein Kinase Inhibitors|
|vemurafenib|BRAF|Inhibitor of Serine/threonine-protein kinase B-raf|

- `biomcp get drug dabrafenib`
- `biomcp search adverse-event -d dabrafenib`

## Trials (26)
|NCT|Title|Status|
|---|---|---|
|NCT03563729|Melanoma Metastasized to the Brain and Steroids|RECRUITING|
|NCT03235245|Immunotherapy With Ipilimumab and Nivolumab Preceded or Not by a Targeted Therap…|ACTIVE_NOT_RECRUITING|
|NCT03175432|Bevacizumab and Atezolizumab With or Without Cobimetinib in Treating Patients Wi…|ACTIVE_NOT_RECRUITING|

- `biomcp search trial --condition melanoma --biomarker BRAF --limit 3 -s recruiting`
- `biomcp search trial --condition melanoma --biomarker BRAF --limit 3 -p phase3`

## Articles (at least 3)
|PMID|Title|Date|
|---|---|---|
|42875213|Accelerated Systemic Atherosclerosis Following Immune-Related Adverse Events in…|2026-10-15|
|24200969|Activities of multiple cancer-related pathways are associated with BRAF mutation…|2014-01-01|
|38774020|The United Kingdom's First Cardio-Oncology Service: A Decade of Growth and Evolu…|2024-02-06|

- `biomcp search article --gene BRAF --disease melanoma --limit 3 --type research-article`
- `biomcp search article --gene BRAF --disease melanoma --limit 3 --type review`

## Pathways (unknown)
Error: API request to Reactome failed. Retry the remote source.
- `biomcp search pathway --query BRAF --limit 3`



