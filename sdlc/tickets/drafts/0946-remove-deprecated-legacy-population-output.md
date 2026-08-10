---
flow: build
priority: 1
deps: ["0878"]
---
# Remove deprecated legacy population output in the following minor release

Held as a draft until the first public release containing ticket 0878 has been
available for its documented compatibility cycle. If that release is v0.8.26,
promote this only for v0.9.0 or a later breaking release.

## Removal contract

Remove the deprecated `legacy_population` JSON/Markdown object and its
MyVariant/legacy gnomAD/ExAC retrieval from variant population output. Direct
`gnomad_r4` fields, exact dataset/release identity, missing/unavailable states,
and GRCh38 requirement remain. No legacy value becomes a fallback or is copied
into the v4 model during removal.

Design may restate schema snapshots, fixtures, skills, examples, and migration
documentation that deliberately preserve the one-release compatibility field.
The release notes name the breaking removal and its replacement fields.

The src line ceiling must fall by the removed compatibility implementation or
the design review must identify the exact retained shared code.
