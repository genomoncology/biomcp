# From BRAF V600E to 3D Protein Context in 3 BioMCP Commands

*BioMCP can take a familiar linear variant name, resolve it to a canonical record, join residue-level protein structure context, and then pivot straight into the literature.*

**TL;DR:** `biomcp variant structure "BRAF V600E"` turns a plain variant phrase into residue, InterPro domain, PDB, AlphaFold, and Cancerhotspots context. In one run, BioMCP maps BRAF V600E to residue 600 of UniProt P15056, shows that the residue sits inside the BRAF kinase domain, lists structures that cover the residue, links the AlphaFold model, reports Cancerhotspots recurrence, and suggests the article follow-up command.

---

## Why this workflow matters

Variant names usually start as linear text: a report says **BRAF V600E**. Structural questions are different: where is residue 600 in the protein, which domain contains it, are there experimental structures covering it, and is the position recurrent in tumors?

BioMCP keeps that as a short command sequence instead of a manual handoff across variant lookup, UniProt, InterPro, PDB, AlphaFold, Cancerhotspots, and PubMed-style article search.

---

## Step 1: Resolve the variant

```bash
$ biomcp get variant "BRAF V600E"
```

```
# BRAF p.V600E (rs113488022)

rsID: rs113488022
ID: chr7:g.140453136A>T
Protein: p.V600E
Legacy Name: BRAF V640E
cDNA: c.620T>A
Consequence: missense_variant
COSMIC: COSM476
Significance: Pathogenic
Source: MyVariant.info / ClinVar
## Population (gnomAD via MyVariant.info)
gnomAD AF: 0.000004 (< 0.01%)
## Predictions (MyVariant.info)
- CADD: 32.0
- PolyPhen: Probably damaging
More:
  biomcp get variant "chr7:g.140453136A>T" predict   - additional detail
  biomcp get variant "chr7:g.140453136A>T" predictions   - additional detail
  biomcp get variant "chr7:g.140453136A>T" clinvar   - additional detail
All:
  biomcp get variant "chr7:g.140453136A>T" all
See also:
  biomcp get gene BRAF
  biomcp search drug --target BRAF
  biomcp variant trials "chr7:g.140453136A>T"
  biomcp variant articles "chr7:g.140453136A>T"
```

The first call does the identity work. It resolves the user-facing phrase to **BRAF p.V600E**, rs113488022, genomic coordinate `chr7:g.140453136A>T`, COSMIC `COSM476`, and a pathogenic ClinVar significance. That gives the structure command a precise anchor.

---

## Step 2: Join residue, domain, structure, and hotspot context

```bash
$ biomcp variant structure "BRAF V600E"
```

```
# Variant structure: BRAF V600E

Gene: BRAF
Residue: 600
Position confidence: requested_hgvsp_exact_match

## Protein

Accession: P15056
Entry: BRAF_HUMAN
Length: 766

## Domains (InterPro)

- Protein kinase domain (IPR000719) 457-717
- Serine-threonine/tyrosine-protein kinase, catalytic domain (IPR001245) 457-712
- Protein kinase-like domain superfamily (IPR011009) 449-717
- Serine/Threonine Kinases and Pseudokinases (IPR051681) 338-717

## Structures (PDB / AlphaFold)

- PDB 1UWH (covers residue)
- PDB 1UWJ (covers residue)
- PDB 2FB8 (covers residue)
- PDB 2L05 (does not cover residue)
- PDB 3C4C (covers residue)
- PDB 3D4Q (covers residue)
- PDB 3IDP (covers residue)
- PDB 3II5 (covers residue)
- PDB 3NY5 (does not cover residue)
- PDB 3OG7 (covers residue)
- AlphaFold: https://alphafold.ebi.ac.uk/entry/P15056

## Cancerhotspots

Source: cancerhotspots.org
Position count: 897
Same amino-acid count: 833

## Warnings

- MyVariant.info returned additional transcript/isoform protein positions; mapped domain/structure context uses the requested HGVSp position.

See also:
  biomcp get protein P15056 structures
  biomcp get protein P15056 domains
  biomcp variant articles "BRAF V600E"
```

This is the main payoff. BioMCP selects residue **600** with `requested_hgvsp_exact_match` confidence, maps the protein to UniProt **P15056 / BRAF_HUMAN**, and shows that V600 sits in overlapping InterPro kinase-domain annotations.

The structure section gives both experimental and predicted context. Several PDB entries cover residue 600, and the AlphaFold link points to the full BRAF model. The Cancerhotspots section reports recurrence at the same position and amino-acid change, which is the bridge from structure to cancer relevance.

The warning is useful too: upstream variant annotations include additional transcript or isoform positions, so BioMCP tells you that the structure join used the requested HGVSp position.

For the reference form of this workflow, see [Annotate variant structure context](../how-to/annotate-variant-structure.md).

---

## Step 3: Pivot to papers

```bash
$ biomcp variant articles "BRAF V600E"
```

```
# Articles: variant=BRAF V600E

Found 10 articles


> PubTator variant annotation recall


Semantic Scholar: enabled

Ranking: calibrated PubMed rescue + lexical directness (top-ranked weak PubMed unique/led rows with at least one anchor hit > title coverage > title+abstract coverage > study/review cue > citation support > source-local position)


| PMID | Title | Source(s) | Date | Why | Cit. |
|---|---|---|---|---|---|
|35647194|Risk and Prognostic Factors for BRAFV600E Mutations in Papil…|PubTator3|2022-05-18|-|61|
|34851044|BRAFV600E mutation test on fine-needle aspiration specimens…|PubTator3|2022-01-01|-|28|
|36276003|Novel thiazole derivatives incorporating phenyl sulphonyl mo…|PubTator3|2022-09-27|-|16|
|38607456|Genetic alterations and allele frequency of BRAF V600E and T…|PubTator3|2024-04-12|-|13|
|35053476|Nuclear Localization of BRAFV600E Is Associated with HMOX-1…|PubTator3|2022-01-09|-|11|
|35047385|Diagnostic Efficacy of Ultrasound, Cytology, and BRAFV600E M…|PubTator3|2022-01-03|-|8|
|33980169|BRAFnon-V600E more frequently co-occurs with IDH1/2 mutation…|PubTator3|2021-05-12|-|7|
|38282867|Retrospective study of BRAFV600E mutation and CT features of…|PubTator3|2024-01-24|-|4|
|39979881|Extracellular vesicle-mediated gene therapy targets BRAFV600…|PubTator3|2025-02-20|-|4|
|40703409|Up-regulation of NGEF via the BRAFV600E /ERK/AP1 pathway enh…|PubTator3|2025-07-17|-|0|



Use `get article <pmid>` for details.
```

The article pivot keeps the same variant anchor and returns papers with PubTator variant annotation recall. From here an agent can inspect a PMID, add disease or date filters, or switch from structure context to therapeutic evidence.

---

## Try it

Run the same workflow locally:

```bash
biomcp get variant "BRAF V600E"
biomcp variant structure "BRAF V600E"
biomcp variant articles "BRAF V600E"
```

Use JSON when you want to hand the structure context to another tool:

```bash
biomcp --json variant structure "BRAF V600E"
```

The structure helper is intentionally opt-in because it joins several network sources. For everyday variant annotation, keep using `biomcp get variant`; reach for `variant structure` when the residue, domain, PDB, AlphaFold, or Cancerhotspots context is the question.
