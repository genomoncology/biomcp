use crate::error::BioMcpError;

const LIST_REFERENCE: &str = include_str!("list_reference.md");

pub fn render(entity: Option<&str>) -> Result<String, BioMcpError> {
    match entity.map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(list_all()),
        Some(raw) => match raw.to_ascii_lowercase().as_str() {
            "gene" => Ok(list_gene()),
            "variant" => Ok(list_variant()),
            "article" => Ok(list_article()),
            "trial" => Ok(list_trial()),
            "organization" => Ok(list_organization()),
            "intervention" => Ok(list_intervention()),
            "biomarker" => Ok(list_biomarker()),
            "drug" => Ok(list_drug()),
            "disease" => Ok(list_disease()),
            "pgx" => Ok(list_pgx()),
            "gwas" => Ok(list_gwas()),
            "pathway" => Ok(list_pathway()),
            "protein" => Ok(list_protein()),
            "adverse-event" | "adverse_event" | "adverseevent" => Ok(list_adverse_event()),
            other => Err(BioMcpError::InvalidArgument(format!(
                "Unknown entity: {other}\n\nValid entities:\n- gene\n- variant\n- article\n- trial\n- organization\n- intervention\n- biomarker\n- drug\n- disease\n- pgx\n- gwas\n- pathway\n- protein\n- adverse-event"
            ))),
        },
    }
}

fn list_all() -> String {
    let has_oncokb = std::env::var("ONCOKB_TOKEN")
        .ok()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);

    let mut out = LIST_REFERENCE.to_string();

    if has_oncokb {
        out = out.replace(
            "- `variant articles <id>`\n",
            "- `variant articles <id>`\n- `variant oncokb <id>`\n",
        );
    }
    out
}

fn list_gene() -> String {
    r#"# gene

## Commands

- `get gene <symbol>` - basic gene info (MyGene.info)
- `get gene <symbol> pathways` - pathway section
- `get gene <symbol> ontology` - ontology enrichment section
- `get gene <symbol> diseases` - disease enrichment section
- `get gene <symbol> protein` - UniProt protein summary
- `get gene <symbol> go` - QuickGO terms
- `get gene <symbol> interactions` - STRING interactions
- `get gene <symbol> civic` - CIViC evidence/assertion summary
- `get gene <symbol> all` - include every section
- `gene definition <symbol>` - same card as `get gene <symbol>`
- `gene get <symbol>` - alias for `gene definition <symbol>`

## Search filters

- `search gene <query>`
- `search gene -q <query>`
- `search gene -q <query> --type <protein-coding|ncRNA|pseudo>`
- `search gene -q <query> --chromosome <N>`
- `search gene -q <query> --region <chr:start-end>`
- `search gene -q <query> --pathway <id>`
- `search gene -q <query> --go <term>`
- `search gene -q <query> --limit <N> --offset <N>`

## Search output

- Includes Coordinates, UniProt, and OMIM in default result rows.

## Helpers

- `gene trials <symbol>`
- `gene drugs <symbol>`
- `gene articles <symbol>`
- `gene pathways <symbol> --limit <N> --offset <N>`
"#
    .to_string()
}

