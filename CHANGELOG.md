# Changelog

## 0.9.3

### New features

- `variant articles` with a gene plus protein change or an rsID now also asks NCBI LitVar2 and merges the papers it links to that variant into the union result with `litvar` source attribution, so variant papers PubTator and Europe PMC miss appear (`CDKN2A A57V` gains the 39-paper rs372266620 set, 32 of which the text routes missed): only an autocomplete entry whose gene and change, or rsID, matches the request exactly is kept, the merge honors the search's deadline and per-source limits, a LitVar2 failure degrades that source only, and `--disable-sources litvar2` plus a `biomcp health` row cover the new source. (2131)
- `variant articles --strategy exact NAME` searches one exact variant name with no resolution and no alias expansion: the caller's string goes to PubMed and Europe PMC as phrase-quoted title/abstract queries, to Semantic Scholar's bulk phrase endpoint, and to PubTator as search text, and the result reports which sources it reached; `--debug-plan` also gains a per-alias, per-source yield table for every alias-driven route (strict, exact-lexical, best-effort, and exact-name) listing hits received, hits kept after merging, and whether each alias call's 25-result cap cut the source's list. (2144)
- `--disable-sources` and `BIOMCP_DISABLE_SOURCES` turn any source, or any recorded group, off in one setting: the list takes comma- or pipe-separated ids and group names, an unknown name fails with the valid names, `commercial-restricted` covers every source whose recorded terms restrict commercial use, a disabled source is never called and its carried fields drop from output, `biomcp health` and the MCP server description name what is off, and `BIOMCP_DISABLE_KEGG` keeps working as an alias. (2061)
- GWAS Catalog queries, openFDA label searches, and openFDA adverse-event counts are now held for a day: a repeat within twenty-four hours is answered from BioMCP's cache without a network request, an older stored answer goes back to the provider, `--no-cache`, `BIOMCP_CACHE_MODE=off`, and an openFDA key still bypass the cache, and a stored GWAS body that no longer decodes is replaced by one fresh fetch. (2091)

### Fixes

- Suppress unresolved structural hotspot counts, preserve cached-answer age notes, keep drug and disease identity refusals precise, hide health error paths, validate protein residues, and handle installer paths with spaces. (2128)
- `variant articles --strategy annotation` searches PubTator by the variant entity id autocomplete returns when the request spells a gene and a protein change: `variant articles "ABCC8 R1436Q" --strategy annotation` keeps the `@VARIANT_p.R1436Q_ABCC8_human` entity — its display name carries only the change `p.R1436Q`, which is why the entity was dropped and the debug plan showed bare queries such as `R1436Q` — and returns the seven papers PubTator annotates with that variant, where the run returned no results with `complete: true`; a run whose autocomplete reply holds no variant entity now says so in its source status with `complete: false` and exit 0 instead of a silently complete empty page, and the debug plan lists the entity-id queries the route ran. (2138)
- `batch article --sections` reports every section it was asked for: article batches refuse `assets` and `asset <asset-key>` before any request, naming the supported sections (annotations, fulltext, indexing, tldr, all) and pointing asset lists at `get article`, an unrequested `tldr` section no longer appears in the JSON section outcomes and provenance rows when only other sections were asked for, and each requested section keeps its data, empty or unavailable outcome — the same fix covers `get article` section requests through the shared detail path. (2147)
- A PMC bot challenge in place of the article page is named instead of called unusable content: when PMC answers the HTML step with its "Checking your browser - reCAPTCHA" interstitial, the full-text attempt records `reason: "bot_challenge"` (coverage `unusable`, outcome `unavailable`) so users and agents know a retry may work, the resolver keeps running its next source, and the failure fold stays as it was. (2141)
- A variant article search keeps a paper matched only by a genomic coordinate alias when the paper is about the requested gene and drops it otherwise: MyVariant's genomic id and confirmed ClinGen allele-registry genomic aliases now travel with the gene symbol in every alias the exact-lexical and strict routes send, so PubMed's loose reading of a bare `chr20:g.35533834A>G` ("chr20" AND "G") no longer pulls unrelated chromosome-20 papers into a `SAMHD1 I448T` search (the cotton paper 28794480 dropped) or `chr9:g.21971188G>A` into a `CDKN2A` search (the citrus paper 41894487 dropped), while rsIDs, transcript and protein aliases keep matching alone because they name the variant uniquely. (2140)
- `get article <id> fulltext --pdf` returns the PDF's text when Semantic Scholar lists an open PDF: the open-PDF link was refused before any download because the outbound policy allowlisted only Semantic Scholar's own two origins, while the recorded links point at publisher hosts such as `diabetesjournals.org` (PMID 10615958 reproduced `source_unavailable` with no text on main), so the policy now treats the listed link as provider-vouched at any HTTPS default-port DNS-name origin while keeping every transport guard — no credentials, no IP-literal hosts, DNS answers that cannot resolve to private, loopback, link-local, CGNAT or metadata addresses, and redirects revalidated by the same rules — and each PDF-step failure now names the failed part in its attempt reason (`open_pdf_link_unavailable`, `open_pdf_link_absent`, `open_pdf_link_rejected`, `open_pdf_download_failed`, `open_pdf_download_absent`, `open_pdf_text_unusable`) instead of a bare `source_unavailable` or `no_content`. (2142)
- An article search by rsID keeps the PubTator variant entity whose `db_id` carries that rsID and searches by it, so `search article` for `rs113488022` returns PubTator's annotated papers as well as the text matches, where the entity was dropped because neither its id nor its display name spelled the rsID; the entity's own label (`p.M600E`) stays out of the match when the rsID matched. An empty variant-articles search where every source answered is now an ok result with exit 0 — ambiguous resolution, indeterminate provider validation, and unfinished identity or equivalence checks keep `complete: false` with no error — and `source_unavailable` with exit 1 stays for a zero-row result where a provider reported unavailable or timed out, with the error naming the failing sources instead of claiming every route was down. (2130)
- `search article` follows the same incomplete-result rule as `variant articles`: JSON carries `complete`, a zero-row search while a source was unavailable or timed out exits 1 with `complete: false` and a structured `source_unavailable` error naming the down sources instead of a silent empty page, an empty search where every source answered keeps exit 0 with `complete: true`, and rows that arrived beside a down source keep exit 0 with `complete: false`. (2145)
- An NCBI API key no longer lifts PubTator3 past its published limit: BioMCP keeps PubTator at three requests a second (334 ms between requests) whether or not `NCBI_API_KEY` is set, because PubTator3's own documentation asks every caller for no more than three requests a second and names no keyed rate, the keyed ten-a-second budget stays with E-utilities, and the API-keys guide, configuration reference, and data-source registry no longer list PubTator among the paths the key improves. (2149)
- An empty supplement reply is a failed download instead of a zero-byte file: `get article <id> asset <key> --output FILE` exits nonzero naming the new `empty_response` coverage outcome and writes no output file, Europe PMC's zero-length supplementary members (observed on PMC6181777's NIHMS1501643-supplement-2.xlsx, which Europe PMC stores empty while the PMC OA package holds the full 2.3 MB file) are marked in `coverage` instead of listed in `assets`, the same rule covers PMC OA, Figshare, and linked-file routes that return no bytes, and when one filename has several routes the route that returned bytes keeps its asset key while the empty route keeps the key it would have carried so an older key still reaches the refusal. (2146)
- A supplement whose only copy is empty or of the wrong type is looked up in the other official copies: when Europe PMC's archive holds a supplement without usable bytes (observed on PMC4858276's pone.0153788.s001.pdf and PMC6168352's NIHMS1501645-supplement-2.docx), BioMCP fetches the PMC open-data bucket object the package manifest names and tries the PMC article page's file route, so a package download that failed on one oversized object no longer hides the files the bucket still serves. Bytes that do not match the file name's type are refused with the new `unusable_asset` outcome the way an empty reply is (observed on PMC7787686's pgen.1009231.s004.docx, where both archive routes return a vendor HTML page): `asset` exits nonzero naming the outcome, writes no file, and the wrong-type route keeps the asset key it would have carried, so a caller never takes a vendor page for the supplement. When every copy is empty, wrong-type, or missing, the manifest marks the file in `coverage` with that reason. (2150)
- A command's start-up no longer walks the whole HTTP cache: the tree is secured once on a fresh, migrated, or first-seen cache and recorded in a marker, later files are secured as they are written, the size check reads a kept byte total that the day's clean and every eviction rewrite instead of statting every content file, and `BIOMCP_CACHE_MODE=off` and `--no-cache` skip every cache start-up step, so a command with a hundred-thousand-entry cache starts as fast as with an empty one. (2133)
- A disabled source is now refused at the one place every source client builds its HTTP client, so `--disable-sources` and `BIOMCP_DISABLE_SOURCES` stop every source's requests instead of only the sources and carried-field sets that checked the setting themselves, and `biomcp serve` and `biomcp serve-http` accept the flag so a server honors the same list a one-shot command does; a disabled source answers as an unavailable section naming the setting that turned it off. (2124)
- `variant structure` verifies its residue against the chosen protein's own sequence before claiming a placement: `variant structure 'MUTYH G368D'` no longer reports position 368 as an exact match on Q9UIF7, whose sequence holds S at 368 and G at 393; the placement is reported as unresolved with a warning naming the finding, no residue-specific domain or PDB coverage claim, and the protein-level rows kept. `variant structure 'BRAF V600E'` on P15056, whose sequence holds V at 600, is unchanged. (2114)
- `get disease MM` lists multiple myeloma as its first refusal candidate, ahead of the source's Miyoshi muscular dystrophy holder, and `search trial -c MM --source nci` refuses with a `biomcp search trial -c "multiple myeloma" --source nci` retry instead of a `get disease` command that leaves a trial user no next step; `article entities 22663011` no longer links the "skin neoplasms" row to the skin benign neoplasm card, because a crosswalk hit whose card title narrows the row's concept (every concept word plus at least one more) counts as no holder and the row keeps its search command. (2066)
- `get drug` brand lists now name only products whose openFDA label names the card's drug as its own product: another drug's record that only matched the label search never lists its brand (trastuzumab no longer lists Enhertu, which is trastuzumab deruxtecan), DrugBank synonyms such as chemical names stop listing as brands, and a brand the user typed that the discover rescue resolved stays listed (Tarceva lists on the erlotinib hydrochloride card). (2068)
- MCP `health` replies no longer name local filesystem paths: local dataset rows and the cache dir keep their bare names in Markdown and drop the `local_path` field in JSON, while the CLI keeps its paths, and MCP error text now keeps every line break the CLI prints, so a multi-line refusal such as an ambiguous-name retry list arrives as separate lines instead of one merged sentence. (2065)
- Every documented install path works as the docs say: a pinned installer run reads the true version from the staged binary and records it in the receipt instead of the build-time field, the PyPI package keeps its `biomcp-cli` name and installs only the `biomcp` command — uvx users run `uvx --from biomcp-cli biomcp serve` — the install docs state the Python 3.10 floor, that Homebrew covers macOS only, and how to pull or pin the Docker image, the Homebrew formula the release renders carries a `test do` block, and the release publishes the changelog section as the GitHub release notes. (2064)
- `get drug` now refuses a product name several drug products share: "Sleep Aid" and "Cold and Flu" exit 2 as ambiguous drug names, list ingredients from labels that own the exact name, and name the ingredient form to try. When no label owns the exact name, as in the recorded "Pain Relief" reply, the refusal suggests a drug search without listing unrelated products. (2085)
- `search all` and `search gwas` with both a gene and a trait wait for one GWAS Catalog query instead of two: the gene query and the trait query are sent at the same time, so `search all --gene BRAF --disease melanoma` finishes its GWAS section inside the 12-second section limit with the same rows. (2089)
- `get drug` now fetches the openFDA label, the CIViC context, the Open Targets data and the openFDA adverse-event preview at the same time once MyChem has named the drug, so the card waits for its slowest source instead of the sum of all four and returns byte-identical output. (2090)
- `get trial NCT99999999` reports the trial as not found with its search suggestion instead of telling the caller to retry the remote source, `get article 999999999999` treats every all-digit identifier as a PMID and reports no article with that PMID instead of rejecting the identifier format, and `get variant "BRAF Z999Q"` exits 2 naming the residue letter Z that is not an amino acid instead of suggesting a search. (2087)
- Every follow-up command a variant card prints uses the headline coordinate: a card whose population section resolved a GRCh37 rsID to a GRCh38 coordinate through dbSNP no longer points its CIViC pointer or its trials and articles commands at the resolved coordinate, so following the printed civic command returns the evidence the card counted, and each direct ClinVar row now prints `N submission(s)` from its submission count instead of an empty submitter count. (2067)
- The stale-cache tests pass on a nearly full filesystem: the fixtures behind the article-search, search-all and ClinGen gene-card stale serves pin the smallest legal `BIOMCP_CACHE_MIN_DISK_FREE` value beside their cache directory, and every test built on the shared fixture guard gets the same pin, so disk-pressure eviction cannot remove the cached reply before the stale serve. Reported by Seungwon Lee, GitHub #291. (2100)
- Single-source relevance searches on Europe PMC and PubTator3 rank each complete 25-row provider page before slicing a requested window. Requests within the same provider page retain a consistent order; windows crossing a provider-page boundary do not promise the same ordering as a single larger request. (2084)
- An explicit `--assembly` on `get variant` constrains the lookup to that build: `--assembly hg38 'chr7:g.140453136A>T'` reports not found when GRCh38 holds no record instead of answering MyVariant's GRCh37 record, a flag that contradicts a coordinate prefix refuses naming both builds, and `BIOMCP_DEFAULT_ASSEMBLY` keeps ordering the dual-build search for a bare coordinate without ever constraining it. (2112)

