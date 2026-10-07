# 2018 build record — article entity variant links name one allele

Landed 2026-10-07, merge 6b504219e (branch tip 3c94532ae). Review
verdicts live on sdlc/tickets/2018-article-entity-variant-links-name-one-allele.md.

Outcome: a multi-allele rsID no longer links every article mention row
to the variant one allele opens. When the document's own annotations
show the rsID naming several protein changes, each row prints the
gene-qualified change that reproduces its own variant; single-allele
rsIDs keep their rsID links. PMID 30738221's G12A/G12D/G12V rows each
open their own variant. Gates: yellow rc=0 at 3c94532ae under the
corrected sync-python-dev protocol; merged-tree CI green at 6b504219e
(the landcheck run; the branch's own CI was pool-cancelled three
times, recorded here). Record filed at landing.
