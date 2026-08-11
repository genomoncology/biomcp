# BioMCP build and test performance intervention — 2026-08-11

## Decision

Pause the BioMCP queue and do the performance work directly in one persistent
worktree. Keep one ticket per independently reviewable behavior, complete them
sequentially on `performance/build-flow`, and archive each ticket with its
before/after evidence. Do not pay a fresh worktree, preflight, design, and
review setup cost for every performance ticket.

The first objective is the fastest truthful local feedback loop. Quality gates
are added back only where they catch a distinct class of failure. Re-running
the same complete gate against the same commit is not additional evidence.

## Pinned baseline

- Repository: `/home/ian/workspace/repos/biomcp`
- Worktree: `/home/ian/workspace/worktrees/biomcp-performance`
- Branch: `performance/build-flow`
- Base: `bf55ce01e8c5d3b074769e09cd02c987f8ab148c`
- Machine: Ryzen 7 5825U, 16 logical CPUs, 27 GiB RAM
- Toolchain: Rust/Cargo 1.93.1, nextest 0.9.132, sccache 0.14.0,
  mold 2.30.0, uv 0.8.0
- Conditions: Queue and SDLC flights were active, so these are loaded-machine
  measurements representative of normal use, not isolated best cases.

| Gate | Wall time | Important decomposition |
|---|---:|---|
| cold `make lint` | 130.06s | Clippy itself finished in 57.67s |
| warm `make lint` | 84.35s | warm Clippy 0.57s; `bin/lint` 40.22s; quality ratchet 33.17s |
| cold `make test` | 884.19s | Rust compile 108s; 2,838 Rust tests 49.45s; 463 Python tests 593.65s |
| warm-binary `make spec` | 592.42s | 53.07s user CPU and 14.74s system CPU: mostly serialized waiting |

The three ordinary gates therefore cost 26m46s once, before any agent reading,
design, implementation, or review time. A completed representative flight,
0951, took 2h16m54s: design 12m54s, design review 14m51s, code 37m53s, code
review 46m47s, and verify 24m29s. The current build flow can invoke lint six
times, test four times, and spec once, while stage agents often invoke the same
complete suites themselves.

## Where the time goes

1. `make test` spends about ten minutes in sequential Python tests. Existing
   ticket 0914 identified the main cause: negative fixtures repeatedly launch
   the complete 33-second quality ratchet instead of the one audit they mutate.
2. `make spec` is serial after the migration from pytest-xdist to the standalone
   Mustmatch binary. Mustmatch has no parallel option. Historical ticket 0187
   added file-level parallelism, but commit `44ffdd31` removed that execution
   model during the binary-runner migration.
3. `spec/entity/variant-article-identity.md` launches the same expensive fixture
   script six times. Each invocation performs six BioMCP requests; the page
   recomputes the same JSON document for separate assertions.
4. `spec/surface/cli-contract-ratchet.md` launches pytest from inside `make
   spec`, including a complete quality-ratchet run already owned by `make test`.
   Other nested Cargo builds are already owned by ticket 0892.
5. Warm `bin/lint` takes 40 seconds even though Clippy takes less than one
   second. Its shell implementation repeatedly starts grep for individual files
   and, in the documentation scan, for individual lines.
6. Routine Cargo lanes do not use the same feature graph. Tests and specs use
   `--no-default-features`; lint enables the default AlphaGenome gRPC/protobuf
   stack. This defeats dependency reuse and makes ordinary lint run `protoc`
   for a feature the routine corpus cannot exercise.
7. `build.rs` watches Git HEAD and exports commit identity to the whole package.
   Every stage commit can therefore invalidate compilation of the roughly
   196,000-line main crate even when the product source is unchanged.
8. A small group of Rust tests writes production-sized caches, rewrites more
   than a thousand session files, or waits through production retry behavior.
   Existing ticket 0964 already owns those seams. Rust execution as a whole is
   currently only about 49 seconds, so this follows the larger Python/spec wins.

## Existing tickets

### Pull forward and implement in this session

1. **0914 — Run only the quality audit a test mutates.** Largest measured
   `make test` win. Keep one full wrapper integration test.
2. **0892 — Build routine specification artifacts once.** Remove nested Cargo
   work and give every spec an explicit prepared artifact.
3. **0936 — Keep normal builds from rewriting generated source.** Removes an
   unsafe build side effect and makes cache inputs stable.
4. **0964 — Inject small persistence and retry test boundaries.** Removes large
   writes, 1,030-file rewrites, and real retry waits after the larger lane-level
   costs are gone.

### Inspect, but do not assume they are speed wins

- **0965 — Ratchet large modules and dead-code allowances.** Use its inventory
  to guide local cleanup, but do not split files or delete dependencies without
  compile-profile evidence. More modules can increase compile overhead if the
  dependency boundary is wrong.
- **0895 — Doc-only pre-commit path.** Useful for developer hooks, but it does
  not fix the factory or this one-worktree intervention.
