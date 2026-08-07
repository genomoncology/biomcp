---
base: 290ee9158db3b363478929e8b56061cc4249de43
head: f067ff92e2c26666f310516fda714249ea8c499c
---
`cargo audit` reports three active advisories against the committed lockfile: `RUSTSEC-2026-0049` (rustls-webpki 0.103.9) and `RUSTSEC-2026-0067`/`RUSTSEC-2026-0068` (tar 0.4.44). The `tar` crate is used in three archive-handling paths (`cbioportal_download`, `update`, `pmc_oa`). While the cBioPortal path already performs its own entry-path validation, shipping a tagged release with known advisories is not acceptable.

Imported from March ticket 207. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/207-update-tar-and-rustls-webpki-for-rustsec-advisories
