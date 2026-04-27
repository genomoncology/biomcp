//! Molecular command-reference pages for `biomcp list`.
pub(super) fn list_gene() -> String {
    r#"# gene

## When to use this surface

- Use `get gene <symbol>` for the default card when you need the canonical summary first.
- Add `protein`, `hpa`, `expression`, `diseases`, `diagnostics`, or `funding` when you need deeper function, localization, disease, diagnostic-test, or NIH grant context.
- Use `gene articles <symbol>` or `search article -g <symbol>` when you need literature tied to one gene.

## Commands

- `get gene <symbol>` - basic gene info (MyGene.info)
- `get gene <symbol> pathways` - pathway section
- `get gene <symbol> ontology` - ontology enrichment section
- `get gene <symbol> diseases` - disease enrichment section
- `get gene <symbol> diagnostics` - diagnostic tests for this gene from GTR
- `get gene <symbol> protein` - UniProt protein summary
- `get gene <symbol> go` - QuickGO terms
- `get gene <symbol> interactions` - STRING interactions
- `get gene <symbol> civic` - CIViC evidence/assertion summary
- `get gene <symbol> expression` - GTEx tissue expression summary
- `get gene <symbol> hpa` - Human Protein Atlas protein tissue expression + localization
- `get gene <symbol> druggability` - DGIdb interactions plus OpenTargets tractability/safety
- `get gene <symbol> clingen` - ClinGen validity + dosage sensitivity
- `get gene <symbol> constraint` - gnomAD gene constraint (pLI, LOEUF, mis_z, syn_z)
- `get gene <symbol> disgenet` - DisGeNET scored gene-disease associations (requires `DISGENET_API_KEY`)
- `get gene <symbol> funding` - NIH Reporter grants mentioning the gene in the most recent 5 NIH fiscal years
- `get gene <symbol> all` - include every standard section (`diagnostics` and `funding` stay opt-in)
- `gene definition <symbol>` - same card as `get gene <symbol>`
- `gene get <symbol>` - alias for `gene definition <symbol>`

## Search filters

- `search gene <query>`
- `search gene -q <query>`
- `search gene -q <query> --type <protein-coding|ncRNA|pseudo>`
- `search gene -q <query> --chromosome <N>`
- `search gene -q <query> --region <chr:start-end>`
- `search gene -q <query> --pathway <id>`
- `search gene -q <query> --go <GO:0000000>`
- `search gene -q <query> --limit <N> --offset <N>`

## Search output

- Includes Coordinates, UniProt, and OMIM in default result rows.

## JSON Output

- Non-empty `search gene --json` responses include `_meta.next_commands`.
- The first follow-up drills the top result with `biomcp get gene <symbol>`.
- `biomcp list gene` is always included so agents can inspect the full filter surface.

## Helpers

- `gene trials <symbol>`
- `gene drugs <symbol>`
- `gene articles <symbol>`
- `gene pathways <symbol> --limit <N> --offset <N>`
"#
    .to_string()
}

pub(super) fn list_variant() -> String {
    let has_oncokb = std::env::var("ONCOKB_TOKEN")
        .ok()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);

    let mut out = r#"# variant

## Commands

- `get variant <id>` - core annotation (MyVariant.info)
- `get variant <id> predict` - AlphaGenome prediction (requires `ALPHAGENOME_API_KEY`)
- `get variant <id> predictions` - expanded dbNSFP model scores (REVEL, AlphaMissense, etc.)
- `get variant <id> clinvar` - ClinVar section details
- `get variant <id> population` - gnomAD population frequencies
- `get variant <id> conservation` - phyloP/phastCons/GERP conservation scores
- `get variant <id> cosmic` - COSMIC context from cached MyVariant payload
- `get variant <id> cgi` - CGI drug-association evidence table
- `get variant <id> civic` - CIViC cached + GraphQL clinical evidence
- `get variant <id> cbioportal` - cBioPortal frequency enrichment (on-demand)
- `get variant <id> gwas` - GWAS trait associations
- `get variant <id> all` - include all sections

## Search filters

- `-g <gene>`
- `--hgvsp <protein_change>`
- `--significance <value>`
- `--max-frequency <0-1>`
- `--min-cadd <score>`
- `--consequence <term>`
- `--review-status <stars>`
- `--population <afr|amr|eas|fin|nfe|sas>`
- `--revel-min <score>`
- `--gerp-min <score>`
- `--tumor-site <site>`
- `--condition <name>`
- `--impact <HIGH|MODERATE|LOW|MODIFIER>`
- `--lof`
- `--has <field>`
- `--missing <field>`
- `--therapy <name>`

## Search output

- Includes ClinVar Stars, REVEL, and GERP in default result rows.

## JSON Output

- Non-empty `search variant --json` responses include `_meta.next_commands`.
- The first follow-up drills the top result with `biomcp get variant <id>`.
- `biomcp list variant` is always included so agents can inspect the full filter surface.

## IDs

Supported formats:
- rsID: `rs113488022`
- HGVS genomic: `chr7:g.140453136A>T`
- Gene + protein: `BRAF V600E`, `BRAF p.Val600Glu`

## Helpers

- `variant trials <id> --source <ctgov|nci> --limit <N> --offset <N>`
- `variant articles <id>`
"#
    .to_string();

    if has_oncokb {
        out.push_str("- `variant oncokb <id>` - explicit OncoKB lookup for therapies/levels\n");
    } else {
        out.push_str("\nOncoKB helper: set `ONCOKB_TOKEN`, then use `variant oncokb <id>`.\n");
    }
    out
}

