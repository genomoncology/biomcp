# 2023 build record — article search carryovers

Landed 2026-10-08. Review verdicts live on
sdlc/tickets/2023-article-search-review-carryovers.md.

Outcome: every sort refuses an offset window above the fetch cap before
any provider request; a deadline error from any primary outranks a fast
PubTator failure (the race the second review diagnosed); the
construction lock test no longer depends on process state and the
PubMed fixture route answers; the cursor walk stops on an absent cursor
after a page with rows; JATS word joining keeps small caps and styled
content whole. Gates: branch CI green at ec27dfbcf; yellow lint and
spec green with the test phase attempted under the interim
deadline-deadlock rule (fourth hang forensically captured); merged-tree
CI green at 05eeda3f3, whose only later delta before this merge commit
was documentation records. Record filed at landing.