### Internal

- Remove duplicate repository review-status tests after the shared landing tool enforced accepted reviews itself. (2063)
- Prepare the routine temporary directory before stress tests and contract checks, so their fixtures use the same prepared checkout paths as the routine suite. (2096, 2148)
- Keep article-validation and disease-survival fixture tests reliable under concurrent load with bounded watchdogs and readiness checks. (2102, 2105)
- Qualify release revisions directly and rehearse complete untagged candidates without publication credentials or external publication. (2161)
- Full main checks build and exercise PNG, SVG and terminal output with the development profile; release qualification and the final rehearsal retain optimized builds and output checks. (2171)
- Keep retry fixtures on real time during loopback requests; preserve the HPO four-attempt policy and compact-command retry cancellation checks. (2167)
- Preserve changelog entries for batches already landed when unfinished work moves to a later release. (2166)
- The credential-redaction contract uses permanent local provider failures, so unrelated randomized retry delays cannot exhaust its command watchdog. (2165)
- Article help is checked through the command line. Redundant help and test-file-layout assertions are removed; parser, label, ranking and refusal edge cases remain. (2081)
- Run retained whole-repository audits in required postmerge and release qualification, while routine Python tests keep focused malformed-input and wrapper checks. (2122)
- Run the local Rust and Python test lanes concurrently after one shared preparation, with bounded workers and separate failure output (2094).
- Source package checks compare archive membership with Cargo's declared package list instead of maintaining a parallel file count, so legitimate source additions preserve the runtime boundary without a counter edit. (2159)
- Measurement suites use a private temporary directory inside the checkout, so inherited temporary-directory aliases cannot invalidate store-security fixtures. (2168)
- Isolate MCP contract servers from inherited credentials, configuration and caches (2160).
- Release work uses reviewed local qualification, explicit ticket prerequisites, comparable measurements and a final untagged publication rehearsal. (2162)
- Landings run the routine lint, tests and specifications through the configured build job gate; main adds the complete shipped-feature check before the next landing, and routine Clippy now checks test targets. (2156)
- The no-entity annotation regression returns an empty reply for both PubTator and LitVar2 autocomplete through one shared fixture handler, so an empty LitVar2 match stays a healthy zero-row answer and the test no longer mistakes a missing fixture route for an outage. (2154)
- The frozen ERepo specification removes its final output byte with Python instead of a GNU-only `head` option, so its exact Markdown assertions also run on macOS. (2157)
- CI and documentation publication run only when started manually; branch pushes, main pushes and pull requests start neither workflow. Manual CI runs exercise every check on the selected revision, and manual documentation publication keeps its production-main revision guard. Release tag triggers and publication safeguards are unchanged. (2155)
- The 0.9 gates run through pm job on the gate hosts: lint, spec, release-gate and builds run on the macOS host, where the lint bootstrap installs the pinned ShellCheck and actionlint on Darwin-arm64, the spec runner and fixture scripts read process data through ps, fcntl and lsof where /proc and the GNU-only tools are absent, and the runner's cleanup and replay greeting work on the bundled Bash 3.2, while the offline routine gate keeps its verdict on the Linux host under the bubblewrap sandbox and `tools/run-offline` runs the command without the sandbox on macOS as a fallback, printing one stderr line saying the run is not network-isolated; `tools/with-build-identity` reads the package version through cargo metadata instead of Python 3.11's tomllib, and the baseline measure script carries no host locks and runs wherever pm jobs schedule it. (2137)
- The routine spec lane runs five of the pages 2080 dropped again: `skills`, `trial-numeric-filters` and `trial-retirement` run offline against the replay server, with the retirement page checking a recorded ClinicalTrials.gov reply and the numeric-filter page counting its zero requests in the shared replay log, and `clingen-erepo` and `variant-articles-corpus` replay their own frozen captures through their runner scripts; the dropped pages' 35-line disposition table lives on the ticket. (2127)
- The repository records a build and test efficiency audit of the 0.9.3 work: gate-host and CI machine time over two weeks, the costliest tests and CI steps, shared output built more than once, the build flag sets, the coverage report against the shared rules, and the development benchmarks, with each finding's measured cost and expected saving. (2116)
- The baseline measure script never prints a coverage percentage from a failed run: each instrumented suite's exit status and test counts print beside its wall time, a report set fed by a failed or unparseable suite prints `RESULT coverage_<set>_lines_pct = incomplete (<suite>: <n> failed)` with no number, the block states its denominator once (product code with test files and generated code removed; inline test modules stay counted), and the scripted reports merge their profile data again now that the report step also unsets the eval'd `CARGO_LLVM_COV_TARGET_DIR` export, which pinned the search to the plain target directory where no profile data lives. (2117)
- Clippy replaces the size, line-cap and wait ratchet scripts: the two audits and their three inventories, the inventory updater, the wait-ratchet script and their tests are gone, `too-many-lines-threshold` in `clippy.toml` caps every function at the repository's longest function, `disallowed-methods` refuses a new clock sleep in production code unless its call carries an allow with a reason, and ruff's banned-api refuses a new `time.sleep` in tests while the existing test sleeps stay listed in `per-file-ignores` until 2081 removes them. (2076)
- The release checks now run on every branch push instead of only at tag time: a branch that changes shipped behavior fails CI until its changelog bullet lands, a branch that changes only records and tests needs no bullet, the branch lane runs the release's green-CI `gh` query against the merge-base so a flag the real `gh` rejects fails a branch run instead of a release, and lint fails on a ticket number cited in a Rust or Python comment. (2074)
- A ticket-branch push that changes only `sdlc/` records over a tip whose own CI run finished green now skips the Rust gates and finishes in minutes instead of running every gate for half an hour; a failed, missing, or in-progress previous run, a force push, any path outside `sdlc/`, and every other branch still run the full gates, and an `AGENTS.md`-only push runs them too because the contract pages read that file. (2086)
- Under `sdlc/`, only record and planning text still counts as docs-only: tickets, records, decisions, designs, ideas, issues, planning and `sdlc/plan.md` keep skipping the heavy jobs, while a push that changes a program, gate wrapper or configuration under `sdlc/` — the `sdlc/scripts` gate wrappers, the `sdlc/project` lifecycle programs, `sdlc/pm.json`, `sdlc/skills`, `sdlc/release-checklist.md` — runs the full CI jobs again, and the green-tip record route skips only the text folders too, because the repository-contracts job that reads the record text runs on every push. (2115)
- One dev build now serves every routine gate: the empty build script that made cargo rescan the package and recompile the crate after any edit is gone, the Python and spec lanes run the dev test build's binary instead of building a second spec-profile binary, and the documented targeted-test loop sets `CARGO_INCREMENTAL=1` for the edit loop. (2070)
- `cargo deny check advisories` passes again: zip moves from 0.6.6 to 9.0.4, past the archive-extract symlink advisory that flagged every push, the manifest keeps the deflate codec alone through the plain flate2 backend, the self-update and Europe PMC readers keep their behavior, and no advisory ignore is added. (2143)
- `cargo deny check advisories` passes again: rustls moved from 0.23.45 to 0.23.46, past the fresh advisories that turned every lane's lint and CI red, with the rest of the lockfile unchanged. (2093)
- The repository now carries one baseline record and its measuring script for the 0.9.3 work: binary and build-folder size, build times, dependency counts, gate times, test counts, coverage, CI cost, and command run times, each printed with the command behind it and measured twice on the build host against tag v0.9.2, so the 0.9.3 changes have before-and-after numbers from one source. (2069)
- The dev-cycle changelog test checks the section's shape instead of a hand-kept ticket list, and every bullet's cited ticket must be filed for the milestone, so a branch's changelog change stays its own one bullet. (2083)
- The HTTP cache's start-up clean now runs at most once a day per cache root, recorded in a marker under the HTTP cache, and `BIOMCP_CACHE_MODE=off` skips it entirely, so a command no longer scans a large cache index on every start-up. (2092)
- CI builds once and runs the lanes in parallel: a `build` job runs `make prepare-test` and uploads the dev build's nextest archive and binaries as one artifact, the lint, Rust-test, Python and spec lanes download it and each run one Makefile target through a shared composite action, `make test` splits into `test-rust` and `test-python`, and the all-features, Windows, stress and release-panic jobs run only on pushes to `main`, where every release tag still needs their green run. (2075)
- Dev and test builds pin line-tables-only debug info for the crate and none for dependencies, and a `debugging` profile gives full debug info when a debugger needs it; measured on the gate host, builds are byte-identical to before on cargo 1.93.1, which already defaults to this layout. (2101)
- No Python test in the routine lane compiles BioMCP or reads the developer's own HTTP cache: every test gets its own cache root through one conftest fixture, the packaged-crate cold compile moved behind the `release_gate` marker that only `make release-gate` runs, and the short-Unix-socket contract keeps its source pin instead of rerunning a Rust test through cargo. (2071)
- `CHANGELOG.md` merges with git's ordinary rules again, undoing 2083's union driver: a union merge kept both sides of a bullet one branch had reworded, so the 2085 bullet landed twice, one copy misspelled, with no conflict to see, and a landing that appends a bullet to a busy section now resolves a small conflict that names the two new bullets. (2104)
- The routine spec lane runs on one replay server over the recorded replies: `BIOMCP_TEST_RETRY_SLEEP_OFF` turns the client's retry back-off sleeps off and `BIOMCP_TEST_FEDERATED_ARTICLE_SOURCE_TIMEOUT_MS` narrows the federated article-source leg budget beside the deadline's existing seam, the runner starts `tools/replay-server` once instead of ten per-page fixture scripts and no longer locks, reaps, or namespaces request logs, the fixture lifecycle and runner-recovery tests are gone with that machinery, the JSON and request-log checks live in golden cases under `tests/golden/spec`, and the provider-backed pages leave the routine list until their recordings are authored. (2080)
- The first golden output suite lands with the replay server that feeds it: every BioMCP provider base override points at `tools/replay-server`, which serves recorded replies from `testdata/replay`, fails loudly on any unrecorded request, records a live reply once in `--record` mode, and derives its routes from `src/sources`, while `tests/golden` runs `get gene`, `get variant`, and `search all` (counts-only and full) in JSON and Markdown against committed golden files that `--update` rewrites. (2079)
- Deadline-proof and surface tests bound their waits with a shared load-scaled watchdog and private cache roots, so routine suites stay green while other jobs share the build host. (2103)
- Article HTML parsing and readability now run on scraper 0.27 and dom_smoothie 0.18, which drops html5ever 0.26 with its syn 1 build, the direct `getrandom` dependency, and the fxhash advisory exception, and the article fixtures and full-text spec pages keep passing. (2078)
- Routine builds compile fewer C libraries and the shipped binary drops small crates it never ran: the self-update zip reader keeps only deflate, so no bzip2, AES or second zstd C library builds, tokio's test clock is a dev dependency the shipped binary no longer carries, the AlphaGenome client compiles no gRPC server, and std file locks replace the `fs2` crate, so Windows builds pull no `winapi` 0.3. (2072)
- `get variant "BRAF V600E" civic` now queries CIViC with the molecular profile name it knows: the three-letter protein change the resolved variant record carries (`p.Val600Glu`) converts to the one-letter spelling with the existing normalizer before the profile name is built, so `BRAF V600E` and `EGFR C797S` report the accepted evidence CIViC holds (115 evidence items and 3 assertions, and 2 items, on the recorded replies) where the card printed an empty section, and a complex alteration the normalizer cannot spell keeps its own name. (2113)
- A release tag now counts only a finished push run of CI on main as its green run, so a ticket-branch run of the same commit can no longer release a tag; a stale branch's landing is charged only with the files its own branch changed; the packaged-crate cold compile runs from the release workflow before any archive is built; the dev-cycle changelog refuses a repeated bullet and a blank line inside a list; and the manifest states the minimum Rust the code needs. (2125)
- The article-query contract test that spent its whole 60 s deadline against an instant loopback reply now pins a fresh per-test cache root like its sibling tests: its spawned server had inherited the ambient user cache and the start-up secure walk consumed the deadline before any provider was asked. The test passes in a fraction of a second under its product deadline, alone, beside a full suite, and on the Linux gate host with a 206 MB ambient cache present. (2126)
- A build from main between releases reports the coming version as a dev build instead of the last released number: main carries Cargo `0.9.3-dev` and Python `0.9.3.dev0` while 0.9.3 work lands and a build prints `0.9.3-dev+g<sha>`, the version-sync check accepts the dev shape beside the release shape with registry, citation, and Homebrew metadata staying on the published release, the release workflow refuses a tag whose version carries `-dev` and refuses a stable tag while any committed version still does, and after each release main moves to the next patch version with the `-dev` suffix through a ticket branch while the release commit drops it. (2132)
- The unused internal benchmark harness (`src/cli/benchmark/`, its structure test and its ownership decision record) and the hidden `serve-sse` compatibility command are gone: `biomcp serve-sse` now exits 2 as an unrecognized subcommand instead of printing a removal message, and the release-verify spec lane no longer wires the deleted harness ratchet. (2073)

