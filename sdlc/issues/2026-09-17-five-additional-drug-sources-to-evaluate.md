# Five additional drug sources to evaluate for BioMCP

Filed 2026-09-17 after the 0.9.0 release and the drug-source audit.
Ranked by expected impact on the existing drug surface. Each section
names the API, what the source provides, live-probed response shapes,
authentication requirements, and the integration seam in the codebase.

## 1. RxNorm (NIH / UMLS) — canonical drug identity resolution

**Why first.** The CLI resolves brand-to-generic through MyChem text
search (match_kind "broad_text"), which is fuzzy and provider-dependent.
RxNorm makes drug identity deterministic: one RxCUI per normalized drug
concept, covering brand names, generic names, ingredients, and dose
forms. Every downstream surface (search, get, interactions, trials,
pharmacogenomics) would benefit from exact identity resolution instead
of text matching.

**API.** `https://rxnav.nlm.nih.gov/REST/` — no authentication for the
public RxNav API. Live-probed:

```
GET /REST/rxcui.json?name=imatinib
→ {"idGroup":{"rxnormId":["282388"]}}

GET /REST/Prescribe/drugs.json?name=vemurafenib
→ conceptGroup with rxcui, name, synonym (e.g., "vemurafenib 240 MG Oral
  Tablet [Zelboraf]" with synonym "Zelboraf 240 MG Oral Tablet")
```

Key endpoints: `rxcui` (name→RxCUI), `drugs` (name→all dose-form
concepts), `allProperties` (RxCUI→all attributes), `interactions`
(RxCUI→drug interactions via DrugBank data).

**Integration seam.** New module `src/sources/rxnorm.rs`. Call after
MyChem search to resolve the query to an exact RxCUI; surface the RxCUI
on the drug card alongside DrugBank ID. Add an `exact_identity` field
to the drug search result when RxNorm confirms the match.

**Notes.** The existing UMLS integration (already in the codebase for
other sources) covers the UMLS API key path if the richer UMLS REST API
is needed. The public RxNav API is sufficient for the core use case.
BioMCP already has `BIOMCP_MEDLINEPLUS_BASE` exported in the
disease-survival fixture, suggesting NIH-family integration is
precedented.

## 2. DailyMed (FDA) — structured package insert sections

**Why second.** The current `get drug <name> label` command uses OpenFDA
for label detail, but OpenFDA returns the full label as a flat JSON
document. DailyMed provides structured label sections (boxed warnings,
contraindications, drug interactions, adverse reactions, dosage and
administration) with predictable section names. Agents need these
sections discretely for safety checking, not as a wall of text.

**API.** `https://dailymed.nlm.nih.gov/dailymed/services/v2/` — no
authentication. Live-probed:

```
GET /services/v2/spls.json?drug_name=imatinib
→ 29 label sets (one per manufacturer), each with setid and title
```

Key endpoints: `spls.json` (drug→label sets), `spls/{setid}.xml`
(structured SPL XML with named sections).

**Integration seam.** New module `src/sources/dailymed.rs`. Add a
`label-sections` subsection to the drug get path that returns the named
sections from the first (or best-matching) label set. The XML is
already structured (SPL schema), so parsing is straightforward.

## 3. PubChem — chemical identity and structure

**Why third.** MyChem provides identity and cross-references, but not
chemical structures. PubChem adds molecular formula, molecular weight,
canonical SMILES, InChI, and InChIKey — the chemistry layer that lets
agents reason about compound identity across databases.

**API.** `https://pubchem.ncbi.nlm.nih.gov/rest/pug/` — no
authentication. Live-probed:

```
GET /compound/name/imatinib/property/MolecularFormula,MolecularWeight,InChIKey/JSON
→ MW=493.6 formula=C29H31N7O InChIKey=KTUFNOKKBVMGRW-UHFFFAOYSA-N
```

Key endpoints: `compound/name/{name}/property/{fields}/JSON` (one
request for all properties). Note: CanonicalSMILES was not returned for
imatinib in the live probe (may need `IsomericSMILES` or a POST
request for long names).

**Integration seam.** New module `src/sources/pubchem.rs`. Add chemical
identity fields (formula, MW, InChIKey, SMILES) to the drug detail
card. Low risk, additive, no breaking changes.

## 4. MedlinePlus Connect — patient-facing drug information

**Why fourth.** The source module `src/sources/medlineplus.rs` already
exists and is wired into the discover path, but it is not surfaced on
the drug get card. MedlinePlus provides consumer-level drug monographs:
plain-language uses, side effects, precautions, dietary instructions.
This is what agents need for patient-facing summaries.

**API.** Already integrated. The module calls the MedlinePlus Connect
API with drug name lookups.

**Integration seam.** Add a `patient-info` subsection to `get drug
<name>` that calls the existing MedlinePlus source and renders the
consumer-level monograph. No new source module needed — this is
surfacing existing infrastructure.

## 5. BindingDB — quantitative drug-target affinities

**Why last.** ChEMBL provides the target list (which proteins a drug
binds) but not binding strength. BindingDB adds quantitative Ki, IC50,
and Kd values, letting agents distinguish a 2 nM inhibitor from a 2
µM one. Valuable for potency ranking, but the most specialized
addition.

**API.** `https://www.bindingdb.org/bind/BindingDB_API.json` — free, no
key. Query by drug name or InChIKey. Returns binding affinity records
with target, Ki/IC50/Kd, and citation.

**Integration seam.** New module `src/sources/bindingdb.rs`. Add a
`binding-affinities` subsection to the drug detail card with the top
targets ranked by affinity. Depends on PubChem (for InChIKey) being
integrated first, or queries by drug name directly.

## Priority order and suggested ticket scope

| Rank | Source | New module | Complexity | Depends on |
|------|--------|-----------|------------|------------|
| 1 | RxNorm | src/sources/rxnorm.rs | Level 2 | None |
| 2 | DailyMed | src/sources/dailymed.rs | Level 2 | None |
| 3 | PubChem | src/sources/pubchem.rs | Level 1 | None |
| 4 | MedlinePlus surface | (existing module) | Level 1 | None |
| 5 | BindingDB | src/sources/bindingdb.rs | Level 2 | PubChem (InChIKey) |

RxNorm and DailyMed together would give the drug surface exact identity
resolution and structured safety labels — the two biggest gaps. PubChem
adds the chemistry layer cheaply. MedlinePlus is a surfacing task, not
a new source. BindingDB is the specialist add-on.

## Boundaries

Each source is an additive enrichment — no existing source is replaced.
The MyChem text-search path stays as the primary discovery mechanism;
RxNorm adds exact identity on top. DailyMed sections supplement, not
replace, the OpenFDA label path. All five sources are free and
keyless (RxNorm's public RxNav API; the UMLS key path exists if richer
data is needed later).
