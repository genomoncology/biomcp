# Explore: Variant to protein-structure annotation

## Spike Question

Can BioMCP reliably connect a variant to protein-structure context — residue, overlapping domain, PDB IDs, AlphaFold link, and Cancerhotspots residue recurrence — using existing read-only sources, without adding ΔΔG/stability computation?

Small scale: BRAF V600E, TP53 R175H, and ROS1 G2032R.

## Prior Art Summary

Existing BioMCP already has most source pieces, but not the joined object.

- Variant lookup (`src/entities/variant/get.rs`, `src/sources/myvariant.rs`) resolves exact gene+protein inputs such as `BRAF V600E` through MyVariant.info and stores `hgvs_p`, `hgvs_c`, rsID, ClinVar, population, prediction, conservation, and other source-backed fields.
- Variant parsing (`src/entities/variant/resolution.rs`) deliberately rejects ambiguous residue-only inputs for `get variant`; this is the right safety model to reuse.
- Cancerhotspots recurrence is already exposed under `get variant ... all` for exact gene+protein inputs with `source`, `matched_transcript`, `position_count`, and `same_aa_count`.
- Protein lookup (`src/entities/protein.rs`, `src/sources/uniprot.rs`, `src/sources/interpro.rs`) already resolves UniProt accessions, extracts PDB/AlphaFold IDs from UniProt cross-references, and retrieves InterPro domain rows.
- Current BioMCP protein domains expose accession/name/type only. InterPro's live response includes residue ranges, but BioMCP does not currently parse or serialize those ranges.
- Current BioMCP structures expose IDs and summary strings; UniProt cross-references often include chain residue coverage strings such as `A/B=448-723`.

Design to reuse:
- Keep structure work opt-in, never in default `get variant`.
- Reuse exact gene+protein variant parsing and MyVariant lookup.
- Reuse UniProt for PDB/AlphaFold IDs and InterPro for domains.
- Keep transcript/isoform honesty explicit; do not silently guess when source positions disagree.

## Approaches Tried

Scripts and results are committed under `architecture/experiments/variant-structure-annotation/`.

### 1. Existing BioMCP CLI composition

What: Call existing surfaces only:

- `biomcp --json --no-cache get variant "<gene change>" all`
- `biomcp --json --no-cache get protein <accession> structures`
- `biomcp --json --no-cache get protein <accession> domains`

Measurements: `results/existing_cli_composition_summary.json`

| Variant | Variant OK | Structures OK | Domains OK | Domain ranges exposed | Total latency |
|---|---:|---:|---:|---:|---:|
| BRAF V600E | yes | yes | yes | no | 25.9s |
| TP53 R175H | yes | yes | yes | no | 11.2s |
| ROS1 G2032R | yes | yes | yes | no | 21.8s |

Finding: Existing public surfaces prove the data families are reachable, but they do not expose enough domain range data to answer residue-overlaps-domain. This approach would force agents to do an unsafe manual join.

### 2. Thin direct-source join

What: Minimal Python implementation joining MyVariant.info, UniProt, InterPro, and existing BioMCP Cancerhotspots output. It parses InterPro residue ranges from the live response.

Measurements: `results/direct_source_join_summary.json` and `results/direct_source_join.json`

| Variant | MyVariant residue | UniProt / structures | Overlapping InterPro domains | Cancerhotspots | Total latency |
|---|---:|---:|---:|---:|---:|
| BRAF V600E | 600 | P15056; 131 PDB; AlphaFold P15056 | 4 | position 897, same-aa 833 | 15.6s |
| TP53 R175H | 175 | P04637; 295 PDB; AlphaFold P04637 | 4 | position 416, same-aa 386 | 17.5s |
| ROS1 G2032R | 2032 | P08922; 5 PDB; AlphaFold P08922 | 5 | no matched hotspot counts | 20.0s |

Important transcript/isoform finding: MyVariant.info frequently returns multiple protein positions for one genomic variant because dbNSFP aggregates transcripts/isoforms.

- BRAF V600E returned positions 207, 600, and 640.
- TP53 R175H returned positions 16, 43, 82, 136, 164, and 175.
- ROS1 G2032R returned positions 2026 and 2032.

The requested gene+protein change still matched the expected canonical position in all three cases. A build must therefore preserve source-returned `hgvsp` values and mark whether the requested position was an exact match, rather than treating the first returned protein position as canonical.

Finding: This approach answers the spike question. Residue/domain/structure context is reachable for all three test variants if BioMCP adds a small InterPro range parser and a joined variant-structure model.

### 3. Thin join plus structure reference links / RCSB probe

What: Same as approach 2, plus AlphaFold/RCSB reference links and a small RCSB accession probe for up to five PDB IDs.

Measurements: `results/direct_source_join_with_structure_links_summary.json` and `results/direct_source_join_with_structure_links.json`

Result:
- AlphaFold links are deterministic from UniProt accession, e.g. `https://alphafold.ebi.ac.uk/entry/P15056`.
- RCSB links are likewise deterministic by PDB ID or accession search.
- RCSB probes confirmed most sampled PDB IDs map back to the UniProt accession, but this did not add enough value for the first contract.
- UniProt PDB cross-reference chain ranges are enough to show likely residue coverage for many structures without adding a new required provider.

Finding: Do not add a hard RCSB dependency in the first build. Add provider links from UniProt IDs, and optionally parse UniProt chain range strings. Treat RCSB as a later enhancement if users need per-chain residue coordinate mapping.

## Decision

**Winner: approach 2 — thin direct-source join using existing federation, with deterministic AlphaFold/PDB links from UniProt IDs.**

