---
flow: build
priority: 16
---
# Say how many assertions a gene has when a variant has none

`variant erepo` on an allele with no ClinGen expert assertion returns "No expert assertions were returned." That is accurate and it is the common case, which is the problem. Measured against ClinVar on 2026-08-19, the ClinGen Evidence Repository holds 81 assertions for BRCA1 against 16,061 ClinVar records, 108 for APC against 17,168, 182 for TP53 against 4,019, and 229 for PTEN against 4,205. An arbitrary variant is far more likely to have no assertion than to have one.

So the first thing a new user sees, for most variants they try, is a blank. Nothing in that message distinguishes "expert curation exists for this gene but not this variant" from "this gene has no expert panel" from "something went wrong." All three read identically, and the most likely reading is the wrong one.

BioMCP already has a gene-level query that returns the count. Naming it turns a dead end into a usable fact: expert curation exists here, this particular variant has not been adjudicated.

Take care not to overstate what the count means. It is the number of assertions the repository holds for that gene, not a coverage percentage and not a statement about the variant's pathogenicity. Absence of an assertion is not evidence of benignity, and the wording must not imply it is.

## Done when

- An empty assertion result for a variant states how many assertions the repository holds for that variant's gene, when the gene is known.
- The wording distinguishes an uncurated variant in a curated gene from a gene with no expert curation at all.
- The message does not imply anything about the variant's classification, in either direction.
- The extra fact does not turn a successful empty result into an error, and does not change the exit code.
- If the gene-level lookup fails or is unavailable, the original message is still returned rather than an error.

## Related

`sdlc/tickets/1023` covers the neighbouring problem of a zero count printed for a source that was never reached. These are different: 1023 is about an unreachable source, this is about a source that answered honestly with nothing.

## Existing tests that pin this

None. The sentence is written at `src/cli/variant/erepo.rs:166` and no shipped test asserts it — `tests/unit/cli/variant.rs` covers only the `--gene` bounds and mutual exclusivity, which this ticket does not touch. Checked 2026-08-20. No restatement is needed or authorized.

## Addendum, 2026-08-20 — what "when the gene is known" fences off

Attempt 1's design refused to approve an implementation: an empty CAid response carries no source-backed gene context, and the authored regression required one, so passing it would have meant inventing a CAid-to-gene resolution the ticket never specified.

The refusal is right and the fence is already in the ticket — the first bullet under "Done when" says "when the gene is known" — but the design read it as a goal to reach rather than a limit to respect. Stating it plainly:

**No new lookup is in scope.** The count is added only when the gene is already in hand from what the command has: the caller supplied it, or the response being rendered names it. Where it is not, the original message is returned unchanged. Resolving an identifier to a gene — from a CAid or from anything else — is a different capability, is not specified here, and is not to be invented to satisfy a proof.

So an empty CAid response with no gene is a case where nothing changes. That is not a gap in the behavior; it is the behavior. A design that authors a proof requiring the count for such a response has written a proof for work this ticket does not ask for, and should say so rather than reaching for an unspecified resolution.

This narrows nothing that was promised. The fifth bullet already says a failed or unavailable gene lookup returns the original message rather than an error, and an absent gene is the same shape of answer.

Everything else stands. In particular the care the ticket asks for keeps full strength: the count is the number of assertions the repository holds for that gene, never a coverage figure and never a statement about the variant, and the wording must not imply that an unadjudicated variant is benign.

If a later ticket does want CAid-to-gene resolution, it is that ticket's to specify, including which source states the mapping and what happens when the mapping is absent.

## Deferred proofs

Added 2026-08-21, after design review refused twice for a deferral
with no named successor. The design may leave this position unproved;
it is carried by the ticket named here:

- how an empty CAid-only ERepo response obtains its gene, including a
  missing mapping and an unavailable source — ticket 1041

This ticket still proves the behavior it is about: when a gene is
already available, from the command or from the response, an empty
result names that gene's assertion count instead of reporting only a
blank, and the unchanged empty message still stands when no gene is
available. No new lookup is in scope here — that fence caused the
code-review refusal of 2026-08-21 at 10:41 and it stays.

## Addendum, 2026-08-21

Three attempts, three different causes, recorded so no fourth
rediscovers them:

- The design review has twice sealed `verdict: refused` in its output
  document while exiting successfully, so the code stage received a
  document saying no design was approved and refused one stage late.
  That is a fault in the assembly, not in this ticket, and it is
  sdlc ticket 0133.
- The code-review refusal at 10:41 was real: the implementation added
  a ClinGen ERepo gene lookup the ticket fences out of scope. That
  attempt's branch is preserved as tag `attempt/1032-20260821-1`.
- The remaining gap was the deferred proof now named above.
