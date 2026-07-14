# Full-scale corpus protocol and audit notes

## Freeze

The explore implementation was frozen at commit `df61a64bd74467f2917b1c244506bc98e012a543` before full-scale scoring. Its `scripts/evaluate.py` SHA-256 is `0a389bc6422dfa14a0aac4735b4128cc8bc452e746e33253e12f61a48f48e6cf`.

## Sampling

PubMed title/abstract searches sampled recent papers from eight explicit event strata and four lexical-trap strata:

- `chromosomal translocation`, `chromosomal deletion`, `chromosomal gain`, `chromosomal amplification`, and `chromosomal inversion`;
- `chromothripsis OR chromoplexy`, `hyperdiploidy OR hyperdiploid`, and `structural variant`;
- controls for `PCR amplification`, `protein translocation`, `nuclear translocation`, and `molecular inversion probe`.

Five papers with a reviewable event mention were retained per event stratum (40 positives). Five papers were retained per trap stratum (20 controls). The initial query selection contained five presumed positives with no qualifying exact mention; they were screened out before prediction and replaced in the same query order: 40945619, 42037569, 42171660, 42431457, and 42425085. One initially sampled control, 42420503, overlapped the explore corpus and was replaced before final measurement by 42307419. The final 60 PMIDs are fixed in `scripts/build_blind_corpus.py` and have no overlap with the 16 explore PMIDs.

Source text is the PubMed title plus a newline plus the abstract. `scripts/fetch_blind_corpus.py` can refresh the ignored NCBI payload, but gates use only the checked-in text snapshot.

## Annotation

Gold uses minimal semantically complete Unicode-codepoint spans. Broad query-family patterns materialize reviewed occurrences; candidate-specific predictions are not copied into gold. Normalized forms, event type, stated locus where contained in the mention, and source provenance are retained. One explicit `t(4;14)` to NSD2 relationship has a larger independent evidence span and provenance. Controls are lexical traps and have no in-scope gold events.

The experiment validates unique PMIDs/event IDs/exact-match keys, label consistency, exact source-span round trips, all eight event types, at least five events per type, relationship target/evidence/provenance integrity, and corpus quotas.

## Blindness limitation

This is a prospective first-pass stress set, but it is **not independently double-annotated blind gold**. The same exploit session sampled, pattern-assisted, reviewed, and scored it. The frozen explore parser was run before adding new event-family rules, so its first-pass numbers are a useful generalization test; however, they are not sufficient for production promotion. Once misses were inspected, this corpus became development data. The tuned result must never be described as blind acceptance.

A production decision requires another disjoint corpus, frozen implementation and scorer first, shuffled text without strata/predictions, two independent annotators plus adjudication, and an exposure manifest excluding all PMIDs seen here.
