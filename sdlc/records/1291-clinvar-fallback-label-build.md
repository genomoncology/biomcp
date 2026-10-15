# 1291 build record — the ClinVar fallback label names its failure

Landed 2026-10-08, merge e38bab076. Review verdicts live on
sdlc/tickets/1291-reproduce-the-clinvar-section-source-switch.md.

Outcome: a degraded ClinVar section names NCBI, the reason it failed
(timeout, rate limit, or provider error — sustained rate limiting that
outlives the deadline labeled honestly as a merged case), and the
newest evaluation date of the MyVariant fallback copy; the failure
classification lives in its own module under the line cap;
mutation-proof tests pin the newest-date selection, the calendar-valid
gate, and the end-to-end timeout. Gates: branch CI green at 9cc9b761e;
the wait-ratchet delta ACCEPTed; merged-tree CI green at e38bab076.
