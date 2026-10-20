# The aspirin Mac run never tested the single-ingredient anchor path

Disposition (2026-10-20): Closed under the recorded resolution and its landed revision b36ad664f. The original evidence and later dispositions remain below.


Filed 2026-09-28 from the review of the 1255-1261 round. The Mac
DDInter run's aspirin check resolved the MyChem anchor to the
five-ingredient combination product ("epinephrine, albuterol
sulfate, nitroglycerin, diphenhydramine hydrochloride, aspirin"),
which the bundle does not cover — so the run proved the honest
not-covered wording on a combination anchor, not the single-
ingredient path (aspirin alone resolves to acetylsalicylic acid,
which the bundle covers: apixaban's table lists it).

## What is wanted

One more Mac run against the synced bundles:

    biomcp drug interactions "acetylsalicylic acid"

The covered single-ingredient card must show rows, the coverage
line, and freshness, proving the anchor path the original issue
meant to exercise. If plain "aspirin" should prefer a single-
ingredient hit when one matches, that is an anchor-policy choice
to record here first (a future ticket).

## Status

Open. Owner: the biomcp queue. Revisit trigger: the next Mac
session or any DDInter anchor change.

## Closure in2242

Resolved in2242: live current-main study preserved the single-ingredient DB00945/CHEMBL25 identity but exposed omitted selected-source name Acetylsalicylic acid in interaction aliases. Retaining that explicit alias fixes the reproduced DDInter20 match without pooling combination ingredients. Authored outside-in controls qualify the fix; the full DrugBank-containing source reply stays private.

## Release dispositions (2026-10-19)

- Single-ingredient interaction identity: FIXED by 2242 at b36ad664f. The reproduced DB00945/CHEMBL25 path retains the selected-source Acetylsalicylic acid alias, with authored outside-in controls.
- Historical Mac-run evidence: HISTORICAL. The combination-anchor observation remains true; this disposition does not claim that the original single-ingredient Mac run was recreated.
