# Structural-event article annotation experiment

This experiment measures occurrence-level structural-event extraction from PubMed
title/abstract text. It is isolated from BioMCP production commands and has no runtime
dependency.

## Reproduce

The reviewed corpus and ontology vocabulary are checked in. PubTator is a dated live
baseline snapshot and must be refreshed before evaluation:

```bash
architecture/experiments/structural-variant-article-annotations/scripts/fetch_pubtator.py
architecture/experiments/structural-variant-article-annotations/scripts/evaluate.py
```

To rebuild source text from NCBI (followed by the reviewed span resolution):

```bash
architecture/experiments/structural-variant-article-annotations/scripts/fetch_corpus.py
architecture/experiments/structural-variant-article-annotations/scripts/build_corpus.py
```

Live and intermediate payloads stay in ignored `work/`. `measurements.json` is the
small durable summary; the repository intentionally forbids tracked
`architecture/experiments/**/results/` payloads.

## Corpus and annotation rule

The corpus contains 16 papers: nine positives and seven controls. It includes the three
reported PMIDs, six further positive papers, point-mutation controls, and lexical traps
such as PCR amplification, nuclear/protein translocation, molecular inversion probes,
ordinary chromosome prose, and ordinary economic "losses". Evaluation uses title plus
abstract joined by one newline.

Gold events are minimal, semantically complete mention spans. Repeated mentions are
separate events. Abbreviations such as `SV`/`CNA` are annotated after the document has
introduced their meaning. Gene-pair fusion notation is retained as
`free_text_structural_variant`, demonstrating how unknown event families remain
representable. A prediction is correct only when Unicode-codepoint start, end, and event
type all match.

Gene consequences are not derived from notation. `gold_gene_relationships` links an
event occurrence to genes only where a separate exact evidence span states the
relationship, and gives source/PMID provenance.

## Candidate schema

```json
{
  "event_id": "PMID:start:end",
  "event_type": "translocation | deletion | gain | amplification | inversion | complex_event | ploidy_state | free_text_structural_variant",
  "verbatim": {"text": "t(17;19)(q22;p13)", "start": 123, "end": 142, "offset_unit": "unicode_codepoints"},
  "normalized": {"form": "t(17;19)(q22;p13)", "chromosomes_or_loci": ["17q22", "19p13"], "copy_number_direction": null},
  "parse_status": "complete | partial | ambiguous | verbatim_only",
  "provenance": {"source": "PubMed title/abstract", "pmid": "...", "passage": "title | abstract"},
  "gene_relationships": [{"relation": "produces_fusion", "genes": ["TCF3", "HLF"], "evidence_span": {}, "provenance": {}}]
}
```

Copy-number direction is explicit (`gain`, `amplification`, or `deletion/loss`) rather
than inferred from a locus alone. Ambiguous notation and karyotype strings retain the
full verbatim span with `partial`, `ambiguous`, or `verbatim_only`; parsed child events
may be attached later without discarding the parent evidence.
