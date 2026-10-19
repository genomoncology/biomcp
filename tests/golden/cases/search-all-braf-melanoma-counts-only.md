# Search All: gene=BRAF disease=melanoma

## Genes (1)
Rows omitted (`--counts-only`).
- `biomcp get gene BRAF`
- `biomcp search trial --biomarker BRAF`

## Variants (42)
Rows omitted (`--counts-only`).
- `biomcp get variant "chr7:g.140449165A>C"`
- `biomcp search variant --gene BRAF --condition melanoma --max-frequency 0.01`

## Diseases (90)
Rows omitted (`--counts-only`).
- `biomcp get disease MONDO:0005105`
- `biomcp search trial -c melanoma`

## Drugs (2)
Rows omitted (`--counts-only`).
- `biomcp get drug dabrafenib`
- `biomcp search adverse-event -d dabrafenib`

## Trials (26)
Rows omitted (`--counts-only`).
- `biomcp search trial --condition melanoma --biomarker BRAF --limit 3 -s recruiting`
- `biomcp search trial --condition melanoma --biomarker BRAF --limit 3 -p phase3`

## Articles (at least 1)
Rows omitted (`--counts-only`).
- `biomcp search article --gene BRAF --disease melanoma --limit 3 --type research-article`
- `biomcp search article --gene BRAF --disease melanoma --limit 3 --type review`

## Pathways (at least 3)
Rows omitted (`--counts-only`).
- `biomcp get pathway R-HSA-167224`

## PGx (at least 0)
Rows omitted (`--counts-only`).

## GWAS (0)
Rows omitted (`--counts-only`).

