# Structural-Variant Annotation Layer for Biomedical Articles

> **Exploit correction:** This historical report and `measurements.json` describe the
> original 87-event gold. Exploit found that PMID 35637217 omitted a repeated capitalized
> `Chromothripsis` mention at offsets 1047:1061. The checked-in corpus now has 88 events;
> see `exploit.md` for the corrected 88/0/0 control and correctness carveout.

## Spike Question

Can BioMCP add a general structural-event annotation layer for biomedical articles that materially closes PubTator's cytogenetic-event gap without disease-specific answer tables or event-to-gene inference masquerading as extraction?

The pre-registered promotion bar was micro precision >= 0.90, micro recall >= 0.80, positive-document coverage >= 0.85, zero false positives in the control set, and nonzero recall for every represented event type.

## Prior Art Summary

BioMCP already exports PMID BioC JSON through `src/sources/pubtator.rs`. The source payload can contain locations, but the production deserializer retains annotation text/type/identifier only. `src/transform/article/annotations.rs` then case-folds and counts gene, disease, chemical, and mutation mentions for a compact display bucket. That is appropriate for entity summaries but cannot represent occurrence-level event evidence, exact spans, copy-number direction, ambiguous parses, or sourced event-to-gene relationships.

The experiment reuses the PubTator source boundary and provenance model. It deliberately does not add structural events to the existing mutation count bucket and does not infer genes from cytogenetic shorthand.

## Corpus and Measurement

The checked-in corpus has 16 PMID title/abstract documents: nine positives and seven controls, 87 exact-span events, all eight requested event types, and six separately sourced gene relationships. It includes PMIDs 30709865, 35637217, and 37449980 plus six additional positive papers. Controls include point mutations, PCR amplification, nuclear/protein translocation, molecular inversion probes, and ordinary chromosome/gene prose.

Gold records include Unicode-codepoint start/end offsets, verbatim text, normalized form, event type, stated chromosome/locus, and separate provenance-bearing gene relationships. Exact start, end, and event type must all match. The durable result is `measurements.json`; live payloads remain ignored under `work/`.

## Approaches Tried

### 1. Current PubTator baseline

Fetched PubTator BioC JSON for each PMID on 2026-07-14 and accepted only mutation/variant-class annotations as candidate events. Ten documents were available; six very recent papers returned HTTP 400. Available documents had gene, disease, chromosome, chemical, species, and cell-line annotations, but no structural-event annotation class. In particular, PMID 35637217 still exposed RB1/TP53 but not its translocations/complex events, and PMID 37449980 exposed chromosome mentions but not copy-number/structural events.

Result: 0/87 events, 0/9 positive-document coverage, no false positives. This confirms an ontology/NER gap rather than a rendering-only gap.

### 2. Conservative deterministic parser

A generic parser recognizes cytogenetic notation and bounded contextual phrases. It normalizes syntax only. There are no PMIDs, diseases, event-to-gene tables, or disease answers in the rules. Ambiguous words are context-bound: bare `amplification`, `translocation`, `inversion`, and `losses` are not accepted, preventing the control traps.

Result: 87 TP, 0 FP, 0 FN; micro and macro precision/recall/F1 1.000; 9/9 positive-document coverage; 0 false positives in seven controls.

Per event type:

| Type | Gold | Precision | Recall | F1 |
|---|---:|---:|---:|---:|
| amplification | 8 | 1.000 | 1.000 | 1.000 |
| complex_event | 17 | 1.000 | 1.000 | 1.000 |
| deletion | 7 | 1.000 | 1.000 | 1.000 |
| free_text_structural_variant | 34 | 1.000 | 1.000 | 1.000 |
| gain | 8 | 1.000 | 1.000 | 1.000 |
| inversion | 5 | 1.000 | 1.000 | 1.000 |
| ploidy_state | 2 | 1.000 | 1.000 | 1.000 |
| translocation | 6 | 1.000 | 1.000 | 1.000 |

This is a development-corpus result, not a claim of general-world perfection. The corpus review informed rule boundaries; a blind held-out set is the main exploit risk.

### 3. Ontology-grounded lexical matcher

A compact vocabulary was retrieved through EMBL-EBI OLS4 from Sequence Ontology and NCI Thesaurus. It includes identifiers such as SO:0001537 (structural variant), SO:1000044 (chromosomal translocation), SO:1000029 (chromosomal deletion), SO:1000030 (chromosomal inversion), SO:0002062 (complex chromosomal rearrangement), SO:0001742 (copy-number gain), NCIT:C80336 (hyperdiploidy), and NCIT:C129355 (chromothripsis).

