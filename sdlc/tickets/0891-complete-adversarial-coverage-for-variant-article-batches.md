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
factory. The text below is as filed.

Add focused language-native tests around the existing request owner and injected execution seams. Keep error-path and redaction coverage out of `spec/*.md`; the shipped mustmatch assertions already own the public happy path.