- **0896/0897 and 0941.** Fixture supervision and short socket paths improve
  reliability. Pull them forward only if parallel execution exposes a real
  lifecycle or path failure.
- **0939.** The compatibility executable cleanup is product/package work, not a
  leading compile-time cause.

### Already completed and still valuable

- 0187 adopted nextest and file-level pytest-xdist execution.
- 0501 reused one spec-profile binary across test and spec.
- 0591 moved live contracts out of routine test.
- 0621, 0624, and 0625 removed live/network, duplicate-spec, and AlphaGenome
  costs from routine validation.
- 0644 hardened fixture lifecycle behavior.

The current measurements show two regressions from that work: Python unit tests
no longer use xdist, and executable Markdown pages lost file-level parallelism
when the Mustmatch runner changed.

## New tickets

The following records capture measured gaps that no existing runnable ticket
owns:

1. **0966 — Scan tracked lint inputs once.** Replace per-file/per-line shell
   subprocesses while keeping every existing lint finding and message.
2. **0967 — Execute each expensive routine spec fixture once.** Share one
   captured result across the assertions on a page.
3. **0968 — Restore bounded parallel execution to routine spec pages.** Start
   with the isolation contract and a conservative worker cap; preserve useful
   failure output and cleanup.
4. **0969 — Keep routine specs from launching gates owned by `make test`.** A
   spec proves product behavior; source-policy pytest belongs in the test lane.
5. **0970 — Isolate build identity from reusable Rust compilation.** A commit
   change must not rebuild the entire library merely to update version output.
6. **0971 — Use one small Cargo graph for routine lint, test, and spec.** Run a
   full-feature check in the release lane, not at every ordinary stage.

## Implementation order

Use measured payoff and risk, not ticket number:

1. 0914, then 0966: remove the known repeated scans from test and lint.
2. 0967 and 0969: remove duplicated work inside the serial spec lane.
3. 0968: parallelize the now-minimal page set, starting with a low worker cap
   and raising it only while the isolation contract remains green.
4. 0892: make all remaining executable artifacts preparation-owned.
5. 0971, 0936, and 0970: make routine compilation shareable and stable across
   commits without weakening release-feature coverage.
6. 0964: shrink the remaining slow Rust tests.
7. Profile the resulting build graph before selecting any 0965 dead-code or
   dependency deletion. Remove only code proven unused and dependencies whose
   removal materially reduces the graph.

After every ticket, run the smallest affected red/green test first. Run the
complete three-gate baseline only at meaningful checkpoints and once at the
sealed end. Record wall, user CPU, system CPU, peak memory, commit, feature set,
and whether the machine was loaded.

## Fastest safe ticket flow

For this intervention:

1. Write the behavior and its failing focused test.
2. Review the behavior and test diff without running unrelated complete gates.
3. Implement and run the focused test until green.
4. Review the code diff and run tests for changed surfaces.
5. At a checkpoint, run each affected complete lane once.
6. At the end, run one sealed `make lint`, `make test`, and `make spec`, then
   merge the whole branch.

For the future factory, preserve independent design/code review but stop using
complete-suite repetition as the handoff mechanism. Cache a green verdict by
exact commit, let stage gates select changed surfaces, and require one sealed
full validation at the final commit. The existing SDLC issue
`prepare-reruns-the-whole-suite-on-every-claim.md` already owns reuse of an
exact-commit green preflight and should be promoted in the SDLC repository.

## Success criteria

- Warm `make lint`, `make test`, and `make spec` together are at least 3x faster
  than the 26m46s intervention baseline.
- The final run retains every existing assertion and every distinct full gate.
- Repeated runs leave the worktree clean and no fixture processes alive.
- A commit that changes only planning or tests does not force an unrelated full
  crate rebuild solely because Git HEAD moved.
- The recommended factory flow performs one complete final validation per
  candidate commit, not one per conversational stage.

## Completed results

### 0914 — named quality audits

Commit `5175790c` added named audit selection, kept one full wrapper integration,
and shared one Rust source snapshot between the two whole-tree Rust audits. The
complete sequential Python contract lane fell from 593.65s to 93.77s on the
same loaded machine: 6.3x faster, saving 499.88s per `make test`. All 465 tests
passed.

### 0966 — one-pass tracked-text lint

Commit `b408f23c` replaced shell subprocess loops with one tracked-file
collection and at most one read per file. All 16 lint-contract tests pass.
Warm `bin/lint` fell from 40.22s to 8.82s (4.6x), and complete warm `make lint`
fell from 84.35s to 26.35s (3.2x).

### 0969 — no nested test gates in routine specs

Commit `df0dce06` removed the duplicate CLI-surface pytest launch from
`make spec` while retaining its direct `make test` ownership. A shell-aware
source ratchet now rejects nested test/lint gates in executable Markdown and
fixture helpers without matching prose, quoted output, or `make -n`. All 49
focused tests and the complete named spec-lint audit pass.
