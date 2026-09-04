# Annotate variant structure context

Use `variant structure` when you need the protein-structure context for one exact variant. The helper is opt-in because it joins several network sources.

```bash
biomcp variant structure "BRAF V600E"
biomcp --json variant structure "BRAF V600E"
```

The JSON response includes:

- `variant`, `gene`, and `input_kind`
- `residue` with the selected protein position, matched HGVSp aliases, other source positions, and confidence
- `protein` with the UniProt accession and entry
- overlapping InterPro `domains` with `start` and `end`
- UniProt `structures.pdb` rows plus `structures.alphafold.url`
- nullable, source-labelled `cancerhotspots` recurrence
- top-level `lookup_outcomes` for `domains` and `cancerhotspots`
- `warnings` and `_meta.next_commands`

A successful provider check with no overlap or hotspot match is `empty` and may
name the checked source. If the selected residue or normalizable protein change
is missing, the lookup is `inapplicable`; if the provider fails, it is
`unavailable`. Those states do not claim a checked absence or provider source,
and an inapplicable/unavailable Cancer Hotspots recurrence is JSON `null`.

BioMCP selects the requested HGVSp position when MyVariant.info/dbNSFP returns multiple transcript or isoform positions. Ambiguous residue-only inputs are rejected with the same guidance as `get variant`; search first, then run this helper on the exact variant.

This helper does not run ΔΔG or stability prediction and does not change default `get variant` output.
