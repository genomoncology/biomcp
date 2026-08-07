---
flow: review
priority: 7
---
# Review: post-v0.8.21 shipped surface and changelog draft

Approximately 30 tickets (221–260) have shipped on main since the v0.8.21 tag, forming the v0.8.22 candidate batch. Feature areas span: diagnostic entity (GTR, WHO IVD, FDA device, gene/disease pivots), drug extensions (WHO active pharmaceutical ingredients, CVX vaccine identity, WHO prequalified vaccines), adverse events (VAERS), HATEOAS improvements (JSON suggestions, cross-entity article suggestions), BioASQ skill recipes, disease clinical features (MedlinePlus retmax, extraction/enrichment/rendering), article fulltext plumbing (resolver boundary + license gate + PMC HTML + opt-in PDF fallback), architecture work (HPO phenotype enrichment port, PDF/HTML/JATS crate adoption), quality work (spec bash-block lint, drug JSON envelope normalization, gene/all runtime reduction, repo cleanup, cargo install fix, test scratch dir hygiene, health probe parallelization, spec consolidation, docs-integrity test relocation). Before cutting v0.8.22 we need a thorough review of the shipped surface to find bugs, usability issues, and spec gaps, plus a draft changelog for the v0.8.22 tag.

Completed under March on 2026-04-20, as March ticket 246. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/246-review-post-v0-8-21-shipped-surface-and-changelog-draft

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.
