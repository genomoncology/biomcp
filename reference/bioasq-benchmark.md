# BioASQ Benchmark

BioMCP treats BioASQ as an offline benchmark input, not as a live runtime
source. The benchmark module under `benchmarks/bioasq/` exists so the repo can
ingest public benchmark artifacts, preserve provenance, and document the
official competition workflow without collapsing those two lanes together.

## Two lanes

BioMCP keeps a **public historical benchmark lane** and an
**official competition lane** on purpose.

- The public historical benchmark lane supports internal regression tracking,
  prompt iteration, and longitudinal product measurement.
- The official competition lane supports registered Task B participation and
  external leaderboard claims.

These lanes do not carry the same evidence value. Public availability does not
turn a mirror into an official participant download.

## Public historical lane

The public historical lane currently has two manifest-defined bundles:

| Bundle id | Packaging | Purpose | Notes |
|---|---|---|---|
| `hf-public-pre2026` | `public_mirror` | Recommended regression bundle | Pinned to `jmhb/BioASQ` revision `8eb56db5f3f43ce7c4102169b24158ad2dc53a74`; deduped output count is `5399` |
| `mirage-yesno-2024` | `public_derived_benchmark` | Supplemental yes/no slice | Pinned to the MIRAGE commit URL and normalizes `618` yes/no records |

Generate the public bundles with:

```bash
uv run --script benchmarks/bioasq/ingest_public.py --bundle hf-public-pre2026
uv run --script benchmarks/bioasq/ingest_public.py --bundle mirage-yesno-2024
```

The ingester writes raw source exports under `benchmarks/bioasq/datasets/raw/`
and canonical JSONL output under `benchmarks/bioasq/datasets/normalized/`.

## Recommended bundle

`hf-public-pre2026` is the recommended public bundle for longitudinal BioMCP
benchmarking. It has the broadest public coverage and produces a stable
normalized JSONL export.

The repo keeps the public versus official count mismatch explicit:

- `hf-public-pre2026` normalizes to `5399` public mirror-derived questions
- `official-task-b-participant-download` records `5389` questions from the
  official participant download reviewed on `2026-03-25`

That mismatch is provenance to surface, not a bug to hide.

## Ranking calibration

Article-ranking calibration lives in the repo-local guide at
`benchmarks/bioasq/ranking-calibration/README.md`. That guide records the
worked-example fixture surface used for future ranking tuning, points back to
the committed provenance files, and records the lexical-regression,
hybrid-default, entity-only fallback, and weight-override proofs that gate
ranking changes.

This is not a new benchmark lane. The calibration surface stays in Rust tests
and benchmark docs because live ranking-order assertions against upstream
article responses are unstable.

## Skill Prompt Integration

eval runners should call `biomcp skill render` immediately before building the
system prompt for a measurement run. That command is the canonical
agent-facing BioMCP prompt surface and avoids pinning a stale `SKILL.md`
snapshot in benchmark harnesses.

Snapshot-based eval runners should refresh their snapshot before measurement
with:

```bash
biomcp skill render > <snapshot-path>
```

This repository contains BioASQ data preparation and benchmark documentation,
but it does not contain the external eval CLI startup path. BioMCP therefore
ships the stable render command and documents the startup/snapshot contract; it
does not claim that external eval startup code was switched here.

## Provenance and terms

The public bundle metadata lives in `benchmarks/bioasq/datasets/manifest.json`.
Each normalized record carries a `provenance` object with lane, source,
packaging, pinned source ref, and source record id.

Terms and source boundaries:

- The HF mirror bundle uses `jmhb/BioASQ` with source packaging
  `public_mirror`. The HF card references BioASQ participation terms at
  <https://bioasq.org/participate>.
- The MIRAGE slice uses a commit-pinned public benchmark URL with source
  packaging `public_derived_benchmark`.
- Neither public artifact is labeled as `official_download`.

## Validity overlay

Future stale or invalid question review belongs in the validity overlay, not in
the raw corpus. The module ships:

- `benchmarks/bioasq/annotations/validity.schema.json`
- `benchmarks/bioasq/annotations/validity.jsonl`

Join records with `question_id` plus `bundle_id`. That keeps future review
layered on top of the normalized bundles without rewriting the source material.

## Official competition lane

The official competition lane is documented from the public participants-area
page reviewed on `2026-03-25`:
<https://participants-area.bioasq.org/general_information/Task14b/>

Operational points for Task B:

- Registered users can download the `5389`-question development dataset from
  the participant area.
- Teams must declare their systems in **Edit Profile** before uploading runs.
- The test set is released in four batches.
- **Phase A** and **Phase A+** submissions are due within `24 hours` of each
  batch's question release.
- **Phase B** submissions are due within `24 hours` of the gold
  article/snippet release for that batch.

What a BioMCP system submission would look like:

1. Register the team and system in the participants area.
2. Download the official development set from the participant portal.
3. Run the Task B answer-generation stack on each released batch.
4. Package the batch answers in the official submission format and upload them
   inside the task window.

Official results support competition and leaderboard claims. They do not
replace the public historical benchmark lane for product regression tracking.

## Evidence value matrix

| Lane | Supports | Does not support |
|---|---|---|
| Public historical benchmark lane | Internal regression tracking, prompt comparisons, repeatable product measurement | Official leaderboard claims or claims that the bundle is an official participant archive |
| Official competition lane | Registered Task B submissions, leaderboard placement, external claims tied to BioASQ participation | A stable always-available public regression corpus for every operator |

## Grounding references

- Repo-local artifacts: `benchmarks/bioasq/ingest_public.py`,
  `benchmarks/bioasq/datasets/manifest.json`,
  `benchmarks/bioasq/annotations/validity.schema.json`
- HF mirror terms link: <https://bioasq.org/participate>
- MIRAGE pinned source:
  <https://raw.githubusercontent.com/gzxiong/MIRAGE/3490d7b5b5fcb96288860ec74d18c3e398a56703/benchmark.json>
- Official Task B lane:
  <https://participants-area.bioasq.org/general_information/Task14b/>