## 0.9.2 — 2026-10-11

### Fixes
- `get variant` now states the current ClinVar classification in its headline when the `clinvar` section answers from NCBI ClinVar, and the quick default view names its cached copy and prints the command that returns the current classification. (1290)
- When NCBI ClinVar drops, the degraded ClinVar section now names NCBI ClinVar and the reason it failed (timeout, rate limit, or provider error) and dates the MyVariant.info fallback copy by its newest evaluation. (1291)
- `get variant` now resolves transcript deletion ranges with intronic offsets and bare ClinVar VariationIDs, with the ClinVar alias fallback answering only on an exact transcript match. (1292)
- Article search now finishes within its 60-second deadline and returns the rows that already answered, with each silent or slow source named as degraded. (1293)
- `get article` and batch detail now return whole abstracts instead of a capped form while search rows keep a named short excerpt of the abstract, and full text renders reference identifiers, clean PMC HTML, and ragged tables instead of dropping them. (1294)
- `search disease` now finds the parent disease for common abbreviations such as NSCLC and DLBCL by matching the exact synonym fields MyDisease holds. (1295)
- A protein-change query with several compatible hits refuses when no single hit names the requested change, naming every candidate and the working input forms. (1297)
- Europe PMC searches now return distinct rows for every requested page by following the provider's own page markers, where the ignored page parameter collapsed every search to its first page. (1298)
- A search that hits its deadline mid-pagination now keeps the rows it already fetched and names the silent source as degraded, exits at the deadline instead of outliving it, names the deadline in its error instead of a generic I/O failure, and honors the deadline while building its cache even when another caller is reading the same cache. (1299)
- `get drug NAME label` now returns whole label sections in JSON while the safety and interaction sections keep their own caps and Markdown keeps a capped short form pointing to the full label, says why a label is missing (no SPL record matched, or the matched record carries no section text) instead of a silent null, and resolves sparse-metadata labels such as osimertinib through a guarded full-text search that never returns another drug's label. (1300)
- Citation evidence now degrades to OpenCitations when Semantic Scholar refuses with a rate limit or a server error, and a live 429 that took twenty-two seconds to fail answered in 2.7 seconds in one live measurement. (1302)
- `get author` and `author papers` now read ORCID's lowercase visibility values, so an `orcid:` lookup no longer fails and papers no longer silently return zero works. GitHub #289. (1303)
- `who sync` now accepts the header set the WHO exports ship today, reports the true outcome per file, and exits nonzero when required files are missing. GitHub #288. (1304)
- The degraded citation path now reuses the first seed's resolved record when the second seed refuses, reports a malformed citation identifier as such instead of claiming the index was unavailable, and names the DOI-resolution search in its status row instead of reporting it not requested. (1306)
- A protein-change query now returns a variant only when the transcript the card names in its headline spells the requested change, with ClinVar presence breaking ties just among those named matches; a change that matches only on another transcript's numbering now carries a note naming that transcript and its own spelling, and a request whose reference residue the gene's standard protein sequence does not carry refuses with the honest reason instead of returning a different variant. (2016)
- `get disease` now refuses an abbreviation that several human diseases hold, naming every candidate with its label; one- and two-letter abbreviations refuse even when a single disease holds them, naming the holder; and a resolved card no longer attaches another disease's genes to its definition. (2017)
- Variant rows in `article entities` no longer share one rsID command when the article names two alleles of that rsID; each row prints the gene-plus-change command that opens its own variant. (2018)
- A failed WHO Prequalification export now degrades every caller honestly: drug search names the reason on the card, in JSON and through MCP, `who sync` names the failing file and its missing column, and a partial sync reports each file's outcome with a nonzero exit instead of claiming it synchronized. (2021)
- Article search now refuses a deep offset window on every sort order, names the deadline as the error whenever any source hit it, and full-text rendering joins small-caps and styled text such as "T ABLE" and "PD L1" without splitting the words. (2023)
- Every variant row in `article entities` that carries a gene symbol and a protein change now prints the gene-plus-change command, including when the article mentions only one allele of an rsID that names several. (2029)
- `get drug NAME` now returns the named drug: a record counts when the query matches its name, one of its brand names, a listed synonym, or the leading words of a longer name for the same drug such as a salt form; brand queries such as TAGRISSO resolve through the record the brand lives on; a name several unrelated drugs share, such as "Pain Relief", refuses and names what matched instead of returning one drug's card carrying another's identifiers, while a name owned by exactly one drug resolves to that drug's own card, a synonym belonging to another drug never answers for it, and a card never takes its name from an unrelated product's packaging row. (2031, 2039)
- An ambiguous disease refusal now includes the oncology reading a clinician likely means, so `get disease MM` lists multiple myeloma; trial search refuses the same ambiguous condition with its choices on the NCI source only, and the default ClinicalTrials.gov source still runs its keyword search. (2032)
- Variant rows in `article entities` keep the gene-plus-change command even when the article spells the gene as K-ras or K-RAS, resolving the official gene symbol from the row's gene identifier when no symbol-shaped mention exists. (2034)
- A gene-plus-protein-change request now resolves on the standard MANE Select transcript with its current version: it never returns a variant whose change matches only on a shorter or longer protein isoform without a note, it refuses and names the standard transcript's own spelling when only an isoform alias spells the request, and it resolves the one candidate the standard transcript names when exactly one exists. (2036)
- `get drug` keeps a brand's own drug: a brand that lives in MyChem only on a nameless record and holds no current openFDA label, such as Tarceva, Lartruvo, Portrazza or Lumoxiti, resolves through the guarded discover rescue before any refusal, and a brand card takes the name its own product row carries, so Zejula names niraparib rather than the Akeega combination and Lonsurf says trifluridine and tipiracil again. (2037)
- A missing WHO export now leaves the local data-folder path in the terminal only, so JSON replies and MCP callers carry the reason without it; the working command a routed or refused variant search suggests keeps every flag you passed; and variant and article requests now also read MyGene and UniProt, which the sources list and the licensing page name. (2038)
- A gene-plus-change request now checks the change's numbering against the standard protein even when that protein cannot be reached, when no standard transcript is known, or when the record carries more annotation rows than the reader used to keep: it answers with the standard transcript's own change, a refusal, or a note that names the other numbering or says the numbering could not be checked. (2042)
- MCP tool errors now carry the public form of a failure, so a missing WHO dataset no longer names the server's local data folder, and the hint a refused variant search prints keeps the `--hgvsp` and `--consequence` flags you passed. (2044)
- `get drug` now takes the label that matches what was asked: a brand query takes its own brand's label (Herceptin no longer shows the OGIVRI biosimilar's, Keytruda the plain KEYTRUDA record rather than the under-the-skin QLEX pairing), an ingredient query takes the plain single-ingredient product's label and card name (niraparib names niraparib with the Zejula label, never the Akeega combination; trastuzumab, pembrolizumab, amivantamab, bevacizumab and rituximab follow the same rule), Darzalex resolves to daratumumab, and a refusal that consults the discover rescue runs it once instead of twice. (2043)
- `article entities` disease rows print `get disease <ontology-id>` only when the disease crosswalk holds exactly one record naming the identifier's PubTator3 concept name; every other disease row keeps the v0.9.1 `search disease` command, so a broad MeSH descriptor such as `MESH:D008175` for Lung Cancers no longer opens lung benign neoplasm's card. (2047)
- `get variant <rsid> gwas` and `all` degrade the GWAS section with the retirement note when the retired legacy GWAS Catalog REST endpoint answers 410 upstream, instead of failing the whole variant answer on the dead source, and a 404 from that endpoint now means the variant holds no GWAS associations. (2047, 2050)
- A gene+protein query never answers an isoform spelling silently: when the canonical protein facts prove the request follows another transcript's numbering, the note prints even with no MANE marker; a record that offers no headlineable protein change refuses with wording that claims only what the record headlined; the numbering refusals name the isoform that spells the request instead of a bare candidate list, and their line breaks survive text mode; and the UniProt record read now serves from BioMCP's HTTP cache. (2047)
- Trial search on the default ClinicalTrials.gov source now tells you when a condition is an ambiguous abbreviation: the note names the diseases that hold the token and the clinical reading the trial corpus carries most, so `search trial -c MF` explains its mixed results instead of returning them silently; the `get disease` refusals for `MF` and `MM` name the myelofibrosis and multiple myeloma readings with their sources; and an NCI-source condition longer than 512 bytes truncates with a visible note instead of failing the search. (2040)
- A drug's product rows count against a request only when the record owns the name and the label's ingredient fits the card, the sparse-record reader keeps multi-word brands, a refusal's discover rescue stops at two seconds, and short abbreviations behave the same in any letter case; the tests share one scratch-cache helper so they run at the same speed on any machine. (2050)
- `get drug` now takes the drug and the label the ingredient names: the label lookup follows the resolved ingredient, a brand whose labels name one ingredient resolves to that ingredient, a plain ingredient such as acetaminophen gets its own card and its own label, a card name never carries a product row's strength, and a combination card shows no single-ingredient identifier as the whole product's; the variant card no longer prints a stray blank line after its transcript when no note follows. (2052)
- A release tag now builds only from a commit whose CI run finished green, and the 0.9.2 changelog states only what actually shipped. (2053)

