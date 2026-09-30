# Review of the CI rework and records round

Filed 2026-09-29 from an independent read-only review of main at `37951529`, range `5bb12367..37951529`. Two fresh reviewers covered the CI rework and the honesty of the new evidence, and the completeness of every finding in the three earlier review files. Reviewers used scratch clones, offline fixture tests, read-only GitHub queries and the local cargo registry.

The three earlier review files stay open until each finding below is closed: `2026-09-28-review-follow-ups-after-1263-and-1264.md`, `2026-09-29-review-follow-ups-after-the-0.9.1-resolution-round.md` and `2026-09-29-review-of-the-enrichr-cache-and-records-round.md`. The round is not closed. More remains than the changelog bullets and the version bump.

## What holds

- Main CI run 36636929043 at `37951529` passed all eight jobs.
- Every action in `ci.yml` and `release.yml` is pinned to a commit that exists upstream. The cache action's v2 tag dereferences to `6323deb1`.
- Nextest installs through `scripts/install-nextest.sh`, which checks a real SHA-256 with `sha256sum --check`.
- The restored search-all end-to-end test fails when `src/cli/outcome.rs:150` passes an empty list to `json_body`.
- The dispatch records for the 1267 and 1268 reviews exist, and their findings are specific and were folded in.
- The four 1.0 plan files cite Ian's approval.
- `AGENTS.md` states the red-main rule correctly.
- `sources.json` corrects the CPIC and WikiPathways notes, and the evidence file header says 45/2/1.
- Ticket 1264's Deferred gaps are complete.

## CI

- A green branch run can sit on a failing tree. `scripts/ci-classify-push.sh` diffs against the previous tip, whatever that tip's CI result was. Commit `95a4998b` failed its branch run (36622672905). Commit `7e9b42c5` added only markdown, skipped every Rust job, and passed (36625824357). The ruff and rustfmt fixes that followed (`8c202753`, `4660b29e`) show the tree was still broken. Diff against the merge-base with main, or against the last commit that passed full CI.
- `*.md` counts as docs, but some markdown is executable or compiled:
  - `spec/*.md` holds the executable specs.
  - `skills/*.md` is embedded in the binary (`src/skill_assets.rs:7`).
  - `src/cli/list_reference.md` is compiled in through `include_str!`.
  - `docs/user-guide/cli-reference.md` is read by a Rust test (`alias_alignment_tests.rs:66`).
  A change to any of these skips the Rust jobs. Classify by directory with an allow list, such as `sdlc/`, `notes/` and plain `docs/` pages that nothing compiles or reads.
- The `needs_binary` guard (`tests/test_ci_workflow_contract.py:122-153`) matches a regex and a fixed list of module names. A new module that runs `["cargo","run"]` passes it, and a comment containing `pytest.mark.needs_binary` counts as a marker.
- Tag pushes never start `ci.yml`, and `release.yml` does not wait for CI. The release bar still needs the tagged commit's green run, so the go request must name a main run on that exact commit.

## Evidence and reviews

- All five reviews ran on Pi GLM-5.3. Four ran as the worker agent, which can write. The fifth exited with code 1. The workspace rule requires fresh read-only reviewers from the owning agent's vendor unless Ian routes otherwise. Pi is for grunt work, and its reports need verification.
- Ticket 1268 had three REJECTs before its ACCEPT, not two as reported.
- The final ACCEPT on 1268 covers `95a4998b`. Five later commits have no review, including the ruff fix, the rustfmt pass and the move of the trust tests into a directory module.
- Records 1267 and 1268 still say `Code review: pending`, and their tickets say ACCEPT.

## GWAS runtime pin

- The report says the earlier review was wrong about `extensions()`. The earlier review was right. reqwest-middleware 0.4.2 has `pub fn extensions(&mut self)` on `RequestBuilder` (`client.rs:572`). The failing test called `.build()` and then `reqwest::Request::extensions()`, which is `pub(crate)` in reqwest 0.12.28. That produced E0624 in job 109581878353 of run 36619750222. `build()` also drops the middleware's extensions, so that test could never have passed.
- A working pin reads the extension from the middleware builder before `build()`, for example `let mut rb = client.request_no_store(&plan); assert_eq!(rb.extensions().get::<CacheMode>(), Some(&CacheMode::NoStore));`. Use the real builder and plan names.
- Correct the false text: "vendored reqwest" and "exactly the recorded reason" in `tests/test_gwas_no_store_contract.py` and record 1268. The 1255-1261 disposition (line 75) says GWAS is "now pinned at runtime", which is false today.

## Still open from the earlier files

