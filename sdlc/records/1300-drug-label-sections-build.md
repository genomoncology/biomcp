# 1300 build record — whole drug label sections with honest missing reasons

Landed 2026-10-08, merge f75292875. Review verdicts live on
sdlc/tickets/1300-return-whole-drug-label-sections-and-say-why-missing.md.

Outcome: whole label sections in JSON with a capped Markdown pointer
printed only when the view cut a section; missing labels name their
reason where callers read it; the identity guard parses the label
element structure so inactive excipients cannot return another drug's
label (verified by exhaustive trace over the full multi-strength
captures); the elements-field fallback returns the sparse records
(Cisapride/Propulsid) the field-scoped queries miss, so empty means
the source has nothing; oversize answers settle with an honest reason.
Gates: branch CI green at c6966f982; yellow lint, spec, and test green
via the build host (one deadline-deadlock hang forensically captured
per the interim rule, restart clean); merged-tree CI green at
f75292875 with MAX_PACKAGE_FILES 1_405 measured. Record filed at
landing.
