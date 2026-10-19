---
priority: 2
---

# 2245 — Name the CGI section's source

Status: OPEN.

Milestone: 0.9.4

## Outcome

The CGI section of variant output names its source as MyVariant's variant-level CGI subset and its load date, so an empty result reads as "no record in that subset", not "CGI has nothing on this gene".

## Evidence

Research on 0.9.3: KRAS G12C returns "No CGI drug associations". BioMCP reads CGI through MyVariant, whose CGI load is dated 2023-02-27 and holds 323 variant-level records; KRAS G12C has none, while BRAF V600E returns 32. A reader can mistake the empty section for CGI having no data on the gene.

## Keeps

Only the section label and empty-state wording change, in Markdown and as a source note in JSON. No data or routing change.

## Proof

Replay tests for a variant with CGI rows and one without show the source label and the subset-specific empty wording. Changelog bullet.