Why:
- It reached residue, domain range, PDB IDs, AlphaFold ID, and Cancerhotspots recurrence for the small-scale set.
- It stays inside BioMCP's read-only federation boundary.
- It requires modest production changes: add InterPro location parsing, add a variant structure join/model, add renderer/docs/specs.
- It avoids a new RCSB dependency for the first contract.
- It makes the transcript/isoform risk explicit instead of pretending the first MyVariant protein position is safe.

Recommended opt-in CLI surface:

```bash
biomcp variant structure "BRAF V600E"
biomcp --json variant structure "BRAF V600E"
```

Recommended behavior:
- Accept exact variant inputs BioMCP already supports for `get variant`: rsID, genomic HGVS, transcript HGVS after normalization, and gene+protein change.
- For ambiguous residue-only shorthand, return the existing guidance style and suggest `search variant` first.
- Resolve the variant, extract requested/selected protein residue, resolve UniProt accession from gene, then fetch InterPro domains and UniProt structure IDs.
- Keep this as a helper pivot, not a default `get variant` section. A future non-default `get variant ... structure` section can wrap the same service if needed, but the helper is the clearer first public contract.

Proposed JSON contract fields:

```json
{
  "variant": "BRAF V600E",
  "gene": "BRAF",
  "input_kind": "gene_protein_change",
  "residue": {
    "requested_change": "V600E",
    "position": 600,
    "reference_aa": "V",
    "alternate_aa": "E",
    "source": "MyVariant.info/dbNSFP",
    "matched_hgvsp": ["p.Val600Glu", "p.V600E"],
    "other_source_positions": [207, 640],
    "position_confidence": "requested_hgvsp_exact_match"
  },
  "protein": {
    "accession": "P15056",
    "entry": "BRAF_HUMAN",
    "length": 766,
    "source": "UniProt"
  },
  "domains": [
    {
      "accession": "IPR000719",
      "name": "Protein kinase domain",
      "type": "domain",
      "start": 457,
      "end": 717,
      "source": "InterPro"
    }
  ],
  "structures": {
    "pdb": [
      {
        "id": "1UWH",
        "method": "X-ray",
        "resolution": "2.95 A",
        "chains": "A/B=448-723",
        "residue_covered": true,
        "source": "UniProt cross-reference"
      }
    ],
    "alphafold": {
      "id": "P15056",
      "url": "https://alphafold.ebi.ac.uk/entry/P15056",
      "source": "UniProt cross-reference / AlphaFold DB"
    }
  },
  "cancerhotspots": {
    "source": "cancerhotspots.org",
    "matched_transcript": "ENST00000288602",
    "position_count": 897,
    "same_aa_count": 833
  },
  "warnings": [
    "MyVariant.info returned additional transcript/isoform protein positions; mapped domain/structure context uses the requested HGVSp position."
  ],
  "_meta": {
    "next_commands": [
      "biomcp get protein P15056 structures",
      "biomcp get protein P15056 domains",
      "biomcp variant articles \"BRAF V600E\""
    ]
  }
}
```

Graceful degradation:
- If no exact protein position can be selected, return partial variant/protein context with `position_confidence: "unresolved"`, no overlapping domains, and a warning.
- If InterPro is unavailable, return residue/protein/structures plus source-status warning.
- If UniProt has no PDB IDs, still return AlphaFold when present.
- If Cancerhotspots has no matched row, return source-labelled null counts when the source answered, matching the current behavior.
- If any remote source fails, return the successful source data and source-status warnings; do not fabricate zeros.

ΔΔG/stability-effect computation is explicitly scoped **out** because it would cross BioMCP's read-only annotation boundary and require heavyweight modeling dependencies better owned by a downstream structural pipeline.

## Outcome

**promote**

Create a follow-on build ticket for `biomcp variant structure <variant>`.

Files likely to touch:
- `src/entities/variant*` — add structure helper orchestration and output model.
- `src/sources/interpro.rs` — parse location fragments/ranges.
- `src/sources/uniprot.rs` — expose typed PDB/AlphaFold structure rows, including chain coverage where available.
- `src/render/markdown/variant*` — render variant structure helper output.
- `src/render/json.rs` or JSON helper layer — include `_meta.next_commands` and source-status metadata.
- CLI command wiring under `src/cli/*` for `variant structure`.
- Docs: `docs/user-guide/cli-reference.md`, `architecture/ux/cli-reference.md`, and the variant user guide.
- Specs: `spec/entity/variant.md` or `spec/entity/variant-hotspots.md` depending on where the behavioral contract fits best.

Required spec/test layers:
- Request-contract test for the new command and JSON shape.
- Renderer unit test for markdown and JSON `_meta.next_commands`.
- Real behavioral check on the operator / real-local lane using live sources; do not fake the remote source path for the canary.

## Risks for Exploit

- MyVariant/dbNSFP returns multiple transcript/isoform positions for common variants. The build must match the requested HGVSp when possible and expose mismatch warnings.
- InterPro locations are available in live responses but not currently represented in BioMCP models; parser tests need fixtures.
- UniProt PDB chain coverage strings are compact text. Parsing `A/B=448-723` is easy, but edge cases may exist.
- RCSB residue-level mapping is useful but not needed for the first contract; adding it now would increase latency and failure modes.
- Latency is acceptable only as an opt-in helper. The measured direct join was roughly 15–20s when including current Cancerhotspots CLI calls, so default `get variant` must remain unchanged.
- Cancerhotspots recurrence is not universal: ROS1 G2032R returned a source-labelled object with null counts in this run.
