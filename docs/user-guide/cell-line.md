# Cell Line

Use cell line commands to turn any common spelling of a cell line into a Cellosaurus accession, then read its identity, its cross-references, and its curated sequence variations.

Key boundaries:

- Cellosaurus is the only source. The CVCL accession is the join key for every other cell line dataset.
- Search compares normalized names: lowercase, with every non-alphanumeric character dropped. `MOLM13`, `MOLM-13`, and `Molm 13` are the same query.
- BioMCP ranks candidates and picks no winner. When two lines match exactly, both rows appear.
- The base card never fetches cross-references. Ask for `xrefs` when you need the join keys.
- Every output names the Cellosaurus release and carries the CC BY 4.0 attribution line.

## Search cell lines

`QUERY` is required. It can be a name, a synonym, or a CVCL accession.

```bash
biomcp search cell-line MOLM13
biomcp search cell-line "MV4;11"
biomcp search cell-line KG1 --limit 5
biomcp search cell-line CVCL_2119
```

Each row carries `match` (`exact` or `partial`) and, on exact rows, `matched_on` (`name` or `synonym`). Exact identifier matches sort first, then exact synonym-only matches, then partial matches. Human lines come first within each group.

A hyphenated query runs twice, once as typed and once with the hyphens removed, and the two windows merge by accession. `KG-1` and `KG1` therefore return the same set.

When Cellosaurus fills its 1000-row window, the total is unknown and the output carries a note. Use a more specific name or the accession.

## Get cell line records

```bash
biomcp get cell-line CVCL_2119
biomcp get cell-line CVCL_0064
```

The card carries the accession, the RRID, the identifier name, synonyms, species, disease, category, sex, and age.

A source ID also works. BioMCP resolves it to the accession first and prints the ID it resolved from.

```bash
biomcp get cell-line ACH-000362      # DepMap
biomcp get cell-line SIDM00437       # Cell Model Passports
biomcp get cell-line CHEMBL3706573   # ChEMBL
biomcp get cell-line MOLM13_950_2019 # PharmacoDB
```

## Request cell line sections

Cross-references, the join keys other datasets use:

```bash
biomcp get cell-line CVCL_2119 xrefs
```

Every key is printed. A line with no link to a resource shows an empty list, so `biomcp get cell-line CVCL_0007 xrefs` prints `gdsc` and `cosmic_clp` as empty.

Curated sequence variations:

```bash
biomcp get cell-line CVCL_1844 variants
```

Rows appear as Cellosaurus published them, with the HGVS description, the HGNC link, zygosity, and the PubMed sources. BioMCP adds no interpretation.

ChEMBL assay coverage:

```bash
biomcp get cell-line CVCL_2119 chembl
```

The section prints the ChEMBL cell line record that names the accession: the ChEMBL ID, the EFO and CLO IDs, and how many ChEMBL assays were run on the line. The count tells you whether ChEMBL literature assays exist. BioMCP lists no assays and no activity values. The join goes through the accession alone, so `biomcp get cell-line CVCL_0004 chembl` finds the line ChEMBL spells `K562`.

PharmacoDB drug response:

```bash
biomcp get cell-line CVCL_2119 drug_response
```

The section counts the experiments PharmacoDB published for the line in each dataset. MOLM-13 has 1117 across CTRPv2, gCSI, GDSC1, and GDSC2. The join goes through the PharmacoDB ID in `xrefs` first and the line name second, and BioMCP keeps the answer only when the accession PharmacoDB returns is the accession asked for. The section lists no rows, because the unscoped listing for a heavily screened line such as K-562 is 19.4 MB of body.

The Cellosaurus sections together:

```bash
biomcp get cell-line CVCL_2119 all
```

`all` covers `variants` and `xrefs`. The `chembl` and `drug_response` sections each cost their own request, so they are asked for by name.

## Helper commands

One helper reads the PharmacoDB rows behind the `drug_response` counts, one dataset at a time:

```bash
biomcp cell-line drug-response CVCL_2119 --dataset GDSC1
```

`--dataset` is required. Without it the command fails before any request and names the ten PharmacoDB datasets, because an unscoped listing is megabytes of body. Each row names the compound, the experiment ID, and the published AAC, IC50, EC50, Einf, HS, and DSS1. PharmacoDB gives no units, BioMCP interprets no value, and an absent metric prints as `-`. A compound tested twice in one dataset stays two rows with two experiment IDs.

To go from a compound back to the lines it was tested on, run [`biomcp drug cell-lines <name>`](drug.md).

To go the other way, from a gene to the cell lines that express it, run [`biomcp gene cell-lines <symbol> --group <group>`](gene.md). That helper prints the Human Protein Atlas nTPM of one gene across one cancer group and carries the Cellosaurus accession on each row it could resolve by name.

## JSON mode

```bash
biomcp search cell-line KG1 --json
biomcp get cell-line CVCL_2119 xrefs --json
```

JSON carries `data_as_of` and `data_as_of_kind` at the top level. `data_as_of_kind` is `release` when BioMCP read the Cellosaurus release, and `retrieved` when that read failed and the value is a retrieval time instead. A filled provider window appears in `_meta.notes`, and `pagination.total` is null.

## Practical tips

Two names that look alike are often two lines. HL-60 is `CVCL_0002`; the NCI-60 line HL-60(TB) is `CVCL_A794`, a separate record. The `NCI-DCTD` link on HL-60(TB) carries the name `HL-60`, so a name-based join from NCI-60 lands on the wrong line. Join on the accession, not the name.

`NB4` is the identifier of `CVCL_0005` and also a synonym of the neuroblastoma line SJNB-4 (`CVCL_8821`). Both rows are exact matches. The `matched_on` column tells them apart.

`KG1` matches three records, two of them mouse lines. Read the species column before you pick.

## Related guides

- [Cellosaurus](../sources/cellosaurus.md)
- [Variant](variant.md)
- [CLI Reference](cli-reference.md)
