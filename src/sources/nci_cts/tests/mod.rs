//! nci_cts tier tests.
//!
//! Tier 2 (`construction`) asserts the pure `RequestPlan` builders; Tier 3 (`parsing`)
//! decodes committed fixture bytes. Both are pure: no local server, env var, or lock.
//! Tier 4 (`live`) hits the real API (needs `NCI_API_KEY`) and is `#[ignore]`d.

mod live;
mod parsing;