### New features

- Article annotation rows now carry namespace identifiers (NCBIGene for genes, MESH and OMIM for diseases and chemicals, rsID then HGVS for variants), and markdown rows print a `get` command only where the real parsers accept the identifier form. (1296)
- The variant headline pairs its ClinVar classification with that classification's own evaluation date; `search variant` routes a free-text phrase whose first token is an official gene symbol to that gene filter with the remaining words as the condition, accepts the first token only when it spells the official symbol, moves a leading protein change after the gene to its own filter, and prints how a zero-row phrase was read beside a working alternative; and `BIOMCP_VARIANT_QUERY_GENE_ROUTING=off` restores the whole-phrase condition search. (1301, 2022)
- Release binaries now print the commit they were built from, every release job runs on a pinned runner image so the pin, not GitHub's moving labels, decides the host, and `make spec` prepares the Python environment the executable contracts need. (2024)

### Docs

- The guides match the shipped behavior: the variant guide explains gene-symbol-first routing, the article guide states the 60-second search deadline, the deep-offset refusal and the OpenCitations fallback, the drug guide names the sections that keep their caps, and the ClinVar source page and the variant-structure blog post show the current headline format. (1290, 1293, 1300, 1301, 1302)

### Internal
- The release's green-CI check now counts successful runs with flags the shipped gh CLI actually supports, after a rehearsal release failed on a gh flag the runner does not have. (2060)
- Ticket 2055's landing carries its release note, which the first gated rehearsal caught missing. (2057)
- The release notes close with this entry, so every ticket in the 0.9.2 tag carries its bullet. (2058)
- The repository's working rules sit in one section of its agent guide — targeted tests while working, the full gates once before landing — and the retired March tooling is gone from the tree. (2055)
- The release guards stop the rehearsal repository from publishing: the wheel upload and the Homebrew push run only in the production repository, the container image and its source label follow the repository running the workflow, and the changelog gate reads a bullet's closing ticket group wherever punctuation trails it. (2049)
- The tickets from the final review rounds carry their verdicts on the ticket tool's own review line. (2051)
- A held test reply that nothing releases now ends on its own at a bound above every real test wait, so a run that loses its caller fails at a named test instead of sitting forever. (2045)
- The release workflow's smoke legs now compare the printed version with the tag exactly, the workflow lint refuses duplicate option keys and variables a container script never receives, and the changelog gate covers every landing shape and stays inside the tag's own release line. (2046)
- Records written by the ticket tool now pass the repository's review contract unchanged, and every landed ticket closes through the tool that recorded it. (2048)
- The `who sync` report moved into its own module, bringing its command file back under the repository's line limit and retiring the entry that had allowed the overage. (2010)
- The release check that every fixed ticket has a changelog bullet now counts every way a change can land — merge commits, `Land NNNN:` subjects, ticket files marked complete, and build records — so a ticket cannot ship undescribed, and a push that changes only the changelog now runs the changelog test instead of skipping as documentation. (2019)
- The repository tree no longer ships a stray packaging file: the package count returned to its true value, the forbidden-name check reads a local untracked file, and the article specification's deadline blocks run again. (2020)
- Article test fixtures hold replies on async primitives instead of blocking runtime workers, so a held reply cannot deadlock the deadline tests; a hung test fails at its own named 120-second timeout, and a hang that blocks even the timers is killed at the process level once the test runner has marked it slow twice on its 120-second marks, about 240 seconds in. (2030)
- The third review round before this release corrected what the tickets and records claim: overstated CI evidence, specification blocks that a merge had left unable to run, and stale review lines now state what actually happened. (2033)
- The fourth review round before this release closed or answered its findings, from the brand-name drug rescue to protein-change refusals that name the standard transcript's own spelling of the requested change; the findings it left open moved to ticket 2038. (2035)
- Maintenance: repository housekeeping after 0.9.1, the Semantic Scholar rate-limit diagnosis behind the citation-evidence fix, and release planning that shipped no behavior change (1287, 1288, 1289, 1305).

## 0.9.1 — 2026-10-01

### Fixes

- The typed MCP `search` and `get` tool schemas now declare a top-level `"type": "object"` and are published flat for client authors, and malformed tool arguments return a structured error naming the field instead of a silent success. GitHub #284. (1223, 1240)
- A panicking tool call no longer takes the MCP server down: the server isolates the failure, answers your client with an error, and keeps serving the rest of the session. (1230, 1235)
- Drug label output now shows boxed warnings and legacy warning text that was previously dropped, so the strongest FDA warnings are visible on every drug card. (1232, 1237)
- Provider failures now render honestly on cards: a failing source shows its failure instead of an empty section that reads as "no data". (1242)
- Windows: `biomcp serve` no longer lets `icacls.exe` write into the stdio MCP stream, which broke strict clients between JSON-RPC frames on localized consoles. GitHub #283. (1246)
- DDInter coverage is carried through to drug cards, and interaction evidence rows are no longer dropped. (1241)
- Simple queries no longer crash the executable: `trial search`, drug trials, drug interactions and `drug adverse-events <unknown>` aborted on the 0.9.0 wheel because it carried a debug-profile binary; the wheel now builds with the locked release profile and every one of those paths runs. GitHub #282. (1243, 1225)
- An unusable `SSL_CERT_FILE` now degrades to a clear warning at startup instead of aborting the run, and a CA bundle supplied through `BIOMCP_CA_BUNDLE` is parsed once at startup and trusted for every outbound connection. GitHub #250. (1221, 1231, 1236)
- Stale cached data now says so: search results carry the cache age and the true source in `_meta.notes`, GWAS never serves a stale body, and certificate-trust failures fail immediately instead of retrying. (1256, 1263, 1268)
- A missing skill or empty catalog exits with a clear diagnostic instead of an empty success, and debug builds embed the same skill and chart assets as release builds. GitHub #287. (1267)

### New features

- A new `cell-line` entity searches and resolves cell lines: `biomcp search cell-line <name>` normalizes spellings against Cellosaurus (exact identifiers and human lines rank first), and `biomcp get cell-line <CVCL_xxxx>` prints one card with synonyms, species, disease, cross-references and variants, accepting DepMap, Cell Model Passports, ChEMBL and PharmacoDB identifiers too. (1202)
- Cell-line research data: `biomcp gene cell-lines <symbol> --group` reports RNA levels (nTPM) for the gene across one cancer group of cell lines from the Human Protein Atlas, drug response and cell-line drug panels come from PharmacoDB, and `get cell-line <id> chembl` lists a line's ChEMBL records with their assay counts — each naming its source and release. (1205, 1213, 1214)
- Linux wheels target `manylinux_2_28` (built and verified in the official containers) and a Linux ARM64 wheel ships, so `pip install biomcp-cli` works on current glibc systems and ARM servers. (1245)
- The Linux tarballs build in the same containers as the wheels and pass the same glibc floor scan, so the documented 2.28 floor holds for both artifact families. (1249)
- The release publishes a multi-architecture container image (`linux/amd64` and `linux/arm64`), smokes both platforms from the registry, and moves the `latest` tag only after the smokes pass. (1219, 1229)
- The release gates now test real properties of the artifacts — versions, changelog contents, wheel behavior, live documentation revision — so a mismatched artifact cannot publish, and a failed tag run can be deleted and retried. (1226, 1227, 1233, 1234)
- The Homebrew formula updates only after the release is public and PyPI has the artifacts, so its tarball URLs are installable the moment it lands. (1222, 1266)

### Docs

- The 48 biomedical sources' licensing evidence lives in the repository, Enrichr is labelled restricted with its terms stated plainly, and the README carries a data-terms warning linking the per-source breakdown. (1276)
- The documentation is current with the shipped binary: new commands, renamed flags, and the configuration reference match 0.9.1. (1224)

### Internal

- The cache mode no longer leaks between tests: a test that flips it restores it, closing the cross-test pollution behind flaky suites. GitHub #286. (1261)
- CI runs on every ticket branch and is the merge gate: a branch merges only after its own run finishes green, and the docs-only skip is decided by the files changed since the merge-base with main; the commit message decides nothing. A push that skips the Rust jobs still runs the record tests. (1220, 1250, 1252, 1275, 1278)
- Flaky tests were fixed at their mechanisms: waits that polled the clock now wait on signals, the load-sensitive lanes are pinned to a deterministic CPU set, the known-flaky set runs in its own stress lane, and the release-panic, TLS and Windows contracts run the real behavior they name. (1239, 1247, 1248, 1252, 1257)
- The check suite that guards the repository (review-status grammar, workflow provenance, wait ratchet, licensing tiers) was rebuilt as structural checks that fail when the behavior they guard is removed. (1251, 1255, 1258, 1264, 1269, 1279)
- Maintenance: records, docs, and test hygiene across the release cycle (1224, 1228, 1238, 1244, 1254, 1259, 1265, 1270, 1279, 1280, 1281, 1282, 1283, 1284, 1285, 1286, 1287).

## 0.9.0 — 2026-09-16

### Breaking changes

- Trial searches now refuse ambiguous `--status active` values for both NCI and
  ClinicalTrials.gov. Use `--status recruiting` for open and accruing trials or
  `--status active_not_recruiting` for enrolled trials that are no longer accruing.
- Replaced the legacy MyVariant/ExAC variant-detail population fields with a
  direct, GRCh38-only gnomAD v4 population result. Exomes and genomes now keep
  raw numeric allele frequencies and counts, ancestry rows, grpmax FAF95, and
  quality filters separate. This breaking response change requires the next
  public release to use a new minor version. Removed fields: `gnomad_af`,
  `allele_frequency_raw`, `allele_frequency_percent`, and
  `population_breakdown`; replacement: `population` with dataset `gnomad_r4`.

### New features

- Fixed ClinicalTrials.gov trial search so hyphenated terms (anti-PD-1, CAR-T,
  PD-L1, dMMR/MSI-H combinations) match what the registry actually holds: the
  ESSIE query builder no longer backslash-escapes hyphens, restoring the
  expected result counts for `--criteria`, `--mutation`, `--biomarker`,
  `--sponsor`, `--study-type`, `--prior-therapies`, `--progression-on`, and
  quoted intervention literals. (1198)
- Added honest zero reporting for trial searches: when ClinicalTrials.gov
  matched trials on the eligibility text but registry eligibility verification
  removed all of them, the output names the upstream count and suggests a
  `--mutation` relaxation, both in the Markdown hint and in the optional JSON
  `_meta.upstream_total` member. (1198)
- Added a persistent sidecar cache for `article citation-evidence`: repeated
  queries for the same directed edge are served from a local thirty-day record
  without re-fetching, keyed on resolved Semantic Scholar paper-ID pairs so
  every hit is paired with a live provider resolution. `biomcp cache clear`
  removes the sidecar. (1199)
- Added a sixth `article citation-evidence` outcome,
  `reference_confirmed_without_passage`: when the passage is not openly
  available but OpenCitations confirms the directed edge, the command returns
  a validated citation record so callers can distinguish "the passage is not
  open" from "this citation may not exist." (1200)
