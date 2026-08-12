---
base: 9259bc2c4ab8379602f666322ebb82dbe4a9692b
head: 1e963c32ff1674731d10e25e54aa8030eb1a7052
---

Top-level and article batches now share one concurrent settlement boundary.
Every validated item runs, output remains in input order, successful results
survive neighboring failures, and JSON and human forms both end with the same
total, succeeded, and failed counts.

All-success batches exit zero; any item failure exits one only after the full
envelope is written. Mixed, all-failure, ordering, safe-error, Markdown, and
JSON contracts passed without routine public-network access.
