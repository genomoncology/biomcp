# Live ClinGen CSpec diagnostic

This diagnostic calls the real ClinGen Criteria Specification Registry using a
complete provider resource IRI. Provider content and capture hashes can change, so
it checks only that the selected document returns the bounded provenance required
to retrieve its original bytes through the CLI.

## Live CSpec selection returns capture provenance

ATM GN020 version 1.5.1 has a provider display version that may differ from its
full resource IRI. The frozen contract owns exact source facts; this live check
catches a broken real route or missing capture provenance without comparing a
volatile response hash to historical bytes.

```bash
biomcp --json gene cspec ATM --version https://cspec.genome.network/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1 | mustmatch like '"capture_id"
"source_sha256"
"byte_length"
"resource_iri"
"specification_id"
"display_version"
"semantic_subset_version"
"semantic_subset_sha256"'
```