- Added `discover <query> --search`: when no concepts resolve, the suggested
  article keyword search runs inline and returns its results alongside the
  concept metadata in one response, with a new `## Article search` Markdown
  section and `article_search` JSON member. (1201)
- Added `article citation-evidence <citing-id> <cited-id>` returning bounded source
  text for one directed citation pair: Semantic Scholar context by default, and an
  exact open-access Europe PMC JATS reference resolution when that edge carries no
  context or `--fulltext` is passed. The result retrieves evidence only and never
  interprets the citation. (1145)
- Added read-only ClinGen Allele Registry (`clingen_car`) normalization for supported
  versioned RefSeq HGVS values, including bounded CLI JSON batches and the typed
  `variant_normalize_car` MCP tool.

- Added a private bounded provider-capture store that retains verified public
  response bytes behind content-addressed handles for future source integrations;
  it does not yet add a source command or provider route.
- Added `variant erepo` and typed `variant_erepo` MCP retrieval of versioned ClinGen
  Evidence Repository assertions, preserving source-listed criterion coverage and
  selected SEPIO provenance without ACMG/AMP interpretation.
- Added `gene cspec` and typed `gene_cspec` retrieval of versioned ClinGen Criteria
  Specification source documents, with exact local raw-capture streaming and no
  ACMG interpretation or classification.

- Resolved recognized JATS- and PMC-HTML-linked article supplements through stable
  asset keys, bounded provider fetches, merged discovery provenance, and typed
  per-file coverage without exposing provider URLs.
- Changed `variant articles` to union exact annotation, normalized-alias, and
  source-citation routes with per-row provenance, deterministic global ranking,
  one-pass pagination, diagnostic strategies, and explicit incomplete status.
- Added additive `canonical_equivalence` CAR CAid observations for independently
  supplied versioned RefSeq transcript/coding and genomic/build identities. These
  facts preserve MyVariant resolution, do not perform liftover, and retain
  received-response hashes for audit.
- Added bounded optional ClinGen LDH article identity observations that bind
  applicable CAids, genes, known PMCIDs, and exact text/table selectors without
  changing article discovery, ranking, pagination, or missing-coverage semantics.
- Added bounded `variant articles --input <path|->` retrieval for 1-10 structured
  variants, ordered compact rows, opt-in route/work plans, parseable article
  follow-ups, and a typed in-memory MCP `variant_articles` tool.
- Made top-level article-search JSON compact by default, with `--full` restoring
  detailed rows and in-band warnings when date sorting replaces relevance.
- Added typed PMID/PMCID/DOI/arXiv/Semantic Scholar identifiers to article
  search and related-paper tables.
- Added Semantic Scholar-only provider-exact `search author` and `get author`,
  including JSON/Markdown provenance, honest unresolved-ORCID warnings,
  list/help discovery, and typed MCP schema support.
- Added opt-in PubMed article indexing with associated author affiliations,
  ORCID, structured MeSH descriptors/qualifiers, explicit availability, and
  inclusion under `get article <id> all`.

### Fixes

- Emitted the `servers` key VS Code actually reads from `biomcp mcp-config
  --client vscode`, instead of the `mcpServers` key the other stdio clients
  use. VS Code parsed the old snippet without complaint and then ignored it,
  so the server never appeared and nothing said why. Public MCP client docs
  now show the same corrected shape.
- Bumped the `h2` lockfile entry to 0.4.16 for RUSTSEC-2026-0258, which
  accepted and queued empty DATA frames without limit. `h2` is a transitive
  dependency of hyper and tonic, so no source change was needed.
- Hardened CLI correctness before the next development candidate: local study
  discovery now treats a missing data directory as an empty catalog and drops
  invalid survival times; phenotype search has truthful 50-result paging and a
  ten-term cap; impossible variant filters and inapplicable batch sources fail
  before provider contact.
- Bounded chart dimensions and PNG scale before study loading, restored canonical
  chart names in JSON, made skill-install JSON report real no-op/repair state,
  rejected unusable HTTP port zero, and documented the typed JSON form of
  `cache path`.
- Labeled the compact MCP measurements from ticket 0932 as historical and made
  current documentation point to the reproducible offline measurement and its
  enforced 16,000-byte / 4,000-token budgets.
- Made global CLI modifiers truthful: cache commands now reject `--no-cache`
  before managed-state access, while JSON help and version flags return
  successful typed output instead of `invalid_argument` errors.
- Advanced the private package identity to Rust `0.9.0-dev.6` and Python
  `0.9.0.dev6` after pinning the documentation toolchain and silencing the
  Material banner, binding Markdown assertions to their owning row and block,
  preserving both PMC3040717 stored-fixture proof-of-work routes, declaring the
  supported test lane, and fixing the indel ID round trip. Public citation,
  directory, updater, and release metadata remain at the latest published
  release, v0.8.25.
- Advanced the private package identity to Rust `0.9.0-dev.5` and Python
  `0.9.0.dev5` after parsed-command MCP authorization, pre-bind HTTP host-policy
  validation, and complete HTTP build-identity reporting. Public citation,
  directory, updater, and release metadata remain at the latest published
  release, v0.8.25.
- Advanced the private package identity to Rust `0.9.0-dev.4` and Python
  `0.9.0.dev4`. Public citation, directory, updater, and release metadata remain
  at the latest published release, v0.8.25.
- Advanced the private package identity to Rust `0.9.0-dev.3` and Python
  `0.9.0.dev3`. Public citation, directory, updater, and release metadata remain
  at the latest published release, v0.8.25.
- Advanced the private package identity to Rust `0.9.0-dev.2` and Python
  `0.9.0.dev2`. Public citation, directory, updater, and release metadata remain
  at the latest published release, v0.8.25.

- Preserved exact GWAS Catalog mantissa/exponent p-values in structured JSON,
  Markdown, filtering, and ordering instead of displaying underflowed values as
  zero. Mixed-result batches now keep their complete report on stdout while
  returning a nonzero status, and gene-alias discovery uses a bounded typed
  identity lookup so `ERBB1` resolves to canonical EGFR.
- Made downloaded full-text artifacts private on creation and during managed
  state repair. Article-asset discovery now bounds optional providers, reports
  distinct provider outcomes, and reuses fresh manifests for five minutes so
  paging does not immediately repeat a slow or unavailable provider ladder.
- Added region-specific Markdown continuation commands to combined drug search,
  removed the unsupported GWAS region example, enforced allowed Host values on
  every HTTP route, made canonical lint bootstrap its pinned shell tools, and
  added a read-only pre-commit hook currency check.

- Bare genomic coordinates now prefer GRCh38 while retaining GRCh37 collision
  evidence. `--assembly` overrides `BIOMCP_DEFAULT_ASSEMBLY`, which overrides
  the GRCh38 default; typed MCP variant get exposes the same assembly choice.

- Repaired MyVariant consequence, ClinVar review-status, and field-presence search filters, including typed rejection of unsupported values and removal of the consequence-dropping fallback.
- Standardized optional source-state inventory and provenance reporting, including
  disease survival `data`/`empty`/`unavailable` outcomes and in-band Markdown
  unavailability without failed-provider credit.
- Added an `inapplicable` lookup state so variant prerequisite checks no longer
  credit uncontacted providers, and exposed honest treatment/trial and variant
  structure helper outcomes without turning optional failures into command errors.
- Sanitized terminal-active controls and invisible bidi markers at human CLI,
  MCP, Markdown, diagnostic, and chart-scalar boundaries while preserving typed
  JSON values and trusted chart-backend ANSI styling.
- Restored MyVariant BayesDel predictions as separate `BayesDel add-AF` and
  `BayesDel no-AF` source scores, and repaired `--gerp-min` filtering without
  adding clinical thresholds or pathogenicity classification.
- Restored figure JATS context and complex-table omission metadata from normal
  DTD-bearing PMC OA archives by sharing the bounded article XML parser policy.
- Kept command-owned JSON collection paths present as empty arrays on empty
  success and parsed structured errors, while preserving error objects, nonzero
  exits, nested drug-region shapes, and legacy bare-array batch successes.
- Sanitized standard structured JSON errors so provider bodies, request URLs,
  credentials, parser details, and internal local paths are not exposed.
- Recovered article assets through bounded Europe PMC supplementary ZIPs when a
  PMC OA archive fails, while distinguishing confirmed absence from source
  unavailability and preserving retained PMC OA license provenance.
- Restored live PubMed article indexing for normal DTD-bearing citation XML with
  finite body/node bounds, and added sanitized failure codes/messages while
  preserving the base article when indexing degrades.

### Docs

- Added the `--search` flag to the discover command guide and the article
  search fallback examples.
- Updated the article user guide with the sixth citation-evidence outcome
  state and the OpenCitations confirmation record.
- Refreshed the trial search guide to document hyphenated term behavior and
  the verification-emptied zero-result hint.
- Updated the release-process reference for the 0.9.0 stable version.

### Internal

- Aligned all serial-test keys for provider and cache environment mutation,
  eliminating random parallel test failures from env-var collisions.
- Fixed the URL policy to prefer an explicitly selected loopback origin over
  the unpaced-origin environment signal, restoring trial document retrieval
  under combined fixtures.
- Added three load-sensitivity eliminations: GenCC store deadline bounds only
  lock acquisition, lifecycle runner death waits tolerate lane load, and the
  execute-thread stack margin is pinned at a real boundary.
- Bounded the docs-verifier probe window and the FDA orphan cache disk floor
  for deterministic test behavior across hosts.

## 0.8.25 — 2026-07-07

### New features

- Added Homebrew formula and tap release automation so macOS/Linuxbrew users can
  install BioMCP from the project tap after release publication. (478)
- Added a `biomcp mcp-config` generator and unified MCP-client setup docs so
  users can copy local stdio config snippets for supported clients. (477)
- Added Docker image publication with CI build and smoke coverage so releases can
  publish `ghcr.io/genomoncology/biomcp` images. (476)
- Published official MCP Registry metadata for `biomcp-cli`, including the PyPI
  package distinction from the unrelated `biomcp` package. (473)
- Annotated MCP tools with read-only hints and human-readable titles for safer
  client presentation. (474)
- Added opt-in variant structure annotation for residue, domain, PDB, AlphaFold,
  and hotspot context without ddG prediction claims. (452)

### Fixes

- Emitted JSON usage errors for clap parse failures under `--json`, keeping CLI
  error output machine-readable on parse errors. (466)
- Fixed `variant normalize` JSON output so failures no longer look like silent
  successes.
- Normalized protein and phenotype JSON search next commands. (460)
- Added broadening hints for zero-result filtered trial searches and clarified
  `--mutation` versus `--biomarker` usage.
- Surfaced the CIViC actionability pointer and currency caveat on default variant
  cards without adding a new network call.
- Accepted advertised `drug adverse-events --count` and clarified FAERS Summary
  output as sample rows rather than an aggregate.
- Bounded CLI exit latency after disease-survival and CTGov helper output, and
  sped up CTGov alias fan-out tests. (467, 470)
- Stabilized MCP/spec routing with dynamic reserved ports and live-lane routing
  fixes. (468)

### Docs

- Added the unified "Add BioMCP to your MCP client" guide and refreshed MCP
  deployment docs while keeping leaked internal spec text out of public docs.
- Documented environment configuration parity with runtime behavior and added
  ratchet coverage. (465)
- Added shell-safe HGVS command examples and separated typed next-command
  ownership from markdown shell quoting. (469)
- Added the variant-structure blog walkthrough from linear sequence to 3D protein
  context. (453)
- Added the skill-authoring guide and normalize-to-codes worked example. (450)
- Refreshed release-prep metadata and public release notes for v0.8.25. (479)

