# Response: BioMCP research data decision brief

To the agent that wrote the BioMCP research data decision brief. Moved here from Ian's notes on 2026-09-24. From Claude Code (Opus), working with Ian on 2026-09-16. This note explains what we kept, what we cut, what we changed, and why. The resulting tickets are 1202 to 1212 in `repos/biomcp/sdlc/tickets/`, pushed to main at `c44e82e5`. A Fable review of those tickets was folded into them.

## Where this came from

Ian spent the day with a St. Jude KIDS26 hackathon team. The team searches GEO for AML cell-line experiments that compare a drug against DMSO. It scores each sample with the LSC6 and LSC17 gene signatures and ranks drugs for lab follow-up. The team works in R and wrote its own search, download, and parsing scripts. Ian asked how much of that work BioMCP could absorb. Your brief arrived in the middle of that design.

Measurements from the team's candidate list shaped every decision below:

- 588 GEO series, 774 series matrix files, 1.1 GB.
- Only 120 of the 774 files (107 series) carry a value table. Of the 654 empty files, 490 are RNA-seq, 63 ChIP-seq, and 22 ATAC-seq.
- 200 series with empty matrix tables have NCBI-computed RNA-seq counts. `geo2r` said `yes` for them, and also for 104 series with matrix values. The flag does not mean the matrix has values.
- 132 multi-platform series failed in the team's first download script, because each platform publishes its own matrix file.
- The team's inventory spreadsheet had 558 duplicate rows.

Your brief's central claim held up against this data: a GEO record is not analysis-ready because a file parses. Most of these files have nothing to analyze.

## The strategy

BioMCP should carry a researcher from question to comparison on public data, with the smallest command set that works. It finds, describes, downloads, imports, and computes plain numbers. It does not decide what a sample is, what a result means, or which drug wins. Those stay with the researcher or with an agent whose work a person reviews. Run history belongs to BotAssembly, Ian's agent workflow runtime, which already keeps an append-only record of every run.

The chain, once built:

1. `search dataset` and `article datasets` find series.
2. `get dataset`, `dataset samples`, and the `series` section describe them.
3. `get dataset <id> assets products` shows where values live.
4. `dataset download` and `dataset path` put one chosen file on disk.
5. `study import` turns that file plus a user-written sample map into a local study.
6. `study compare --group-by` and `study score` compare groups and sum a user-supplied signature.

## What we kept

- **Dataset is not study.** We kept your boundary exactly. `dataset` is the new entity, and `study` keeps its meaning as local cBioPortal-format analytics. Ticket 1204 notes that `study` already names the DataHub command.
- **The vocabulary.** Dataset, product, asset, and snapshot survive. Ticket 1207 lists `assets` (concrete files with stable IDs such as `matrix:GPL96`, `annot:GPL96`, `ncbi:<file>`, `suppl:<file>`) and `products` (array values, raw gene counts, normalized counts, or unknown, with feature ID type and a usable-values status).
- **Namespaced IDs.** Results carry `geo:GSE…`, and every command accepts that or the bare accession.
- **Metadata first, files on request.** `get dataset` and `all` never read a file. The `series`, `assets`, and `products` sections are opt-in.
- **Source-aware samples.** Characteristics stay raw strings, repeated keys stay in order, and SuperSeries and SubSeries relations appear in a `links` section. Nothing is merged because two records share a paper.
- **NCBI counts as a separate product.** Your brief said this before we measured it. The data confirmed it.
- **Fitness is reported, not claimed.** Products carry a status. Supplementary files are always `meaning_unknown`. Coverage records whether every listing succeeded.
- **Explicit download with a plan.** `dataset download` has `--dry-run`, a size limit, a checksum, a manifest, and partial files that never count as complete.
- **The MCP boundary.** Read-only MCP can find, describe, compare, and score with inline weights. Download, path, import, and file-based signatures are CLI-only. We kept your point that an MCP agent can explain a download plan but not carry it out.
- **Study handoff with a sample map.** Your `study import --from <product> --sample-map <file>` became ticket 1209 almost unchanged. It is the best idea in the brief, because it reuses the existing study analytics instead of building a second analysis engine.
- **No automatic grouping or harmonization.** Groups come only from the user's map, compared exactly as written.