fn list_variant() -> String {
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

## IDs

Supported formats:
- rsID: `rs113488022`
- HGVS genomic: `chr7:g.140453136A>T`
- Gene + protein: `BRAF V600E`

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

fn list_article() -> String {
    r#"# article

## Commands

- `get article <id>` - get by PMID/PMCID/DOI
- `get article <id> annotations` - PubTator entity mentions
- `get article <id> fulltext` - download/cache full text
- `get article <id> all` - include all article sections
- `article entities <pmid> --limit <N>` - annotated entities with next commands

## Search

- `search article -g <gene>` - gene filter (PubTator autocomplete)
- `search article -d <disease>` - disease filter (PubTator autocomplete)
- `search article --drug <name>` - chemical/drug filter (PubTator autocomplete)
- `search article <query>` - positional free text keyword
- `search article -k <keyword>` (or `-q <keyword>`) - free text keyword
- `search article --type <review|research|case-reports|meta-analysis>`
- `search article --date-from <YYYY-MM-DD> --date-to <YYYY-MM-DD>`
- `search article --journal <name>`
- `search article --open-access`
- `search article --exclude-retracted`
- `search article --include-retracted`
- `search article --sort <date|citations|relevance>`
- `search article ... --limit <N> --offset <N>`

## Notes

- Set `NCBI_API_KEY` to increase throughput for NCBI-backed article enrichment.
"#
    .to_string()
}

fn list_trial() -> String {
    r#"# trial

## Commands

- `get trial <nct_id>` - protocol card by NCT ID
- `get trial <nct_id> eligibility` - show eligibility criteria inline
- `get trial <nct_id> locations` - site locations section
- `get trial <nct_id> outcomes` - primary/secondary outcomes
- `get trial <nct_id> arms` - arm/intervention details
- `get trial <nct_id> references` - trial publication references
- `get trial <nct_id> all` - include every section
- `search trial [filters]` - search ClinicalTrials.gov (default) or NCI CTS (`--source nci`)

## Useful filters (ctgov)

- `--condition <name>` (or `-c`)
- `--intervention <name>` (or `-i`)
- `--status <status>` (or `-s`)
- `--phase <NA|1|1/2|2|3|4>` (or `-p`)
- `--facility <name>`
- `--age <years>`
- `--sex <female|male|all>`
- `--mutation <text>`
- `--sponsor-type <nih|industry|fed|other>`
- `--prior-therapies <text>`
- `--progression-on <drug>`
- `--line-of-therapy <1L|2L|3L+>`
- `--lat <N>` + `--lon <N>` + `--distance <miles>`
- `--results-available`
- `--has-results` (alias)
- `--study-type <interventional|observational|...>`
- `--date-from <YYYY-MM-DD> --date-to <YYYY-MM-DD>`
- `--count-only`
- `--limit <N> --offset <N>`
"#
    .to_string()
}

fn list_drug() -> String {
    r#"# drug

## Commands

- `get drug <name>` - get by name (MyChem.info aggregation)
- `get drug <name> label` - show key FDA label sections inline
- `get drug <name> shortage` - query current shortage status
- `get drug <name> targets` - enrich with ChEMBL/OpenTargets targets
- `get drug <name> indications` - enrich with OpenTargets indications
- `get drug <name> interactions` - DrugBank interaction rows from cached MyChem payload
- `get drug <name> civic` - CIViC therapy evidence/assertion summary
- `get drug <name> approvals` - Drugs@FDA approval/application details
- `get drug <name> all` - include all sections

## Search

- `search drug <query>`
- `search drug -q <query>`
- `search drug --target <gene>`
- `search drug --indication <disease>`
- `search drug --mechanism <text>`
- `search drug --atc <code>`
- `search drug --pharm-class <class>`
- `search drug --interactions <drug>`
- `search drug ... --limit <N> --offset <N>`

## Helpers

- `drug trials <name>`
- `drug adverse-events <name>`
"#
    .to_string()
}

fn list_organization() -> String {
    r#"# organization

## Commands

- `search organization <query>` - positional NCI organization vocabulary search
- `search organization -q <query>` - flag form (equivalent)
- `search organization --type <academic|industry|...>` - constrain organization type
- `search organization --city <city> --state <state>` - location filters
- `search organization ... --limit <N> --offset <N>`

## Notes

- Requires `NCI_API_KEY`
"#
    .to_string()
}

fn list_intervention() -> String {
    r#"# intervention

## Commands

- `search intervention <query>` - positional NCI intervention vocabulary search
- `search intervention -q <query>` - flag form (equivalent)
- `search intervention --type <drug|device|procedure|...>`
- `search intervention --category <category>`
- `search intervention --code <code>`
- `search intervention ... --limit <N> --offset <N>`

## Notes

- Requires `NCI_API_KEY`
"#
    .to_string()
}

fn list_biomarker() -> String {
    r#"# biomarker

## Commands

- `search biomarker <query>` - positional NCI biomarker vocabulary search
- `search biomarker -q <query>` - flag form (equivalent)
- `search biomarker --type <type>`
- `search biomarker --eligibility <text>`
- `search biomarker --assay-purpose <text>`
- `search biomarker --code <code>`
- `search biomarker ... --limit <N> --offset <N>`

## Notes

- Requires `NCI_API_KEY`
"#
    .to_string()
}

fn list_disease() -> String {
    r#"# disease

## Commands

- `get disease <name_or_id>` - resolve MONDO/DOID or best match by name
- `get disease <name_or_id> genes` - Monarch associations augmented with CIViC drivers
- `get disease <name_or_id> pathways` - Reactome pathways from associated genes
- `get disease <name_or_id> phenotypes` - HPO phenotypes with resolved names
- `get disease <name_or_id> variants` - CIViC disease-associated molecular profiles
- `get disease <name_or_id> models` - Monarch model-organism evidence
- `get disease <name_or_id> prevalence` - OpenTargets prevalence-like evidence
- `get disease <name_or_id> civic` - CIViC disease-context evidence
- `get disease <name_or_id> all` - include all disease sections
- `search disease <query>` - positional search by name
- `search disease -q <query>` - search by name
- `search phenotype "<HP terms>"` - HPO term set to ranked diseases
- `search disease -q <query> --source <mondo|doid|mesh>` - constrain ontology source
- `search disease -q <query> --inheritance <pattern>`
- `search disease -q <query> --phenotype <HP:...>`
- `search disease -q <query> --onset <period>`
- `search disease ... --limit <N> --offset <N>`

## Helpers

- `disease trials <name>`
- `disease articles <name>`
- `disease drugs <name>`
"#
    .to_string()
}

fn list_pgx() -> String {
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
"#
    .to_string()
}

fn list_gwas() -> String {
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

## Related

- `list pgx` - pharmacogenomics command family
"#
    .to_string()
}

fn list_pathway() -> String {
    r#"# pathway

## Commands

- `search pathway <query>` - positional pathway search (Reactome)
- `search pathway -q <query>` - pathway search (Reactome)
- `search pathway -q <query> --type pathway`
- `search pathway --top-level`
- `search pathway -q <query> --limit <N> --offset <N>`
- `get pathway <id>` - base pathway card
- `get pathway <id> genes` - pathway participant genes
- `get pathway <id> events` - contained events
- `get pathway <id> enrichment` - g:Profiler enrichment from pathway genes
- `get pathway <id> all` - include all sections

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
- To inspect pathway composition, run `biomcp get pathway <id> genes events`.
- To pivot to clinical context, run `biomcp pathway trials <id>` and `biomcp pathway articles <id>`.
"#
    .to_string()
}

fn list_protein() -> String {
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
- To inspect structural context, run `biomcp get protein <accession> structures`.
- To continue result browsing, run `biomcp search protein <query> --limit <N> --offset <N>`.
"#
    .to_string()
}

fn list_adverse_event() -> String {
    r#"# adverse-event

## Commands

- `search adverse-event --drug <name>` - FAERS reports (OpenFDA)
- `search adverse-event --drug <name> --outcome <death|hospitalization|disability>`
- `search adverse-event --drug <name> --serious <type>`
- `search adverse-event --drug <name> --date-from <YYYY|YYYY-MM-DD> --date-to <YYYY|YYYY-MM-DD>`
- `search adverse-event --drug <name> --suspect-only --sex <m|f> --age-min <N> --age-max <N>`
- `search adverse-event --drug <name> --reporter <type>`
- `search adverse-event --drug <name> --count <field>` - aggregation mode
- `search adverse-event ... --limit <N> --offset <N>`
- `get adverse-event <report_id>` - retrieve report by ID

## Other query types

- `search adverse-event --type recall --drug <name>` - enforcement/recalls
- `search adverse-event --type device --device <name>` - MAUDE device events
- `search adverse-event --type device --manufacturer <name>` - MAUDE by manufacturer
- `search adverse-event --type device --product-code <code>` - MAUDE by product code
"#
    .to_string()
}
