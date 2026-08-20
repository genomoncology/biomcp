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
