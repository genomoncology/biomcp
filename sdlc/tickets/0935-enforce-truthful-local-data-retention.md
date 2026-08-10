---
flow: build
priority: 9
---
# Expire local sessions and cache entries truthfully

The policy says BioMCP stores no request payloads and defines no retention
period. Article sessions persist query terms and identifiers, and HTTP response
cache entries can remain physically present after their stated age until a
later write or storage-pressure cleanup.

## Retention contract

Document article-session records and the managed HTTP response cache as local
storage. Session records expire after their existing ten-minute lifetime.
HTTP cache entries obey configured `max_age_secs`. Opening the relevant store
for any read, write, stats, or clean operation physically removes expired
records even when no new record is written and no size/disk threshold is
crossed.

`cache stats` reports response-cache and session counts separately, while
existing `cache clean` and `cache clear` remove the documented managed data
without following links outside the root. Ticket 0948 owns private filesystem
permissions. Ticket 0949 owns the no-local-state invocation mode.

## Done when

- Clock-controlled tests create expired and unexpired sessions/cache entries,
  then prove a read/stats path physically removes only expired managed data.
- Cleanup occurs below all size and disk-pressure thresholds.
- Policy, cache, article-session, configuration, and troubleshooting pages name
  stored fields, locations, lifetimes, controls, and upstream-provider
  retention as separate concerns.
- No test stores a real biomedical query or reaches a public provider.

## Authorized test changes

Design commits may restate retention, session, cache manager/config/clean, and
policy documentation tests in
`src/cli/article/session.rs`, `src/cache`, cache CLI tests, and public policy
pages. Existing cache-key isolation, symlink safety, atomic writes, size/disk
limits, and article loop-breaking behavior remain covered.

The src line ceiling may rise by at most 200 lines.