### Internal

- Added whole-surface CLI contract ratchets for help, list, docs, specs, and JSON
  metadata. (459)
- Completed helper and parent help discovery for helpers and skills. (463)
- Restored deterministic routine disease-survival bounded-exit coverage with a
  SEER fixture. (472)
- Moved contract and release smokes to tag/release time, removed stale daily
  release cron behavior, and repaired stale WikiPathways probe coverage.
- Removed leaked internal working docs, retired validation profiles, and public
  root clutter from the repo.

## 0.8.24 — 2026-06-23

### New features

- Added Claude Code plugin marketplace metadata so users can install the BioMCP
  plugin through `/plugin`. (239)
- Changed `serve-http --allowed-hosts` to be opt-in, leaving the Host guard
  open by default for normal proxy and deployment setups while preserving
  explicit host restrictions when configured. (240)
- Surfaced upstream source attribution through MCP responses so agent callers can
  see section-level provenance without separately requesting JSON. (434)
- Added typed MCP `search` and `get` tools with schema enums for entities and
  sections, keeping the raw `biomcp` tool as the escape hatch. (435)
- Made CLI rejection errors more actionable for over-limit requests, multi-word
  `get drug` / `get disease` inputs, and pathway identifiers that should be
  routed to protein, gene, or variant commands. (436)
- Accepted transcript HGVS inputs such as `NM_004333.6:c.1799T>A` through
  `get variant` by reusing the existing normalization seam. (440)
- Generalized `get pathway` redirect hints for Ensembl IDs, gene symbols, and
  dbSNP rsIDs, in addition to UniProt accessions. (443)

### Fixes

- Resolved the RUSTSEC-2026-0186 `memmap2` advisory in the release dependency
  set.
- Removed the Python `mcp` test-client dependency by migrating protocol contract
  coverage to the Rust rmcp-client harness, clearing all seven remaining
  Dependabot alerts tied to the removed Python dependency tree. (433)
- Quieted cache disk-space WARN output after successful cleanup so normal
  commands do not emit misleading warnings on stderr. (437)
- Bounded CTGov trial-helper post-output latency for `--limit 1` gene and
  disease trial searches. (438)
- Preferred canonical HGNC genes for common aliases such as PD-L1, HER2, and P53
  instead of lower-value NCIT/MESH matches. (439)
- Made `-j` / `--json` error paths emit JSON error objects on stdout for command
  errors, including unknown genes and invalid variants. (441)

### Docs

- Refreshed release-prep metadata and public release notes for v0.8.24.
  (432)

### Internal

- Completed a light parity/coherence review of the Rust rmcp-client contract
  migration before tagging v0.8.24, with no release-blocking findings. (432,
  433)
- Restored the parallel-isolation canary to `SPEC_ROUTINE_PATHS`, keeping
  `make verify` green after the rmcp contract migration. (442)
- Updated `build.rs` so the baked version string re-runs on Git HEAD/ref changes
  and release binaries report the current commit metadata. (444)

### Back compatibility

- Changed `search article --source all` to stop querying LitSense2 by default;
  use `--source litsense2` explicitly when you want LitSense2.
- Tightened drug CLI validation: `search drug --region who --product-type vaccine`
  now rejects structured filters, and `drug trials --no-alias-expand` now
  rejects non-CTGov trial sources.

## 0.8.23 — 2026-06-11

### New features

- Added source-first article full-text and asset retrieval improvements,
  including provenance and quality flags, BioC/JATS markdown fixture coverage,
  figure/supplement/table handling, canonical open-access package asset
  manifests, and Figshare supplementary-file retrieval for non-PMC articles.
  (383, 384, 385, 386, 398, 408, 409)
- Added Cancerhotspots recurrence enrichment for somatic variants, surfacing
  graded recurrence counts for OS3/OM3/OP3 evidence without coupling the
  recurrence lane to optional AlphaGenome prediction credentials. (410, 412)

### Docs

- Refreshed migrated architecture, operator, source-versioning, configuration,
  observability, CLI discovery, and JSON contract documentation so public docs,
  `biomcp list`, and spec architecture agree on gettable versus search-only
  entities and current source/operator knobs. (405, 406, 411, 419, 420)
- Distilled architecture experiment policy so durable writeups stay in the repo
  while generated `architecture/experiments/**/results/**` artifacts are
  ignored and blocked from reappearing by the quality ratchet. (369, 381, 415)

### Fixes

- Fixed VAERS live support by using a CDC WONDER-compatible request client and
  pinned the positive VAERS vaccine aggregate canary. (413)
- Validated Figshare provider-supplied asset download URLs before fetch,
  rejecting unsafe schemes, unrelated hosts, localhost, and private-network
  targets while preserving explicit test server seams. (414)
- Classified Monarch phenotype 5xx responses as `Source unavailable` with
  retry guidance instead of generic API failures, keeping temporary upstream
  outages actionable in live verify. (416)
- Pinned extreme `Retry-After` handling through the default shared client so
  authenticated Semantic Scholar retries stay bounded even when an upstream
  returns very large retry windows. (403, 417)
- Tightened public CLI and spec contracts for high-risk request plans,
  update-option behavior, list-only Cargo wrappers, MCP skill allowlisting,
  GTR/diagnostic examples, and source-specific live lanes. (400, 401, 404, 418)
- Stabilized OLS4/discover timeout behavior, deterministic validation routing,
  live-pathway quarantine, and residual spec-runner reliability gaps after the
  mustmatch migration. (389, 390, 397)
- Fixed release and PR validation setup so GitHub Actions installs the
  standalone mustmatch CLI before contract/spec jobs, and refreshed vulnerable
  Python and Rust lockfile packages flagged by Dependabot. (post-0.8.23)

### Internal

- Reset BioMCP validation around request-contract tests, split routine
  `make spec` from release live verify, migrated markdown specs to the
  mustmatch binary, adopted robust ellipsis/run-expect assertions, and moved
  all live upstream specs into the verify lane. (371, 372, 373, 374, 375, 376,
  377, 378, 379, 391, 392, 393, 395, 396)
- Pruned brittle live assertions, quarantined ambiguous drug fallback specs,
  added renderer-envelope fixture contracts, and repaired deterministic
  request-plan coverage for disease, discover, article, and variant surfaces.
  (380, 382)
- Completed the 0.8.23 hardening pass for tickets 412 through 420, including
  full `make lint`, `make test`, `make spec`, variant-hotspots and VAERS live
  lane proofs, and a documented Monarch 502 live-lane caveat. (412, 413, 414,
  415, 416, 417, 418, 419, 420)

## 0.8.22 — 2026-04-30

### New features

- Extended WHO Prequalification drug coverage to include active pharmaceutical
  ingredients (APIs) alongside finished pharmaceutical products;
  `search drug --region who --product-type api` filters explicit WHO searches
  to API rows. (232)
- Added the CDC CVX/MVX vaccine identity bridge for default and EU/all
  vaccine-brand drug search, backed by local CDC bundle downloads and explicit
  refresh via `biomcp cvx sync`. (233)
- Added CDC WONDER VAERS as the aggregate vaccine adverse-event source for
  `search adverse-event --source <faers|vaers|all>`, including combined
  vaccine search, health visibility, and source inventory/docs surfaces. (235)
- Added the local GTR-backed `diagnostic` entity with `search/get diagnostic`,
  first-use bundle refresh, explicit `biomcp gtr sync`, and the supporting
  health/docs/spec contract. (236)
- Expanded `diagnostic` into a local multi-source surface with WHO Prequalified IVD
  infectious-disease products, source-aware search/get behavior, and explicit
  `biomcp who-ivd sync`. (237)
- Added FDA 510(k) and PMA regulatory overlays to
  `get diagnostic <id> regulatory`, rendering source-aware FDA device status
  when records match and a truthful empty regulatory section when they do not.
  (238)
- Added opt-in diagnostic pivots to gene and disease cards with
  `get gene <symbol> diagnostics` and `get disease <name_or_id> diagnostics`,
  backed by local GTR and WHO IVD diagnostic search. (239)
- Extended article search follow-up guidance with cross-entity keyword hints:
  JSON `_meta.next_commands` now includes gene and drug pivots for recognizable
  keyword patterns, while markdown `See also:` adds `discover` and
  `get disease` recovery paths for short disease-like phrases. (242)
- Added opt-in `clinical_features` disease output backed by MedlinePlus
  clinical summaries, reviewed HPO phenotype mapping, source-native evidence
  URLs, and unsupported-disease empty states via
  `get disease <name> clinical_features`. (252, 253, 254)
- Added structured `_meta.workflow` guidance for worked-example prompts. (279)
- Added workflow-ladder sidecars and schema-backed `_meta.ladder[]` payloads
  so routed workflows can carry explicit ladder steps across the CLI/MCP
  contract. (282)
- Added richer CLI help examples for batch/variant-limit usage and preserved
  the parser-level skill uninstall contract through current help and MCP guard
  surfaces. (217, 224)
- Added opt-in article fulltext source-boundary coverage and PDF fallback so
  article retrieval can truthfully report source limits while recovering PDF
  text when structured fulltext is unavailable. (255, 256)
- Added DDInter-backed drug-drug interaction workflows with
  `biomcp drug interactions <name>`, `get drug <name> interactions`, local
  bundle health, source-scoped empty wording, and `biomcp ddinter sync`.
  (303)
- Added explicit transcript-HGVS normalization via
  `biomcp variant normalize <service> <transcript_hgvs>`, backed by Mutalyzer
  and VariantValidator with service-labelled JSON statuses and warnings. (370)

### Docs

- Added BioASQ-oriented how-to routing rows to `skills/SKILL.md` for
  gene-disease association, gene localization/protein-function, drug regulatory
  date, and variant clinical-significance workflows, steering agents to
  structured-data shortcuts before article search. (240)
- Refreshed architecture, source-integration, and current-state docs so the
  shipped architecture and CLI guidance match the post-decomposition repo
  surface. (272, 273, 274, 275)
- Repaired backtick quoting in the mustmatch study/spec contract, published
  BioASQ benchmark guidance, and made the canonical `SKILL.md` render/install
  contract explicit for agents and operators. (276, 280, 281)
- Aligned the public landing-copy contract with the shipped workflow landing
  bullets so `make test-contracts` accepts the current README and docs homepage
  feature counts. (286)
- Refreshed source, licensing, versioning, CLI decomposition, diagnostic,
  clinical-features, API-key, quality-bar, and staging-demo docs to match the
  shipped v0.8.22 surface and current local-runtime key expectations. (222,
  272, 292, 304, 306, 314, 318, 329, 332, 341, 358, 362)

### Fixes

- Changed custom CLI validation failures (`BioMcpError::InvalidArgument`) to exit
  `2`, matching clap parser failures and separating bad usage from runtime
  failures. (353)
- Patched `rustls-webpki` to the safe release line and made `make check` enforce
  the cargo-deny advisory gate alongside the existing license gate. (290)
- Fixed cargo-install compatibility, deterministic EMA freshness checks, OLS4
  smoke routing, WHO drug JSON envelopes, and WHO API cache-limit wording so
  install, health, and explicit WHO/API drug searches stay stable. (218, 220,
  226, 232)
- Added compact diagnostic rows and capped disease diagnostic pivots so
  gene/disease diagnostic follow-ups stay scannable and bounded. (266, 267)
- Replaced the stale GTR sample with a live-valid GTR example and added
  zero-result recovery for the local diagnostic surface. (268, 269)
- Tightened entity-aware article follow-ups with a same-session loop-breaker
  for overlapping suggestions and PubMed ESearch cleanup for bounded
  question-format filler words. (277, 278, 283)
