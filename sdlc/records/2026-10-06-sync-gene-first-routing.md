# Sync current gene-first routing

October 6, 2026. The1.0 coordinator lands3dfd1607893fd467585dfa20abac268d0d28ec19 after fresh High whole-code review. It incorporates main dc7b7acf653989911e9791e4728807e74514ee81 into the accepted25571b04 migration line. Cargo and the common BioData4f06f546 pin remain unchanged.

The merge retains gene-confirmed variant search, explicit protein/consequence filters and migrated interval/typed variant consumers. Dispatch/query/module and package-test conflicts preserve both behaviors; a duplicate re-export is corrected.

The original three-word fallback phrase cannot become an interval. An unconfirmed gene retains the whole phrase; a confirmed gene follows the new upstream behavior. One public fixture case owns the original input, complete GET parameters and complete stdout. It catches a lost suffix or accidental gene/protein filter. The obsolete intermediate-plan row and its oracle entry are removed; all other21 routing rows and later seams stay unchanged.

Actual checks pass:59 initial affected Rust registrations, final21-row routing owner/later seams, all seven gene-first spec checks, two package checks, exact package-pin owner and existing boundary checker. The checker’s two stale revision literals now name the accepted dependency; its policy is not expanded. Final routing body0.32s/process1.804s, test-only build50.601s. Earlier corrected test compilation63.466s/native29.13s. Other exact phase timings and reused unchanged evidence remain in the handoff.

M5 reused1.0 target/offline Cargo/two jobs and loopback fixture endpoints. All owned fixture processes stopped. macOS is not Linux network-namespace isolation. No whole gate or release qualification is claimed. No live provider,0.9 edit, hosted job or old-data deletion ran.

Evidence: /private/tmp/biomcp-main-sync-handoff-20261006.md, /private/tmp/biomcp-ir16-routing-disposition-20261006.md, /private/tmp/biomcp-main-sync-review-20261006.md and raw logs.
