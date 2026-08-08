# Live PubTator contract still blocks the deterministic make-test gate

Severity: should-fix.

March marked this as blocking the next ticket in its area.

Carried over from March, where it was raised against ticket 593
on 2026-07-19 and left open. The text
below is as filed.
## Summary

`make test` still contains a contract that calls live PubTator. During ticket 593's single authoritative `full-blocking` run, `swallowed_source_failures_do_not_log_credentials` timed out after 10 seconds while running `biomcp --no-cache variant articles "BRAF V600E" --limit 1`, failing an otherwise-green gate. The same test passed immediately in isolation in 7.0 seconds.

## Detail

Ticket 591 moved known live full-contract tests out of `make test`, but this security logging contract still reaches PubTator autocomplete over the network. Its assertion is valuable—provider failures must not leak credentials—but a routine blocking gate cannot prove that deterministically through a live service. The failure is unrelated to ticket 593's pure SSRF address classifier, yet it prevents an honest full-blocking approval.

The observed stderr contained only a sanitized PubTator warning and no secret; the failure was solely the command's fixed 10-second timeout. Raising that timeout would make the flake slower rather than deterministic.

## Suggested action

Destination: **gate/test**. Replace the live PubTator dependency with an ephemeral local HTTP fixture that returns a representative provider failure, while preserving the existing no-secret logging assertion. Alternatively move this contract to `make verify` if its purpose truly requires a real provider. Add a ratchet that `make test` cannot perform external network calls so ticket 591's deterministic-gate decision applies to indirect calls as well as explicitly ignored live tests.
