---
flow: build
priority: 9
---
# Take the live PubTator call out of `make test`

## Narrowed 2026-08-09 during review triage

The fixture conversion already landed: the no-secret logging assertion
now runs against `CredentialRedactionFixture` (tests/json_error_contract.rs:608),
a local ephemeral fixture, not live PubTator. What remains of this
ticket is ONLY the ratchet — the fail-closed guarantee that no routine
test can reach the network, so a live call cannot come back indirectly.
Do not redo the fixture work.

## Done when

A ratchet fails the gate if any routine test (`make test`) reaches the
external network — fail-closed, covering the whole routine suite, not a
per-test allowlist that new tests can silently skip. The landed
fixture-backed credential assertion keeps passing unchanged.

## Why here, why now

This sits at priority 9 because it is inside a gate rung. A live call in
`make test` can fail any flight for reasons the agent did not cause and
cannot fix, which is the exact thing the gate ladder exists to prevent.

## The finding

Raised under March and carried over when BioMCP moved to the sdlc
factory. Reproduced in full below; `severity` is March's word, and
this ticket's priority is the one that counts.

<!-- from 593-live-pubtator-contract-still-blocks-make-test.md -->

## Summary

`make test` still contains a contract that calls live PubTator. During ticket 593's single authoritative `full-blocking` run, `swallowed_source_failures_do_not_log_credentials` timed out after 10 seconds while running `biomcp --no-cache variant articles "BRAF V600E" --limit 1`, failing an otherwise-green gate. The same test passed immediately in isolation in 7.0 seconds.

## Detail

Ticket 591 moved known live full-contract tests out of `make test`, but this security logging contract still reaches PubTator autocomplete over the network. Its assertion is valuable—provider failures must not leak credentials—but a routine blocking gate cannot prove that deterministically through a live service. The failure is unrelated to ticket 593's pure SSRF address classifier, yet it prevents an honest full-blocking approval.

The observed stderr contained only a sanitized PubTator warning and no secret; the failure was solely the command's fixed 10-second timeout. Raising that timeout would make the flake slower rather than deterministic.

## Suggested action

Destination: **gate/test**. Replace the live PubTator dependency with an ephemeral local HTTP fixture that returns a representative provider failure, while preserving the existing no-secret logging assertion. Alternatively move this contract to `make verify` if its purpose truly requires a real provider. Add a ratchet that `make test` cannot perform external network calls so ticket 591's deterministic-gate decision applies to indirect calls as well as explicitly ignored live tests.
