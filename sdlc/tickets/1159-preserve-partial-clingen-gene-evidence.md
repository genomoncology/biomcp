---
flow: build
priority: 8
---

# Preserve partial ClinGen gene evidence

## Goal

A slow or failed ClinGen request does not erase evidence returned by another ClinGen dataset. On 2026-09-04, two no-cache TP53 requests returned no ClinGen facts after the combined section timed out. Direct ClinGen requests responded separately, and the implementation can discard completed validity or dosage data when its sibling request fails. The reproduction and code path appear in `sdlc/issues/2026-09-04-one-clingen-failure-erases-other-results.md` at commit `d9d29dd1`.

## Desired functionality

BioMCP retains completed ClinGen gene-validity and dosage-sensitivity results
independently. The existing `clingen.validity`, `clingen.haploinsufficiency`,
and `clingen.triplosensitivity` fields keep their names, value types, ordering,
caps, and omission behavior. Two required additive fields make the state of
each result family explicit:

```json
{
  "clingen": {
    "haploinsufficiency": "Sufficient Evidence for Haploinsufficiency",
    "validity_status": {
      "status": "timed_out",
      "op": "gene_validity_download",
      "message": "ClinGen gene-validity download timed out."
    },
    "dosage_status": {
      "status": "data",
      "op": "gene_dosage_download"
    }
  }
}
```

`validity_status.status` and `dosage_status.status` use only `data`, `empty`,
`failed`, or `timed_out`. The `op` vocabulary is also closed:
`client_init`, `gene_lookup`, `gene_validity_download`, and
`gene_dosage_download`. `message` is omitted for `data` and `empty`. A failure
or timeout uses one of these stable public messages rather than an upstream
body, URL, local path, or parser diagnostic:

| Status owner | `op` | Public message |
|---|---|---|
| client construction | `client_init` | `ClinGen client initialization failed.` |
| validity download/decode/parse | `gene_validity_download` | `ClinGen gene-validity download failed.` |
| validity deadline | `gene_validity_download` | `ClinGen gene-validity download timed out.` |
| dosage download/decode/parse | `gene_dosage_download` | `ClinGen gene-dosage download failed.` |
| dosage deadline | `gene_dosage_download` | `ClinGen gene-dosage download timed out.` |
| lookup blocks a validity absence | `gene_lookup` | `ClinGen gene lookup failed; gene-validity absence is unconfirmed.` or `ClinGen gene lookup timed out; gene-validity absence is unconfirmed.` |
| lookup blocks a dosage absence | `gene_lookup` | `ClinGen gene lookup failed; dosage-sensitivity absence is unconfirmed.` or `ClinGen gene lookup timed out; dosage-sensitivity absence is unconfirmed.` |

`data` means at least one retained validity row or dosage classification.
`empty` is allowed only after a successful bounded response, recognized schema,
successful parse, and conclusive identity match with zero retained evidence.
Non-success HTTP, an HTML document in place of data, invalid encoding or CSV,
missing required headers, an oversized response, and any other acquisition or
parse error are `failed`; an expired operation deadline is `timed_out`. The
lookup endpoint's existing compatibility for a JSON-shaped body mislabeled
with an HTML media type remains lookup-specific; an HTML body is still a
failure. Validity requires `GENE SYMBOL`, `GENE ID (HGNC)`, `DISEASE LABEL`,
`CLASSIFICATION`, `CLASSIFICATION DATE`, and `MOI`. Dosage requires
`GENE SYMBOL`, `HGNC ID`, `HAPLOINSUFFICIENCY`, `TRIPLOSENSITIVITY`, and `DATE`.

HGNC lookup is one shared identity operation owned by the combined ClinGen
acquisition, not by either evidence family. A valid lookup result is supplied
to both parsers. If lookup fails or times out, an exact gene-symbol match in a
valid download may still produce `data`; a zero symbol match is inconclusive
and that family's status inherits the lookup failure or timeout with
`op: gene_lookup`. A valid lookup that returns no HGNC identifier still permits
the documented exact-symbol match and a healthy `empty`. This preserves useful
rows without turning a failed identity lookup into proof of absence.

One `ClinGenClient` is constructed before any ClinGen network work. A
construction failure starts no requests and marks both families `failed` with
`op: client_init`. Otherwise lookup, validity download, and dosage download
start concurrently, each under its own configured optional-enrichment
deadline. The implementation owns and settles those futures; it does not spawn
per-operation tasks. Any outer ClinGen prefetch handle is awaited on the normal
path and aborted on every early return, so the command leaves no detached
ClinGen work. One slow operation therefore neither consumes its sibling's
deadline nor erases its result.

