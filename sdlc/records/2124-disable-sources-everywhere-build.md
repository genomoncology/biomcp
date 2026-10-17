# 2124 build record — every source off in one setting

Branch `ticket/2124-disable-sources-everywhere`, tip `f50cf9949` (2026-10-16). One gate in
`disable::ensure_not_disabled` refuses an HTTP client for a disabled registry id, and every
shared client constructor in `src/sources/mod.rs` plus the six clients that build their own
now carry their registry id through it, so no source request can reach the network with its
source disabled. The serve commands pass the parsed `--disable-sources` flag into
`disable::init`, and `disable::init` now keeps the server's resolution when the in-process
CLI worker behind each tool call re-enters without a flag, so every tool call enforces the
flag the operator passed. A disabled source answers as an unavailable section naming the
setting that turned it off.

## Proofs

Each proof fails on current main first and passes on the branch.

- **The walk** (`tests/test_disable_sources_replay_walk.py`) covers every id in
  `docs/reference/sources.json` (80 of 80; the file's own assertion fails if a new id joins
  without an entry). Each routed entry runs its command against the golden suite's replay
  server with the source disabled and asserts the provider's wire prefix stays silent, while
  an enabled baseline of the same command proves the command uses the source. Sixty-three
  entries are routed; seventeen are `carried` — fields another provider passes along, a
  local dataset with no routable download URL, or a chain the recorded replies stop short of
  — and each names why, asserts its exit parity, and, where the source owns an exclusive
  wire prefix, that the prefix stays silent. Three recorded replies already under
  `testdata/sources` (UniProt P15056, dbSNP rs113488022, MyDisease MONDO:0007947) became
  replay routes to anchor the protein, population, and disease chains. Red on main: dozens
  of `disabled <id> still made requests` failures (civic, mygene-info, string, uniprot,
  clingen, disgenet, gtex, hpa, dgidb, opentargets, gprofiler, reactome, wikipathways,
  mydisease, monarch, seer, hpo, ols4, and more). Green on the branch in about six seconds.
- **The serve proofs** (`tests/disable_sources_cli.rs`, `serve_flag_disables_civic_on_the_stdio_server`
  and `serve_http_flag_matches_the_env_variable_form`) spawn `biomcp --disable-sources civic
  serve` and `serve-http` on the replay server, assert the initialize instructions list
  CIViC off with the flag's reason, call the typed `get` tool for a variant's CIViC section,
  and assert no `/civic` request reached the server; the serve-http test pins the
  environment-variable form to the same description. Red on main: `the disabled CIViC source
  still reached the network: ["POST /civic/graphql#d5ba12178da005e7"]` and the missing
  disabled-sources sentence. Green on the branch. The contract harness grew
  `spawn_stdio_client_with_args` and `spawn_http_server_with_args` for leading global flags
  and runtime-built environment pairs.

## Gates (macOS build host via pm job; the landing bar stays ci.yml green at the exact post-rebase tip)

- `cargo fmt --check`: pass at `e8559f5f9` (job 1792184722826, after the fmt commit).
- Rust disable-sources tests: 13/13 pass at `e8559f5f9` (job 1792184722826) and again at
  `f50cf9949`; the walk passes at every tip from `57cd1ae12` on (job 1792184418492).
- Full Rust suite on the macOS build host: the branch's failures are a subset of the failures pure main
  produces in the same job worktree (main: 20, branch: 12, every branch failure also fails
  main except one load-flaky test that passes in isolation; jobs 1792185263926 and
  1792185400313). The macOS build host runs without the offline sandbox, with sccache forcing
  `RUSTC_WRAPPER=` and `CARGO_INCREMENTAL` unset, under load; Linux CI is authoritative.
- Python lane on the macOS build host: identical failure set on pure main and the branch (20 tests,
  all packaging/release-script/tooling tests; jobs 1792185681198 and 1792185777143);
  `mkdocs build --strict` passes.
- `make lint`: blocked on the macOS build host (no pinned ShellCheck/actionlint archive for darwin-arm64);
  clippy under the macOS build host's rustc 1.95.0 fails on pure main as well, so the pinned CI toolchain
  is the bar.
- `make spec`: prepare-spec and spec-static (7/7) pass; `scripts/run-specs.sh` needs bash 4
  FD redirection and the macOS build host has bash 3.2, so the mustmatch pages run in Linux CI.
- Red runs: branch `tickets/2124-red-proof` (main plus the probes) at `4a2c0562e`, jobs
  1792184285179 and 1792183481950.

## Findings

- `get protein P15056 structures` panics on main and the branch alike once a UniProt reply
  is available: `section outcome credits a provider outside the source-state registry:
  protein/structures`. Pre-existing, exposed by richer replay data, not caused by this
  ticket; needs its own ticket.
- `gencc sync` fails before any download when its per-run dataset directory is empty
  (Store open/lock refuses), so the walk carries gencc with exit parity; the gene card's
  GenCC section reads the local store either way.
- The GTR sync's replay-visible request is its version probe; the condition-gene download
  follows only when the version differs. The cbioPortal data-hub endpoint shares the
  cbioportal id and stays unqueried by the variant card.
- The macOS build host cannot run `make lint`, the offline-enforced lanes, or `run-specs.sh` (bash 3.2);
  ticket 2137 covers bringing the gates to the macOS host. The remote pm on the macOS build host was
  updated to protocol 4 mid-run to unblock dispatch.