- Improved sparse drug research-code recovery, relational `discover` filtering,
  trial help example/flag consistency, and CTGov intervention alias preservation
  so generated follow-up commands stay useful and shell-safe for degraded or
  ambiguous inputs. (302, 310, 313, 338, 339, 340, 342, 351, 357)
- Repaired the targeted `SPEC_SMOKE_ARGS` lane so it stores stable smoke
  section IDs and resolves them to current mustmatch pytest item IDs at runtime;
  the quality ratchet now checks collectability before stale line-qualified
  selectors can reach `make spec-smoke`. (288)
- Hardened update and machine-readable CLI contracts: `biomcp update` fails
  closed when checksum sidecars are missing, `biomcp --json version` documents
  its plain-text exception, `biomcp --json list` remains parseable, and
  short-literal update ratchets were replaced with structural behavior checks.
  (331, 333, 352, 355)
- Stabilized MCP stdio no-input guidance, cBioPortal study download idle/stall
  handling, disease clinical-feature specs, OLS4 disease/discover fallback IDs,
  WikiPathways parallel tests, and protein ComplexPortal specs with regression
  coverage. (326, 336, 345, 346, 350, 354, 358)
- Surfaced Semantic Scholar authentication/degradation status in article search
  and honored `Retry-After` during authenticated retries so throttled searches
  disclose source health and back off correctly. (364, 365, 366)

### Internal

- Completed release/docs contract cleanup for the `0.8.22` line so the latest
  release notes, ticket inventory, and release metadata stay aligned. (264, 265)
- Made `make check` include the public Python/docs contract lane and added
  `make release-gate` as the named local release-blocking command, so
  release-readiness docs and validation profiles share one contract. (287)
- Reworked `SPEC_SMOKE_ARGS` handling around stable smoke inventory and current
  mustmatch pytest item IDs, and cleaned up `.march` artifact handling for
  repo submission paths. (270, 271)
- Migrated the release gate to the spec-v2 corpus, made `spec-pr` self-contained
  for March kickoff, restored release-gate reliability, and kept stable smoke,
  canary, cache-warm, and test-contract lanes aligned with the current shipped
  docs/spec surface. (294, 297, 298, 299, 300, 301, 307, 308, 344)
- Decomposed oversized CLI modules, absorbed residual line-cap allowlists,
  clarified the benchmark harness as internal, and added runtime-wiring/line-cap
  ratchets so structural cleanup cannot reintroduce public-surface drift. (309,
  319, 320, 321, 322, 323, 324, 325, 334, 335, 343, 347)
- Strengthened release readiness with gene-all warm-budget coverage, local gate
  uv-build fixes, leaked-artifact cleanup, binary asset attributes, CI wrapper
  stale-binary warnings, Python 3.12/Rust 1.95 CI compatibility, self-contained
  local-data spec fixtures, and final v0.8.22 readiness audits before publish.
  (247, 258, 262, 263, 285, 289, 315, 316, 317, 327, 328, 330, 348, 363, 367)
- Recorded the diagnostic-entity, HPO clinical-feature, Obsidian vault,
  biomedical news, post-v0.8.21 shipped-surface, and neural-reranking reviews
  as architecture or deferred work unless runtime wiring shipped in the bullets
  above. (230, 243, 244, 245, 246, 284)

## 0.8.21 — 2026-04-16

### New features

- Added article date-range filtering (`--date-from`/`--date-to` and
  `--year-min`/`--year-max` aliases) to `search article`. (182)
- Added EMA regulatory region (`--region eu`) to `search drug` and `get drug`,
  backed by local EMA human-medicine feeds; data auto-downloads on first use via
  `biomcp ema sync`. (203)
- Added entity-aware HATEOAS suggestions (`_meta.next_commands`) to article
  search JSON output; gene-anchored and keyword searches return contextually
  appropriate follow-up commands. (195, 202)
- Added article search fallback suggestions when `discover` resolves no entities,
  so dead-end queries get recovery paths. (204)
- Surfaced EuropePMC and PubMed `first_index_date` per article in
  `search article --json`; `null` for sources that do not expose index dates. (196)
- Expanded trial search with drug alias union so searches match trials registered
  under any approved drug name variant. (198)
- Surfaced opt-in section hints (`More:` block) in gene and disease base card
  responses; `get gene <symbol>` and `get disease <name>` now show `funding`
  and `survival` opt-in hints. (209)

### Fixes

- Fixed `search all --counts-only --json` so sections correctly omit `results`,
  `links`, and `total` fields per the counts-only contract. (206)
- Disambiguated FAERS 404 (drug not in post-marketing database) from "found but
  no matching events"; added ClinicalTrials.gov fallback for adverse-event
  lookups when FAERS returns 404. (197)
- Improved error messages for `study download` 403/404 (actionable, with
  `--list` recovery hint) and DisGeNET 403 (env var name and docs URL). (212)
- Added study subcommand help descriptions and a regression test so help
  completeness is enforced going forward. (210)

### Docs

- Refreshed architecture docs to the post-decomposition state: source map,
  modes prose, and MCP allowlist contract. (213)
- Fixed documentation drift: README source count, CHANGELOG WHO entry, and
  root help command count. (211)
- Published two blog posts on biomcp.org: daraxonrasib six-commands workflow
  article and a follow-up that removes biomcp-python references. (200, 201)

### Internal

- Added `mustmatch lint` for spec bash blocks so spec sections that silently
  skip due to a missing pipe are now caught. (221)
- Untracked `.march` build artifacts from git so worktree state stays clean. (193)
- Added Compact Approval Fields spec block for EMA and EU regulatory data
  contract assertions. (194)
- Consolidated `EnvVarGuard` and `TempDirGuard` into a central `test_support`
  module shared across all test crates. (199)
- Added docs-only validation profile so `make docs` can run without triggering
  the full build gate. (205)
- Updated `tar` and `rustls-webpki` for active RustSec advisories. (207)
- Fixed 7 `make test-contracts` failures from assertion drift after the
  decomposition batch. (208)

## 0.8.20 — 2026-03-30

### Security

- Security fix: remove partial API key from health output so shared operator
  reports no longer leak credential fragments. (067)
- Hardened the MCP allowlist so mutating `study download` paths stay blocked
  while the safe `--list` path remains available. (068)

### Fixes

- Fixed `batch --json` so responses always include the `_meta` contract field.
  (069)
- Hardened the GWAS variant section against live decode failures. (070)
- Fixed gene-to-PGx and disease-to-drug See-also routing so those pivots land
  on the correct entity helpers. (077)
- Fixed large PMC OA full-text downloads that were timing out on bigger
  payloads. (080)

### New features

- Added cross-entity See-also links for drug→PGx, gene→PGx, disease→study,
  and zero-result→discover routing. (076)
- Added the quality ratchet to `make check` so the repo gate now audits
  mustmatch lint, MCP allowlist coverage, and source-registry drift. (075)
- Added the study top-mutated helper plus disease literature fallback for
  missing direct evidence paths.
- Added SEER Explorer-backed disease survival sections so `get disease <name_or_id> survival`
  can show 5-year relative survival by sex for mapped cancers. (173)
- Added NIH Reporter funding sections so `get gene <symbol> funding` and
  `get disease <name_or_id> funding` can show top grants from the most
  recent 5 NIH fiscal years. (174)
- Added WHO Prequalification as the shipped local-data drug source for
  `get drug <name> regulatory --region who` and `search drug --region who`;
  run `who sync` to force-refresh the local CSV before freshness-sensitive
  regional drug lookups.

### Improvements

- Reworked CLI help, typed-path guidance, and the skill catalog for
  self-teaching agent navigation. (072, 073, 074)
- `get disease` now shows full disease definitions instead of truncated
  characterizations. (080)
- `get disease` now separates defining phenotypic features from the full HPO
  list with ranked key-feature summaries. (081)
- `get gene` now shows UniProt protein isoforms in the protein section. (082)
- `get drug` now shows drug target family names alongside individual targets.
  (083)
- `get gene` now shows protein legacy names alongside current symbols. (084)
- `get variant` now shows variant legacy nomenclature alongside current HGVS
  names. (085)
- Improved purine-analog mechanism labels so ATC-exact and mechanism-of-action
  coverage stay aligned. (086)
- Closed the remaining BioASQ product gaps around protein function truncation,
  variant-specific targets, and disease feature coverage. (087)
- Replaced the in-shell mustmatch shim with the packaged `mustmatch lint`
  entrypoint plus a small compatibility layer for the repo’s existing ratchet
  contract. (088)

### Docs

- Added architecture contract docs for See-also links, discover grammar, CI,
  and post-tag public proof expectations. (078)
- Documentation consistency audit across the public docs surface.
- Repo cleanup for layout, example paths, and tracked `.march` artifacts.
  (079)

## 0.8.19 — 2026-03-26

### Improvements

- Cleaned up Semantic Scholar health output so unauthenticated setups are
  reported as optional/shared-pool usage instead of looking like a broken
  configuration. (055)
- Added six more built-in study chart types: pie, density, violin,
  ridgeline, heatmap, and stacked bar. (041)
- EMA-backed drug data now auto-downloads on first use and refreshes stale
  local caches, so EU drug coverage no longer requires manual setup. (057)

### CI

- Moved EMA-backed specs out of the PR-blocking stable lane into the smoke
  lane so CI keeps EMA coverage without destabilizing the release gate. (056)

### Docs

- Refreshed the README, docs index, and source landing pages with clearer
  public-facing messaging.
- Added three new blog posts covering agent skills, article search, and
  built-in cancer genomics charts.

## 0.8.18 — 2026-03-25

### New sources

- Added EMA regional drug coverage backed by local EMA human-medicine feeds,
  so `search drug`, `get drug`, and `get drug shortage` support
  `--region us|eu|all` across EU regulatory status, DHPCs, referrals, PSUSAs,
  shortages, and the existing US label surface. (043, 044)
- Added the BioASQ benchmark module with `benchmarks/bioasq/ingest_public.py`,
  dataset and validity manifests, and the published BioASQ benchmark
  reference at `docs/reference/bioasq-benchmark.md`. (045, 046)

### Improvements

- `search drug` now defaults to `--region all`, `get drug --region` is
  surfaced in help, OpenFDA label fallback prefers exact brand-name matches,
  and OpenTargets drug-indication lookups are corrected for mixed US/EU
  workflows. (043, 044)
- `S2_API_KEY` is now optional for Semantic Scholar-backed article
  enrichment and helpers: unauthenticated calls use the shared Semantic
  Scholar pool at 1 req/2sec, while authenticated calls keep the 1 req/sec
  quota with retry backoff. (047)

## 0.8.17 — 2026-03-23

### New sources

- Added WikiPathways as a third pathway source alongside Reactome and KEGG,
  with source-aware section handling plus operator proof coverage in
  `biomcp health` and `scripts/contract-smoke.sh`. (025)
- Added an optional Semantic Scholar search leg to `search article` when
  `S2_API_KEY` is set, merging PubTator3, Europe PMC, and Semantic Scholar
  results with identifier-aware deduplication and directness-first ranking.
  (034)
- Added KEGG as a pathway source alongside Reactome and WikiPathways.
- Added gnomAD constraint metrics for variant-interpretation context.
- Added Human Protein Atlas tissue expression to gene output.
- Added ComplexPortal-backed protein complex data.
- Added DisGeNET gene-disease association scores.

### New commands

- Added `biomcp discover <query>` as the free-text concept-resolution
  entrypoint, using OLS4 as the required backbone with optional UMLS
  crosswalks and MedlinePlus disease/symptom context. (028)
- Added `biomcp article batch <id>...` for compact multi-article summary
  cards with parallel article fetch and optional batched Semantic Scholar
  enrichment; the command is limited to 20 IDs. (037)

### Improvements

