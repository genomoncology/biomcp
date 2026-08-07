---
base: 36d964db4c066f638ae298df6e60b3ac315221c4
head: 5333e03ad4100aa773a027145d39309caa45144c
---
The routine executable spec spends most of its time in `spec/entity/article.md`. The file has 26 bash blocks and starts `setup-article-fulltext-source-fixture.sh` 15 separate times, with each block launching a server, probing readiness, sourcing an env file, and trapping cleanup. During ticket 494 verification, the routine mustmatch run remained in the article file for more than five minutes before advancing to later entities.

Imported from March ticket 502. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/502-share-one-runner-owned-article-fixture-across-routine-specs
