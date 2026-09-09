# Findings from the 2026-09-01 interface study that have no ticket

An empirical study ran BioMCP 0.8.25 (git `e127992e`) against live public
APIs on 2026-09-01. Roughly 60 shell commands measured the tool surface, and
31 agent runs through the stdio MCP server measured behaviour through it.

Most of what it found already became records 1091 through 1100 — the silent
zero on variant search, the pharmacogenomics phenotype collapse, the false
zero from a failed disease label lookup, an unresolved filter reporting zero,
`whyStopped`, one unavailable source discarding the working sections, the MCP
schema not naming its sections, and a plain-text query field silently
accepting filter syntax. Records 1104, 0899 and 0950 cover the adverse-event
percent column and the genome-build labels.

Six findings have no ticket and no record. They are listed here for triage.
Nothing below is validated as a requirement by anyone outside these runs.

## 1. `retrieved_at`, a version stamp and a response digest in the standard `_meta` envelope

The reproducibility story needs a claim-to-retrieval record. None of the
three fields is in the standard envelope. The design already exists and is
scoped to two commands: `variant articles --verify-identity` carries
`verifier_version`, `provider_template_version`, two canonical subset hashes
and an `artifact_id` SHA-256 over the request, response and content chain.
ClinGen normalization carries `CarProvenance { request_template_version,
car_version, response_sha256 }`. A content-addressed provider byte store
already exists in `src/cache/provider_capture.rs`.

Two complications. There is no single envelope — at least four response
shapes exist (`src/render/json.rs`, `src/cli/shared.rs`, an inline struct in
`src/cli/article/dispatch.rs`, plus the discover and error variants), so the
change lands in four places. And `_meta.evidence_urls` are landing pages
reconstructed from identifiers rather than the URL that produced the bytes.

## 2. Protein-numbering tolerance on `--hgvsp`

`--hgvsp K27M` returns 0 and `K28M` returns 1 (H3K27M diffuse midline
glioma). `--hgvsp E6V` returns 0 and `E7V` returns 1 (sickle haemoglobin).
The filter demands initiator-methionine-inclusive numbering; clinical usage,
the literature and the WHO CNS tumour classification all use mature-protein
numbering.

Minimum fix: when an `--hgvsp` lookup resolves to nothing, retry at plus or
minus one and, on a hit, return it with an explicit note that the numbering
was adjusted. That preserves the honest `Unresolved` signal without the trap.

The agent runs found this costs effort rather than accuracy at the model
class tested — agents recovered every time — so it ranks below the items
already ticketed.

## 3. A frozen replay mode for evaluation

Any repeated measurement against mutable live APIs needs one. Two options.
Warm-cache replay (`BIOMCP_CACHE_MODE=infinite` with a snapshotted
`BIOMCP_CACHE_DIR`) needs no new code and is already how CI replays specs,
but the snapshot is an opaque blob store that a reader cannot audit. A
general request-to-fixture router would extend the pattern in
`benchmarks/output-footprint/run.py` across the provider-override environment
variables, backed by `testdata/sources/` and its `capture-receipts.json`.

Caution: `docs/reference/configuration.md` says the base-URL overrides are
not a stable operator API. A harness built on them is built on a surface the
maintainers reserve the right to move.

## 4. Rank suggestions, and prefer the term the user supplied

Three observed cases where the suggestion layer degraded the investigation.

- The medulloblastoma card suggests `search trial -c "cerebellum embryonal
  neoplasm"`, which returns 110 trials and 3 recruiting against 345 and 36
  for the user's own term. The suggestion took `synonyms[0]` over the input.
- A Li-Fraumeni phenotype search returns `fibromatosis, gingival, 6` as the
  top hit and then suggests opening it.
- `discover "H3K27M"` finds the correct concept with its full synonym list
  and suggests exactly one command, `get drug "h3k27me3"`, routing to a
  histone methylation mark misclassified as a drug.

Rule: when the user supplied a term that resolves, keep using it. Do not
suggest opening a low-confidence top hit without marking the confidence.

## 5. An abstention threshold on literature search

`search article -k "zolgensma treatment of retinoblastoma randomized trial"`
pairs a spinal muscular atrophy drug with an unrelated eye tumour and returns
three confidently formatted articles. The `Why` column reads `abstract 1/6`:
one of six query terms matched. The number needed to warn is already computed
and printed. Nothing thresholds on it.

## 6. UTF-8 double-encoding in the GTR bundle

`Ion AmpliSeq(tm)` renders as `Ion AmpliSeqâ„¢` and `CancerNext-Expanded(R)`
as `CancerNext-ExpandedÂ®`. These are lab and manufacturer name fields that an
agent may quote verbatim.

## The pattern worth noting

Several of these are cheap for the same reason: the mechanism already exists
somewhere else in the tree and was not generalized. Provenance exists on two
commands. `section_outcomes` exists in one envelope. The alias resolver
exists on `get gene`. The abstention signal exists in the `Why` column. The
accurate characterization is not that provenance discipline is missing — it
is strong where it is applied, and the coverage is partial.

Raw material for every claim above is archived locally under
`archive/notes-2026-09-09/`, split between the Markdown snapshot and the
non-Markdown command captures. Any published measurement should be re-run and
pinned to a git SHA rather than a version string: `0.8.25` names at least
three materially different binaries.
