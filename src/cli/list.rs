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
            "drug" => Ok(list_drug()),
            "disease" => Ok(list_disease()),
            "phenotype" => Ok(list_phenotype()),
            "pgx" => Ok(list_pgx()),
            "gwas" => Ok(list_gwas()),
            "pathway" => Ok(list_pathway()),
            "protein" => Ok(list_protein()),
            "study" => Ok(list_study()),
            "adverse-event" | "adverse_event" | "adverseevent" => Ok(list_adverse_event()),
            "search-all" | "search_all" | "searchall" => Ok(list_search_all()),
            "discover" => Ok(list_discover()),
            "batch" => Ok(list_batch()),
            "enrich" => Ok(list_enrich()),
            "skill" | "skills" => Ok(crate::cli::skill::list_use_cases()?),
            other => Err(BioMcpError::InvalidArgument(format!(
                "Unknown entity: {other}\n\nValid entities:\n- gene\n- variant\n- article\n- trial\n- drug\n- disease\n- phenotype\n- pgx\n- gwas\n- pathway\n- protein\n- study\n- adverse-event\n- search-all\n- discover\n- batch\n- enrich\n- skill"
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

fn list_discover() -> String {
    r#"# discover

## Commands

- `discover <query>` - resolve free-text biomedical text into typed concepts and suggested BioMCP follow-up commands
- `--json discover <query>` - emit structured concepts plus discover-specific `_meta` metadata for agents
"#
    .to_string()
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
- `get gene <symbol> expression` - GTEx tissue expression summary
- `get gene <symbol> hpa` - Human Protein Atlas protein tissue expression + localization
- `get gene <symbol> druggability` - DGIdb interactions plus OpenTargets tractability/safety
- `get gene <symbol> clingen` - ClinGen validity + dosage sensitivity
- `get gene <symbol> constraint` - gnomAD gene constraint (pLI, LOEUF, mis_z, syn_z)
- `get gene <symbol> disgenet` - DisGeNET scored gene-disease associations (requires `DISGENET_API_KEY`)
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
- `search gene -q <query> --go <GO:0000000>`
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
- `get article <id> tldr` - Semantic Scholar TLDR/influence section (`S2_API_KEY`)
- `get article <id> annotations` - PubTator entity mentions
- `get article <id> fulltext` - download/cache full text
- `get article <id> all` - include all article sections
- `article entities <pmid> --limit <N>` - annotated entities with next commands
- `article citations <id> --limit <N>` - citation graph with contexts/intents (`S2_API_KEY`)
- `article references <id> --limit <N>` - reference graph with contexts/intents (`S2_API_KEY`)
- `article recommendations <id> [<id>...] [--negative <id>...] --limit <N>` - related papers (`S2_API_KEY`)

## Search

- `search article -g <gene>` - gene filter (PubTator autocomplete)
- `search article -d <disease>` - disease filter (PubTator autocomplete)
- `search article --drug <name>` - chemical/drug filter (PubTator autocomplete)
- `search article <query>` - positional free text keyword
- `search article -k <keyword>` (or `-q <keyword>`) - free text keyword
- `search article --type <review|research|case-reports|meta-analysis>`
- `search article --date-from <YYYY-MM-DD> --date-to <YYYY-MM-DD>`
- `search article --since <YYYY-MM-DD>` - alias for `--date-from`
- `search article --journal <name>`
- `search article --open-access`
- `search article --exclude-retracted`
- `search article --include-retracted`
- `search article --sort <date|citations|relevance>`
- `search article --source <all|pubtator|europepmc>`
- `search article --debug-plan` - include executed planner/routing metadata in markdown or JSON
- `search article ... --limit <N> --offset <N>`

## Notes

- Set `NCBI_API_KEY` to increase throughput for NCBI-backed article enrichment.
- Set `S2_API_KEY` to unlock optional Semantic Scholar search fan-out plus TLDR, citation graph, and recommendation helpers.
- `search article` still keeps `--source <all|pubtator|europepmc>` in v1; Semantic Scholar is automatic when the key is present and the filter set is compatible.
- Default `search article --sort relevance` is directness-first rather than citation-first.
"#
    .to_string()
}

fn list_trial() -> String {
    r#"# trial

## Commands

- `get trial <nct_id>` - protocol card by NCT ID
- `get trial <nct_id> eligibility` - show eligibility criteria inline
- `get trial <nct_id> locations` - site locations section
- `get trial <nct_id> locations --offset <N> --limit <N>` - paged location slice
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
- `--age <years>` (decimals accepted, e.g. `0.5`)
- `--sex <female|male|all>`
- `--mutation <text>`
- `--criteria <text>`
- `--biomarker <text>`
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
- `get drug <name> interactions` - OpenFDA label interaction text when available; otherwise a truthful public-data fallback
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
- `search drug --interactions <drug>` - unavailable from current public data sources
- `search drug ... --limit <N> --offset <N>`

## Helpers

- `drug trials <name>`
- `drug adverse-events <name>`
"#
    .to_string()
}

fn list_disease() -> String {
    r#"# disease

## Commands

- `get disease <name_or_id>` - resolve MONDO/DOID or best match by name with OpenTargets gene scores
- `get disease <name_or_id> genes` - Monarch associations augmented with CIViC drivers and OpenTargets scores
- `get disease <name_or_id> pathways` - Reactome pathways from associated genes
- `get disease <name_or_id> phenotypes` - HPO phenotypes with resolved names
- `get disease <name_or_id> variants` - CIViC disease-associated molecular profiles
- `get disease <name_or_id> models` - Monarch model-organism evidence
- `get disease <name_or_id> prevalence` - OpenTargets prevalence-like evidence
- `get disease <name_or_id> civic` - CIViC disease-context evidence
- `get disease <name_or_id> disgenet` - DisGeNET scored disease-gene associations (requires `DISGENET_API_KEY`)
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

fn list_phenotype() -> String {
    r#"# phenotype

## Commands

- `search phenotype "<HP:... HP:...>"` - rank diseases by phenotype similarity
- `search phenotype "<HP:...>" --limit <N> --offset <N>` - page ranked disease matches

## Examples

- `search phenotype "HP:0001250 HP:0001263"`
- `search phenotype "HP:0001250" --limit <N> --offset <N>`
- `search phenotype "HP:0001250,HP:0001263" --limit 10`

## Workflow tips

- Start with 2-5 high-confidence HPO terms for better ranking signal.
- Use specific neurologic/cancer phenotype terms before broad umbrella terms.
- Follow with `get disease <id> all` to inspect phenotypes, genes, and pathways.

## Related

- `search disease -q <query> --phenotype <HP:...>`
- `disease trials <name>`
- `disease articles <name>`
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
"#
    .to_string()
}

fn list_batch() -> String {
    r#"# batch

## Command

- `batch <entity> <id1,id2,...>` - parallel `get` operations for up to 10 IDs

## Options

- `--sections <s1,s2,...>` - request specific sections on each entity
- `--source <ctgov|nci>` - trial source when `entity=trial` (default: `ctgov`)

## Supported entities

- `gene`, `variant`, `article`, `trial`, `drug`, `disease`, `pgx`, `pathway`, `protein`, `adverse-event`

## Examples

- `batch gene BRAF,TP53 --sections pathways,ontology`
- `batch trial NCT04280705,NCT04639219 --source nci --sections locations`
"#
    .to_string()
}

fn list_enrich() -> String {
    r#"# enrich

## Command

- `enrich <GENE1,GENE2,...>` - gene-set enrichment using g:Profiler

## Options

- `--limit <N>` - max number of returned terms (must be 1-50; default 10)

## Examples

- `enrich BRAF,KRAS,NRAS`
- `enrich EGFR,ALK,ROS1 --limit 20`
"#
    .to_string()
}

fn list_search_all() -> String {
    r#"# search-all

## Command

- `search all` - cross-entity summary card with curated section fan-out

## Slots

- `--gene` (or `-g`)
- `--variant` (or `-v`)
- `--disease` (or `-d`)
- `--drug`
- `--keyword` (or `-k`)

## Output controls

- `--since <YYYY|YYYY-MM|YYYY-MM-DD>` - applies to date-capable sections
- `--limit <N>` - rows per section (default: 3)
- `--counts-only` - section counts without row tables
- `--debug-plan` - include executed leg/routing metadata in markdown or JSON
- `--json` - machine-readable sections + links contract

## Notes

- At least one typed slot is required.
- Unanchored keyword-only dispatch is article-only.
- Keyword is pushed into drug search only when `--gene` and/or `--disease` is present.

## Understanding the Output

- Section order follows anchor priority: gene, disease, drug, variant, then keyword-only.
- `get.top` links open the top row as a detailed card.
- `cross.*` links pivot to a related entity search.
- `filter.hint` links show useful next filters for narrowing.
- `search.retry` links appear when a section errors or times out.
- Typical workflow: `search all` -> `search <entity>` -> `get <entity> <id>` -> helper commands.
"#
    .to_string()
}

fn list_pathway() -> String {
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
"#
    .to_string()
}

fn list_study() -> String {
    r#"# study

## Commands

- `study list` - list locally available cBioPortal studies from `BIOMCP_STUDY_DIR`
- `study download [--list] [<study_id>]` - list downloadable study IDs or install a study into `BIOMCP_STUDY_DIR`
- `study filter --study <id> [--mutated <symbol>] [--amplified <symbol>] [--deleted <symbol>] [--expression-above <gene:threshold>] [--expression-below <gene:threshold>] [--cancer-type <type>]` - intersect sample filters across mutation, CNA, expression, and clinical data
- `study query --study <id> --gene <symbol> --type <mutations|cna|expression>` - run per-study gene query
- `study cohort --study <id> --gene <symbol>` - split the cohort into `<gene>-mutant` vs `<gene>-wildtype`
- `study survival --study <id> --gene <symbol> [--endpoint <os|dfs|pfs|dss>]` - summarize KM survival and log-rank statistics by mutation group
- `study compare --study <id> --gene <symbol> --type <expression|mutations> --target <symbol>` - compare expression or mutation rate across mutation groups
- `study co-occurrence --study <id> --genes <g1,g2,...>` - pairwise mutation co-occurrence (2-10 genes)

## Setup

- `BIOMCP_STUDY_DIR` should point to a directory containing per-study folders (for example `msk_impact_2017/`).
- Use `study download --list` to browse remote IDs and `study download <study_id>` to install a study into that directory.
- `study cohort`, `study survival`, and `study compare` require `data_mutations.txt` and `data_clinical_sample.txt`.
- `study survival` also requires `data_clinical_patient.txt` with canonical `{ENDPOINT}_STATUS` and `{ENDPOINT}_MONTHS` columns.
- Expression comparison also requires a supported expression matrix file.

## Examples

- `study list`
- `study download --list`
- `study download msk_impact_2017`
- `study filter --study brca_tcga_pan_can_atlas_2018 --mutated TP53 --amplified ERBB2 --expression-above ERBB2:1.5`
- `study query --study msk_impact_2017 --gene TP53 --type mutations`
- `study query --study brca_tcga_pan_can_atlas_2018 --gene ERBB2 --type cna`
- `study query --study paad_qcmg_uq_2016 --gene KRAS --type expression`
- `study cohort --study brca_tcga_pan_can_atlas_2018 --gene TP53`
- `study survival --study brca_tcga_pan_can_atlas_2018 --gene TP53 --endpoint os`
- `study compare --study brca_tcga_pan_can_atlas_2018 --gene TP53 --type expression --target ERBB2`
- `study compare --study brca_tcga_pan_can_atlas_2018 --gene TP53 --type mutations --target PIK3CA`
- `study co-occurrence --study msk_impact_2017 --genes TP53,KRAS`
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

#[cfg(test)]
mod tests {
    use super::{list_gene, render};

    #[test]
    fn list_root_includes_quickstart_and_skills_tip() {
        let out = render(None).expect("list root should render");
        assert!(out.contains("## Quickstart"));
        assert!(out.contains("`skill install` - install BioMCP skill guidance to your agent"));
        assert!(out.contains("`discover <query>`"));
        assert!(!out.contains("`skill list`"));
        assert!(!out.contains("Run `biomcp skill list` to browse all skills."));
    }

    #[test]
    fn list_discover_page_exists() {
        let out = render(Some("discover")).expect("list discover should render");
        assert!(out.contains("# discover"));
        assert!(out.contains("discover <query>"));
        assert!(out.contains("--json discover <query>"));
    }

    #[test]
    fn list_entity_pages_drop_stale_skill_sections() {
        for entity in ["gene", "variant", "drug"] {
            let out = render(Some(entity)).expect("entity page should render");
            assert!(
                !out.contains("## Recommended skills"),
                "{entity} page should not advertise removed use-case skills"
            );
            assert!(
                !out.contains("## Skills"),
                "{entity} page should not append the generic skills section"
            );
            assert!(
                !out.contains("biomcp skill "),
                "{entity} page should not reference stale skill commands"
            );
        }
    }

    #[test]
    fn list_skill_alias_routes_to_skill_listing() {
        let out = render(Some("skill")).expect("list skill should render");
        assert!(out.contains("No skills found"));
    }

    #[test]
    fn list_batch_and_enrich_pages_exist() {
        let batch = render(Some("batch")).expect("list batch should render");
        assert!(batch.contains("# batch"));
        assert!(batch.contains("batch <entity> <id1,id2,...>"));

        let enrich = render(Some("enrich")).expect("list enrich should render");
        assert!(enrich.contains("# enrich"));
        assert!(enrich.contains("enrich <GENE1,GENE2,...>"));
    }

    #[test]
    fn list_study_page_exists() {
        let out = render(Some("study")).expect("list study should render");
        assert!(out.contains("# study"));
        assert!(out.contains("study download [--list] [<study_id>]"));
        assert!(out.contains(
            "study filter --study <id> [--mutated <symbol>] [--amplified <symbol>] [--deleted <symbol>]"
        ));
        assert!(out.contains(
            "study query --study <id> --gene <symbol> --type <mutations|cna|expression>"
        ));
        assert!(out.contains("study cohort --study <id> --gene <symbol>"));
        assert!(
            out.contains(
                "study survival --study <id> --gene <symbol> [--endpoint <os|dfs|pfs|dss>]"
            )
        );
        assert!(out.contains(
            "study compare --study <id> --gene <symbol> --type <expression|mutations> --target <symbol>"
        ));
    }

    #[test]
    fn list_gene_mentions_new_gene_sections() {
        let out = list_gene();
        assert!(out.contains("get gene <symbol> expression"));
        assert!(out.contains("get gene <symbol> hpa"));
        assert!(out.contains("get gene <symbol> druggability"));
        assert!(out.contains("get gene <symbol> clingen"));
        assert!(out.contains("get gene <symbol> constraint"));
        assert!(out.contains("get gene <symbol> disgenet"));
    }

    #[test]
    fn list_disease_mentions_disgenet_section() {
        let out = render(Some("disease")).expect("list disease should render");
        assert!(out.contains("get disease <name_or_id> disgenet"));
    }

    #[test]
    fn list_trial_and_article_include_missing_flags() {
        let trial = render(Some("trial")).expect("list trial should render");
        assert!(trial.contains("--biomarker <text>"));

        let article = render(Some("article")).expect("list article should render");
        assert!(article.contains("--since <YYYY-MM-DD>"));
    }

    #[test]
    fn list_pathway_describes_source_aware_sections() {
        let out = render(Some("pathway")).expect("list pathway should render");
        assert!(out.contains("get pathway <id> events` - contained events (Reactome only)"));
        assert!(out.contains(
            "get pathway <id> enrichment` - g:Profiler enrichment from pathway genes (Reactome only)"
        ));
        assert!(out.contains(
            "get pathway <id> all` - include all sections supported by that pathway source"
        ));
        assert!(out.contains("biomcp get pathway <id> genes"));
        assert!(out.contains("Reactome pathways, events are also available"));
    }

    #[test]
    fn phenotype_and_gwas_include_workflow_tips() {
        let phenotype = render(Some("phenotype")).expect("list phenotype should render");
        assert!(phenotype.contains("## Workflow tips"));
        assert!(phenotype.contains("2-5 high-confidence HPO terms"));

        let gwas = render(Some("gwas")).expect("list gwas should render");
        assert!(gwas.contains("## Workflow tips"));
        assert!(gwas.contains("--p-value"));
    }

    #[test]
    fn unknown_entity_lists_new_valid_entities() {
        let err = render(Some("unknown")).expect_err("unknown entity should fail");
        let msg = err.to_string();
        assert!(msg.contains("- skill"));
        assert!(msg.contains("- enrich"));
        assert!(msg.contains("- batch"));
        assert!(msg.contains("- study"));
        assert!(msg.contains("- discover"));
    }
}
