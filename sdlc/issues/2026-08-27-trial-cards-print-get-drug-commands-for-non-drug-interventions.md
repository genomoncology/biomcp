# Trial cards print a get-drug command for intervention names that are not drugs

`biomcp get trial NCT06662188` (JAG201, a SHANK3 gene therapy) prints in its
See-also block:

    biomcp get drug JAG201

Running it fails:

    Error: drug 'JAG201' not found. Try searching: biomcp search drug -q "JAG201"

`biomcp drug trials JAG201` (also printed) works, because it is a text
search. Verified 2026-08-27 against 0.9.0-dev.6 (captures:
experiments/193-biomcp-bug-hunt/calls/rt-trial-card.txt, run-get_drug_JAG201.txt).

Mechanism, verified in code: `src/render/markdown/related.rs` (the trial
branch, ~line 799) takes the first intervention string and prints
`get drug {intervention}` — the same guarantee class as ticket 1056 (print
only commands that run), on the trial→drug pivot. A sibling branch two
lines up already does the right thing for aliases: it prints
`search drug -q ...`, the search form.

The fix shape: intervention-derived names print the search form (which the
error message itself recommends), or the card verifies the name resolves
as a drug before printing the get form — design settles it. Same class
should be swept across cards that derive drug names from non-drug fields.
