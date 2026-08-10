---
flow: build
priority: 9
deps: ["0935", "0948"]
---
# Make no-cache leave no managed request state

The global `--no-cache` help says it disables HTTP caching, but users also need
one clear invocation mode that does not read or write article-session query
state. Conflating that behavior with expiry would leave the immediate privacy
choice ambiguous.

## Invocation contract

For one invocation, global `--no-cache` performs no managed HTTP cache reads or
writes and no article-session store reads or writes. A caller that combines
`--no-cache` with `--session` receives a typed invalid-argument error before
transport because cross-call session behavior cannot be honored without local
state. Provider-side logging and retention are unaffected and remain disclosed.

Temporary files needed only to stream or parse a response are removed before
the command returns and are not placed in the managed cache/session root.
Explicit user-requested downloads, study datasets, CSpec captures, and other
durable output commands are not reclassified as caches; their existing command
contracts and paths remain unchanged.

## Done when

- Local request tests snapshot an empty managed root before and after successful,
  empty, provider-failed, interrupted, human, and JSON `--no-cache` commands and
  prove no cache/session entry or invocation temp remains.
- A prepopulated cache cannot satisfy a `--no-cache` request, and the fresh
  response does not update it.
- Every `--no-cache --session` ordering accepted by Clap fails before a counting
  transport observes a request.
- Help, cache, article-session, policy, and troubleshooting documentation use
  the same no-managed-request-state wording and distinguish explicit downloads.
- Existing default caching, session loop-breaking, and explicit durable-output
  behavior remain covered.

## Authorized test changes

Design commits may restate global flag help, cache/session dispatch, temporary
file cleanup, local transport, and documentation tests. Do not weaken default
cache correctness, provider source attribution, or explicit download/capture
contracts.

The src line ceiling may rise by at most 160 lines.
