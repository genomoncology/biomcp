# Drug mechanism field can carry a pharmacologic action instead of a mechanism

In `biomcp search all --gene BRAF --disease melanoma`, the Drugs section reports:

    | dabrafenib  | BRAF | Cytochrome P450 2C9 Inducers |
    | vemurafenib | BRAF | Inhibitor of Serine/threonine-protein kinase B-raf |

Dabrafenib is a BRAF inhibitor. "Cytochrome P450 2C9 Inducers" is a MeSH
pharmacologic-action term, not the drug's mechanism of action, and it sits in a
column the vemurafenib row fills correctly. A clinician reading the card would
take the label at face value.

Found while reviewing marketing capture
`repos/mktg/biomcp/drafts/10-ten-cards-one-command/captures/10-search-all.txt`
(BioMCP 0.9.0-dev.6, captured 2026-08-26). Worth checking whether the mechanism
field takes the first available MeSH pharmacologic action rather than a
mechanism-of-action classification.

Verified in code and data on 2026-08-26 (analysis complete, ticket not yet
filed):

- The mechanism column's fallback is `fallback_mechanism_from_hit`
  (`src/transform/drug.rs`): it takes the FIRST NDC pharm class tagged
  `[MoA]` (`moa_pharm_classes(...).next()`). The preferred source is
  ChEMBL `mechanism_of_action` (`chembl_mechanisms_from_hit`); the
  fallback fires when ChEMBL has nothing for the hit.
- Live MyChem data for dabrafenib: FDA's DrugCentral `fda_moa` lists
  "Protein Kinase Inhibitors" FIRST and a cytochrome-P450 inducer class
  second — the correct answer is present in the upstream data. The NDC
  pharm-class list the code actually reads orders the inducer class into
  the first `[MoA]` slot, so first-match picks it.
- Vemurafenib's row reads correctly only because its single FDA MoA class
  happens to be the mechanism ("Inhibitor of Serine/threonine-protein
  kinase B-raf") — the same code path, a luckier list.

Classification: upstream data ordering surfaced by a code policy
(first-match instead of most-specific). The fix is a BioMCP-side ranking
or preferring the ChEMBL/DrugCentral mechanism-of-action class, not an
upstream correction.
