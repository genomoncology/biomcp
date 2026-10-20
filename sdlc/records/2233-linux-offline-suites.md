# Fresh Linux offline qualification

## Initial snapshot

Job `2233/1792416130767-75930` passed on commit `096c11fc416ddc18ee3b05814a6d0b9fede63648`, tree `d4cad8b1a90efff48a0d9bc380c4e2cb017696fa`. This qualifies that snapshot; it does not qualify later cache or replay changes. The later final-candidate evidence appears below.

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

The initial raw evidence is also retained in the private numbered experiment `170-biomcp-2233-linux-offline-evidence/1792416130767-75930` on the qualifying host. This copy survives job and lane cleanup under the approved retention rule.


## Failed final-candidate attempts

Both attempts remain failed. Each prepared a fresh clone and dependencies online, then built and tested inside the verified Linux network namespace. Each passed preparation, 4,279 routine Rust cases with 34 ignored cases, the empty explicit long profile and the 4,313-case Rust inventory. Each completed Python with 1,587 passed, one failed and five skipped. Neither reached strict docs or the shipped-feature stages.

| Job | Source | Python failure | Repair record |
| --- | --- | --- | --- |
| `2233/1792451222629-265544` | `51f5e03b218c87cedc82bd9e1a67552bd5ab9df8` | `test_initialized_mcp_and_cli_share_cache_during_concurrent_requests`: expected two cache entries, found one | `2283-concurrent-cache-qualification.md` |
| `2233/1792455276832-365318` | `4c582db4186837893e4839f6d47e5e60a129aae7` | `test_parallel_article_sources_fill_cold_cache_then_reuse_answers[command1]`: repeated cacheable requests | `2285-parallel-article-cache-qualification.md` |

The repairs corrected tests that assumed every successful concurrent provider request admitted its response to the best-effort cache. Their controlled probes demonstrated permitted skipped admission and retained strict reuse checks for admitted answers. The original failing case databases were removed by fixture teardown. Their original runtime causes remain unknown; the controlled probes do not establish those causes. The repair records retain the diagnosis and its limits. No product code changed in this ticket.

The raw attempts remain separately retained under `170-biomcp-2233-linux-offline-evidence/final-1792451222629-265544` and `final-1792455276832-365318` on Yellow. Their log SHA256 values are `3d027914386627aa22db3c524ff1779c59aba2c13e4e231677562f311c35db2c` and `9c20b271e1fb2dc49304b60f90b77bff265e5e5d1097ebc9e0eff7a1ace5d88f`. Both owned source and build directories were removed after evidence retention.

## Final candidate

Job `2233/1792458909149-436786` passed with exit zero on `40eac3b38cf7d1f333f6bd34ee6119859547d500`, tree `7009781b5f53f33964cf4fcea01942e4c474f19c`. It used a fresh clone and rebuilt the exact candidate CLI and MCP contract example. Dependency preparation used `cargo fetch --locked` and `make sync-python-dev` before isolation. All subsequent build and test stages ran through `tools/run-offline`, with two Rust workers and serial Python.

The verifier demonstrated blocked public DNS and direct public TCP, working loopback TCP and Unix sockets, zero capabilities, NoNewPrivs and correct host ownership mapping. The final source check verified all 4,357 tracked file hashes and an empty Git status. No live provider qualification or credential-based check ran.

The PM explicitly permitted reuse of the routine Rust, long-profile and inventory evidence from job `2233/1792455276832-365318` on `4c582db4186837893e4839f6d47e5e60a129aae7`. Both the runner and a separate source diff confirmed that the intervening paths were only `tests/test_http_cache_fail_soft.py`, `CHANGELOG.md` and the 2285 ticket and record. Rust, Cargo, build scripts, shared helpers and fixture data files were unchanged. Those Rust components were not rerun on `40eac3b`; the fresh stages below qualify the changed Python test and rebuilt candidate artifacts.

| Selection | Result and source |
| --- | --- |
| Routine Rust | Reused: 4,279 passed; 34 ignored, from `4c582db` |
| Explicit long Rust profile | Reused: zero cases selected; no actual long cases, from `4c582db` |
| Rust inventory | Reused: 4,313 cases, including the declared ignored selections, from `4c582db` |
| Fresh routine CLI and MCP contract example | Built on `40eac3b` |
| Complete Python collection, including audit and release-marker packaging | 1,588 passed; five skipped on `40eac3b` |
| Strict documentation | Passed on `40eac3b` |
| All-feature Clippy, warnings denied | Passed on `40eac3b` |
| Declared AlphaGenome behavior selection | Six passed; 4,124 other library cases filtered on `40eac3b` |
| All-feature development CLI and PNG, SVG and terminal output | Built and passed on `40eac3b` |

Python's five skips were the live update module requiring opt-in, three outer-wrapper or nested-namespace cases incompatible with the enclosing sandbox, and the landing-only repository audit invocation. The complete log and Python XML retain their exact identities and reasons. The reused Rust inventory retains every ignored identity. Non-Linux platform code, optimized release rehearsal, standalone specifications and undeclared feature combinations remain outside this qualification.

The reused Rust commands were `cargo nextest run --archive-file ... -j 2 --config-file ... --no-fail-fast`, the same command with `--profile long --no-tests=pass`, and `cargo nextest list --archive-file ... --config-file ... --ignore-default-filter --message-format json`; the run commands used `tools/report-slow-tests.py`. The fresh job ran `make prepare-routine-test-tmp prepare-test-contracts`, complete `pytest tests/ -v -n 0` with an owned retained base directory and JUnit XML, `mkdocs build --strict`, `cargo clippy --locked --all-targets --all-features -- -D warnings`, `cargo test --locked --all-features --lib sources::alphagenome::tests`, `cargo build --locked --profile dev --all-features --bin biomcp` and `tools/check-png-artifact target/debug/biomcp`. Python commands used `uv run --no-sync` and the prepared `BIOMCP_BIN`; Cargo feature commands used `tools/with-build-identity`. Retained `state` and `log` contain exact commands, environment controls and toolchain identities.

Raw evidence remains on Yellow under `170-biomcp-2233-linux-offline-evidence/final-1792458909149-436786`, including the complete post-exit job log, runner state, qualification result, job result, Python XML, source hashes, isolation output, reuse decision and artifact hashes. The retained job result records exit zero and runtime 2,339,949 milliseconds. The complete log SHA256 is `c107fde84aca562e60b692b118ac3744d6f2b3b6b31f192d930865ec729de30d`; Python XML SHA256 is `973eb35a4d14872dc338e0226134769f235718c30c19b3fb8f7b000b2b9e9eee`. `evidence-sha256.txt` verifies every retained file and has SHA256 `27d61f3b1d9162b4370df337b287580d0dc2e50e881fd23e58a99ecd7bf4cf5a`.

The routine CLI SHA256 is `4cdc0343de03a5db0c84f5bd49ab4fb17abcc894520ced72b2004837cea2eeb6`; the MCP contract example SHA256 is `1c845412289f2c33cf978924c32c52d26b1aac1274102d0b300ba0f3467ba4ca`. The final all-feature CLI SHA256 is `48d8d1175ddac85a6f94e6786346b0c5c105f9614680bd9e7c2aac39c69d9fe4`. These identify separate build stages.

The exit handler removed the owned fresh clone and build directory; a later read-only check confirmed its absence. The initial and all three final-attempt archives remain under the experiment's `OWNER` retention instruction. Remove those archives only when their explicit release authorizes it.
