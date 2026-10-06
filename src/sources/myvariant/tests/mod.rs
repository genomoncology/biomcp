//! myvariant tier tests.
//!
//! Tier 2 (`construction`) asserts the pure `RequestPlan` builders and the filter
//! normalizers; Tier 3 (`parsing`) decodes committed fixture bytes. Both are pure — no
//! mock HTTP server, no env var, no lock. Tier 4 (`live`) hits the real API and is
//! `#[ignore]`d (verify-lane / parity only).

mod construction;
mod live;
mod parsing;

mod dbnsfp_adoption;

mod clinvar_adoption;

mod dbsnp_adoption;

mod gnomad_adoption;

mod cadd_adoption;

mod cosmic_adoption;
mod exac_adoption;
