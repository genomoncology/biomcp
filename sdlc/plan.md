# BioMCP plan

This plan describes future work and release exits. Read status from pm. BioMCP 0.9.4 is the active release; work on the 1.0 line stays paused.

## Outcome

Ship 0.9.4 with BioMCP's cache in one SQLite database, the 0.9.3 release review's follow-ups tested or fixed, development builds back to v0.9.2's speed, and the moved test work on the replay server.

## Order

1. Land the after-release records for 0.9.3 first: the next development version, an empty Unreleased changelog section and the release record.
2. Run the priority-2 lanes in parallel where their claims do not collide: development build time (2191), tests on the production cache paths (2199), the full-disk and disabled-source reviews (2202, 2204), whole-program tests for search all, update and uninstall (2213), and the Windows cache review (2201) once its execution path is approved. Reviews report defects as new tickets rather than widening their own scope.
3. Let the cache reviews' findings set the requirements for the SQLite cache (2134) before it starts.
4. Serialize manifest and profile edits: the tonic and prost upgrade (2077), the toml 1.x trial (2210) and release linking by measurement (2211).
5. Serialize the overlapping replay and fixture work: replay server and golden output (2079), fixture consumers (2111) and fixture identifiers (2163).
6. Fit the remaining product and review tickets around these: rsID routing in plain article search (2184), clearer output and assembly errors (2200), the mixed-version and long-running server reviews (2203, 2205), and the remaining measurement rows (2214). Transcript aliases (2185) wait for a clinical reviewer's ruling.

Six lanes at most. Land accepted changes in batches that share one gate, merge the latest main before testing, and run the full main check after each landing. Before any release rehearsal, run the full Rust and Python suites once on Linux from a fresh clone with the network off.

## Release exit

Every 0.9.4 ticket is landed or withdrawn through pm, main passes the required full local check, the release measurement compares v0.9.3 and the candidate on one build machine, and the complete manual release workflow passes on final main without a tag or publication. Ian then authorizes the tag.

## Hardest problems

These are the hardest problems BioMCP faces today, written for readers outside the team.

1. **Many outside sources, one answer.** BioMCP answers from dozens of public biomedical services with different speeds, rate limits and outages. A combined search waits for its slowest source, and one slow service can dominate the reply time.
2. **Naming variants without guessing.** The same protein change can be written several ways and numbered differently on different transcripts. BioMCP must match the forms a clinician means and refuse rather than return a confident wrong variant.
3. **Telling drugs, diseases and genes apart.** Brand names, combinations, biosimilars and ambiguous abbreviations such as MM or MF can point to the wrong record. BioMCP must pick the reading a clinician means or say why it cannot.
4. **Getting full text and supplements.** Many papers sit behind paywalls, publishers refuse scripted downloads, and old scans carry no text. BioMCP must find what is openly available and say plainly when it is not.
5. **A shared local cache that never breaks a request.** The cache must be fast and safe when many programs and a server use it at once on Windows, macOS and Linux, through upgrades, full disks and damaged files.
6. **Honoring source licenses.** Users must be able to switch off sources whose terms they cannot accept, including data those sources reach through other services, and BioMCP's records of each source's terms must stay accurate.
7. **Honest errors.** A user or AI agent needs to know whether a setting is wrong, a source is down, or the data does not exist, because each calls for a different next step.
8. **Testing against live services without flaky results.** Tests replay recorded provider replies offline, and any test that silently reaches the network or the tester's own files gives results that differ by machine.
9. **The same results on every platform.** Tests can pass on one operating system and fail on another, and routine testing on Windows is limited.
10. **Releases that pass on the first rehearsal.** Each full release rehearsal builds every package from a clean machine, and gaps in clean-machine setup have forced repeated rehearsals.
11. **Build time on a large codebase.** BioMCP has over 300,000 lines of Rust, nearly half of it tests, and development builds got slower in 0.9.3 even as tests got faster.
12. **Tests that prove what users run.** Some code paths are tested through test-only helpers instead of the code users run, and the suite must keep only tests that prove real behavior.
