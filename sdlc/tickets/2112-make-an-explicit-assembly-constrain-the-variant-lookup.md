# 2112 — Make an explicit assembly constrain the variant lookup

Status: COMPLETE.

Milestone: 0.9.3

Reviews: revision 6cb464c, accept

Reviews: revision 6cb464c, accept

## Outcome

`biomcp get variant --assembly hg38 'chr7:g.140453136A>T'` looks the coordinate up on GRCh38 only and reports not found when GRCh38 has no hit, the same as `GRCh38:chr7:g.140453136A>T`. It never answers with the GRCh37 record. An unprefixed coordinate with no flag keeps the dual-build search.

## Evidence

Found 2026-10-16 by an outside QA audit of the v0.9.2 release binary on recorded public replies, and confirmed against main by reading the code and querying MyVariant.

- `src/entities/variant/resolution.rs:175` sets `requires_comparison: genome_build.is_none()`, where `genome_build` comes only from a coordinate prefix, so the CLI flag never turns the comparison off.
- `src/cli/variant/dispatch.rs:326-330` passes `--assembly` through `resolved_default_assembly` (`src/cli/variant/mod.rs:99-104`) as a preference only.
- `src/entities/variant/get.rs:1065-1100` queries both builds and, at 1099-1100, takes the other build when the preferred one is not found, with no warning and empty build candidates.
- MyVariant on 2026-10-16: `chr7:g.140453136A>T` with `assembly=hg38` answers 404; with `assembly=hg19` it answers 200 with BRAF.
- `docs/user-guide/variant.md:33-37` says an explicit `--assembly` always wins; `spec/entity/variant.md:129-131` says the command sends the declared build to MyVariant.
- No test covers an explicit build that misses while the other build hits.

- Starts from: the lines above.
- Decisions:
  - A declared build, from the flag or the coordinate prefix, constrains the lookup to that build. Both declarations behave the same.
  - A flag that conflicts with a coordinate prefix refuses with both builds named.
  - `BIOMCP_DEFAULT_ASSEMBLY` stays a preference: it orders the dual-build search but does not constrain it.
- Changes: claims `src/entities/variant/resolution.rs` `src/entities/variant/resolution/**` `src/entities/variant/get.rs` `src/entities/variant/get/tests.rs` `src/cli/variant/**` `testdata/sources/myvariant/**` `testdata/sources/capture-receipts.json` `spec/entity/variant.md` `docs/user-guide/variant.md` `CHANGELOG.md`.
- Proof: recorded-reply tests, each failing on current main: flag hg38 on the GRCh37-only coordinate refuses as not found; flag hg19 on the same coordinate finds BRAF; flag and prefix agree on a one-build-only hit; a conflicting flag and prefix refuse. The existing dual-build tests for unprefixed coordinates pass unchanged.
