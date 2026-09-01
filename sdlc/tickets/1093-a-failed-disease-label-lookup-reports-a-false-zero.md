---
flow: build
priority: 9
---

# A disease card whose label lookup failed reports a false zero and steers every follow-up wrong

`biomcp get disease medulloblastoma` answers a clinical question with a wrong number. Verified against the repository build `biomcp 0.9.0-dev.6` (`./target/debug/biomcp`) on 2026-09-01:

```
$ biomcp get disease medulloblastoma
# MONDO:0007959
ID: MONDO:0007959
Recruiting Trials (ClinicalTrials.gov): 0
```

The true count is 36:

```
$ biomcp search trial -c medulloblastoma -s RECRUITING --limit 1
Showing 1-1 of 36 results.
```

One failure produces three wrong outputs. Label resolution did not return a display name, so the card fell back to the ontology identifier. The card then headed itself `MONDO:0007959`, ran the recruiting-trial count against the literal string `MONDO:0007959`, and serialized the miss as `0` rather than as an absent value.

The same fallback reaches every suggested command on the card. None of them use the term the caller typed:

```
See also:
  biomcp search trial -c "cerebellum embryonal neoplasm"
  biomcp search article -d "cerebellum embryonal neoplasm"
  biomcp search diagnostic --disease "cerebellum embryonal neoplasm"
  biomcp search drug --indication "cerebellum embryonal neoplasm"
```

`search trial -c "cerebellum embryonal neoplasm"` returns 3 recruiting trials against 36 for `-c medulloblastoma`. A caller who follows the card's own advice loses 92% of the recruiting trials for the disease they asked about.

The failure is per-entity and gives no warning. Neuroblastoma, retinoblastoma, Ewing sarcoma and sickle cell all report exact counts on the same command, so nothing in the output distinguishes a card whose lookup succeeded from one whose lookup failed.

## Required behavior

A count that could not be computed must not render as a number. A reader must be able to tell "no recruiting trials exist" from "the count query did not run".

A card must not silently substitute a term the caller did not supply. When the caller's own term resolved the entity, follow-up commands and counts use it. When the card offers a different term, the output says which term it used and why.

The header must name the disease when a name is available anywhere in the record the card already holds.

## Done, observably

- `get disease medulloblastoma` reports the recruiting-trial count that `search trial -c medulloblastoma -s RECRUITING` reports, or reports no number at all and says the count was unavailable.
- No section of any disease card prints `0` for a count whose underlying query failed or was never run.
- The suggested commands on a disease card run against a term that returns the entity the card describes.
- A disease whose label lookup fails is distinguishable in output from one whose label lookup succeeds, without the reader running a second command.

## Boundary

This ticket does not change how MyDisease.info is queried, does not add a new source, and does not change the disease card's section list. Cards whose label resolution succeeds keep their current output. The suggestion ranking question on other entity cards (a low-confidence top hit being offered without its confidence) is a separate concern and is not in scope here.
