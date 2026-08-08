# Align linked assets with the fulltext not-included contract

Severity: should-fix.

March marked this as blocking the next ticket in its area.

Carried over from March, where it was raised against ticket 600
on 2026-07-20 and left open. The text
below is as filed.
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
