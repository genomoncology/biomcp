# BioMCP 0.9.3: dependencies and dead code

A read-and-measure study of BioMCP at tag `v0.9.2` (commit `4049666f5`). No code was changed. Numbers marked "measured" come from commands run on the build host (4 cores, 14 GB, Rust 1.93.1) in a fresh clone with its own target directory, one build at a time. Numbers marked "inferred" are estimates from those measurements. The earlier iteration-speed study (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`) is not repeated; its coverage figures are reused where they answer a question here.

## Summary

1. **Unneeded dependencies.** None of the 58 direct dependencies is wholly unused: `cargo machete` reports nothing and every crate has at least one production reference. Seven are used in one place each: `md5` (one download cache key), `humantime` (one `parse_duration`), `bincode` (one `serialize`), `toml` (one `from_str`), `http-body-util` (one error type check), `zstd` (one `decode_all`), `htmd` (one `convert`). Two are misplaced: tokio's `test-util` is used only inside `#[cfg(test)]` modules, and `tokio-stream` is used only by the optional AlphaGenome client yet is not optional. The real waste is in default features. `zip 0.6` with default features pulls bzip2, AES, PBKDF2 and a second `zstd`, so the C build of `zstd-sys` (97 s of release CPU, 23 s dev, measured) runs even when AlphaGenome is off.
2. **Large dependencies.** By release compile time (measured): `zstd-sys` 97.7 s, `kuva` 47.6 s (plus 13.8 s for its PNG backend), `unpdf` 39.1 s (plus 14.8 s of rayon and PDF-encryption crates), `ring` 24.2 s, `regex-automata` 23.2 s, `rmcp` 22.7 s, `rustls` 17.9 s, `minijinja` 17.4 s, `reqwest` 17.0 s, `tokio` 14.8 s, `toml_edit` 12.5 s, `tonic` 12.3 s. By binary size (measured): the crate's own code 10.3 MiB, `std` 6.4 MiB, `kuva` 1.6 MiB, then `tokio`, `serde_json`, `rmcp`, `minijinja` at 0.6 to 0.7 MiB each. Kuva renders eleven study chart types, the `chart` documentation command, and the inline SVG charts the MCP server returns; 81 `--chart` mentions in docs and spec pages depend on it. Keep kuva and `charts-png`, `unpdf` and `minijinja`. Trim and upgrade `tonic`; trim `zip`.
3. **Vendoring candidates.** Few and small: `fs2` (std has `File::lock` since Rust 1.89; removes `winapi 0.3` from Windows builds), `humantime` and `bytesize` (about 80 lines together), `md5` (use the `sha2` already present), `getrandom` (one 16-byte nonce; use `uuid`, already in the tree). Together they remove 5 crates and about 1.2 s of release compile (measured per crate). `toml` is the only candidate with real cost (6 crates, 18.7 s release, 9.2 s dev), but it parses a documented user config file, so try `toml 1.x` before writing a parser.
4. **Duplicates and features.** 34 package names appear in two or three versions (measured, `cargo tree -d`). Three choices cause almost all of them: `tonic 0.12`, the old HTML stack of `scraper 0.18` and `readability-rust 0.1` (against `htmd` on html5ever 0.38), and `zip 0.6`. `unpdf` adds the RustCrypto 0.2 pre-release family. Heavy defaults to turn off: `zip` (keep only `deflate`), `tonic` (`router` and `server` are unused; BioMCP is a gRPC client), `scraper` (`main` pulls `getopts`), tokio `test-util` (move to dev-dependencies). Regex Unicode tables cannot be dropped cheaply, and `chrono` cannot leave while `rmcp` and `kuva` need it.
5. **Dead code.** The compiler finds little. A build with every root module private and every `allow(dead_code)` neutralised produced 157 spans with all features on (measured): 27 in the generated AlphaGenome proto, 58 struct fields never read (provider JSON shapes), and 72 functions and types totalling about 500 lines. Of the 130 hand-written spans, 121 are referenced from test code, 9 only from strings or docs, and one five-line item from nothing. `tools/dead-code-exceptions.json` already names nearly all of them. The larger candidates are structural: `src/cli/benchmark` (2,560 lines, compiled only under `#[cfg(test)]`, not a command), the removed `serve-sse` command's remnants, and the test code the prior study measured. The 17,160 production lines the prior study's coverage called unreached are reachable from documented commands; they are untested, not dead. No source integration is unreferenced.
6. **Optimization targets.** The crate itself, not its dependencies. Of the 843 s release wall time, 394 s is the library and 262 s is the link with thin LTO and one codegen unit (measured); dependencies finish by 186 s. `cargo llvm-lines` counts 6.8 million lines of LLVM IR in 138,257 function copies. Generic JSON handling dominates: `serde_json`'s `deserialize_struct` 447 copies (181,284 lines), the crate's `render::json::to_pretty` 86 copies (24,596 lines), `sources::decode_json` 66 copies (13,512 lines), the `settle_batch` closure 12 copies (17,181 lines). Sorting is instantiated for 292 element types, and tokio's task harness exists in 248 copies.

## Measurements

### Counts

| Measure | Value |
|---|---|
| Direct dependencies | 58 normal (3 optional: `tonic`, `prost`, `zstd`), 1 Windows-only, 5 dev, 1 Unix dev |
| Packages in `Cargo.lock` | 488 |
| Crates compiled on Linux, default features | 372 (384 with dev-dependencies) |
| Crates compiled, routine `--no-default-features` | 339 (351 with dev-dependencies) |
| Crates compiled for Windows, default features | 390 |
| Rust source | 763 files, 292,003 lines under `src/`; production by area: entities 79,030, sources 50,669, cli 29,147, render 16,865, transform 9,999, cache 8,380, mcp 4,194 |
| Generated proto | 1,486 lines |

### Release build (default features)

`CARGO_PROFILE_RELEASE_STRIP=false cargo build --release --locked --timings`, measured.

| Measure | Value |
|---|---|
| Wall time | 842.7 s |
| CPU time | 1,604.6 s user, 49.2 s system |
| Peak memory | 4.6 GB |
| Dependencies done by | 186 s |
| `biomcp-cli` library unit | 393.6 s |
| `biomcp` binary unit (thin LTO, one codegen unit) | 262.4 s |
| Binary, stripped | 39,119,080 bytes |
| `.text` section | 27.4 MiB |
| Wrapper binary `biomcp-cli` | 2,148,112 bytes unstripped, 4.2 s to build |

Top dependency units by release compile time, seconds: zstd-sys build script run 97.3, kuva 47.4, unpdf 38.4, regex-automata 23.2, rmcp 22.5, ring build script run 19.3, rustls 17.8, minijinja 17.4, reqwest 17.0, tokio 14.8, zerocopy 12.8, toml_edit 12.5, tonic 12.3, clap_builder 11.5, regex-syntax 9.9, http-cache 8.1, htmd 7.8, axum 0.8 7.8, aho-corasick 7.7, syn 2 7.7, h2 7.7, syn 1 7.6, axum 0.7 7.3, tracing-subscriber 6.3, chrono 5.8, scraper 5.4.

### Dev build (all features, cold)

175.3 s wall, 470.3 s CPU (measured). Dependencies done by 102 s; the crate 73.2 s. Largest dependency units: zstd-sys build script run 22.6 s, rmcp 13.8, kuva 12.8, tokio 9.2, unpdf 8.5, syn 2 8.3, rustls 7.8, ring 7.0, syn 1 6.8, regex-automata 6.7, axum 0.7 6.1, axum 0.8 6.0.

### Binary size by crate

`cargo bloat --release --crates`, measured, share of `.text`: biomcp_cli 10.3 MiB (37.6%), std 6.4 MiB (23.4%), kuva 1.6 MiB (5.7%), tokio 717 KiB, serde_json 711 KiB, rmcp 681 KiB, minijinja 581 KiB, h2 473 KiB, rustls 393 KiB, regex_automata 380 KiB, unpdf 374 KiB, clap_builder 302 KiB, ring 289 KiB, html5ever 273 KiB. The remaining 148 crates hold 1.6 MiB together. The tool calls its attribution approximate.

### Per-dependency exclusive cost

Crates that compile only because of each direct dependency, with their summed release compile time (measured):

| Dependency | Exclusive crates | Release s | Note |
|---|---|---|---|
| kuva 0.5 | 11 | 61.6 | 47.4 s is kuva; the PNG backend is 13.8 s |
| unpdf 0.4.5 | 16 | 53.8 | 38.4 s itself; rayon, crossbeam, rc4, cbc, md-5 and RustCrypto 0.2 14.8 s |
| rmcp 1.7 | 18 | 42.0 | schemars, darling, uuid, rand 0.10 |
| scraper 0.18 + readability-rust 0.1 | 33 shared | 34.5 | the html5ever 0.26 stack, syn 1 among it |
| tonic 0.12 | 17 | 32.5 | router and server extras 17.9 s |
| toml 0.8 | 6 | 18.7 | toml_edit 12.5, winnow 2.9 |
| minijinja 2 | 1 | 17.4 | |
| htmd 0.5.4 | 14 | 17.3 | the html5ever 0.38 stack |
| axum 0.8 | 4 | 12.1 | |
| tracing-subscriber | 7 | 8.6 | |
| zip 0.6 (default features) | 10 release, 12 routine | 4.8 release; 26.8 routine dev | in the routine build zip is zstd-sys's only user |
| tar | 3 | 3.3 | |
| roxmltree, http-cache-reqwest, csv, rust-embed, zstd, reqwest-retry, dirs, semver, humantime, bytesize, md5, fs2 | 1 to 3 each | under 2.3 each | |

Every other direct dependency has no exclusive crates: removing it would remove nothing from the graph.

### LLVM lines

`cargo llvm-lines --release --lib`, measured: 6,819,551 lines, 138,257 copies. The crate's own functions are 39%, `core`, `alloc` and `std` 30%, `tokio` 6%, the serde family about 9%, `rmcp` 4%.

## Unneeded dependencies

Method: `cargo machete --with-metadata` (no findings), then a grep of every `use`, path and macro reference per crate, then a read of each low-count call site.

| Dependency | Production use | Verdict |
|---|---|---|
| md5 0.7 | `utils/download.rs:9`, download cache key | replace with `sha2` |
| getrandom 0.3 | `sources/gencc/store.rs`, a 16-byte temp-name nonce | replace |
| humantime 2 | `cli/cache.rs`, one duration parse | replace |
| bytesize 1 | 4 sites in cache config, cache CLI and health | replace, keeping the printed format byte-identical |
| fs2 0.4 | file locks in 5 files, free space in `cache/limits.rs` | replace with std and the platform crates already present |
| bincode 1.3 | `cache/manager.rs`, cache entry encoding | keep; a format change costs one cold cache |
| toml 0.8 | `cache/config.rs`, four optional keys under `[cache]` | keep a real parser; try toml 1.x |
| zstd 0.13 | `sources/alphagenome.rs`, chunk decoding the server chooses | keep while AlphaGenome is a default feature |
| tokio-stream | 2 sites, both AlphaGenome | make optional under the `alphagenome` feature |
| readability-rust 0.1 | one call in `transform/article/html.rs` | replace with a maintained port (below) |
| scraper 0.18 | 3 files | upgrade |
| kuva, minijinja, rust-embed, unpdf, htmd | charts, 34 templates, embedded assets, PDF text, HTML to Markdown | keep |

tokio `test-util` is used by `tokio::time::advance` only after `#[cfg(test)]` lines, so it belongs in `[dev-dependencies]`.

## Large dependencies

**kuva 0.5.0** (MIT). BioMCP uses its SVG, terminal and PNG backends and the bar, line, scatter, histogram, density, pie, waterfall and survival plots, through 13 functions in `src/render/chart.rs` and the `chart` command in `src/cli/chart.rs`. Users reach it through `biomcp study ... --chart`, `biomcp chart`, and MCP study calls that return inline SVG. Cost: 47.4 s release, 12.8 s dev, 1.6 MiB of binary; the PNG backend adds 13.8 s release and the `ttf-parser` advisory exception. Without kuva users lose every chart; without `charts-png` they lose PNG files from study charts. Keep both. The routine build already turns PNG off, and kuva's other features are already off.

**unpdf 0.4.5** (MIT). One call site; converts article PDFs to Markdown. 38.4 s release, 16 exclusive crates; none is optional in 0.4.5 or in the current release. Users would lose full text from PDF-only providers. Keep. If build time ever outweighs that, use a `pdf` feature that is on in releases and off in the routine build, like `charts-png`.

**minijinja 2** (Apache-2.0). 34 templates, 17.4 s, 1 exclusive crate. Keep.

**rmcp 1.7** (Apache-2.0). The MCP server. 22.5 s, 18 exclusive crates. Keep.

**tonic 0.12 and prost 0.13** (AlphaGenome). 12.3 s plus 17.9 s of crates only `router` and `server` need. Trim the features now and upgrade later. Users lose nothing.

## Vendoring candidates

Vendored code keeps its licence notice; MIT and Apache-2.0 are fine.

| Crate | Used | Lines to write | Removed | Verdict |
|---|---|---|---|---|
| fs2 | file lock calls, `available_space`, `total_space` | about 30 (std locks, `statvfs`, `GetDiskFreeSpaceExW`) | 1 crate on Linux; `winapi 0.3` on Windows | do it |
| humantime | `parse_duration` | about 40 | 1 crate, 0.4 s | do it |
| bytesize | parse and print | about 40 | 1 crate, 0.3 s | do it |
| md5 | `md5::compute` | 0, use `sha2` | 1 crate | do it; download cache entries rename once |
| getrandom | `getrandom::fill` | 0, use `uuid` | one of three `getrandom` versions once scraper 0.18 goes | do it with the HTML change |
| dirs | 11 sites | about 40 | 3 crates, 0.6 s | not worth it |
| toml | `from_str` | about 60 for a subset parser | 6 crates, 18.7 s | keep a real parser; measure toml 1.x |
| reqwest-retry | retry middleware | about 250 | 2 crates, 0.8 s | keep |

Vendoring buys little Linux build time. The reasons to do it are duplicate removal, Windows build time and advisory hygiene.

## Duplicates and features

| Caused by | Duplicated names | Release s in the extra copies |
|---|---|---|
| tonic 0.12 | axum 0.7, axum-core 0.4, matchit 0.7, tower 0.4, indexmap 1, hashbrown 0.12, socket2 0.5 | 17.9 |
| scraper 0.18 and readability-rust 0.1 | html5ever 0.26, markup5ever 0.11, string_cache 0.8, tendril 0.4, phf 0.10 and 0.11, siphasher 0.3, syn 1, unicode-width 0.2, rand 0.8 | 34.5 |
| zip 0.6 | zstd 0.11, zstd-safe 5, cipher 0.4, inout 0.1 | 4.8 release; 26.8 routine dev |
| unpdf 0.4.5 | cipher 0.5, crypto-common 0.2, block-buffer 0.12, inout 0.2 | 14.8 with rayon |
| rmcp 1.7 | rand 0.10, rand_core 0.10, getrandom 0.4 | small |
| the cache stack | base64 0.21, thiserror 1, unicode-width 0.1 | small |

Fixes:

- `zip = { version = "0.6", default-features = false, features = ["deflate"] }`. Both call sites read deflate archives (self-update release zips and Europe PMC supplementary files). If upgrading zip at the same time, choose `deflate-flate2` to stay on the `flate2` already present.
- `tonic` with `default-features = false` and `codegen`, `prost`, `channel`, `tls`, `tls-roots` removes `router` and `server`. Upgrading to tonic and prost 0.14 removes the rest and the `rustls-pemfile` advisory exception, at the cost of regenerating `src/generated/`.
- HTML: `readability-rust` has one release and pins scraper 0.18. `scraper 0.27` with a maintained Readability port on html5ever 0.39 (for example `dom_smoothie`, MIT) removes the whole 0.26 stack (33 crates, 34.5 s release, 24.8 s dev, syn 1 among them) and the `fxhash` advisory exception. Article extraction output may change; the article fixtures are the check.
- `scraper` with `default-features = false` drops `getopts` before the upgrade.
- Not recommended: dropping regex Unicode tables (the patterns use `\d`, `\s`, `\b`, and `tracing-subscriber` keeps `regex-automata` anyway); removing `chrono` (rmcp and kuva need it); replacing the HTTP cache manager before its 1.0.

## Dead code

**Method.** The scratch clone became a binary crate with every root module private and every `allow(dead_code)` neutralised, built with `-W dead_code` and all features (73 s). Items named in test files or after a file's `#[cfg(test)]` line count as test-only. Line extents were measured by brace matching.

| Class | Spans | Lines | What they are |
|---|---|---|---|
| Generated proto | 27 | | response types for AlphaGenome calls BioMCP does not make; ignore |
| Fields never read | 58 | about 120 | provider JSON shapes kept for fixture fidelity |
| Referenced only by tests | 63 | about 450 | request-plan structs and builders in pubtator, mydisease, semantic_scholar, europepmc, litsense2, mutalyzer, variantvalidator; `cli::outcome::execute`; `gene::search` and builder methods; `variant::search`; several `*_search_markdown` renderers; `article::query::build_pubmed_esearch_params` (41 lines, the largest) |
| Named only in strings or docs | 9 | about 75 | |
| No reference anywhere | 1 | 5 | a tuple field in `src/sources/reactome.rs:386` |

These go when their construction tests go, which the speed study's replay-server step makes possible.

| Structural candidate | Lines | Evidence |
|---|---|---|
| `src/cli/benchmark/` | 2,560 | `#[cfg(test)] mod benchmark;` in `src/cli/mod.rs`; no command variant; compiles into the test binary only. Delete, or move to a script |
| `serve-sse` remnants | about 30 | hidden command that only prints a deprecation message; two match arms and four Python tests |
| `biomcp-cli` wrapper binary | 61 lines, a second 2 MB executable, 4.2 s per release build | execs its sibling `biomcp`; a packaging decision |
| `src/entities/source_state_registry.rs` | 946, under `#![allow(dead_code)]` | three production readers; only `aggregation` and `labels()` are dead |

**Reachable but never exercised** (prior study's coverage). 17,160 production lines run under no test. The largest are in `entities/gene.rs`, `variant/gwas.rs`, `adverse_event.rs`, `cli/system/dispatch.rs`, `disease/associations.rs` and `article/assets.rs`. Whole surfaces no outside-in test touches: `search all`, `update` and `uninstall`, `cli/health/local.rs`, `entities/variant/structure.rs`, and nine source clients (`cbioportal_download`, `disgenet`, `pharmgkb`, `request_plan_transport`, `gprofiler`, `archive_budget`, `oncokb`, `litsense2`, `interpro`). All are reachable from documented commands. Each needs an outside-in test or a decision to drop the surface.

**Source integrations.** All 79 modules under `src/sources` are referenced. Nothing is an orphaned client.

## Optimization targets

The crate is 78% of release wall time and 42% of cold dev wall time (measured). The link's 262 s comes from `codegen-units = 1` with thin LTO; the trade against runtime speed was not measured.

| Target | Lines | Copies | Remedy |
|---|---|---|---|
| `serde_json` `deserialize_struct` | 181,284 | 447 | shrinks only when provider shapes shrink |
| `render::json::to_pretty<T>` | 24,596 | 86 | serialize to `serde_json::Value` at call sites; one non-generic printer |
| `settle_batch` closures | 23,685 | 36 | box the per-item future or project to a common type |
| `sources::decode_json<T>` | 13,512 | 66 | move status and error handling to a non-generic function |
| `cli::outcome::run` | 10,087 | 1 | one very large async state machine on every command's path |
| `study::dispatch::handle_command` | 7,914 | 1 | same shape |
| stable sort internals | about 147,000 | 292 element types | sort keys or indices of a few common types |
| tokio task harness | about 67,000 | 248 future types | box futures at fan-out spawn sites |

The largest user-visible runtime cost remains the HTTP cache's start-up clean on every command (speed study); nothing in the dependency graph compares to it.

## A recommended order

1. **Trim `zip` to deflate.** Payoff: 10 fewer crates in release and 12 in the routine build, including the `zstd-sys` C build (26.8 s of CPU per cold routine build). Proof: `cargo tree -e normal,build --no-default-features | grep -c zstd-sys` prints 0; a fixture test extracts a deflate zip through both call sites.
2. **Move tokio `test-util` to dev-dependencies and make `tokio-stream` optional.** Payoff: the shipped binary drops the test clock. Proof: `cargo tree -e features -i tokio` shows no `test-util` for a release build.
3. **Trim `tonic` and `scraper` features.** Payoff: about 13 s release (inferred) and five duplicates. Proof: `cargo tree -d` loses axum 0.7, axum-core, matchit, getopts, unicode-width 0.2.
4. **Replace `fs2`, `md5`, `getrandom`, `humantime`, `bytesize`.** Payoff: 5 direct dependencies, 5 crates, `winapi 0.3` on Windows. Proof: `cargo tree --target x86_64-pc-windows-msvc -i winapi` is empty; `biomcp cache` and health output stay byte-identical.
5. **Delete dead items.** Delete `src/cli/benchmark` and the `serve-sse` remnants now; delete the 63 test-only items and 58 fields as the speed study retires their construction tests. Proof: the dead-code scan reports zero hand-written spans; `tools/dead-code-exceptions.json` shrinks toward empty.
6. **Upgrade `tonic` and `prost` to 0.14** and regenerate the proto. Proof: one `tower` and one `indexmap` in `cargo tree -d`; `cargo deny check` passes without the `rustls-pemfile` exception.
7. **Replace the html5ever 0.26 stack.** Payoff: 33 crates and about 35 s release CPU, syn 1 and the `fxhash` exception gone. Proof: `cargo tree -i syn@1` is empty; article extraction fixtures pass.
8. **Try `toml 1.x`.** Payoff: up to 18.7 s release (inferred). Proof: the timings report.
9. **Reduce monomorphization** in the order of the table above. Proof: `cargo llvm-lines` totals and the library unit time fall.
10. **Measure `codegen-units` for the release link.** Measure build time and runtime speed before deciding.

Steps 1 to 8 remove about 60 crates and roughly 110 s of release CPU and 50 s of dev CPU from a cold build (inferred from per-crate sums). Wall-time gains are smaller because dependencies compile four wide and are cached between edits. The per-edit cost stays with the crate itself, which the speed study's plan addresses.

## What I could not measure

- Function-level `cargo bloat` for the main binary.
- Build time after each change. Each payoff is the measured compile time of the crates that would leave, an upper bound for wall time.
- `cargo udeps`, which needs a nightly toolchain; `cargo machete` and the grep agree there are no unused dependencies.
- Compile time of the replacements (`scraper 0.27`, a Readability port, `toml 1.x`, `tonic 0.14`) and of `winapi` on Windows.
- Fresh coverage; the speed study's figures were reused.
