# Drug card reports DDInter not covered as no interactions

Filed 2026-09-23 from an independent review of `v0.9.0..f2549676`.

## Symptom

With a sample DDInter bundle:

- `drug interactions aspirin` correctly says the drug is not covered.
- `get drug aspirin interactions` prints "no matching rows… 0 of 0". `src/entities/drug/interactions.rs:163` drops the coverage status.

Name matching (`:110`) uses only the query, the resolved name, and brand names. A row filed under "acetylsalicylic acid" was missed from the aspirin side and found from the warfarin side. The name lookup also resolved "aspirin" to a combination product.

`src/sources/ddinter.rs:208` caches the index for the life of the process, while the freshness label is re-read from file times at `:149`. A long-running server can label old rows fresh. A corrupt file surfaces as a generic "API request failed".

The fixed path has never run against a full real bundle. Every test host lacks it.

## Fix

- Carry the coverage status onto the drug card.
- Match on synonyms. Add a test pairing aspirin with acetylsalicylic acid.
- Tie the freshness label to the loaded index.
- Name the file in the corrupt-bundle error.
- Check the 8 MB per-file download cap against the real file sizes.
- Run `drug interactions` once on a host with the real bundle and record the result here.
