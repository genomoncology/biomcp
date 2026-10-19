# Release linking measurement

## Decision

Keep thin link-time optimization and one codegen unit. The sixteen-unit candidate had lower observed release-build times and a larger binary, but the runtime collection did not establish no worse program speed. Program speed takes precedence. No profile, source, dependency or measurement-helper change is proposed.

The bounded collection stopped after the full-article output check failed. Its changing field was emitted timing metadata, not a demonstrated product defect. The remaining three replay cases were not run. The incomplete runtime collection cannot justify adoption; no extra samples, candidates or live-provider calls were collected to change that conclusion.

## Product and settings

The published measurement revision was `304a5f578b671ee3cb472fa24b0330772a6d701d`. Its product files match main `08b74a340658f2c30e158fd00459330baebacd74`; only this ticket's metadata differs. Both arms used that identical revision and locked dependencies. The qualified main passed `make main-gate` in job `2156/1792409329667-86161`. That development-profile qualification supplies no release-build timing.

The current release profile sets `lto = "thin"`, `codegen-units = 1`, `panic = "unwind"` and `strip = true`. The candidate changed only the effective codegen-unit setting through `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16`; the current arm explicitly used `1`. No tracked manifest or source file was edited. Each arm's tracked-status check was empty.

All samples ran on one Apple arm64 build host with 18 logical CPUs, Rust/Cargo 1.95.0 and sccache 0.17.0. Runtime preparation selected Python 3.12.13. The ordinary populated compiler cache remained enabled and shared, with the platform linker. Each build started in an immutable clone with an empty target directory. `CARGO_INCREMENTAL` and `CARGO_TARGET_DIR` were unset; host configuration disabled incremental compilation. No Rust flags or worker-count environment override was present. Other jobs could run throughout; no quiet window was requested. Global cache counters cannot attribute cache activity to these samples.

The existing helper was frozen at SHA-256 `98ed6f7128e4a2122c0ff69d33b142327c911c1c85fa1d81f9de87ccb9dc8c6b`. Its selected invocation was:

```sh
bash <frozen-helper> <arm-clone> --only release-build,stripped-binary
```

The helper timed this command with the native timing utility and retained Cargo timing reports:

```sh
tools/with-build-identity cargo build --release --locked --timings
```

The frozen helper's banner describes the current one-unit profile even in the candidate arm. The per-attempt command records and environment override identify the effective candidate setting. The helper's unused TOML-parser fallback can invoke `uv run`; the selected release and size sections do not execute that fallback.

## Release-build samples

The fixed pair order was current/candidate, candidate/current, current/candidate. All six build commands passed in job `2211/1792410321987-31873`. Queue admission, Python preparation, cloning, binary capture and target cleanup were outside the timed commands. No build was repeated after these six samples.

The table uses native timing output with finer precision than the helper's rounded wall result. CPU time and peak resident memory are the timing utility's process-tree observations, not host-wide accounting of compiler-cache server work. Load is the one-minute host average before and after the measured section; complete vectors and cache snapshots remain in the raw evidence.

| Pair | Arm | Wall s | User s | System s | Peak resident bytes | Load before | Load after |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | Current | 325.48 | 216.04 | 10.80 | 4,511,907,840 | 10.07 | 11.63 |
| 1 | Candidate | 108.14 | 406.52 | 16.02 | 5,025,038,336 | 11.98 | 47.52 |
| 2 | Candidate | 97.90 | 400.59 | 15.27 | 5,046,534,144 | 47.52 | 51.39 |
| 2 | Current | 324.69 | 211.04 | 10.48 | 4,494,311,424 | 51.39 | 6.85 |
| 3 | Current | 336.88 | 221.34 | 10.04 | 4,521,115,648 | 7.58 | 10.43 |
| 3 | Candidate | 104.40 | 381.67 | 16.76 | 4,975,214,592 | 10.43 | 69.78 |

Current median wall time was 325.48 seconds, range 324.69–336.88. Candidate median was 104.40 seconds, range 97.90–108.14. These paired observations show substantial current release-build cost on this host. Shared load and compiler-cache history prevent attributing the entire difference to codegen units or predicting an idle-host saving.

Every current stripped binary was 34,015,216 bytes. Every candidate stripped binary was 44,654,720 bytes. The candidate adds 10,639,504 bytes. The experiment retained all six immutable binaries and Cargo timing reports before deleting each clone's target output.

## Local runtime samples and stopping boundary

The runtime runner imported the existing `benchmarks/output-footprint/run.py` replay server, cases and isolated environment. It did not change that file. The fixed corpus contains compact and full PubMed article search, variant search, gene pathways and trial search. Inputs were the repository's committed provider fixtures plus the replay handler's existing synthetic responses. Existing source terms remain described in `docs/reference/source-licensing.md`; no production data, new data acquisition or provider licence was required.

The runner measured only each CLI subprocess using `time.perf_counter`, retaining stdout, stderr, exit status, output hash and load vectors. Replay-server setup, module import and tokenization were outside the measurement; no tokenization was performed by this timing wrapper. It used the captured first-pair binaries, one private cache root and the existing cache-off environment. Provider endpoints used the existing loopback replay configuration. This was not an operating-system network-isolation test.

