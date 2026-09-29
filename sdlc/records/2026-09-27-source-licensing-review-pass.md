# Source licensing review pass (delegated), 2026-09-27

Ian moved the source licensing review from himself to the developer
on 2026-09-27
(`sdlc/issues/2026-09-27-review-follow-ups-after-the-1238-close.md`,
direction section). This is that review's first pass. Its immediate
job was the 2027-03 crossing: 48 sources carried `reviewed_on`
2026-03-20 and would cross the guard's 365-day fail on 2027-03-20.

## Method

Four read-only research agents split the 48 (12 each). Each read
the recorded entry in `docs/reference/sources.json` and
`docs/reference/source-licensing.md`, then fetched the live terms
page (curl, or web search where the page is JS-walled) and reported
per source: reachable and date-stamped, contradiction or not, one
verbatim governing sentence, its URL. DepMap got a separate
evidence pass recorded in its own issue file.

## Findings

**Two sources changed materially and are updated in the registry:**

- COSMIC — the recorded `sanger.ac.uk/legal/cosmic-licensing/` URL
  is dead; cancer.sanger.ac.uk/cosmic redirects to cosmickb.org,
  whose licensing page (© 2026) restricts academic use to
  not-for-profit organisations and requires a commercial licence
  for commercial use, expressly including "patient services and
  clinical reporting". First write applied that clause to
  BioMCP's cached fields. Ian corrected it the same day: BioMCP
  reads COSMIC only through MyVariant.info, and MyVariant's
  metadata
  (`http://myvariant.info/metadata`, checked 2026-09-27) says
  "COSMIC v68 was imported from UCSC database dump. This is the
  last freely available somatic variants from COSMIC before their
  licence change." The v68-era terms — not the current clause —
  govern the fields BioMCP surfaces; the current licence governs
  COSMIC's current direct offering only. The registry records
  both, with the snapshot's age (not its licence) as the practical
  limit.
- CPIC — cpicpgx.org now 302-redirects to ClinPGx and the former
  CC0 licence pages are gone. The successor data-usage policy page
  states ClinPGx data is CC BY-SA 4.0; no live page says whether
  that licence governs CPIC content. The registry records exactly
  that state and treats CPIC as attribution-plus-share-alike until
  CPIC publishes terms on its own domain again.

**One summary corrected:** ChEMBL is CC BY-SA 3.0; the earlier
summary omitted the share-alike obligation.

Corrected 2026-09-28 (twice): CIViC and WikiPathways ARE
verifiable — their current docs and terms pages are plain HTML
(CIViC's FAQ states CC0 1.0 Universal; WikiPathways' terms page
adopts the CC0 waiver; both cited in the evidence table) — and
PharmGKB verified against the same ClinPGx policy page CPIC uses.
Only Enrichr stays undated (its terms loader 404s provider-side).
The guard warns for Enrichr on 2027-01-15 and fails on 2027-03-21;
the owner of the retry is the developer agent working Ian's ordered
BioMCP queue.

**Improvements folded in:** the four BioThings services
(MyGene/MyVariant/MyDisease/MyChem) now carry the shared footer's
research-purposes disclaimer in their reuse notes — a clinical-facing
tool should show it. InterPro, OLS4, and QuickGO terms URLs now
point at the plain-HTML EMBL-EBI terms-of-use page instead of their
JS-walled app pages.

**Everything else unchanged** — 45 of 48 after the 2026-09-28
corrections (42 verified on the day, CIViC, WikiPathways and
PharmGKB verified the next day against pages the first pass could
not read). The JS-walled terms of DisGeNET, g:Profiler,
GTEx, CGI, and cbioportal were read from their live JS bundles,
which are the citable sources. The per-source evidence table with
URL, access date, and finding for all 48 is committed at
`docs/reference/source-licensing-evidence-2026-09-27.md`
(corrected 2026-09-28: PharmGKB, CIViC and WikiPathways all verified against live pages, leaving one unverifiable; the
warning date is 2027-01-15 and the fail date 2027-03-21 for
Enrichr alone — the 2027-03 crossing is handled for the other 47).

## Registry state

45 markdown sections dated 2026-09-27 (the batch-verified entries
plus the PMC OA section, verified directly against
<https://pmc.ncbi.nlm.nih.gov/about/copyright/> the same day: US
government journal articles are public domain, and open-access
articles still carry article-level licences and third-party
material, matching the recorded summary). The 2026-09-28
corrections re-dated CIViC and WikiPathways to 09-28 in both
registries; Enrichr alone stays 2026-03-20. `tools/check-source-registry.py`
status=pass; the licensing docs contract suite is green (17
passed).

## Decision authority

No money, agreement, or provider contact was involved. Ian can
overturn any of this; the DepMap decision (separate file) names its
own overturn path.