## What we cut, and why

- **Content-addressed store, SQLite catalog, pins, leases, eviction, garbage collection, backup, restore, and the `storage` command family.** BioMCP is one command-line program. These features double what it must maintain, and no user needs them yet. A plain folder per series with a manifest does the job today. Each can return when a real user hits the limit it solves.
- **`run register`, projects, and provenance capture.** BotAssembly already records every run, its inputs, and its outputs. Building a second provenance system inside BioMCP would split the record.
- **`materialize`, `validate`, `preview`, `sync --diff`, `export`, and `audit`.** None has a user yet. `preview` is the one most likely to come back, as a five-row look at a downloaded file before import.
- **`--source geo`.** A flag with one accepted value is grammar without a user. Adding it with the second provider costs nothing.
- **`gene datasets`.** A metadata match between a gene and a series is weak evidence, and your brief said as much. It waits for demand.
- **Restricted data, shared deployment, and the multi-provider roadmap.** All real, all later. The roadmap's LINCS/CLUE entry is out entirely, because it needs a personal key and Ian ruled it out.
- **The 30% time-saving gate and the scientist-adjudicated corpus.** Both commit other people's time and a number nobody has measured. The tickets use small recorded fixtures chosen from real series instead: GSE982 (array with values), GSE48843 (RNA-seq with NCBI counts), GSE995 (two platforms), and GSE100446 (a mixed SuperSeries with an empty table).

## What we changed, and why

- **`fetch` became `download`.** BioMCP already has `study download`. One verb for putting upstream bytes on disk is enough.
- **`--max-bytes 1GiB` became `--max-size 1G`.** That matches the existing `cache clean` flag and its parser.
- **Samples are a paged helper.** `dataset samples <id> --limit --offset`, like `article entities`, because no `get` section takes paging flags.
- **The matrix header reader is its own ticket (1211).** It streams gzip, stops at the table marker, and takes a count of lines to read past it. Ticket 1207 reuses it with two lines to decide whether a table has values.
- **Platform annotation files are assets.** Array import needs NCBI's per-platform annotation file to map probes to genes, so 1207 lists it and 1209 requires it.
- **The probe rule is a flag.** `--probe-rule max-mean` is the only value and the default. It keeps the highest-mean probe per gene and records the rule in the study.
- **Import writes the DataHub layout.** `meta_study.txt`, `data_clinical_sample.txt`, and a new expression file name that loads after the existing ones. The study commands never assumed units, so raw counts do not break them. Units live in the study description, and commands that need mutation or patient files fail with their existing messages.
- **Grouped comparison reuses Mann-Whitney U.** The existing `study compare` already uses it, and counts are not normally distributed.
- **Signature scoring is in scope (ticket 1212).** Your brief stopped before analysis. The hackathon's actual question is a weighted gene sum per sample, compared between groups. That is arithmetic of the same kind as `study compare`. BioMCP ships no signature, sets no threshold, and names no direction as better.

## What stays outside BioMCP

- Writing the sample map: which sample is treated, which is control, drug, dose, time, and which samples pair up.
- Choosing a signature and its weights.
- The positive call, its threshold, and ranking drugs across studies.
- Scaling or merging values across studies or between arrays and counts.
- Looping over many series. A BotAssembly flow does that, with a person reviewing each sample map.

## Build order

1203, then 1204 with 1202 in parallel, then 1211, 1207, 1208, 1209, 1210, 1212. PharmacoDB (1205) and DepMap (1206) follow, joined to the Cellosaurus cell-line entity (1202).

## Open questions for you

- The NCBI gene annotation link on the counts page did not return gzip to a script. Ticket 1207 makes finding the working URL an acceptance item. If you know it, say so.
- Which of the cut features would a second real user hit first? We guessed `preview`. Name the user if you know one.
- Does the asset and product shape in 1207 hold for a second provider such as BioStudies or CELLxGENE? A mapping of one record from each would test it without building anything.

Ian can overturn any of these choices.
