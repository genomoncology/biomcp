# 2032 build record — ambiguous disease refusals offer the oncology choice

Landed 2026-10-08, merge d18d30e7f. Review verdicts live on
sdlc/tickets/2032-ambiguous-disease-refusals-offer-the-oncology-choice-and-reach-trial-search.md.

Outcome: the MM refusal names the oncology reading the source cannot
see (a one-entry clinical-reading pointer, message-only, preferences
deferred); an ambiguous condition in NCI trial search refuses with its
named candidates and sends no request, while an ungroundable condition
keeps the visible keyword degrade. Gates: branch CI run 37798569759
green at the code tip 6815a33b7; code review ACCEPT; the merged-tree
run at d18d30e7f carries main's green landings plus this docs-union —
main's own CI on the landing chain covers it, with the grammar fix
commit after the merge recorded here.