The canonical `section_outcomes.clingen` and matching
`_meta.section_sources` entry are derived from the two family statuses exactly
as follows:

| Family result | `section_outcomes.clingen` | sources | message |
|---|---|---|---|
| both healthy, at least one `data` | `data` | `["ClinGen"]` | omitted |
| both `empty` | `empty` | `["ClinGen"]` | omitted |
| at least one `data`, plus any `failed`/`timed_out` | `degraded` | `["ClinGen"]` | `ClinGen gene evidence is partial; one result family is unavailable.` |
| no `data`, and any `failed`/`timed_out` | `unavailable` | `[]` | `ClinGen gene evidence is incomplete; no ClinGen absence can be concluded.` |

`_meta.section_sources` keeps one entry with key `clingen`, label `ClinGen`,
the same aggregate outcome, and the exact sources above. The inner family
vocabulary does not add `failed` or `timed_out` to the canonical section-outcome
vocabulary.

Markdown renders the validity and dosage statuses independently, including the
stable operation and message for a failure, and continues to show a healthy
sibling's evidence. It prints only dosage classifications actually present in
the selected upstream row. In particular, a missing haploinsufficiency or
triplosensitivity value is not rendered as `No evidence`; an upstream
classification such as `No Evidence for Triplosensitivity` remains data and is
preserved verbatim.

## Success criteria

- A deterministic TP53 provider fixture proves that a dosage response delayed
  beyond its own deadline still returns the completed validity rows with
  `validity_status.status: data`, `dosage_status.status: timed_out`, and the
  `degraded` aggregate mapping above. The inverse case returns the newest
  dosage row when validity fails.
- Fixed tests cover all four family states and exact status object shape,
  operation identifiers, stable messages, and aggregate truth-table rows. A
  client-construction failure produces two `client_init` failures and a zero
  request log.
- A failed and a timed-out HGNC lookup each preserve exact-symbol data but make
  a zero match failed or timed out rather than empty. One shared lookup request
  serves both families.
- Valid zero-match CSVs are `empty`. Separate malformed CSV, HTML-body, missing
  required-header, HTTP error, timeout, and body-limit cases are never `empty`
  and never expose raw failure details. The fixture rejects unexpected routes.
- Timing/request-log tests prove the three operations begin concurrently, each
  deadline is independent, the section settles within one configured deadline
  plus bounded local parsing, and no local ClinGen future or task remains live
  after the parent operation completes or is cancelled.
- Existing validity behavior remains pinned: results are newest review date
  first with the deterministic existing tie-breaks and remain capped at five.
  Existing dosage behavior remains pinned: the newest dated matching dosage
  row wins and its exact classifications are retained.
- A one-sided dosage row renders only the classification supplied. A literal
  upstream `No Evidence for ...` classification remains distinguishable from a
  missing field in Markdown and JSON.
- CLI JSON, CLI Markdown, raw MCP `biomcp` in text and JSON modes, and typed MCP
  `get` for `entity: gene` and section `clingen` all preserve the healthy
  sibling, expose both family statuses, and agree on
  `section_outcomes.clingen` and `_meta.section_sources`. Typed MCP keeps its
  existing request schema; this ticket adds no new tool or section name.
- The public gene/CLI source documentation defines the additive `GeneClinGen`
  schema, both vocabularies, the aggregate mapping, and the distinction between
  missing dosage data and an actual ClinGen no-evidence classification.
- Provider-shaped positive fixtures used to prove ClinGen fields and schema are
  traceable to receipt-backed public captures with request, capture date, hash,
  and any minimization recorded in the source fixture inventory/README.
  Synthetic delay, failure, malformed, and oversize variants are labeled as
  derived test fixtures and are not represented as byte-faithful captures.

## Boundaries

This ticket changes failure isolation and additive result status for existing
ClinGen gene evidence. It does not add GenCC, add clinical actionability, change
ClinGen classifications, expand the five-row validity cap, change date ordering
or newest-dosage selection, create a consensus across sources, or rename/remove
the three existing `GeneClinGen` evidence fields.

Ticket 1158 has no implementation dependency on this ticket: GenCC is a
separate named section with bulk-cache and stale-data semantics, while the
status type and operations defined here are ClinGen-specific. Its `deps` entry
is therefore removed rather than implying a reusable abstraction that this
ticket does not introduce.
