---
flow: build
priority: 10
---
# Recover article assets through Europe PMC without false not-found

`biomcp --no-cache --json get article 38821914 assets` resolves PMID 38821914 to PMC11143360, receives a PMC OA manifest, then follows the advertised archive URL to an HTTP 404. BioMCP discards that source error and returns `not_found`, falsely telling the caller that the article has no supported assets. On 2026-07-14 the Europe PMC `PMC11143360/supplementaryFiles` endpoint returned a valid 204,916-byte ZIP containing seven files, including `41408_2024_1068_MOESM1_ESM.docx`. This is a live false negative on an agent-facing evidence path.

Completed under March on 2026-07-14, as March ticket 530. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/530-recover-article-assets-through-europe-pmc-without-false-not-found