1. The changelog gate still caps ticket numbers at 1999 (`scripts/check-changelog-coverage.py:26`, `:71`). The date-shape rejection was added next to the cap instead of replacing it. No test uses a ticket number of 2000 or above. Narrowing the regex to `[01][0-9]{3}` still passes all 20 gate tests.
2. The `--release --locked` check (`tests/test_release_workflow_provenance.py:703-708`) still matches substrings across the whole step.
3. The raise-review check still accepts `Code review: pending; reviewer returns ACCEPT or findings` and "will ACCEPT after fixes". A raise can still cite any old accepted ticket.
4. The trust-failure test covers only `error_chain_carries`. Deleting the early return in `NoTrustFailureStrategy::handle` (`src/sources/mod.rs:1193-1197`) passes every test. Test `handle` itself.
5. The README check (`tests/test_public_landing_copy_docs_contract.py:65`, `:118-123`) is unchanged since `ac51eceb`. It still accepts any "terms" or "restrict", and the hero check is still `>= 1`.
6. `### Enrichr` (`docs/reference/source-licensing.md:346`) still sits under `## Tier 1`. AlphaFold DB, ClinVar, dbSNP, Disease Ontology, Drugs@FDA, LitSense2, MONDO and PDB are tier 1 and sit under the Tier 3 heading. The tier-agreement test (`tests/test_source_licensing_docs_contract.py:436`) checks table rows only and ignores headings. Six sources skip the tier check.
7. Records 1265-1269 cite no green run. Record 1269 cites only failed runs 36530006688 and 36530006673. Record 1265 says "the cap stays". Record 1268 names `trust_failure_tests.rs`, which is now `trust_failure_tests/mod.rs`. Tickets 1276 and 1278 have no records.
8. The 0.9.1 resolution-round review file has no disposition section. The 1263/1264 disposition cites only failed runs. The Enrichr-round disposition cites no run.
9. The 1255-1261 disposition still has no lines for DDInter, MyDisease.info, the stderr bound, the duplicate sentences, COSMIC, the GenCC stand-in or the yellow fragment.
10. The 2026-09-26 disposition (lines 68-80) still says "pending-review merge", "the 1251 fix merge", "1250 fix merge", "1252/1248", "(in flight)" and "1242 b1".
11. Ticket 1253 lines 13-17 are a garbled sentence with "1254 b2". Ticket 1278 keeps a stale "must be filled in before merge" paragraph and "~700/850 s".
12. Shorthand remains: "535747d9's chain" (1263 file lines 108 and 114, 1255 file line 75), "round-three" (record 1269, ticket 1264) and "P2s" (record 1265). The pass record line 59 keeps the fragment "The owner of the BioMCP queue."
13. Still hard-wrapped: tickets 1253, 1260, 1264, 1267, 1268 and 1275, the pass record and the evidence file header.
14. The Enrichr and UniProt plain-send path still retries trust failures. Record 1268 defers it with no approval from Ian. Fix it, or bring it to Ian as a deferral with its reason.
15. The changelog gate reports 41 missing bullets for a v0.9.1 tag at `37951529`. `Cargo.toml` is at 0.9.1-dev.1.

## Disposition (2026-09-30, ticket 1280 branch)

1. The green-run-on-broken-tree hole: closed — the classify base is the merge-base with origin/main, proven by a behavioral test that replays the exact 95a4998b/7e9b42c5 shape (a code commit followed by a markdown commit classifies full CI).
2. Executable markdown: closed — the allow list is sdlc/, notes/, CHANGELOG.md, AGENTS.md and .github/*.md; spec/, skills/, src/, docs/ and README.md all run the full suite (docs/charts is compiled in and benchmark_cli_structure reads README and every docs page), each pinned by contract and behavioral tests.
3. The needs_binary guard: tightened — the marker must parse as a real module-level pytestmark (AST), and any quoted cargo argv marks the module unless it is a recorded fake-fixture; the poisoned marker inside a generated fixture string was caught by exactly this check.
4. Tag pushes never start ci.yml: recorded in the release record and the runbook — the go request names a main run on the exact tag commit; the release workflow runs on the tag push itself.
5. Review vendor: the round's ACCEPT came from a read-only Claude-vendor reviewer (dispatch 4e671b58); its file-read-only limitation is recorded in ticket 1279, and the go-request review re-verified the post-ACCEPT commits with mutations.
6. Three REJECTs on 1268: the ticket's history records all three; the earlier report's "two" was corrected.
7. Records 1267/1268 verdicts: both match their tickets (ACCEPT with dispatch records); record 1268's plain-send boundary text and test path corrected 2026-09-30.
8. The GWAS extension claim: resolved — reqwest-middleware 0.4.2 exposes RequestBuilder::extensions publicly; the pin reads it before build(); the false "vendored reqwest" text is gone from the contract, the record and the 1255-1261 disposition.
9. Gate cap: removed; the gate test plants tickets 2000 and 2027 counting.
10. --release --locked per command, raise-review promise phrases, README/hero strictness, licensing headings, records, dispositions, shorthand and wraps: closed on the 1279 branch and re-verified by the 2026-09-30 review's mutations.
11. The plain-send trust retry (Enrichr, UniProt): fixed, not deferred — the loop returns at the first trust failure with an async single-attempt proof.
12. The changelog and version bump: landed after the ACCEPT and re-verified; the go-request review confirmed a scratch tag passes version sync, release checks and the 53-ticket changelog gate.
13. Twelve bisect-breaking intermediate commits: recorded in ticket 1279's evidence; the tip runs are green and the branch is deleted.
