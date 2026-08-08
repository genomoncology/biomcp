---
flow: quickfix
priority: 4
---
# Correct the `attach_not_included` design note to match what shipped

## Done when

Ticket 600's design note and `spec/entity/article.md` agree: ordinary
fulltext `not_included` reports package supplementary files only. No
public contract or fixture changes; the note changes.

## Why here, why now

**The decision is made — do not reopen it.** Ian chose, on 2026-08-08,
to keep ordinary fulltext `not_included` package-only rather than route
it through the shared linked-asset resolver. Reason: routing it through
would add linked-asset acquisition to every ordinary fulltext request,
which is extra network on a common path for a benefit callers can
already get from `get article <id> assets`.

So this ticket makes the *design note* right, not the code.

## The finding

Raised under March and carried over when BioMCP moved to the sdlc
factory. The text below is as filed.

## Summary

Ticket 600's final design says `attach_not_included` should consume the shared
linked-asset resolver, but the shipped fulltext spec still requires the old
package-only summary. The design and public contract need one explicit decision.

## Detail

The implementation intentionally leaves ordinary fulltext `not_included`
package-only because `spec/entity/article.md::Fulltext Reports Assets Not
Included` requires one supplementary file. The fixture now also names a
JATS-linked supplement, so routing `attach_not_included` through the linked
resolver would change that contract and add linked-asset acquisition to ordinary
fulltext requests.

This is not safe to repair as a code-only change: design must choose whether
ordinary fulltext summaries include every linked asset or preserve package-only,
low-latency behavior. Verification confirmed the current package-only behavior
and the explicit `assets` resolver both work as documented.

## Suggested action

Destination: `spec` plus `test` and architecture docs. Decide the intended
fulltext behavior first. If linked assets belong in `not_included`, author a
behavioral `spec/entity/article.md` assertion that names the package and linked
supplements without pinning an exact count, then route `attach_not_included`
through a resolver that avoids duplicate acquisition. If fulltext stays
package-only, amend `design-final.md`/architecture wording and add a focused test
that the explicit asset resolver—not ordinary fulltext—owns linked fetching.
