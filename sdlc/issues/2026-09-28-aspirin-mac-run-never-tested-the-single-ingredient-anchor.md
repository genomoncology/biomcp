# The aspirin Mac run never tested the single-ingredient anchor path

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
