# Unify variant transcript numbering in lookup and display

Status: OPEN.

Milestone: 0.9.4

Priority note 2026-10-19: moved into 0.9.4 because every known bug ships fixed. MANE preference and naming the matched numbering already landed; the remaining scope is lookup by either transcript cDNA numbering and a not-found answer that says the identifier belongs to a different transcript. Follow sdlc/decisions/2026-10-18-transcript-aliases-need-the-same-genomic-change.md, and route clinical questions to a clinical reviewer.

From issue `2026-09-11-variant-display-mixes-transcript-numbering.md` (a real clinical-report failure: a report's `c.1187G>A` finds no match because the display mixes protein numbering from one transcript with cDNA numbering from another; the not-found reads as "nothing to see" instead of "wrong transcript"). The 2026-09-27 review ordered a real ticket with an owner and trigger, not a parked line.

## Problem

Variant output names a protein consequence and a cDNA change whose numbering can come from different transcripts (MUTYH c.1187G>A: p.Gly396Asp on the common transcript, p.Gly382Asp as legacy numbering on another). A consumer matching a report identifier against that output fails, and the failure mode is silent.

## Scope

1. Every variant rendering that pairs cDNA and protein numbering names the transcript each comes from, once, plainly. 2. The variant lookup path accepts either transcript's cDNA numbering as a key and answers on the transcript it matched, saying so — the not-found case distinguishes "not in the source" from "that identifier belongs to a different transcript". 3. Curated transcript rules (which transcript is primary per gene) are data, recorded with provenance, not per-callsite choices.

## Acceptance

- The MUTYH c.1187G>A case from the issue: lookup by either numbering finds ClinVar 5294 and the output names its transcript.
- A test with two transcript numberings of one variant proves both keys resolve and the display labels its transcripts.
- The transcript-priority data records its source and date.

## Retained behavior

Single-transcript output is unchanged. The ClinVar Variation ID and rsID lookups keep their current resolution. No existing identifier stops resolving; the change only adds alternate keys and names the transcript in use.

## Code paths

- `src/entities/variant/` — the entity's parse and normalization sites, including the ClinVar and MyVariant payload readers that hold per-transcript numbering today.
- `src/cli/variant/dispatch.rs`, `src/cli/variant/normalization_json.rs` and `src/transform/variant.rs` — where the cDNA/protein pair renders without naming its transcripts.
- The variant lookup argument parser in `src/cli/variant/dispatch.rs` (argument handling), `src/cli/variant/normalization_json.rs` (the JSON payload builder) and `src/transform/variant.rs` (transcript mapping) — where an alternate numbering must be accepted as a key.

## Proof

The acceptance tests are the proof: both transcript numberings of the MUTYH case resolve to ClinVar 5294 with the transcript named, and a two-numbering fixture proves both keys work and the display labels its transcripts.

## Deferred gaps

The curated transcript-priority table (which transcript is primary per gene, with source and date) does not exist yet; sourcing it is the first implementation step, not an afterthought. Prior evidence: two independent agents hit the same failure on MUTYH c.1187G>A during a molecular-tumor-board experiment (2026-09-11) — the report's `c.1187G>A` found no match because the display mixed protein numbering from one transcript with cDNA numbering from another (p.Gly396Asp on the common transcript, p.Gly382Asp as legacy numbering); the variant is ClinVar 5294, rs36053993, Pathogenic/Likely pathogenic. A not-found result read as "nothing to see" rather than "wrong transcript." Anything else deferred during implementation gets its own issue file with an owner and a revisit trigger, per house rule.

## Owner and trigger

Owner: the biomcp queue (developer agents; no clinical judgement needed to render provenance already present in the data). Revisit trigger: any new molecular-tumor-board experiment that matches report identifiers against BioMCP output, or the start of the 1.0 feature track, whichever comes first. Ian can re-prioritize at any time.

## Coordinator study brief

Restated remaining outcome: look up an allele through either source-backed transcript cDNA representation, while distinguishing another reported transcript from an unverified or ordinary not-found input. MANE/display work already landed. Before any product edit, reproduce current-main lookup and propose exact files plus an outside-in failing case; if the remaining problem does not reproduce, report that for withdrawal rather than invent work.

Clinical review 2026-10-19 confirmed the existing same-genomic-change decision. Use explicit transcript versions and authoritative DNA-level anchors; gene/protein/token similarity alone is insufficient. ClinVar5294 lists NM_001128425.2:c.1187G>A and current MANE Select NM_001048174.2:c.1103G>A. Archived RCV000079501.9 also lists NM_001048171.1:c.1145G>A with the same NC_000001.11:g.45331556C>T and NC_000001.10:g.45797228C>T. Do not substitute NM_001048171.2 for version1, or call c.1187G>A MANE Select. Sources: https://www.ncbi.nlm.nih.gov/clinvar/RCV000079501.9/?cmd=HTOn and https://www.ncbi.nlm.nih.gov/clinvar/variation/5294/. The current record supplies canonical SPDI NC_000001.11:45331555:C:T and ClinGenCA011561. These are a source-backed case, not proof that current lookup passes or that a new ClinGen reply was captured.

Use the three exact versioned inputs as identity/control cases. Require matching transcript provenance and the same allele only when the recorded DNA anchor proves it. With normalizers unavailable, another transcript's spelling alone must not establish identity. Say an identifier belongs to another transcript only when source evidence establishes that fact; otherwise say the requested transcript remains unverified. Existing BRCA1 recorded paper-anchor responses CA002402 and alternate-allele CA10594475 can support controls but do not prove lookup behavior. All execution is through the configured build job. Preserve raw exact commands/responses in a numbered build-host experiment. Read-only study first; no source claim is granted yet.
