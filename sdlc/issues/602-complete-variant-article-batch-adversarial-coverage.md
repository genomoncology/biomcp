# Complete adversarial coverage for variant-article batches

Severity: nice-to-have.

Carried over from March, where it was raised against ticket 602
on 2026-07-20 and left open. The text
below is as filed.
Ticket 602 covers the core happy path, bounds, compact projection, two-worker ordering, work ceilings, invalid-item retention, MCP schema, and the review-found validation regressions. Its approved native test matrix also named several combinations that remain unproved directly: non-object item decoding, generated/explicit request-ID collisions, mixed valid plus terminal-hard outcomes and exit behavior, usable partial degradation, and exhaustive route-plan secret/URL/body/path redaction.

Add focused language-native tests around the existing request owner and injected execution seams. Keep error-path and redaction coverage out of `spec/*.md`; the shipped mustmatch assertions already own the public happy path.