Result: 18 TP, 26 FP, 69 FN; micro precision 0.409, recall 0.207, F1 0.275; 5/9 coverage; six false positives in controls. It missed notation such as `t(4;14)` and produced exact-span/type errors by matching bare `gain`, `deletion`, and `inversion`. Concrete control errors included `PCR amplification`, `NF-κB nuclear translocation`, `protein translocation`, and `molecular inversion probes`. Ontology identity is useful normalization metadata after detection, but lexical matching alone is not viable detection.

### 4. Parser plus ontology union

The deduplicated union tested whether ontology prose recall complements notation parsing.

Result: 87 TP, 26 FP, 0 FN; precision 0.770, recall 1.000, F1 0.870; 9/9 coverage; six control false positives. The union added no true positives beyond the parser and inherited every ontology ambiguity. It fails the quality bar.

## Decision

The deterministic parser wins and clears the pre-registered bar on this small corpus. Promote only that bounded approach; do not union raw ontology lexical matches into detection. Ontology IDs may be attached after a parser has established event context.

The candidate production concept is `ArticleStructuralEvent`:

- `event_id`
- `event_type`: `translocation | deletion | gain | amplification | inversion | complex_event | ploidy_state | free_text_structural_variant`
- `verbatim`: text, exact start/end, offset unit
- `normalized`: canonical form, chromosomes/loci, explicit copy-number direction
- `parse_status`: `complete | partial | ambiguous | verbatim_only`
- source provenance: provider, PMID, passage
- `gene_relationships`: relation, target genes, independent evidence span, independent provenance

Unknown events remain `free_text_structural_variant` with verbatim evidence; they are never dropped or forced into a known class. A fusion-like gene pair may be represented this way until a shared schema gains a dedicated fusion type.

### Bounded follow-up build ticket

Add an opt-in article structural-events section over PubMed title/abstract text, separate from current PubTator annotation counts. Implement only the measured notation/context families. Do not add disease mappings, inferred event-to-gene links, full-text extraction, or a general ISCN parser.

Acceptance thresholds on a new blind corpus are:

- at least 40 positive and 20 control papers, including at least five gold events per supported type;
- exact-span/type micro precision >= 0.95 and recall >= 0.85;
- per-type recall >= 0.80 where at least five examples exist;
- positive-document coverage >= 0.90;
- zero false positives in the 20 lexical-trap controls;
- every relationship has a separate evidence span and source provenance;
- `make lint`, `make test`, and `make spec` green with no live downloads in gates.

If these blind thresholds fail, keep the feature experimental and narrow the supported families rather than adding disease-specific exceptions.

## Outcome

**promote** — to the bounded, opt-in exploit above, contingent on blind-corpus acceptance.

## Risks for Exploit

- **Development-set optimism:** rules and span policy were refined together on only 16 abstracts. The perfect parser score must be challenged blind.
- **Abstract-only evidence:** full text, tables, figure captions, supplements, and OCR were not measured.
- **Ambiguous notation:** `+1q`, `-7`, `amp`, slash-separated loci, and abbreviations can be arithmetic, dosage, or prose. Preserve verbatim and mark ambiguous unless chromosome context is sufficient.
- **Karyotype strings:** full ISCN strings were not measured. Store the parent string as `verbatim_only`/`partial`; do not emit confident child events until a dedicated ISCN corpus validates them.
- **Copy-number direction:** direction must come from explicit gain/amplification/deletion/loss syntax. A bare chromosome/locus or generic CNA must not acquire direction.
- **Gene provenance:** `t(11;14)` must not silently become CCND1. Gene relationships require their own quoted span and provider/PMID provenance.
- **Offsets and text versions:** PubTator, PubMed, and Europe PMC may normalize whitespace or Unicode differently. Keep offset unit and exact source text/version together.
- **Acronym drift:** `SV`/`CNA` should require local definition or strong article context outside the measured corpus.
- **Ontology scope:** ontology labels help normalization but their broad lexical forms are unsafe as detectors.
- **Schema alignment:** Trials3/Nucleus alteration grammar is not yet available in this repo; obtain cross-team review before freezing public JSON names.
