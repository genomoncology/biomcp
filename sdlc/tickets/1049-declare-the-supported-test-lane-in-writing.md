---
flow: build
priority: 27
---

# Declare the supported test lane in writing

Decision recorded 2026-08-23, Ian's ruling: the supported test lane is
`make test` — the offline `--no-default-features` nextest lane the gates
already run. Direct bare `cargo test` invocations are unsupported, and their
baseline failures are known and accepted rather than reconciled. This ticket
writes that ruling into the repo so a developer hitting a direct-run failure
can learn, from the repo alone, that they are in an unsupported lane and
what the supported one is.

What was verified on this machine on 2026-08-23, so the writing need not
guess:

- `make test` passes; every gate that runs it has been green.
- A full-suite direct `cargo test --locked --lib` run fails
  `sources::provider_url_policy::tests::selected_fixture_origin_allows_only_exact_ip_loopback`
  in **both** feature lanes, while the same test **passes in isolation** —
  the failure is suite-order-dependent, not feature-dependent.
- A default-feature full run additionally showed run-varying failures across
  two runs: `article_source_urls_keep_their_live_pacing_policies` once, and
  two `cli::skill::tests::install` symlink tests once.
- The failing test's panic is the policy rejecting a non-HTTPS fixture
  origin, the same shape the filed issue recorded on 2026-08-22.

## Done when

- The repo's testing documentation — wherever a developer who just ran a
  bare `cargo test` would look first — states the supported lane, states
  that direct bare invocations are unsupported, and names the known
  order-dependent failure(s) so the reader can recognize theirs.
- The statement is dated to the ruling and says the reconciliation path was
  declined, so a future reader knows this is a decision and not an oversight.

## What this replaces

The alternative — reconciling the default-feature and direct-run
expectations — was considered and declined by the ruling above. This ticket
does not change any test or any source file; it documents the lane. If a
future ticket wants to make direct runs green, that ticket replaces this
statement and says so.

Filed from `sdlc/issues/1039-default-feature-cargo-test-baseline.md` and
`sdlc/issues/2026-08-22-provider-url-fixture-origin-routine-test.md` (the
latter reproduced on 2026-08-23 in both feature lanes and is folded into
this documentation).
