---
flow: build
priority: 3
---
# Prove search continues past a fully filtered page

## Done when

An integration test covers an alias-union fanout where one page
contributes no surviving result, and shows the search continuing to the
next page rather than stopping. The test fails if continuation is
removed.

## The finding

Raised under March and carried over when BioMCP moved to the sdlc
factory. The text below is as filed.

Ticket 590 updates alias-union worker state before shared detail filtering, but no integration test proves search continues when a page contributes no surviving candidates and a later page contains a qualifying trial.

## Detail

The current overlap fixture now proves that search and count actually request a later duplicate-only page, and that an NCT ID repeated across workers/pages receives one detail request. Both first-page fixture studies pass the eligibility filter, however. A regression that stops after a round whose newly discovered candidates are all rejected by detail-backed filtering could therefore escape the current suite.

The implementation order is correct on inspection: worker page tokens and exhaustion are updated before `apply_ctgov_post_filters`, and the search threshold is checked afterward. The missing proof is not release-blocking, but it is worth making durable because this ordering is easy to break during later pagination work.

## Suggested action

Destination: `test`.

Extend the loopback CTGov integration fixture with a criteria-specific path where one fanout page has only post-filter-rejected new candidates and its next page has a qualifying unique candidate. Drive the shipped binary with a result limit that requires the later candidate, then assert the qualifying result shape and that a paginated list request occurred. Keep the public mustmatch fixture mapping unchanged and avoid exact request counts or token prose except where provider-I/O multiplicity itself is the behavior.
