# Fresh Linux offline qualification

## Initial snapshot

Job `2233/1792416130767-75930` passed on commit `096c11fc416ddc18ee3b05814a6d0b9fede63648`, tree `d4cad8b1a90efff48a0d9bc380c4e2cb017696fa`. This qualifies that snapshot; it does not qualify later cache or replay changes. The required final-candidate run will supply separate evidence.

The job prepared a fresh clone and dependencies online with `cargo fetch --locked` and `make sync-python-dev`. Every actual build and test stage then ran through `tools/run-offline`. Its verifier demonstrated blocked public DNS and direct TCP, working local transports, zero capabilities, NoNewPrivs, and correct host ownership mapping. Cargo and nextest used two workers; Python ran sequentially. Exact commands, compiler identities, source inventory and output hashes remain with the job evidence.

| Selection | Result |
| --- | --- |
| Routine Rust | 4,377 passed; 34 ignored |
| Explicit long Rust profile | No cases selected; this revision declares no actual long cases |
| Full Python collection, including audit and release-marker packaging | 1,345 passed; five skipped |
| Strict documentation | Passed |
| All-feature Clippy, warnings denied | Passed |
| Declared AlphaGenome behavior selection | Six passed |
| All-feature development CLI and PNG, SVG and terminal output | Passed |

The Rust exclusions are 31 live cases and three subprocess helpers. Python exclusions are the explicitly opted-out live-update module, one landing-only audit case, and three namespace or outer-wrapper checks requiring an unisolated caller or unavailable nested namespace. `evidence/exclusions.json` retains the exact names and reasons. The AlphaGenome selection filtered 4,234 other library tests according to the declared selection. Non-Linux platform code, optimized release rehearsal, standalone specifications and undeclared feature combinations were outside this run.

No demonstrated gap required a fix ticket. All 4,204 tracked files remained unchanged. Cleanup job `2233/1792418181849-127273` verified source identity, retained the evidence and removed only the owned fresh clone and build output.

Retained evidence: `job-runs/biomcp/2233/1792416130767-75930/`, including `log`, `state`, source verification, commands, Python XML, exclusions and artifact hashes under `evidence/`. Log SHA256: `d14d5e69e7717a863b41993f0c102caa1fe4d3b7b46407d04fb107877567b500`.
