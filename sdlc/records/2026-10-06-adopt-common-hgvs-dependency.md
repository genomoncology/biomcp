# Adopt the common HGVS dependency

October 6, 2026. The 1.0 coordinator accepts fresh Medium review of6f95040fae14868017bba22c5bb3323cbaf9c39d and lands this dependency Quick Fix on biodata/biomcp-1.0.

Only Cargo.toml and Cargo.lock change. BioData remains0.0.36 and advances to accepted4f06f546b76c1eb60837fee7e56d25bbd4a9e051. One package identity remains; no other dependency, provider, API, model or test expectation changes.

All86 selected existing consumer tests pass. They cover complete coding edit retrieval, gene coding detail, coding/genomic interpretation, callable/CLI/MCP outputs, output mapping, typed schema, article admission/error envelopes and credential privacy. M5 reused1.0 target, offline locked dependencies, two compile jobs. Test compilation59.72s/process59.75s; native compilation30.59s/process30.62s. Selected bodies total41.30s. These are directional affected checks, not a full release gate.

The common dependency adoption is complete. Broader migration and release qualification remain in the existing queue. The0.9 line remains untouched. No live provider, source acquisition, service, hosted job or deletion ran.

Evidence: /private/tmp/biomcp-common-hgvs-adoption-handoff-20261006.md, /private/tmp/biomcp-common-hgvs-adoption-review-20261006.md and their raw logs.
