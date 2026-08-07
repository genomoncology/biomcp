---
flow: build
priority: 5
---
# Share one runner-owned article fixture across routine specs

The routine executable spec spends most of its time in `spec/entity/article.md`. The file has 26 bash blocks and starts `setup-article-fulltext-source-fixture.sh` 15 separate times, with each block launching a server, probing readiness, sourcing an env file, and trapping cleanup. During ticket 494 verification, the routine mustmatch run remained in the article file for more than five minutes before advancing to later entities.

Completed under March on 2026-07-11, as March ticket 502. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/502-share-one-runner-owned-article-fixture-across-routine-specs
