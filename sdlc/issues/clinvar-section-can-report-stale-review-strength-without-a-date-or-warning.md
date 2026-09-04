# ClinVar section can report stale review strength without a date or warning

Severity: should-fix

BioMCP 0.9.0-dev.6 reported one submitter and one review star for the HSD17B4 allele `rs1753477498`:

```text
$ biomcp --json get variant rs1753477498 clinvar
"clinvar_condition_reports": 1
"clinvar_review_stars": 1
"clinvar_review_status": "criteria provided, single submitter"
"clinvar_id": "974782"
```

NCBI's current ClinVar VCV record disagrees. A live official EFetch request on 2026-09-04 returned `SubmissionCount="2"`, `NumberOfSubmitters="2"`, and `criteria provided, multiple submitters, no conflicts`. It contains Rady Children's `SCV001426412`, last evaluated 2020-08-04, and LabCorp `SCV006072505`, last evaluated 2025-03-18.

```text
https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=clinvar&rettype=vcv&is_variationid=true&id=974782
```

The defect comes from inherited upstream staleness. BioMCP's ClinVar section reads only MyVariant fields. A direct MyVariant request for the same GRCh37 allele still reports `last_evaluated: 2020-08-04`, `number_submitters: 1`, and the single-submitter review status. `MYVARIANT_FIELDS_GET` at `src/sources/myvariant.rs:18-52` does not request either freshness field. The rendered JSON identifies the section source as ClinVar and omits MyVariant, the evaluation date, and the number of submitters. A caller cannot tell that it received an indirect historical snapshot.

BioMCP owns the misleading provenance and missing freshness signal. MyVariant owns the stale payload.

## Cheapest useful shape

1. Request and show MyVariant's ClinVar evaluation date and submitter count. Label the data as ClinVar through MyVariant. This makes the current limitation visible without adding a provider.
2. Make `get variant <id> clinvar` retrieve the current VCV record through NCBI EFetch. Preserve the aggregate classification separately from each SCV submission, including accession, submitter, classification, condition, review status, evaluation date, assertion method, citations, and public comments when supplied.
3. Keep the fast indirect summary on the default variant card if direct retrieval would slow every lookup. Mark its provenance and age. Use the direct record for the explicit `clinvar` section.

## Success criteria

- The fixed HSD17B4 record reports the current two-submitter aggregate without conflicts.
- Human-readable and JSON output identify direct ClinVar data separately from ClinVar data carried by MyVariant.
- Every aggregate and submission-level classification carries the available evaluation date and accession.
- The output never turns several submissions into one unqualified assertion.
- An unavailable direct request leaves the dated indirect result visible with a clear partial or stale status.
- A deterministic NCBI fixture with two SCVs proves the aggregate and submission contract.

NCBI documents EFetch by Variation ID and the VCV XML as the complete public record. The verified response was 23,379 bytes, so an exact on-demand lookup does not require the bulk monthly archive.
