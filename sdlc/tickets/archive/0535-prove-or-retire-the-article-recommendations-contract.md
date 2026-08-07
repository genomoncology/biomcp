---
flow: review
priority: 8
---
# Prove or retire the article recommendations contract

`biomcp article recommendations` is documented as finding related papers from positive seeds and accepts negative seeds. In two independent systematic-review update cases it returned about 340 distinct papers and recovered none of 28 known added studies, while ordinary article search recovered most of them. Current main does return recommendation rows for an individual seed, so the remaining uncertainty is whether identifier resolution or request construction weakens the signal, or whether Semantic Scholar's related-paper model simply does not support the implied evidence-gap use case.

Completed under March on 2026-07-15, as March ticket 535. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/535-prove-or-retire-the-article-recommendations-contract

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.
