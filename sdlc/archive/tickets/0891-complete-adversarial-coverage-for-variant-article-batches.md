---
flow: build
priority: 3
---
# Complete adversarial coverage for variant-article batches

## Done when

The adversarial cases named in the body have tests, and each fails if
its guard is removed.

## The finding

Raised under March and carried over when BioMCP moved to the sdlc
factory. Reproduced in full below; `severity` is March's word, and
this ticket's priority is the one that counts.

<!-- from 602-complete-variant-article-batch-adversarial-coverage.md -->

# Complete adversarial coverage for variant-article batches

Ticket 602 covers the core happy path, bounds, compact projection, two-worker ordering, work ceilings, invalid-item retention, MCP schema, and the review-found validation regressions. Its approved native test matrix also named several combinations that remain unproved directly: non-object item decoding, generated/explicit request-ID collisions, mixed valid plus terminal-hard outcomes and exit behavior, usable partial degradation, and exhaustive route-plan secret/URL/body/path redaction.

Add focused language-native tests around the existing request owner and injected execution seams. Keep error-path and redaction coverage out of `spec/*.md`; the shipped mustmatch assertions already own the public happy path.
