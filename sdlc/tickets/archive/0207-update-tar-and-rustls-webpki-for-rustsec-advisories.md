---
flow: build
priority: 8
---
# Update tar and rustls-webpki for RustSec advisories

`cargo audit` reports three active advisories against the committed lockfile: `RUSTSEC-2026-0049` (rustls-webpki 0.103.9) and `RUSTSEC-2026-0067`/`RUSTSEC-2026-0068` (tar 0.4.44). The `tar` crate is used in three archive-handling paths (`cbioportal_download`, `update`, `pmc_oa`). While the cBioPortal path already performs its own entry-path validation, shipping a tagged release with known advisories is not acceptable.

Completed under March on 2026-04-14, as March ticket 207. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/207-update-tar-and-rustls-webpki-for-rustsec-advisories
