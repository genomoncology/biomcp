# 2017 build record — ambiguous disease abbreviations refuse in get disease

Landed 2026-10-08, merge d8afa297f. Review verdicts live on
sdlc/tickets/2017-ambiguous-disease-abbreviations-refuse-in-get-disease.md.

Root cause: the 1295 query returned every disease holding a token as
an exact synonym, the resolver scored names and synonyms equally, and
an unlabelled record's ID fallback ("mondo 0020481") out-ranked the
labelled disease on shortest-name tie-breaks — so MF resolved to
Myotonia fluctuans and cards mixed definition and genes from different
diseases, because the gene section re-queried Open Targets with the
requested abbreviation.

Outcome: two or more exact human holders refuse naming every candidate
with labels filled (non-human records never count — the veterinary
myeloma record descends from the non-human ancestry marker); full
words are ambiguous only by name (myeloma resolves to multiple
myeloma); the NCI trial search surfaces its keyword degrade in the
response; one-disease cards never mix another disease's genes; two-
letter single-holder tokens refuse with the short-token reason.

Gates: branch CI green at 84a647904; yellow gates fully green at the
same sha (lint, spec, and test rc=0 in the isolated gate clone); code
review ACCEPT at 5b2ee79c2 plus the delta ACCEPT through 84a647904.
Landing deviation, corrected from the third review's finding 6: the
merge reached main via the queue owner's local-chain push before the
landcheck finished; the landcheck run completed green eight minutes
later, and main's own run for that push was runner-cancelled — the
landcheck run, not main's, is the covering evidence.
