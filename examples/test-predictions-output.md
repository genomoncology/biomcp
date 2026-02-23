Used BioMCP commands:

- `biomcp --json get variant "BRAF V600E"`
- `biomcp --json get variant "BRAF V600E" predictions`
- `biomcp --json get variant "BRAF V600E" cgi`

### 1) Variant details (BioMCP `get variant`)
- Variant resolved to **`chr7:g.140453136A>T`** (`HGVS p.V600E`, `HGVS c.620T>A`), gene **BRAF**.
- **rsID:** `rs113488022`; **COSMIC ID:** `COSM476`.
- Clinical status from annotation: **Pathogenic**, gnomAD AF **3.98e-6**.
- ClinVar data (from the full record): **clinvar_id 13961**, significance review status “criteria provided, multiple submitters, no conflicts”.

### 2) Expanded prediction scores
From `biomcp ... predictions`:
- **REVEL:** **0.931**
- **AlphaMissense:** **0.9853** (Prediction: Pathogenic)
- **ClinPred:** **0.993496775627136** (Prediction: D)
- **MetaRNN:** **0.88336486** (Prediction: D)
- **BayesDel:** not returned in the `expanded_predictions` payload for this variant in current Biomcp output (only 4 tools were present).

### 3) CGI drug–variant associations
From `biomcp ... cgi`:
- **Responsive (evidence):**  
  **PLX4720 (BRAF inhibitor)**, **BRAF inhibitor + MEK inhibitors**, **Pan-RAF inhibitors**, **BRAF inhibitors**, **Trametinib (MEK inhibitor)**, **Vemurafenib + Cobimetinib**, **BRAF inhibitor + PI3K pathway inhibitors**
- **Resistant (evidence):**  
  **EGFR TK inhibitors**, **Cetuximab (EGFR mAb inhibitor)**

### 3-sentence clinical actionability summary
BRAF V600E is a clearly pathogenic, recurrent oncogenic missense change (rs113488022 / chr7:g.140453136A>T, p.V600E) with very high pathogenicity signal across integrated scores (REVEL 0.931, AlphaMissense 0.9853, ClinPred 0.9935, MetaRNN 0.8834).  
CGI-linked evidence indicates strong actionable support for BRAF/MEK pathway targeting (including FDA-guideline-supported **Trametinib** and **Vemurafenib + Cobimetinib**) and other BRAF-directed combinations (**Pan-RAF inhibitors**, **BRAF inhibitor + MEK inhibitors**, **PLX4720**).  
However, resistance associations (e.g., **EGFR TK inhibitors** and **Cetuximab**) suggest context-specific non-responsiveness in some tumors, so treatment selection should be tissue- and protocol-specific rather than assuming uniform sensitivity across all cancer types.