pub(super) fn list_pgx() -> String {
    r#"# pgx

## Commands

- `get pgx <gene_or_drug>` - CPIC-based PGx card by gene or drug
- `get pgx <gene_or_drug> recommendations` - dosing recommendation section
- `get pgx <gene_or_drug> frequencies` - population frequency section
- `get pgx <gene_or_drug> guidelines` - guideline metadata section
- `get pgx <gene_or_drug> annotations` - PharmGKB enrichment section
- `get pgx <gene_or_drug> all` - include all PGx sections
- `search pgx -g <gene>` - interactions by gene
- `search pgx -d <drug>` - interactions by drug
- `search pgx --cpic-level <A|B|C|D>`
- `search pgx --pgx-testing <value>`
- `search pgx --evidence <level>`
- `search gwas -g <gene>` - GWAS-linked variants by gene
- `search gwas --trait <text>` - GWAS-linked variants by disease trait

## Examples

- `get pgx CYP2D6`
- `get pgx codeine recommendations`
- `search pgx -g CYP2D6 --limit 5`
- `search gwas --trait "type 2 diabetes" --limit 5`

## JSON Output

- Non-empty `search pgx --json` responses include `_meta.next_commands`.
- The first follow-up drills the top result with `biomcp get pgx <gene_or_drug>`.
- `biomcp list pgx` is always included so agents can inspect the full filter surface.
"#
    .to_string()
}

pub(super) fn list_gwas() -> String {
    r#"# gwas

## Commands

- `search gwas -g <gene>` - GWAS-linked variants by gene
- `search gwas --trait <text>` - GWAS-linked variants by disease trait
- `search gwas --region <chr:start-end>`
- `search gwas --p-value <threshold>`
- `search gwas ... --limit <N> --offset <N>`

## Examples

- `search gwas -g TCF7L2 --limit 5`
- `search gwas --trait "type 2 diabetes" --limit 5`
- `search gwas --region 7:55000000-55200000 --p-value 5e-8 --limit 10`

## Workflow tips

- Use `--trait` for phenotype-first discovery and `-g` for gene-first review.
- Tighten noisy results with `--p-value` and locus-focused `--region`.
- Pivot high-interest hits into `get variant <id>` and `variant trials <id>`.

## Related

- `list pgx` - pharmacogenomics command family
- `search trial --mutation <text>`
- `search trial --criteria <text>`
- `search article -g <gene>`

## JSON Output

- Non-empty `search gwas --json` responses include `_meta.next_commands`.
- The first follow-up drills the top hit with `biomcp get variant <rsid>`.
- `biomcp list gwas` is always included so agents can inspect the full filter surface.
"#
    .to_string()
}

pub(super) fn list_pathway() -> String {
    r#"# pathway

## Commands

- `search pathway <query>` - positional pathway search (Reactome + KEGG)
- `search pathway -q <query>` - pathway search (Reactome + KEGG)
- `search pathway -q <query> --type pathway`
- `search pathway --top-level`
- `search pathway -q <query> --limit <N> --offset <N>`
- `get pathway <id>` - base pathway card
- `get pathway <id> genes` - pathway participant genes
- `get pathway <id> events` - contained events (Reactome only)
- `get pathway <id> enrichment` - g:Profiler enrichment from pathway genes (Reactome only)
- `get pathway <id> all` - include all sections supported by that pathway source

## Search filters

- `search pathway <query>`
- `search pathway -q <query>`
- `--type pathway`
- `--top-level`
- `--limit <N> --offset <N>`

## Helpers

- `pathway drugs <id>`
- `pathway articles <id>`
- `pathway trials <id>`

## Workflow examples

- To find pathways for an altered gene, run `biomcp search pathway "<gene or process>" --limit 5`.
- To inspect pathway composition, run `biomcp get pathway <id> genes`.
- For Reactome pathways, events are also available: `biomcp get pathway R-HSA-5673001 events`.
- To pivot to clinical context, run `biomcp pathway trials <id>` and `biomcp pathway articles <id>`.

## JSON Output

- Non-empty `search pathway --json` responses include `_meta.next_commands`.
- The first follow-up drills the top result with `biomcp get pathway <id>`.
- `biomcp list pathway` is always included so agents can inspect the full filter surface.
"#
    .to_string()
}

pub(super) fn list_protein() -> String {
    r#"# protein

## Commands

- `search protein -q <query>` - protein search (UniProt, human-only by default)
- `search protein <query>` - positional query form
- `search protein -q <query> --all-species`
- `search protein -q <query> --reviewed`
- `search protein -q <query> --disease <name>`
- `search protein -q <query> --existence <1-5>`
- `search protein ... --limit <N> --offset <N>`
- `get protein <accession_or_symbol>` - base protein card
- `get protein <accession> domains` - InterPro domains
- `get protein <accession> interactions` - STRING interactions
- `get protein <accession> complexes` - ComplexPortal protein complexes
- `get protein <accession> structures` - structure IDs (PDB/AlphaFold)
- `get protein <accession> all` - include all sections

## Search filters

- `search protein <query>`
- `search protein -q <query>`
- `--all-species`
- `--reviewed` (default behavior uses reviewed=true for safer results)
- `--disease <name>`
- `--existence <1-5>`
- `--limit <N> --offset <N>`
- `--next-page <token>` (cursor compatibility alias; `--offset` is preferred UX)

## Helpers

- `protein structures <accession> --limit <N> --offset <N>`

## Workflow examples

- To find a target protein from a gene symbol, run `biomcp search protein BRAF --limit 5`.
- To inspect complex membership, run `biomcp get protein <accession> complexes`.
- To inspect structural context, run `biomcp get protein <accession> structures`.
- To continue result browsing, run `biomcp search protein <query> --limit <N> --offset <N>`.
"#
    .to_string()
}