Five pairs per case alternated current/candidate and candidate/current order. No warm-up samples were discarded. The existing invocation arguments were:

```sh
biomcp --json search article -g BRAF --source pubmed --limit 2
biomcp --json search article -g BRAF --source pubmed --limit 2 --full
```

All twenty collected subprocesses exited zero in job `2211/1792411663763-42983`. Samples below are seconds in collection order for each arm.

| Case | Arm | Five samples | Median | Range |
| --- | --- | --- | ---: | --- |
| Compact articles | Current | 0.963063, 0.352794, 0.354855, 0.349840, 0.350439 | 0.352794 | 0.349840–0.963063 |
| Compact articles | Candidate | 1.312261, 0.351021, 0.349545, 0.352536, 0.346326 | 0.351021 | 0.346326–1.312261 |
| Full articles | Current | 0.349191, 0.347169, 0.348015, 0.348793, 0.349612 | 0.348793 | 0.347169–0.349612 |
| Full articles | Candidate | 0.352858, 0.351906, 0.350155, 0.347472, 0.350044 | 0.350155 | 0.347472–0.352858 |

All compact outputs matched at 1,501 bytes and SHA-256 `23a9ec71c9b92b6ea7cf27819df48e68353a32c1ff9bb4af55d8617c329ee29a`. Full outputs were all 2,545 bytes but their raw hashes varied. Inspection of saved outputs found the emitted `elapsed_ms` field changing, including between repeated current-profile runs. All ten parsed full outputs were equal after excluding that timing field. This inspection used saved output only; the fixed raw-output check was not relaxed and the samples were not rerun.

The failed output check stopped the runner before variant, gene and trial samples. Runtime load ranged from 42.74 down to 37.24 in the one-minute readings. The candidate's compact median was slightly lower, while its full-article median and observed maximum were higher. These observations and the incomplete corpus do not establish no worse executable speed. They do not demonstrate a profile-caused slowdown either.

Historical live-provider observations in `2069-0-9-3-baseline.md` remain context. They include substantial provider/cache latency and cannot isolate executable speed. This experiment did not repeat them.

## Attempts and retained evidence

One owned disposable experiment, `2077-biomcp-release-linking`, used `OWNER` naming `biomcp/2211`. Review preserved its small evidence in `job-runs/biomcp/2211/1792411663763-42983/release-linking-evidence.tar.gz` on the measurement host, beside the finished job logs. The archive keeps the original experiment-relative paths: scripts, frozen helper, settings, preparation logs, raw runtime output, command settings, native timing output, Cargo timing reports, tracked-status checks, binary hashes and cache/load snapshots. Its `attempt-2/builds.json` and `attempt-2/runtime/samples.json` hold the raw structured observations. All six binary sizes and hashes were verified before cleanup. The disposable experiment folder, clones and binaries were removed after review; the evidence archive and job logs remain. These are experiment outputs, not tracked product files.

Preparation had two separate failures, both retained:

- Job `2211/1792410148605-80671` attempted plain `uv sync --locked`, which started building the local package before measurement. The verified owned process group was terminated, exit 143. No measured arm began. Its preparation log and original script remain at the experiment root.
- Job `2211/1792410321987-31873` used `uv sync --locked --no-install-project`, completed all six measured builds, then failed before any runtime sample because the optional development dependency `tiktoken` was absent. The missing dependency was confirmed from its saved runtime log. Its overall exit 1 is not reported as a wholly passing job.

The runtime-only continuation used `uv sync --locked --extra dev --no-install-project`, then `uv run --no-sync python <runtime-script> <experiment-attempt>`. It rebuilt no product. Job `2211/1792411663763-42983` then stopped at the full-article output check, exit 1. The failed preparation was not counted as a runtime sample.

Captured binary SHA-256 identities:

| Pair | Arm | SHA-256 |
| --- | --- | --- |
| 1 | Current | `3187c7132d410a422adec7f356ad720454a582433f52375d4ea26a745736a7c0` |
| 1 | Candidate | `29d4c0260a87b4cbc2ec29cba9acc9cf3ecd172f4dd58478851a534b45a572fe` |
| 2 | Candidate | `8b2fad8a96021bcd83f08d6e698c01796056bf6b1ab3b646ba67a93377eb90bc` |
| 2 | Current | `20eb3279dea053956938e37a910993782f4530dffb35c3668b75d396ddd3969e` |
| 3 | Current | `071a66785ae8f953c9255aac0831420a6ff0cda7637be5050903717838eb8e91` |
| 3 | Candidate | `fef55a65558537cfffee654883bc6e5f8953a919c559b708de0616dc3644abc6` |

Earlier records are reused for their actual scope: 2191 addresses development builds, 2210 evaluates TOML compatibility/build cost, and 2116 records an older contended release build on a different host. None supplies a missing sample in this release-profile comparison. The report changes no product or gate input and reuses the existing exact-product qualification. It makes no new full-suite or release-qualification claim.
