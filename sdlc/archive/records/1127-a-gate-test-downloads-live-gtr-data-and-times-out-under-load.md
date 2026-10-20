---
base: 120e4484446a347c2a8f3b148c92eb94922907d6
head: 2ed57677955741cb520db6ea891631e646c8521c
---

The diagnostic regulatory overlay could exceed its eight-second guard because
OpenFDA client construction scanned the managed cache before Tokio polled the
timeout.

The overlay now uses an uncached OpenFDA client with the standard transport and
retry policy. Focused coverage verifies its timeout, success, and unavailable
fallback behavior while other OpenFDA callers retain managed caching.
