---
priority: 2
---

# 2269 — Report article metadata and section completeness truthfully

Status: OPEN.

Milestone: 0.9.4

## Outcome

Article search reports an internal metadata failure in its JSON source status and marks affected rows incomplete, and full-text quality flags agree with the parsed output.

## Evidence

Research ran released 0.9.3 on benchmark questions and paper fetches and verified each finding against the provider's official API; raw outputs are in research's store and may be copied as fixtures (ask the PM for the path). R3: `biomcp search article -d glioblastoma -k "recurrence cell states mesenchymal shift" --full --json --limit 5` warns on stderr that the PubTator3 visible-article metadata fallback failed (internal), yet JSON says complete:true and source_status omits PubTator3; PubTator3 biocjson answers 200 for those PMIDs, so check whether the failure is ours. R4: `biomcp get article 28736637 fulltext --json` (PMC HTML fallback) gives full_text_manifest.quality.has_sections:false while full_text_summary.section_count is 31.

## Keeps

Build on 2235's recovery. Compute flags from the parsed output.

## Proof

Replay tests: the failing fallback appears in source_status with complete:false; the 28736637 case reports has_sections:true. If the PubTator3 failure is BioMCP's request, fix it.
