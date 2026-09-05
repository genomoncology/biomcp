# Structural-Variant Article Annotations — Harden

## Decomposition

The optimized implementation was a single 201-line Python file containing detection rules, normalization, occurrence-schema assembly, corpus adaptation, JSONL rendering, argument parsing, and filesystem writes. It is now split into:

- `architecture/experiments/structural-variant-article-annotations/scripts/structural_variant_annotations/parser.py` — reusable detection, normalization, annotation, batch adaptation, and deterministic JSONL rendering.
- `architecture/experiments/structural-variant-article-annotations/scripts/structural_variant_annotations/types.py` — shared occurrence, normalization, provenance, and relationship contracts.
- `architecture/experiments/structural-variant-article-annotations/scripts/structural_variant_annotations/__init__.py` — the supported package facade.
- `architecture/experiments/structural-variant-article-annotations/scripts/structural_events.py` — a 40-line compatibility CLI that only parses paths, reads JSON, calls `render_jsonl`, and writes the result.

The library has no argument parsing, filesystem writes, subprocess calls, or network behavior. Independent regex scans remain deliberate: optimization showed that combining same-type alternatives lost overlapping occurrences. The extraction does not change the experimental boundary: no PMID/disease rules, no event-to-gene lookup table, and no inferred gene relationships.

The generic harden instructions referenced an unavailable cross-repository spike plan. Ticket 515 is therefore authoritative: the consumers are a later BioMCP opt-in article-annotation proof and a downstream alteration-grammar consumer. They need occurrence records over already-retrieved article text. This package gives them an in-process interface; they do not need to shell out or copy parser code.

## Public API

Import from `structural_variant_annotations`:

- `detect(text) -> list[Detection]` — sorted exact Unicode-codepoint detections for scoring and custom adapters.
- `annotate(text, pmid, source="PubMed title/abstract") -> list[StructuralEvent]` — candidate occurrence records with normalized form, loci, copy-number direction, parse status, and provenance.
- `annotate_documents(documents, source="PubMed title/abstract") -> Iterator[DocumentAnnotations]` — streaming batch adapter for article records containing `pmid` and `text`; an optional per-document `source` overrides the default.
- `render_jsonl(documents, source="PubMed title/abstract") -> str` — deterministic JSONL writer used by the CLI.
- Shared contracts: `EventType`, `ParseStatus`, `CopyNumberDirection`, `Detection`, `Document`, `DocumentAnnotations`, `VerbatimSpan`, `NormalizedEvent`, `Provenance`, `GeneRelationship`, and `StructuralEvent`.

`EventType` is the measured eight-value vocabulary: `translocation`, `deletion`, `gain`, `amplification`, `inversion`, `complex_event`, `ploidy_state`, and `free_text_structural_variant`. Gene relationships remain a separate sourced type and `annotate` emits an empty relationship list rather than inferring genes from event notation.

Single-article use:

```python
from structural_variant_annotations import StructuralEvent, annotate

text = "Study title\nThe sample carried t(11;14)."
events: list[StructuralEvent] = annotate(text, pmid="12345678")
assert events[0]["event_type"] == "translocation"
assert events[0]["verbatim"]["text"] == "t(11;14)"
assert events[0]["gene_relationships"] == []
```

Batch use from a downstream spike:

```python
from structural_variant_annotations import annotate_documents

articles = [
    {"pmid": "12345678", "text": "Title\nchromosomal deletion was observed"},
    {"pmid": "87654321", "text": "Control\nPCR amplification was performed"},
]

for article in annotate_documents(articles):
    consume(article["pmid"], article["structural_events"])
```

The second document yields no event because ordinary PCR amplification is a measured lexical trap.

## Build System

This repository is Rust/Python, not Zig, and has no `build.zig`. The equivalent build-system work is `architecture/experiments/structural-variant-article-annotations/pyproject.toml`, which packages `scripts/structural_variant_annotations` as the distribution `biomcp-structural-variant-annotations`. `uv build` successfully produced an sdist and wheel outside the worktree.

A sibling Python spike can add the experiment as an editable path dependency:

```bash
uv add --editable ../biomcp/architecture/experiments/structural-variant-article-annotations
```

Or declare it directly:

```toml
[project]
dependencies = ["biomcp-structural-variant-annotations"]

[tool.uv.sources]
biomcp-structural-variant-annotations = { path = "../biomcp/architecture/experiments/structural-variant-article-annotations", editable = true }
```

It can then use the imports above. The existing JSONL executable remains available for humans, but downstream code should import the package.

## Regression Check

Seven-run end-to-end benchmarks after decomposition met or exceeded the optimization-final medians while preserving byte-identical output:

| Corpus | Optimize final | Harden | Latency | Output SHA-256 | Correctness |
|---|---:|---:|---:|---|---|
| Full scale, 60 docs | 832.54 docs/s | 834.69 docs/s | 1.1981 ms/doc | `c12760adda62c54bb684db6f40a542edeacea6b2538ad8db5578afba49225a94` | 91 TP / 0 FP / 0 FN |
| Regression control, 16 docs | 404.04 docs/s | 406.94 docs/s | 2.4573 ms/doc | `495c5fca2c7c37b411176d0d2bede256667dd48c514dcfc1b7ebfc552c2bcd00` | 88 TP / 0 FP / 0 FN |

The extracted API and wrapper passed all seven isolated contracts, including direct package import, batch annotation, deterministic JSONL, Unicode span round trips, lexical traps, schema validation, and CLI compatibility.

Repository validation is green:

- `make lint`
- `make test`: 2,438 Rust tests passed (28 skipped), 328 Python contracts passed, strict MkDocs build passed
- `make spec`: 123 routine specs passed (3 skipped), 30 surface contracts passed

The quality conclusion is unchanged: the tuned parser is reusable experimental evidence, but its honest frozen blind first pass failed recall and document coverage. This hardening does not promote it into BioMCP production.

## Reusable Assets

- Stable occurrence-level event, span, normalization, provenance, and relationship `TypedDict` contracts.
- The measured eight-value event taxonomy and explicit copy-number-direction vocabulary.
- Exact-span deterministic detector with control-safe contextual families and overlap-preserving semantics.
- Normalizer for notation, chromosomes/loci, plural forms, ploidy, and copy-number direction.
- Single-article and streaming batch annotation functions over local text.
- Deterministic JSONL renderer with the existing byte-level output contract.
- A buildable Python path dependency and thin compatibility CLI pattern.
- Offline corpus/schema validators, seven library/CLI contracts, 60-document benchmark fixture, 16-document regression control, and fixed output checksums.
- A provenance boundary that keeps gene relationships separate and never rewrites event notation into genes.
