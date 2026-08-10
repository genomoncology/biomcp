---
flow: build
priority: 3
---
# Search across criteria specifications

Held in drafts after the 2026-08-10 adversarial review. Do not promote this
file as written.

## Why it is not ready

The repository has manifests for eight genes but only one selected CSpec
document capture. That corpus cannot support claims about all 122 released
specifications or trustworthy negative findings. A search implementation
would be precise over an accidentally tiny corpus and misleading about its
coverage.

## Decisions required before promotion

1. Which upstream snapshot is canonical: all currently released
   specifications, every historical version, or both?
2. May the exact documents be redistributed in BioMCP test/package artifacts,
   or should BioMCP store only an index plus receipts and let users sync the
   corpus locally?
3. What update policy and version identifier make a negative result honest?
4. Which fields must be indexed: criterion code, descriptor text, gene,
   condition, operator, threshold, version, and publication state?
5. What default page size and maximum context budget should search use?

## Recommended split after those decisions

First create a corpus-foundation ticket: acquire and receipt a complete
versioned snapshot, settle licensing, record coverage, and build a bounded
local index. Only after it lands should this search ticket expose commands
such as:

    biomcp search spec --criterion BS1 --limit 25 --offset 0

Search results must name corpus version, covered/released document counts,
filters, exact versus text matches, and pagination. A no-hit answer must state
the corpus coverage rather than imply no panel has ever used the term.
