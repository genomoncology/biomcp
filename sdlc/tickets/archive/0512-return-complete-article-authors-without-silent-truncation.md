---
flow: build
priority: 5
---
# Return complete article authors without silent truncation

`get article <pmid> --json` deliberately reduces author lists longer than four to first and last author. PMIDs 35637217, 37449980, and 38821914 return 2 authors although PubTator/PubMed carry 16, 28, and 18. The field looks complete, so middle-author attribution fails silently. `article batch` omits authors entirely even though its underlying article objects already have them.

Completed under March on 2026-07-13, as March ticket 512. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/512-return-complete-article-authors-without-silent-truncation