- Restored pathway progressive disclosure and exact-title-first ranking, so
  default pathway cards stay concise and exact pathway-title matches rise to
  the top across sources. (017)
- Expanded operator proof surfaces and refreshed source-aware architecture docs
  so the shipped source inventory, API-key handling, proof expectations, and
  section-availability rules are explicit and consistent. (018, 019, 030)
- Improved protein complexes terminal layout and normalized pathway CLI usage
  errors and remediation. (020, 021)
- Added source labels and missing evidence URLs across entity detail outputs in
  Markdown and JSON for stronger traceability. (022, 025)
- Added paper/reproducibility and community-packaging artifacts, including
  `paper/`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, and `.zenodo.json`. (023)
- `get gene` and `get drug` misses now return discovery-backed canonical
  suggestions with structured `_meta.alias_resolution` /
  `_meta.next_commands` guidance instead of dead-end errors. (029)
- Help, `list`, and docs now surface working typed paths, and `search all` /
  `search article` can emit `--debug-plan` routing metadata in Markdown or
  JSON when requested. (032)
- Typed variant retrieval now accepts common shorthand forms, emits
  variant-scoped recovery guidance on misses, and accepts long-form protein
  notation such as `AKT2 p.Pro50Thr`. (033, 039)
- `search all` cross-routing now stays inside controlled typed fallback rules,
  reducing noisy drill-downs and duplicate follow-up commands. (035)
- Typed retrievals now expose compact answer-bearing summaries for approvals,
  population frequencies, and disease/variant evidence without removing the
  detailed sections. (036)
- Deepened OpenTargets integration with tractability and
  genetic-association sections.
- Added JATS-aware PMC full-text extraction for article detail views.
- MCP chart responses can now return SVG inline as a base64 image payload.
- Polished Kuva chart output with human-readable labels and KM curve
  improvements.
- Suppressed retry-middleware warnings from stderr.
- Pathway output now keeps section availability truthful and gives better
  guidance when a source lacks a section.

### Fixes

- Fixed drug interaction output so supported drugs no longer return empty
  interaction sections.
- Fixed g:Profiler enrichment timeouts.
- Fixed the DisGeNET template crash on sparse or null payloads such as
  `biomcp get gene KYNU disgenet`. (031)
- Fixed default article retraction filtering so only confirmed retractions are
  excluded; PubTator3 and Semantic Scholar rows with unknown retraction status
  remain visible. (038)

## 0.8.16 — 2026-03-17

- Adopted the shared `bin/lint` script and wired `make lint` / `make check`
  to the March repo convention for Rust repos.
- Refreshed the README, homepage, CLI reference, and helper docs so the public
  release docs match the current BioMCP command surface.
- Expanded `skills/SKILL.md` for article graph helpers, `enrich`, `batch`,
  chart discovery, chart flags, and updated drug interaction guidance.
- Updated the Kuva chart blog post with checked-in SVG outputs for all nine
  worked examples plus placeholder references for terminal screenshots.
- Added Semantic Scholar article enrichment and helpers for TLDR, citations,
  references, and recommendations.
- Added Kuva-backed study chart rendering plus chart discovery/docs for SVG
  study outputs.
- Trial search now accepts fractional ages such as `0.5 years`.
- Generated `_meta.next_commands` now pass a parser-level validity gate, and
  article JSON guidance is cleaner and more truthful.
- Fixed federated article search offset and sort semantics with correctness
  coverage.
- Breaking MCP runtime change: renamed the MCP execution tool from `shell` to
  `biomcp`; update MCP clients and demos to call `biomcp` after `tools/list`.
- Added `CITATION.cff` and citation pointers in the README and public docs.
- Fixed Semantic Scholar references to use cited papers instead of referenced
  papers.
- Fixed article search timeouts on `--exclude-retracted`.
- Fixed article retraction filtering and unary `NOT` parsing.
- Stabilized CTGov trial pagination and age-filtered totals: improved cursor
  behavior, aligned count/search params, clarified trial-filter guidance, and
  exposed approximate age-only counts when exact totals are too expensive.
- Expanded release-quality gates with contracts CI, version-sync checks, a
  PR-gated spec suite, stable/smoke lane splitting, and protein entity spec
  coverage.
- Updated lockfiles for the `quinn-proto` and `PyJWT` vulnerability
  advisories.
- Refined the Streamable HTTP demo into a tested end-to-end BRAF/melanoma
  workflow with a focused README and Python 3.11 compatibility.

## 0.8.15 — 2026-03-11

- Fixed the planning-docs CI path regression so release validation uses the
  repo-local planning fixtures by default instead of an Ian-local absolute
  path. This is the fix from PR #191 that unblocks release packaging on
  GitHub Actions.
- Refreshed the public discovery docs so `search all` is taught as the unified
  cross-entity entry point in the README and docs index. This is the docs
  alignment from PR #190.

## 0.8.14 — 2026-03-10

- Promoted remote Streamable HTTP in newcomer docs with a dedicated getting-started page for `biomcp serve-http`.
- Added a runnable PEP 723 demo client at `demo/streamable_http_client.py` for `/mcp`, including a `tools/list` flow and `biomcp version` tool call.
- Surfaced the canonical remote HTTP routes more prominently across public docs: `/mcp`, `/health`, `/readyz`, and `/`.
- Kept `serve-sse` migration guidance visible while shipping the Streamable HTTP release/docs/demo verification package together.
- Reranked disease search results so exact-match canonical names surface first.
- `search article` now rejects unsupported identifiers with explicit contract guidance instead of falling through ambiguously.
- Defaulted article search sorting to relevance across CLI entry points.
- Normalized search aliases, identifiers, and trial phase semantics across entity searches.
- Broadened trial mutation matching across title, summary, keyword, and eligibility fields.
- Removed stale skill-discovery UX and refreshed `list` / help output to match the shipped CLI surface.

## 0.8.13 — 2026-03-09

- `study survival` now reports Kaplan-Meier median survival, 1/3/5-year survival landmarks, and two-group log-rank p-values.
- Fixed survival median calculation: replaced raw follow-up median with Kaplan-Meier median survival.
- `study compare --type expression` now reports Mann-Whitney U and p-value.
- Added `study download <study_id>` and `study download --list` for local cBioPortal datahub installs.
- Hardened `study download`: no total body timeout for large archives, path-like study ID rejection, stream-to-disk download, and cleanup on failed extraction.
- Refreshed `skills/SKILL.md` with real study command usage, including `cohort`, `filter`, and `survival --endpoint` examples.

## 0.8.12 — 2026-03-07

- Added `study` subcommand: local cBioPortal study adapters with CLI query surface (`download`, `cohort`, `survival`, `compare`).
- Added `study filter`: multi-omics cross-table sample joins across mutation, CNA, expression, and clinical data.
- Added Fisher exact p-values to co-occurrence pair analysis.
- Added skill installer auto-discovery: scans existing agent config directories in priority order and supports the `.agents/skills/` cross-tool standard.
- Refreshed skill guidance and reference material: adopted the measured `SKILL.md` refresh and added drift validation for examples, schemas, and jq snippets.
- CLI quality pass: positional search arguments, multi-token input handling, variant fallback, NCT ID validation, bare-entity shortcuts, staged trial fill, article request timeout, and `--until` alias.

## 0.8.11 — 2026-03-06

- Added `expression` section to gene output (GTEx tissue-specific TPM data).
- Added `druggability` section to gene output (DGIdb drug-gene interactions and categories).
- Added `clingen` section to gene output (gene-disease validity and dosage sensitivity).
- Added evidence URLs (`_meta.evidence_urls`) to all entity output — includes Ensembl, OMIM, NCBI Gene, and UniProt links where available.
- Added `spec/` BDD documentation suite with 54 passing executable specifications.
- Unified article search: fan-out across PubTator3 and Europe PMC in parallel with PMID deduplication, source-grouped rendering, and `--source <all|pubtator|europepmc>`.
- Added infinite cache mode via `BIOMCP_CACHE_MODE=infinite` for offline/demo workflows.
- Consolidated CI into a single job; streamlined release pipeline.
- Added reusable presentations infrastructure with an intro deck, branded theme assets, and slide templates for BioMCP talks.
- Hardened PyPI release packaging for arm64 and portable version syncing in the release workflow.

## 0.8.10 — 2026-03-04

- Polished `search all` output: GWAS trait relevance filtering, clinical significance sorting, uninformative variant suppression.
- Gene-anchored article search via `GENE_PROTEIN` field for higher precision.
- Added trial search improvements (`--mutation` boolean handling and `--criteria` support).
- Added PyPI trusted publisher setup and release workflow updates.

## 0.8.9 — 2026-03-03

- Added `search all` cross-entity command — parallel fan-out across major entities (genes, variants, diseases, drugs, trials, articles, pathways, PGx, GWAS, and adverse events) with counts-first display and HATEOAS deep-dive links.

## 0.8.8 — 2026-03-02

- Output cleanup across entity renderers for consistent markdown formatting.
- Improved help discoverability and completeness across all subcommands.
- Rewrote footer output with streamlined next-step suggestions.

## 0.8.7 — 2026-02-27

- Improved help text discoverability and completeness for all CLI subcommands.

## 0.8.6 — 2026-02-27

- Fixed trial filter accuracy: age-based post-filtering, exclusion-criteria detection, and pathway source correctness.

## 0.8.5 — 2026-02-26

- Fixed `install.sh` to resolve `latest` tag to a release that has downloadable assets.
- Hardened input safety with SSRF prevention on user-supplied URLs.
- Enforced pagination and validation contracts across search endpoints.

## 0.8.4

- Simplified CI/CD workflow triggers to avoid redundant post-merge runs.
- Added runner-aware `actions/cache@v4` protoc caching in CI, docs deploy, and release workflows.
- Pinned workflow protoc installation to `28.3` with cache-aware conditional setup.

## 0.8.3

- Added inclusion-first eligibility post-filtering to reduce exclusion-only false positives in trial matching.
- Applied cryptography dependency security updates and refreshed release artifacts.
- Tightened CLI/docs consistency around trial filter behavior and output contracts.

## 0.8.2

- Added location-level facility and geo post-filtering for more accurate trial site matching.
- Embedded git tag version metadata in the binary for reliable `biomcp version` output.
- Improved pagination and result-window handling across search command families.

## 0.8.1

- Expanded clinical-grade trial filtering with ~45 new search flags and validation rules.
- Added MCP tool auto-generation to reduce drift between CLI capabilities and server tool surface.
- Upgraded help/discoverability content and list-based guidance for agent-driven workflows.

## 0.8.0

Complete rewrite from Python to Rust. Single static binary, no runtime dependencies.

### Highlights

- Single-binary CLI and MCP server — no Python, no pip, no virtual environments
- 15 biomedical data sources, unified command grammar, compact markdown output
- 14 embedded skills (guided investigation workflows)
- MCP server (stdio + SSE) with tool and resource support
- HTTP proxy (`serve-http`) for multi-worker shared rate limiting
- Production installer with SHA256 verification (5 platforms)
- Progressive disclosure: search returns summaries, get returns full detail with selectable sections
- NCBI API key support for improved rate limits
- 429 retry with Retry-After backoff

### Data sources

MyGene.info, MyVariant.info (ClinVar, gnomAD, CIViC, OncoKB), ClinicalTrials.gov,
NCI CTS API, PubMed/PubTator3, MyChem.info, Monarch/MONDO, Reactome, UniProt,
OpenFDA FAERS, PharmGKB/CPIC, GWAS Catalog, Monarch (phenotypes), AlphaGenome (gRPC).

### Breaking changes from Python BioMCP

- Python package `biomcp-python` is no longer maintained
- MCP tool names and signatures have changed
- Configuration via environment variables only

## 0.7.3

Legacy Python BioMCP. See branch `python-0.7.3` for source code.

