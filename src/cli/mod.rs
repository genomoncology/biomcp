//! Top-level CLI parsing and command execution.

use clap::{Parser, Subcommand};
use futures::{StreamExt, future::try_join_all};
use tracing::{debug, warn};

pub mod health;
pub mod list;
pub mod skill;
pub mod update;

#[derive(Parser, Debug)]
#[command(
    name = "biomcp",
    about = "Query genes, variants, trials, articles, drugs, diseases, and more from 15 biomedical sources",
    version,
    after_help = "Note: flags marked (best-effort) are applied client-side or via imprecise API matching; results may include false positives."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output as JSON instead of Markdown
    #[arg(short, long, global = true)]
    pub json: bool,

    /// Disable HTTP caching (always fetch fresh data)
    #[arg(long, global = true)]
    pub no_cache: bool,
}

#[derive(Subcommand, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Commands {
    /// Search for entities
    Search {
        #[command(subcommand)]
        entity: SearchEntity,
    },
    /// Get entity by ID
    Get {
        #[command(subcommand)]
        entity: GetEntity,
    },
    /// Variant cross-entity helpers
    Variant {
        #[command(subcommand)]
        cmd: VariantCommand,
    },
    /// Drug cross-entity helpers
    Drug {
        #[command(subcommand)]
        cmd: DrugCommand,
    },
    /// Disease cross-entity helpers
    Disease {
        #[command(subcommand)]
        cmd: DiseaseCommand,
    },
    /// Article cross-entity helpers
    Article {
        #[command(subcommand)]
        cmd: ArticleCommand,
    },
    /// Gene cross-entity helpers
    Gene {
        #[command(subcommand)]
        cmd: GeneCommand,
    },
    /// Pathway cross-entity helpers
    Pathway {
        #[command(subcommand)]
        cmd: PathwayCommand,
    },
    /// Protein cross-entity helpers
    Protein {
        #[command(subcommand)]
        cmd: ProteinCommand,
    },
    /// Check external API connectivity
    Health {
        /// Check external APIs only
        #[arg(long)]
        apis_only: bool,
    },
    /// Run MCP server over stdio
    Mcp,
    /// Alias for `mcp` (Claude Desktop friendly)
    Serve,
    /// Run MCP server over HTTP (SSE transport)
    ServeHttp {
        /// Host address to bind
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Port to listen on
        #[arg(long, default_value = "8080")]
        port: u16,
    },
    /// Embedded BioMCP skills (use-cases) for agents
    #[command(after_help = "\
EXAMPLES:
  biomcp skill <name|number>
  biomcp skill 03
  biomcp skill variant-to-treatment
  biomcp skill list")]
    Skill {
        #[command(subcommand)]
        command: Option<skill::SkillCommand>,
    },
    /// Update the biomcp binary from GitHub releases
    Update {
        /// Check for updates, but do not install
        #[arg(long)]
        check: bool,
    },
    /// Uninstall biomcp from the current location
    Uninstall,
    /// Command reference for entities and flags
    List {
        /// Optional entity name (gene, variant, article, trial, drug, disease, pgx, gwas, pathway, protein, adverse-event)
        entity: Option<String>,
    },
    /// Parallel get operations (comma-separated IDs, max 10)
    Batch {
        /// Entity type (gene, variant, article, trial, drug, disease, pgx, pathway, protein, adverse-event)
        entity: String,
        /// Comma-separated IDs (max 10)
        ids: String,
        /// Optional comma-separated sections to request on each get call
        #[arg(long)]
        sections: Option<String>,
        /// Trial source when entity=trial (ctgov or nci)
        #[arg(long, default_value = "ctgov")]
        source: String,
    },
    /// Gene set enrichment against g:Profiler
    Enrich {
        /// Comma-separated HGNC symbols (e.g., BRAF,KRAS,NRAS)
        genes: String,
        /// Maximum enrichment terms (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Show version
    Version,
}

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug)]
pub enum SearchEntity {
    /// Search genes by symbol, name, type, or chromosome (MyGene.info)
    #[command(after_help = "\
EXAMPLES:
  biomcp search gene BRAF
  biomcp search gene -q kinase --type protein-coding --region chr7:140424943-140624564 --limit 5

See also: biomcp list gene")]
    Gene {
        /// Free text query (gene name, symbol, or keyword)
        #[arg(short, long)]
        query: Option<String>,
        /// Optional positional query alias for -q/--query
        #[arg(value_name = "QUERY")]
        positional_query: Option<String>,
        /// Filter by gene type (e.g., protein-coding, ncRNA, pseudo)
        #[arg(long = "type")]
        gene_type: Option<String>,
        /// Filter by chromosome (e.g., 7, X)
        #[arg(long)]
        chromosome: Option<String>,
        /// Filter by genomic region (chr:start-end)
        #[arg(long)]
        region: Option<String>,
        /// Filter by pathway ID/name (e.g., R-HSA-5673001)
        #[arg(long)]
        pathway: Option<String>,
        /// Filter by GO term ID/text (e.g., GO:0004672)
        #[arg(long = "go")]
        go_term: Option<String>,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Search diseases by name or ontology (Monarch/MONDO)
    #[command(after_help = "\
EXAMPLES:
  biomcp search disease \"lung cancer\"
  biomcp search disease -q melanoma --inheritance \"autosomal dominant\" --phenotype HP:0001250 --onset adult --limit 5

See also: biomcp list disease")]
    Disease {
        /// Free text query (disease name or keyword)
        #[arg(short, long)]
        query: Option<String>,
        /// Optional positional query alias for -q/--query
        #[arg(value_name = "QUERY")]
        positional_query: Option<String>,
        /// Restrict results by ontology source (mondo, doid, mesh)
        #[arg(long)]
        source: Option<String>,
        /// Filter by inheritance pattern
        #[arg(long)]
        inheritance: Option<String>,
        /// Filter by phenotype term (e.g., HP:0001250)
        #[arg(long)]
        phenotype: Option<String>,
        /// Filter by clinical onset period
        #[arg(long)]
        onset: Option<String>,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Search pharmacogenomic interactions
    #[command(after_help = "\
EXAMPLES:
  biomcp search pgx -g CYP2D6
  biomcp search pgx -d warfarin --cpic-level A

See also: biomcp list pgx")]
    Pgx {
        /// Filter by gene symbol
        #[arg(short = 'g', long)]
        gene: Option<String>,
        /// Filter by drug name
        #[arg(short = 'd', long)]
        drug: Option<String>,
        /// Filter by CPIC level (A/B/C/D)
        #[arg(long = "cpic-level")]
        cpic_level: Option<String>,
        /// Filter by PGx testing recommendation
        #[arg(long = "pgx-testing")]
        pgx_testing: Option<String>,
        /// Filter by evidence level (best-effort)
        #[arg(long)]
        evidence: Option<String>,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Search disease matches from an HPO term set (Monarch semsim)
    #[command(after_help = "\
EXAMPLES:
  biomcp search phenotype \"HP:0001250 HP:0001263\"
  biomcp search phenotype \"HP:0001250\" --limit 5

See also: biomcp list disease")]
    Phenotype {
        /// HPO term list (space- or comma-separated, e.g., \"HP:0001250 HP:0001263\")
        terms: String,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Search GWAS associations by gene or trait
    #[command(after_help = "\
EXAMPLES:
  biomcp search gwas -g TCF7L2
  biomcp search gwas --trait EFO_0000305 --region 7:140000000-141000000 --p-value 5e-8

See also: biomcp list gwas")]
    Gwas {
        /// Filter by gene symbol
        #[arg(short = 'g', long)]
        gene: Option<String>,
        /// Filter by disease trait text
        #[arg(long = "trait")]
        trait_query: Option<String>,
        /// Filter by genomic region (chr:start-end)
        #[arg(long)]
        region: Option<String>,
        /// Filter by p-value threshold
        #[arg(long = "p-value")]
        p_value: Option<f64>,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Search articles by gene, disease, drug, keyword, or author (PubMed/PubTator3)
    #[command(after_help = "\
EXAMPLES:
  biomcp search article \"BRAF resistance\"
  biomcp search article -q \"immunotherapy resistance\" --limit 5
  biomcp search article -g BRAF --date-from 2024-01-01
  biomcp search article -d melanoma --type review --journal Nature --limit 5

See also: biomcp list article")]
    Article {
        /// Filter by gene symbol
        #[arg(short, long)]
        gene: Option<String>,

        /// Filter by disease name
        #[arg(short, long)]
        disease: Option<String>,

        /// Filter by drug/chemical name
        #[arg(long)]
        drug: Option<String>,

        /// Filter by author name
        #[arg(short = 'a', long)]
        author: Option<String>,

        /// Free text keyword search (alias: -q, --query)
        #[arg(
            short = 'k',
            long = "keyword",
            visible_short_alias = 'q',
            visible_alias = "query"
        )]
        keyword: Option<String>,
        /// Optional positional query alias for -k/--keyword/--query
        #[arg(value_name = "QUERY")]
        positional_query: Option<String>,

        /// Published after date (YYYY-MM-DD)
        #[arg(long = "date-from", alias = "since")]
        date_from: Option<String>,
        /// Published before date (YYYY-MM-DD)
        #[arg(long = "date-to")]
        date_to: Option<String>,

        // `long = "type"` is used instead of deriving from the field name because
        // `type` is a Rust reserved keyword. Internally we use `article_type`.
        /// Filter by publication type [values: research-article, review, case-reports, meta-analysis]
        #[arg(long = "type")]
        article_type: Option<String>,
        /// Filter by journal title
        #[arg(long)]
        journal: Option<String>,

        /// Restrict to open-access articles (default: off, includes all access models)
        #[arg(long = "open-access")]
        open_access: bool,

        /// Exclude preprints (best-effort; default: off, includes preprints)
        #[arg(long)]
        no_preprints: bool,

        /// Exclude retracted publications from search results
        #[arg(long)]
        exclude_retracted: bool,
        /// Include retracted publications in search results (default excludes them)
        #[arg(long, conflicts_with = "exclude_retracted")]
        include_retracted: bool,

        /// Sort order [values: date, citations, relevance] (default: date)
        #[arg(long, default_value = "date", value_parser = ["date", "citations", "relevance"])]
        sort: String,

        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Search trials by condition, intervention, mutation, or location (ClinicalTrials.gov)
    #[command(after_help = "\
EXAMPLES:
  biomcp search trial -c melanoma -s recruiting
  biomcp search trial -p 3 -i pembrolizumab
  biomcp search trial -c melanoma --facility \"MD Anderson\" --age 67 --limit 5
  biomcp search trial --mutation \"BRAF V600E\" --status recruiting --study-type interventional --has-results --limit 5

Trial search is filter-based (no free-text query).
See also: biomcp list trial")]
    Trial {
        /// Filter by condition/disease
        #[arg(short = 'c', long)]
        condition: Option<String>,

        /// Filter by intervention/drug
        #[arg(short = 'i', long)]
        intervention: Option<String>,

        /// Filter by institution/facility name
        #[arg(long)]
        facility: Option<String>,

        /// Filter by phase [values: NA, 1, 1/2, 2, 3, 4, EARLY_PHASE1, PHASE1-PHASE4]
        #[arg(short = 'p', long)]
        phase: Option<String>,
        /// Study type (e.g., interventional, observational)
        #[arg(long = "study-type")]
        study_type: Option<String>,

        /// Patient age in years for eligibility matching
        #[arg(long)]
        age: Option<u32>,

        /// Eligible sex [values: female, male, all]
        #[arg(long)]
        sex: Option<String>,

        /// Filter by trial status [values: recruiting, not_yet_recruiting, enrolling_by_invitation, active_not_recruiting, completed, suspended, terminated, withdrawn]
        #[arg(short = 's', long)]
        status: Option<String>,

        /// Search eligibility criteria for mutation (best-effort)
        #[arg(long)]
        mutation: Option<String>,

        /// Biomarker filter (NCI CTS; best-effort for ctgov)
        #[arg(long)]
        biomarker: Option<String>,

        /// Prior therapy mentioned in eligibility
        #[arg(long)]
        prior_therapies: Option<String>,

        /// Drug/therapy patient progressed on
        #[arg(long)]
        progression_on: Option<String>,

        /// Line of therapy: 1L, 2L, 3L+
        #[arg(long)]
        line_of_therapy: Option<String>,

        /// Filter by sponsor (best-effort)
        #[arg(long)]
        sponsor: Option<String>,

        /// Sponsor/funder category [values: nih, industry, fed, other]
        #[arg(long = "sponsor-type")]
        sponsor_type: Option<String>,

        /// Trials updated after date (YYYY-MM-DD)
        #[arg(long = "date-from", alias = "since")]
        date_from: Option<String>,
        /// Trials updated before date (YYYY-MM-DD)
        #[arg(long = "date-to")]
        date_to: Option<String>,

        /// Latitude for geographic search
        #[arg(long, allow_hyphen_values = true)]
        lat: Option<f64>,

        /// Longitude for geographic search
        #[arg(long, allow_hyphen_values = true)]
        lon: Option<f64>,

        /// Distance (miles) for geographic search
        #[arg(long)]
        distance: Option<u32>,

        /// Only return trials with posted results (default: off, include trials with/without posted results)
        #[arg(long = "has-results", visible_alias = "results-available")]
        results_available: bool,

        /// Return only total count (no result table)
        #[arg(long = "count-only")]
        count_only: bool,

        /// Trial data source (ctgov or nci)
        #[arg(long, default_value = "ctgov")]
        source: String,

        /// Skip the first N results (pagination)
        #[arg(long, default_value = "0")]
        offset: usize,

        /// Cursor token from a previous response
        #[arg(long = "next-page")]
        next_page: Option<String>,

        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Search variants by gene, significance, frequency, or consequence (ClinVar/gnomAD)
    #[command(after_help = "\
EXAMPLES:
  biomcp search variant BRAF --limit 5
  biomcp search variant -g BRAF --significance pathogenic
  biomcp search variant -g BRCA1 --review-status 2 --revel-min 0.7 --consequence missense_variant --limit 5
  biomcp search variant --hgvsp V600E -g BRAF --limit 5

For variant mentions in trials: biomcp variant trials \"BRAF V600E\"
See also: biomcp list variant")]
    Variant {
        /// Filter by gene symbol
        #[arg(short = 'g', long)]
        gene: Option<String>,
        /// Optional positional query alias for -g/--gene
        #[arg(value_name = "QUERY")]
        positional_query: Option<String>,

        /// Filter by protein change (e.g., V600E or p.V600E)
        #[arg(long)]
        hgvsp: Option<String>,

        /// ClinVar significance (e.g., pathogenic, benign, uncertain)
        #[arg(long)]
        significance: Option<String>,

        /// Max gnomAD allele frequency (0-1)
        #[arg(long)]
        max_frequency: Option<f64>,

        /// Min CADD score (>=0)
        #[arg(long)]
        min_cadd: Option<f64>,

        /// Functional consequence filter (e.g., missense_variant)
        #[arg(long)]
        consequence: Option<String>,
        /// ClinVar review status filter (e.g., 2, expert_panel)
        #[arg(long = "review-status")]
        review_status: Option<String>,
        /// Population AF scope (afr, amr, eas, fin, nfe, sas)
        #[arg(long)]
        population: Option<String>,
        /// Minimum REVEL score
        #[arg(long = "revel-min")]
        revel_min: Option<f64>,
        /// Minimum GERP score
        #[arg(long = "gerp-min")]
        gerp_min: Option<f64>,
        /// Filter by COSMIC tumor site
        #[arg(long = "tumor-site")]
        tumor_site: Option<String>,
        /// Filter by ClinVar condition
        #[arg(long)]
        condition: Option<String>,
        /// Filter by SnpEff impact (HIGH/MODERATE/LOW/MODIFIER)
        #[arg(long)]
        impact: Option<String>,
        /// Restrict to loss-of-function variants
        #[arg(long)]
        lof: bool,
        /// Require presence of a field
        #[arg(long)]
        has: Option<String>,
        /// Require missing field
        #[arg(long)]
        missing: Option<String>,
        /// Filter CIViC therapy name
        #[arg(long)]
        therapy: Option<String>,

        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Search drugs by name, target, indication, or mechanism (MyChem.info)
    #[command(after_help = "\
EXAMPLES:
  biomcp search drug pembrolizumab
  biomcp search drug -q \"kinase inhibitor\" --target EGFR --atc L01 --pharm-class kinase --interactions warfarin --limit 5

See also: biomcp list drug")]
    Drug {
        /// Free text query (drug name, class, etc.)
        #[arg(short, long)]
        query: Option<String>,
        /// Optional positional query alias for -q/--query
        #[arg(value_name = "QUERY")]
        positional_query: Option<String>,

        /// Filter by target gene symbol
        #[arg(long)]
        target: Option<String>,

        /// Filter by indication/disease name
        #[arg(long)]
        indication: Option<String>,

        /// Filter by mechanism text
        #[arg(long)]
        mechanism: Option<String>,

        /// Filter by drug type (e.g., biologic, small-molecule)
        #[arg(long = "type")]
        drug_type: Option<String>,
        /// Filter by ATC code
        #[arg(long)]
        atc: Option<String>,
        /// Filter by pharmacologic class
        #[arg(long = "pharm-class")]
        pharm_class: Option<String>,
        /// Filter by interaction partner drug name
        #[arg(long)]
        interactions: Option<String>,

        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Search pathways by name or keyword (Reactome)
    #[command(after_help = "\
EXAMPLES:
  biomcp search pathway \"MAPK signaling\"
  biomcp search pathway -q \"DNA repair\" --type pathway --top-level --limit 5

See also: biomcp list pathway")]
    Pathway {
        /// Free text query (pathway name, process, keyword)
        #[arg(short, long)]
        query: Option<String>,
        /// Optional positional query alias for -q/--query
        #[arg(value_name = "QUERY")]
        positional_query: Option<String>,
        /// Entity type filter (e.g., pathway)
        #[arg(long = "type")]
        pathway_type: Option<String>,
        /// Include top-level pathways
        #[arg(long = "top-level")]
        top_level: bool,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Search proteins by name or accession (UniProt)
    #[command(after_help = "\
EXAMPLES:
  biomcp search protein kinase
  biomcp search protein -q \"BRAF\" --reviewed --disease melanoma --existence 1 --limit 5

See also: biomcp list protein")]
    Protein {
        /// Free text query (protein name, accession, keyword)
        #[arg(short, long)]
        query: Option<String>,
        /// Optional positional query alias for -q/--query
        #[arg(value_name = "QUERY")]
        positional_query: Option<String>,
        /// Include all species (default: off, human-only)
        #[arg(long)]
        all_species: bool,
        /// Restrict to reviewed entries
        #[arg(long)]
        reviewed: bool,
        /// Filter by disease text
        #[arg(long)]
        disease: Option<String>,
        /// Filter by protein existence level (1-5)
        #[arg(long)]
        existence: Option<u8>,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
        /// Cursor token from a previous response
        #[arg(long = "next-page")]
        next_page: Option<String>,
    },
    /// Search adverse event reports (OpenFDA FAERS)
    #[command(after_help = "\
EXAMPLES:
  biomcp search adverse-event -d pembrolizumab --reaction rash
  biomcp search adverse-event -d carboplatin --serious death --date-from 2020 --date-to 2024 --count patient.reaction.reactionmeddrapt
  biomcp search adverse-event --type recall -d nivolumab

See also: biomcp list adverse-event")]
    AdverseEvent {
        /// Drug name (required for FAERS queries)
        #[arg(short = 'd', long)]
        drug: Option<String>,

        /// Device name (required for --type device)
        #[arg(long)]
        device: Option<String>,

        /// Device manufacturer name (for --type device)
        #[arg(long)]
        manufacturer: Option<String>,

        /// Device product code (for --type device)
        #[arg(long = "product-code")]
        product_code: Option<String>,

        /// Filter by reaction term (MedDRA)
        #[arg(long)]
        reaction: Option<String>,

        /// Filter by reaction outcome [values: death, hospitalization, disability]
        #[arg(long)]
        outcome: Option<String>,

        /// Seriousness filter (optionally specify type: death, hospitalization, lifethreatening, disability, congenital, other)
        #[arg(long, num_args = 0..=1, default_missing_value = "any")]
        serious: Option<String>,

        /// Received after year/date (YYYY or YYYY-MM-DD)
        #[arg(long = "date-from", alias = "since")]
        date_from: Option<String>,
        /// Received before year/date (YYYY or YYYY-MM-DD)
        #[arg(long = "date-to")]
        date_to: Option<String>,
        /// Restrict to suspect drugs only
        #[arg(long = "suspect-only")]
        suspect_only: bool,
        /// Patient sex filter (m|f)
        #[arg(long)]
        sex: Option<String>,
        /// Minimum patient age
        #[arg(long = "age-min")]
        age_min: Option<u32>,
        /// Maximum patient age
        #[arg(long = "age-max")]
        age_max: Option<u32>,
        /// Reporter qualification filter
        #[arg(long)]
        reporter: Option<String>,
        /// Server-side count aggregation field
        #[arg(long)]
        count: Option<String>,

        /// Query type: faers (default), recall, or device
        #[arg(long, default_value = "faers")]
        r#type: String,

        /// Filter by recall classification (Class I, Class II, Class III)
        #[arg(long)]
        classification: Option<String>,

        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
}

#[derive(Subcommand, Debug)]
pub enum GetEntity {
    /// Get gene by symbol
    #[command(after_help = "\
EXAMPLES:
  biomcp get gene BRAF
  biomcp get gene BRAF pathways

See also: biomcp list gene")]
    Gene {
        /// Gene symbol (e.g., BRAF, TP53, EGFR)
        symbol: String,
        /// Sections to include (pathways, ontology, diseases, protein, go, interactions, all)
        #[arg(trailing_var_arg = true)]
        sections: Vec<String>,
    },
    /// Get article by PMID, PMCID, or DOI
    #[command(after_help = "\
EXAMPLES:
  biomcp get article 22663011
  biomcp get article 22663011 annotations

See also: biomcp list article")]
    Article {
        /// PMID (e.g., 22663011), PMCID (e.g., PMC9984800), or DOI (e.g., 10.1056/NEJMoa1203421)
        id: String,
        /// Sections to include (annotations, fulltext, all)
        #[arg(trailing_var_arg = true)]
        sections: Vec<String>,
    },
    /// Get disease by name or ID (e.g., MONDO:0005105)
    #[command(after_help = "\
EXAMPLES:
  biomcp get disease melanoma
  biomcp get disease MONDO:0005105 genes

See also: biomcp list disease")]
    Disease {
        /// Disease name (e.g., melanoma) or ID (e.g., MONDO:0005105)
        name_or_id: String,
        /// Sections to include (genes, pathways, phenotypes, variants, models, prevalence, civic, all)
        #[arg(trailing_var_arg = true)]
        sections: Vec<String>,
    },
    /// Get pharmacogenomics card by gene or drug (e.g., CYP2D6, warfarin)
    #[command(after_help = "\
EXAMPLES:
  biomcp get pgx CYP2D6
  biomcp get pgx warfarin recommendations

See also: biomcp list pgx")]
    Pgx {
        /// Gene symbol or drug name (e.g., CYP2D6, codeine)
        query: String,
        /// Sections to include (recommendations, frequencies, guidelines, annotations, all)
        #[arg(trailing_var_arg = true)]
        sections: Vec<String>,
    },
    /// Get trial by NCT ID (e.g., NCT02576665)
    #[command(after_help = "\
EXAMPLES:
  biomcp get trial NCT02576665
  biomcp get trial NCT02576665 eligibility --source ctgov
  biomcp get trial NCT02576665 locations --offset 20 --limit 20

See also: biomcp list trial")]
    Trial {
        /// ClinicalTrials.gov identifier (e.g., NCT02693535)
        nct_id: String,
        /// Sections to include (eligibility, locations, outcomes, arms, references, all)
        #[arg(trailing_var_arg = true)]
        sections: Vec<String>,
        /// Trial data source (ctgov or nci)
        #[arg(long, default_value = "ctgov")]
        source: String,
    },
    /// Get variant by rsID, HGVS, or "GENE CHANGE" (e.g., "BRAF V600E")
    #[command(after_help = "\
EXAMPLES:
  biomcp get variant rs113488022
  biomcp get variant \"BRAF V600E\" clinvar

See also: biomcp list variant")]
    Variant {
        /// rsID, HGVS, or "GENE CHANGE" (e.g., rs113488022, "BRAF V600E")
        id: String,
        /// Sections to include (predict, predictions, clinvar, population, conservation, cosmic, cgi, civic, cbioportal, gwas, all)
        #[arg(trailing_var_arg = true)]
        sections: Vec<String>,
    },
    /// Get drug by name
    #[command(after_help = "\
EXAMPLES:
  biomcp get drug pembrolizumab
  biomcp get drug pembrolizumab targets

See also: biomcp list drug")]
    Drug {
        /// Drug name (e.g., pembrolizumab, carboplatin)
        name: String,
        /// Sections to include (label, shortage, targets, indications, interactions, all)
        #[arg(trailing_var_arg = true)]
        sections: Vec<String>,
    },
    /// Get pathway by Reactome stable ID
    #[command(after_help = "\
EXAMPLES:
  biomcp get pathway R-HSA-5673001
  biomcp get pathway R-HSA-5673001 genes

See also: biomcp list pathway")]
    Pathway {
        /// Reactome stable ID (e.g., R-HSA-5673001)
        id: String,
        /// Sections to include (genes, events, enrichment, all)
        #[arg(trailing_var_arg = true)]
        sections: Vec<String>,
    },
    /// Get protein by UniProt accession or gene symbol
    #[command(after_help = "\
EXAMPLES:
  biomcp get protein P15056
  biomcp get protein P15056 structures

See also: biomcp list protein")]
    Protein {
        /// UniProt accession or HGNC symbol (e.g., P15056 or BRAF)
        accession: String,
        /// Sections to include (domains, interactions, structures, all)
        #[arg(trailing_var_arg = true)]
        sections: Vec<String>,
    },
    /// Get adverse event report by FAERS safetyreportid or MAUDE mdr_report_key
    #[command(after_help = "\
EXAMPLES:
  biomcp get adverse-event 10222779
  biomcp get adverse-event 10222779 reactions

See also: biomcp list adverse-event")]
    AdverseEvent {
        /// FAERS safetyreportid or MAUDE mdr_report_key
        report_id: String,
        /// Sections to include (reactions, outcomes, concomitant, guidance, all)
        #[arg(trailing_var_arg = true)]
        sections: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum VariantCommand {
    /// Search trials mentioning the variant in eligibility criteria (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp variant trials \"BRAF V600E\" --limit 5
  biomcp variant trials \"BRAF V600E\" --source nci --limit 5
  biomcp variant trials rs113488022 --limit 5

Note: Searches free-text fields (e.g., eligibility criteria). Results depend on source document wording.
See also: biomcp list variant")]
    Trials {
        /// Variant identifier (rsID, HGVS, or "GENE CHANGE")
        id: String,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
        /// Trial data source (ctgov or nci)
        #[arg(long, default_value = "ctgov")]
        source: String,
    },
    /// Search articles mentioning the variant (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp variant articles \"BRAF V600E\" --limit 5
  biomcp variant articles rs113488022 --limit 5

Note: Searches free-text fields (e.g., eligibility criteria). Results depend on source document wording.
See also: biomcp list variant")]
    Articles {
        /// Variant identifier (rsID, HGVS, or "GENE CHANGE")
        id: String,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Explicit OncoKB lookup for a variant (requires ONCOKB_TOKEN)
    #[command(after_help = "\
EXAMPLES:
  biomcp variant oncokb \"BRAF V600E\"
  biomcp variant oncokb rs121913529

See also: biomcp list variant")]
    Oncokb {
        /// Variant identifier (rsID, HGVS, or "GENE CHANGE")
        id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum DrugCommand {
    /// Search trials using this drug (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp drug trials pembrolizumab --limit 5
  biomcp drug trials osimertinib --source nci --limit 5

Note: Searches free-text fields (e.g., eligibility criteria). Results depend on source document wording.
See also: biomcp list drug")]
    Trials {
        /// Drug name (e.g., pembrolizumab)
        name: String,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
        /// Trial data source (ctgov or nci)
        #[arg(long, default_value = "ctgov")]
        source: String,
    },
    /// Search FAERS adverse events for this drug (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp drug adverse-events pembrolizumab --limit 5
  biomcp drug adverse-events carboplatin --serious --limit 5

Note: Searches free-text fields (e.g., eligibility criteria). Results depend on source document wording.
See also: biomcp list drug")]
    AdverseEvents {
        /// Drug name (e.g., pembrolizumab)
        name: String,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
        /// Serious reports only
        #[arg(long)]
        serious: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum DiseaseCommand {
    /// Search trials for this disease (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp disease trials melanoma --limit 5
  biomcp disease trials \"lung cancer\" --source nci --limit 5

Note: Searches free-text fields (e.g., eligibility criteria). Results depend on source document wording.
See also: biomcp list disease")]
    Trials {
        /// Disease name (e.g., melanoma)
        name: String,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
        /// Trial data source (ctgov or nci)
        #[arg(long, default_value = "ctgov")]
        source: String,
    },
    /// Search articles for this disease (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp disease articles melanoma --limit 5
  biomcp disease articles \"glioblastoma\" --limit 5

Note: Searches free-text fields (e.g., eligibility criteria). Results depend on source document wording.
See also: biomcp list disease")]
    Articles {
        /// Disease name (e.g., melanoma)
        name: String,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Search drugs with this disease as an indication (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp disease drugs melanoma --limit 5
  biomcp disease drugs \"breast cancer\" --limit 5

Note: Searches free-text fields (e.g., eligibility criteria). Results depend on source document wording.
See also: biomcp list disease")]
    Drugs {
        /// Disease name (e.g., melanoma)
        name: String,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
}

#[derive(Subcommand, Debug)]
pub enum ArticleCommand {
    /// Surface annotated entities from PubTator as discoverable commands
    #[command(after_help = "\
EXAMPLES:
  biomcp article entities 22663011
  biomcp article entities 22663011 --limit 5
  biomcp article entities 24200969

See also: biomcp list article")]
    Entities {
        /// PMID (e.g., 22663011)
        pmid: String,
        /// Maximum entities per category (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}

#[derive(Subcommand, Debug)]
pub enum GeneCommand {
    /// Show canonical gene definition card (same output as `get gene`)
    #[command(
        alias = "get",
        after_help = "\
EXAMPLES:
  biomcp gene definition BRAF
  biomcp gene get BRAF
  biomcp get gene BRAF

See also: biomcp list gene"
    )]
    Definition {
        /// HGNC gene symbol (e.g., BRAF)
        symbol: String,
    },
    /// Search trials linked to this gene symbol (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp gene trials BRAF --limit 5
  biomcp gene trials EGFR --source nci --limit 5

Note: Searches free-text fields (e.g., eligibility criteria). Results depend on source document wording.
See also: biomcp list gene")]
    Trials {
        /// HGNC gene symbol (e.g., BRAF)
        symbol: String,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
        /// Trial data source (ctgov or nci)
        #[arg(long, default_value = "ctgov")]
        source: String,
    },
    /// Search drugs targeting this gene symbol
    #[command(after_help = "\
EXAMPLES:
  biomcp gene drugs EGFR --limit 5
  biomcp gene drugs BRAF --limit 5

See also: biomcp list gene")]
    Drugs {
        /// HGNC gene symbol (e.g., BRAF)
        symbol: String,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Search articles mentioning this gene
    #[command(after_help = "\
EXAMPLES:
  biomcp gene articles BRAF --limit 5
  biomcp gene articles TP53 --limit 5

See also: biomcp list gene")]
    Articles {
        /// HGNC gene symbol (e.g., BRAF)
        symbol: String,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Show pathways section for this gene symbol
    #[command(after_help = "\
EXAMPLES:
  biomcp gene pathways BRAF
  biomcp gene pathways BRAF --limit 5 --offset 0
  biomcp gene pathways BRCA1

See also: biomcp list gene")]
    Pathways {
        /// HGNC gene symbol (e.g., BRAF)
        symbol: String,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
}

#[derive(Subcommand, Debug)]
pub enum PathwayCommand {
    /// Search drugs linked to genes in this pathway (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp pathway drugs R-HSA-5673001 --limit 5
  biomcp pathway drugs R-HSA-6802957 --limit 5

Note: Searches free-text fields (e.g., eligibility criteria). Results depend on source document wording.
See also: biomcp list pathway")]
    Drugs {
        /// Reactome stable ID (e.g., R-HSA-5673001)
        id: String,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Search articles linked to this pathway (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp pathway articles R-HSA-5673001 --limit 5
  biomcp pathway articles R-HSA-6802957 --limit 5

Note: Searches free-text fields (e.g., eligibility criteria). Results depend on source document wording.
See also: biomcp list pathway")]
    Articles {
        /// Reactome stable ID (e.g., R-HSA-5673001)
        id: String,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Search trials linked to this pathway (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp pathway trials R-HSA-5673001 --limit 5
  biomcp pathway trials R-HSA-5673001 --source nci --limit 5

Note: Searches free-text fields (e.g., eligibility criteria). Results depend on source document wording.
See also: biomcp list pathway")]
    Trials {
        /// Reactome stable ID (e.g., R-HSA-5673001)
        id: String,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
        /// Trial data source (ctgov or nci)
        #[arg(long, default_value = "ctgov")]
        source: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProteinCommand {
    /// Show protein structural identifiers
    #[command(after_help = "\
EXAMPLES:
  biomcp protein structures P15056
  biomcp protein structures P15056 --limit 25 --offset 5

See also: biomcp list protein")]
    Structures {
        /// UniProt accession or HGNC symbol (e.g., P15056 or BRAF)
        accession: String,
        /// Maximum structures to show (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
}

fn empty_sections() -> &'static [String] {
    &[]
}

fn parse_batch_sections(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn extract_json_from_sections(sections: &[String]) -> (Vec<String>, bool) {
    let mut json_override = false;
    let cleaned = sections
        .iter()
        .filter_map(|raw| {
            let trimmed = raw.trim();
            let normalized = trimmed.to_ascii_lowercase();
            if normalized == "--json" || normalized == "-j" {
                json_override = true;
                return None;
            }
            if trimmed.is_empty() {
                return None;
            }
            Some(trimmed.to_string())
        })
        .collect();
    (cleaned, json_override)
}

fn parse_usize_arg(flag: &str, value: &str) -> Result<usize, crate::error::BioMcpError> {
    value.parse::<usize>().map_err(|_| {
        crate::error::BioMcpError::InvalidArgument(format!("{flag} must be a non-negative integer"))
    })
}

type LocationPaging = (Vec<String>, Option<usize>, Option<usize>);

fn parse_trial_location_paging(
    sections: &[String],
) -> Result<LocationPaging, crate::error::BioMcpError> {
    let mut cleaned: Vec<String> = Vec::new();
    let mut location_offset: Option<usize> = None;
    let mut location_limit: Option<usize> = None;
    let mut i = 0usize;
    while i < sections.len() {
        let token = sections[i].trim();
        if token.is_empty() {
            i += 1;
            continue;
        }

        if let Some(value) = token.strip_prefix("--offset=") {
            location_offset = Some(parse_usize_arg("--offset", value)?);
            i += 1;
            continue;
        }
        if token == "--offset" {
            let value = sections.get(i + 1).ok_or_else(|| {
                crate::error::BioMcpError::InvalidArgument(
                    "--offset requires a value for trial location pagination".into(),
                )
            })?;
            location_offset = Some(parse_usize_arg("--offset", value.trim())?);
            i += 2;
            continue;
        }
        if let Some(value) = token.strip_prefix("--limit=") {
            location_limit = Some(parse_usize_arg("--limit", value)?);
            i += 1;
            continue;
        }
        if token == "--limit" {
            let value = sections.get(i + 1).ok_or_else(|| {
                crate::error::BioMcpError::InvalidArgument(
                    "--limit requires a value for trial location pagination".into(),
                )
            })?;
            location_limit = Some(parse_usize_arg("--limit", value.trim())?);
            i += 2;
            continue;
        }
        cleaned.push(sections[i].clone());
        i += 1;
    }

    if location_limit.is_some_and(|value| value == 0) {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--limit must be >= 1 for trial location pagination".into(),
        ));
    }

    Ok((cleaned, location_offset, location_limit))
}

fn normalize_cli_query(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn resolve_query_input(
    flag_query: Option<String>,
    positional_query: Option<String>,
    flag_names: &str,
) -> Result<Option<String>, crate::error::BioMcpError> {
    let flag_query = normalize_cli_query(flag_query);
    let positional_query = normalize_cli_query(positional_query);
    match (flag_query, positional_query) {
        (Some(_), Some(_)) => Err(crate::error::BioMcpError::InvalidArgument(format!(
            "Use either positional QUERY or {flag_names}, not both"
        ))),
        (Some(value), None) | (None, Some(value)) => Ok(Some(value)),
        (None, None) => Ok(None),
    }
}

async fn render_gene_card(
    symbol: &str,
    sections: &[String],
    json_output: bool,
) -> anyhow::Result<String> {
    let gene = crate::entities::gene::get(symbol, sections).await?;
    if json_output {
        Ok(crate::render::json::to_pretty(&gene)?)
    } else {
        Ok(crate::render::markdown::gene_markdown(&gene, sections)?)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct LocationPaginationMeta {
    total: usize,
    offset: usize,
    limit: usize,
    has_more: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PaginationMeta {
    pub offset: usize,
    pub limit: usize,
    pub returned: usize,
    pub total: Option<usize>,
    pub has_more: bool,
    pub next_page_token: Option<String>,
}

impl PaginationMeta {
    fn offset(offset: usize, limit: usize, returned: usize, total: Option<usize>) -> Self {
        let has_more = total
            .map(|value| offset.saturating_add(returned) < value)
            .unwrap_or(returned == limit);
        Self {
            offset,
            limit,
            returned,
            total,
            has_more,
            next_page_token: None,
        }
    }

    fn cursor(
        offset: usize,
        limit: usize,
        returned: usize,
        total: Option<usize>,
        next_page_token: Option<String>,
    ) -> Self {
        let has_token = next_page_token
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty());
        let has_more = match total {
            Some(value) => offset.saturating_add(returned) < value || has_token,
            None => has_token,
        };
        Self {
            offset,
            limit,
            returned,
            total,
            has_more,
            next_page_token,
        }
    }
}

#[derive(serde::Serialize)]
struct SearchJsonResponse<T: serde::Serialize> {
    pagination: PaginationMeta,
    count: usize,
    results: Vec<T>,
}

fn search_json<T: serde::Serialize>(
    results: Vec<T>,
    pagination: PaginationMeta,
) -> anyhow::Result<String> {
    let count = results.len();
    crate::render::json::to_pretty(&SearchJsonResponse {
        pagination,
        count,
        results,
    })
    .map_err(Into::into)
}

fn pagination_footer_offset(meta: &PaginationMeta) -> String {
    crate::render::markdown::pagination_footer(
        crate::render::markdown::PaginationFooterMode::Offset,
        meta.offset,
        meta.limit,
        meta.returned,
        meta.total,
        None,
    )
}

fn pagination_footer_cursor(meta: &PaginationMeta) -> String {
    crate::render::markdown::pagination_footer(
        crate::render::markdown::PaginationFooterMode::Cursor,
        meta.offset,
        meta.limit,
        meta.returned,
        meta.total,
        meta.next_page_token.as_deref(),
    )
}

fn paged_fetch_limit(
    limit: usize,
    offset: usize,
    max_limit: usize,
) -> Result<usize, crate::error::BioMcpError> {
    if limit == 0 || limit > max_limit {
        return Err(crate::error::BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {max_limit}"
        )));
    }
    Ok(limit.saturating_add(offset).min(max_limit))
}

fn truncate_article_annotations(
    mut annotations: crate::entities::article::ArticleAnnotations,
    limit: usize,
) -> crate::entities::article::ArticleAnnotations {
    annotations.genes.truncate(limit);
    annotations.diseases.truncate(limit);
    annotations.chemicals.truncate(limit);
    annotations.mutations.truncate(limit);
    annotations
}

fn paginate_results<T>(rows: Vec<T>, offset: usize, limit: usize) -> (Vec<T>, usize) {
    let total = rows.len();
    let paged = rows.into_iter().skip(offset).take(limit).collect();
    (paged, total)
}

fn version_output() -> String {
    let cargo_version = env!("CARGO_PKG_VERSION");
    let git_tag = option_env!("BIOMCP_BUILD_GIT_TAG");
    let git = option_env!("BIOMCP_BUILD_GIT_SHA").unwrap_or("unknown");
    let build = option_env!("BIOMCP_BUILD_DATE").unwrap_or("unknown");
    let version = git_tag
        .filter(|t| t.starts_with('v') && !t.contains('-'))
        .map(|t| &t[1..])
        .unwrap_or(cargo_version);
    format!("biomcp {version} (git {git}, build {build})")
}

fn log_pagination_truncation(observed_total: usize, offset: usize, returned: usize) {
    if offset.saturating_add(returned) < observed_total {
        debug!(
            total = observed_total,
            offset, returned, "Results truncated by --limit"
        );
    }
}

fn should_try_pathway_trial_fallback(
    results_len: usize,
    offset: usize,
    total: Option<u32>,
) -> bool {
    if results_len != 0 || offset > 0 {
        return false;
    }
    total.is_none_or(|value| value == 0)
}

fn trial_search_query_summary(
    filters: &crate::entities::trial::TrialSearchFilters,
    offset: usize,
    next_page: Option<&str>,
) -> String {
    vec![
        filters
            .condition
            .as_deref()
            .map(|v| format!("condition={v}")),
        filters
            .intervention
            .as_deref()
            .map(|v| format!("intervention={v}")),
        filters.facility.as_deref().map(|v| format!("facility={v}")),
        filters.age.map(|v| format!("age={v}")),
        filters.sex.as_deref().map(|v| format!("sex={v}")),
        filters.status.as_deref().map(|v| format!("status={v}")),
        filters.phase.as_deref().map(|v| format!("phase={v}")),
        filters
            .study_type
            .as_deref()
            .map(|v| format!("study_type={v}")),
        filters.sponsor.as_deref().map(|v| format!("sponsor={v}")),
        filters
            .sponsor_type
            .as_deref()
            .map(|v| format!("sponsor_type={v}")),
        filters
            .date_from
            .as_deref()
            .map(|v| format!("date_from={v}")),
        filters.date_to.as_deref().map(|v| format!("date_to={v}")),
        filters.mutation.as_deref().map(|v| format!("mutation={v}")),
        filters
            .biomarker
            .as_deref()
            .map(|v| format!("biomarker={v}")),
        filters
            .prior_therapies
            .as_deref()
            .map(|v| format!("prior_therapies={v}")),
        filters
            .progression_on
            .as_deref()
            .map(|v| format!("progression_on={v}")),
        filters
            .line_of_therapy
            .as_deref()
            .map(|v| format!("line_of_therapy={v}")),
        filters.lat.map(|v| format!("lat={v}")),
        filters.lon.map(|v| format!("lon={v}")),
        filters.distance.map(|v| format!("distance={v}")),
        filters
            .results_available
            .then(|| "has_results=true".to_string()),
        (offset > 0).then(|| format!("offset={offset}")),
        next_page
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| format!("next_page={value}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(", ")
}

fn amino_acid_one_letter(code: &str) -> Option<char> {
    match code.trim().to_ascii_uppercase().as_str() {
        "A" | "ALA" => Some('A'),
        "R" | "ARG" => Some('R'),
        "N" | "ASN" => Some('N'),
        "D" | "ASP" => Some('D'),
        "C" | "CYS" => Some('C'),
        "Q" | "GLN" => Some('Q'),
        "E" | "GLU" => Some('E'),
        "G" | "GLY" => Some('G'),
        "H" | "HIS" => Some('H'),
        "I" | "ILE" => Some('I'),
        "L" | "LEU" => Some('L'),
        "K" | "LYS" => Some('K'),
        "M" | "MET" => Some('M'),
        "F" | "PHE" => Some('F'),
        "P" | "PRO" => Some('P'),
        "S" | "SER" => Some('S'),
        "T" | "THR" => Some('T'),
        "W" | "TRP" => Some('W'),
        "Y" | "TYR" => Some('Y'),
        "V" | "VAL" => Some('V'),
        "*" | "TER" | "STOP" => Some('*'),
        _ => None,
    }
}

fn normalize_protein_change(value: &str) -> String {
    let trimmed = value
        .trim()
        .trim_start_matches("p.")
        .trim_start_matches("P.");
    if trimmed.is_empty() {
        return String::new();
    }

    let bytes = trimmed.as_bytes();
    let Some(start_digits) = bytes.iter().position(|b| b.is_ascii_digit()) else {
        return trimmed.to_string();
    };
    let end_digits = bytes[start_digits..]
        .iter()
        .position(|b| !b.is_ascii_digit())
        .map(|i| start_digits + i)
        .unwrap_or(bytes.len());

    if end_digits <= start_digits {
        return trimmed.to_string();
    }

    let from = &trimmed[..start_digits];
    let pos = &trimmed[start_digits..end_digits];
    let to = &trimmed[end_digits..];

    let Some(from_aa) = amino_acid_one_letter(from) else {
        return trimmed.to_string();
    };
    let Some(to_aa) = amino_acid_one_letter(to) else {
        return trimmed.to_string();
    };

    format!("{from_aa}{pos}{to_aa}")
}

async fn variant_trial_mutation_query(id: &str) -> String {
    let id = id.trim();
    if id.is_empty() {
        return String::new();
    }

    if let Ok(crate::entities::variant::VariantIdFormat::GeneProteinChange { gene, change }) =
        crate::entities::variant::parse_variant_id(id)
    {
        let normalized = normalize_protein_change(&change);
        if !normalized.is_empty() {
            return format!("{gene} {normalized}");
        }
    }

    if let Ok(variant) = crate::entities::variant::get(id, empty_sections()).await {
        let gene = variant.gene.trim();
        let protein = variant
            .hgvs_p
            .as_deref()
            .map(normalize_protein_change)
            .unwrap_or_default();
        if !gene.is_empty() && !protein.is_empty() {
            return format!("{gene} {protein}");
        }
    }

    id.to_string()
}

async fn pathway_drug_results(
    id: &str,
    fetch_limit: usize,
) -> Result<Vec<crate::entities::drug::DrugSearchResult>, crate::error::BioMcpError> {
    let sections = vec!["genes".to_string()];
    let pathway = crate::entities::pathway::get(id, &sections).await?;

    let search_limit = fetch_limit.clamp(1, 10);
    let mut stream = futures::stream::iter(pathway.genes.into_iter().map(|gene| async move {
        let filters = crate::entities::drug::DrugSearchFilters {
            target: Some(gene.clone()),
            ..Default::default()
        };
        let result = crate::entities::drug::search(&filters, search_limit).await;
        (gene, result)
    }))
    .buffer_unordered(5);

    let mut results: Vec<Vec<crate::entities::drug::DrugSearchResult>> = Vec::new();
    let mut attempted: usize = 0;
    let mut failures: usize = 0;
    while let Some((gene, next)) = stream.next().await {
        attempted += 1;
        match next {
            Ok(rows) => results.push(rows),
            Err(err) => {
                failures += 1;
                warn!(gene = %gene, "pathway drug lookup failed: {err}");
            }
        }
    }

    if attempted > 0 && failures.saturating_mul(2) > attempted {
        return Err(crate::error::BioMcpError::Api {
            api: "pathway-drugs".into(),
            message: format!(
                "Failed to resolve {failures} of {attempted} pathway gene target lookups while collecting drugs"
            ),
        });
    }

    let mut out: Vec<crate::entities::drug::DrugSearchResult> = Vec::new();
    for rows in results {
        for row in rows {
            if out.iter().any(|v| v.name.eq_ignore_ascii_case(&row.name)) {
                continue;
            }
            out.push(row);
            if out.len() >= fetch_limit {
                return Ok(out);
            }
        }
    }

    Ok(out)
}

fn uninstall_self() -> Result<String, crate::error::BioMcpError> {
    let current = std::env::current_exe()?;
    match std::fs::remove_file(&current) {
        Ok(()) => Ok(format!("Uninstalled biomcp from {}", current.display())),
        Err(err) => Ok(format!(
            "Unable to remove running binary automatically ({err}).\nRemove manually:\n  rm {}",
            current.display()
        )),
    }
}

fn enrich_markdown(genes: &[String], terms: &[crate::sources::gprofiler::GProfilerTerm]) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Enrichment: {}\n\n", genes.join(", ")));
    if terms.is_empty() {
        out.push_str("No enriched terms found.\n");
        return out;
    }

    out.push_str("| Source | ID | Name | p-value |\n");
    out.push_str("|--------|----|------|---------|\n");
    for row in terms {
        let source = row.source.as_deref().unwrap_or("-");
        let id = row.native.as_deref().unwrap_or("-");
        let name = row.name.as_deref().unwrap_or("-");
        let p = row
            .p_value
            .map(|v| format!("{v:.3e}"))
            .unwrap_or_else(|| "-".to_string());
        out.push_str(&format!("| {source} | {id} | {name} | {p} |\n"));
    }
    out
}

/// Executes one parsed CLI command and returns rendered output.
///
/// # Errors
///
/// Returns an error if argument validation fails, downstream entity operations fail,
/// rendering fails, or external API requests fail.
pub async fn run(cli: Cli) -> anyhow::Result<String> {
    let no_cache = cli.no_cache;
    crate::sources::with_no_cache(no_cache, async move {
        match cli.command {
            Commands::Get {
                entity: GetEntity::Gene { symbol, sections },
            } => {
                let (sections, json_override) = extract_json_from_sections(&sections);
                let json_output = cli.json || json_override;
                render_gene_card(&symbol, &sections, json_output).await
            }
            Commands::Get {
                entity: GetEntity::Article { id, sections },
            } => {
                let (sections, json_override) = extract_json_from_sections(&sections);
                let json_output = cli.json || json_override;
                let article = crate::entities::article::get(&id, &sections).await?;
                if json_output {
                    Ok(crate::render::json::to_pretty(&article)?)
                } else {
                    Ok(crate::render::markdown::article_markdown(&article, &sections)?)
                }
            }
            Commands::Get {
                entity:
                    GetEntity::Disease {
                        name_or_id,
                        sections,
                    },
            } => {
                let (sections, json_override) = extract_json_from_sections(&sections);
                let json_output = cli.json || json_override;
                let disease = crate::entities::disease::get(&name_or_id, &sections).await?;
                if json_output {
                    Ok(crate::render::json::to_pretty(&disease)?)
                } else {
                    Ok(crate::render::markdown::disease_markdown(&disease, &sections)?)
                }
            }
            Commands::Get {
                entity: GetEntity::Pgx { query, sections },
            } => {
                let (sections, json_override) = extract_json_from_sections(&sections);
                let json_output = cli.json || json_override;
                let pgx = crate::entities::pgx::get(&query, &sections).await?;
                if json_output {
                    Ok(crate::render::json::to_pretty(&pgx)?)
                } else {
                    Ok(crate::render::markdown::pgx_markdown(&pgx, &sections)?)
                }
            }
            Commands::Get {
                entity:
                    GetEntity::Trial {
                        nct_id,
                        sections,
                        source,
                    },
            } => {
                let (sections, location_offset, location_limit) =
                    parse_trial_location_paging(&sections)?;
                let (sections, json_override) = extract_json_from_sections(&sections);
                let json_output = cli.json || json_override;
                let trial_source = crate::entities::trial::TrialSource::from_flag(&source)?;
                let includes_locations = sections
                    .iter()
                    .any(|section| section.trim().eq_ignore_ascii_case("locations"));
                if !includes_locations
                    && (location_offset.is_some() || location_limit.is_some())
                {
                    return Err(crate::error::BioMcpError::InvalidArgument(
                        "--offset and --limit are only valid with the 'locations' section".into(),
                    )
                    .into());
                }
                let mut trial =
                    crate::entities::trial::get(&nct_id, &sections, trial_source).await?;
                let mut location_pagination: Option<LocationPaginationMeta> = None;
                if includes_locations {
                    let offset = location_offset.unwrap_or(0);
                    let limit = location_limit.unwrap_or(20);
                    if let Some(locations) = trial.locations.take() {
                        let total = locations.len();
                        let paged: Vec<_> =
                            locations.into_iter().skip(offset).take(limit).collect();
                        let has_more = offset + paged.len() < total;
                        trial.locations = Some(paged);
                        location_pagination = Some(LocationPaginationMeta {
                            total,
                            offset,
                            limit,
                            has_more,
                        });
                    }
                }
                if json_output {
                    if let Some(loc_page) = location_pagination {
                        #[derive(serde::Serialize)]
                        struct TrialWithLocationPagination {
                            #[serde(flatten)]
                            trial: crate::entities::trial::Trial,
                            location_pagination: LocationPaginationMeta,
                        }
                        Ok(crate::render::json::to_pretty(
                            &TrialWithLocationPagination {
                                trial,
                                location_pagination: loc_page,
                            },
                        )?)
                    } else {
                        Ok(crate::render::json::to_pretty(&trial)?)
                    }
                } else {
                    let mut md =
                        crate::render::markdown::trial_markdown(&trial, &sections)?;
                    if let Some(loc_page) = location_pagination {
                        md.push_str(&format!(
                            "\n\n---\n*Locations: showing {} of {} (offset {}, limit {}{})*",
                            trial.locations.as_ref().map_or(0, |v| v.len()),
                            loc_page.total,
                            loc_page.offset,
                            loc_page.limit,
                            if loc_page.has_more {
                                ", more available"
                            } else {
                                ""
                            },
                        ));
                    }
                    Ok(md)
                }
            }
            Commands::Get {
                entity: GetEntity::Variant { id, sections },
            } => {
                let (sections, json_override) = extract_json_from_sections(&sections);
                let json_output = cli.json || json_override;
                let variant = crate::entities::variant::get(&id, &sections).await?;
                if json_output {
                    Ok(crate::render::json::to_pretty(&variant)?)
                } else {
                    Ok(crate::render::markdown::variant_markdown(&variant, &sections)?)
                }
            }
            Commands::Get {
                entity: GetEntity::Drug { name, sections },
            } => {
                let (sections, json_override) = extract_json_from_sections(&sections);
                let json_output = cli.json || json_override;
                let drug = crate::entities::drug::get(&name, &sections).await?;
                if json_output {
                    Ok(crate::render::json::to_pretty(&drug)?)
                } else {
                    Ok(crate::render::markdown::drug_markdown(&drug, &sections)?)
                }
            }
            Commands::Get {
                entity: GetEntity::Pathway { id, sections },
            } => {
                let (sections, json_override) = extract_json_from_sections(&sections);
                let json_output = cli.json || json_override;
                let pathway = crate::entities::pathway::get(&id, &sections).await?;
                if json_output {
                    Ok(crate::render::json::to_pretty(&pathway)?)
                } else {
                    Ok(crate::render::markdown::pathway_markdown(&pathway, &sections)?)
                }
            }
            Commands::Get {
                entity: GetEntity::Protein {
                    accession,
                    sections,
                },
            } => {
                let (sections, json_override) = extract_json_from_sections(&sections);
                let json_output = cli.json || json_override;
                let protein = crate::entities::protein::get(&accession, &sections).await?;
                if json_output {
                    Ok(crate::render::json::to_pretty(&protein)?)
                } else {
                    Ok(crate::render::markdown::protein_markdown(&protein, &sections)?)
                }
            }
            Commands::Get {
                entity:
                    GetEntity::AdverseEvent {
                        report_id,
                        sections,
                    },
            } => {
                let (sections, json_override) = extract_json_from_sections(&sections);
                let json_output = cli.json || json_override;
                let event = crate::entities::adverse_event::get(&report_id).await?;
                if json_output {
                    return Ok(crate::render::json::to_pretty(&event)?);
                }
                match event {
                    crate::entities::adverse_event::AdverseEventReport::Faers(ref r) => {
                        Ok(crate::render::markdown::adverse_event_markdown(r, &sections)?)
                    }
                    crate::entities::adverse_event::AdverseEventReport::Device(ref r) => {
                        Ok(crate::render::markdown::device_event_markdown(r)?)
                    }
                }
            }
            Commands::Variant { cmd } => match cmd {
                VariantCommand::Trials {
                    id,
                    limit,
                    offset,
                    source,
                } => {
                    let _ = crate::entities::variant::parse_variant_id(&id)?;
                    let mutation_query = variant_trial_mutation_query(&id).await;
                    let trial_source = crate::entities::trial::TrialSource::from_flag(&source)?;
                    let filters = crate::entities::trial::TrialSearchFilters {
                        mutation: Some(mutation_query.clone()),
                        source: trial_source,
                        ..Default::default()
                    };
                    let (results, total) =
                        crate::entities::trial::search(&filters, limit, offset).await?;
                    if let Some(total) = total {
                        log_pagination_truncation(total as usize, offset, results.len());
                    }
                    if cli.json {
                        #[derive(serde::Serialize)]
                        struct SearchResponse {
                            count: usize,
                            total: Option<u32>,
                            results: Vec<crate::entities::trial::TrialSearchResult>,
                        }

                        Ok(crate::render::json::to_pretty(&SearchResponse {
                            count: results.len(),
                            total,
                            results,
                        })?)
                    } else {
                        let mut query_parts = vec![format!("mutation={mutation_query}")];
                        if matches!(trial_source, crate::entities::trial::TrialSource::NciCts) {
                            query_parts.push("source=nci".to_string());
                        }
                        if offset > 0 {
                            query_parts.push(format!("offset={offset}"));
                        }
                        let query = query_parts.join(", ");
                        Ok(crate::render::markdown::trial_search_markdown(
                            &query, &results, total,
                        )?)
                    }
                }
                VariantCommand::Articles { id, limit, offset } => {
                    let id_format = crate::entities::variant::parse_variant_id(&id)?;
                    let (gene, keyword) = match id_format {
                        crate::entities::variant::VariantIdFormat::RsId(rsid) => (None, Some(rsid)),
                        crate::entities::variant::VariantIdFormat::HgvsGenomic(hgvs) => {
                            (None, Some(hgvs))
                        }
                        crate::entities::variant::VariantIdFormat::GeneProteinChange { gene, change } => {
                            (Some(gene), Some(change))
                        }
                    };

                    let filters = crate::entities::article::ArticleSearchFilters {
                        gene,
                        disease: None,
                        drug: None,
                        author: None,
                        keyword,
                        date_from: None,
                        date_to: None,
                        article_type: None,
                        journal: None,
                        open_access: false,
                        no_preprints: true,
                        exclude_retracted: false,
                        sort: crate::entities::article::ArticleSort::Date,
                    };

                    let query = vec![
                        filters.gene.as_deref().map(|v| format!("gene={v}")),
                        filters.keyword.as_deref().map(|v| format!("keyword={v}")),
                        (offset > 0).then(|| format!("offset={offset}")),
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(", ");

                    let fetch_limit = paged_fetch_limit(limit, offset, 50)?;
                    let rows = crate::entities::article::search(&filters, fetch_limit).await?;
                    let (results, total) = paginate_results(rows, offset, limit);
                    log_pagination_truncation(total, offset, results.len());
                    if cli.json {
                        #[derive(serde::Serialize)]
                        struct SearchResponse {
                            total: Option<usize>,
                            count: usize,
                            results: Vec<crate::entities::article::ArticleSearchResult>,
                        }

                        Ok(crate::render::json::to_pretty(&SearchResponse {
                            total: Some(total),
                            count: results.len(),
                            results,
                        })?)
                    } else {
                        Ok(crate::render::markdown::article_search_markdown(
                            &query, &results,
                        )?)
                    }
                }
                VariantCommand::Oncokb { id } => {
                    let result = crate::entities::variant::oncokb(&id).await?;
                    if cli.json {
                        Ok(crate::render::json::to_pretty(&result)?)
                    } else {
                        Ok(crate::render::markdown::variant_oncokb_markdown(&result))
                    }
                }
            },
            Commands::Drug { cmd } => match cmd {
                DrugCommand::Trials {
                    name,
                    limit,
                    offset,
                    source,
                } => {
                    let trial_source = crate::entities::trial::TrialSource::from_flag(&source)?;
                    let filters = crate::entities::trial::TrialSearchFilters {
                        intervention: Some(name.clone()),
                        source: trial_source,
                        ..Default::default()
                    };
                    let (results, total) =
                        crate::entities::trial::search(&filters, limit, offset).await?;
                    if let Some(total) = total {
                        log_pagination_truncation(total as usize, offset, results.len());
                    }
                    if cli.json {
                        #[derive(serde::Serialize)]
                        struct SearchResponse {
                            count: usize,
                            total: Option<u32>,
                            results: Vec<crate::entities::trial::TrialSearchResult>,
                        }

                        Ok(crate::render::json::to_pretty(&SearchResponse {
                            count: results.len(),
                            total,
                            results,
                        })?)
                    } else {
                        let query = if offset > 0 {
                            format!("intervention={name}, offset={offset}")
                        } else {
                            format!("intervention={name}")
                        };
                        Ok(crate::render::markdown::trial_search_markdown(
                            &query, &results, total,
                        )?)
                    }
                }
                DrugCommand::AdverseEvents {
                    name,
                    limit,
                    offset,
                    serious,
                } => {
                    let filters = crate::entities::adverse_event::AdverseEventSearchFilters {
                        drug: Some(name.clone()),
                        serious: serious.then_some("any".to_string()),
                        ..Default::default()
                    };
                    let query_summary = crate::entities::adverse_event::search_query_summary(&filters);
                    let fetch_limit = paged_fetch_limit(limit, offset, 50)?;
                    let response =
                        crate::entities::adverse_event::search_with_summary(
                            &filters,
                            fetch_limit,
                            0,
                        )
                        .await?;
                    let (results, observed_total) =
                        paginate_results(response.results, offset, limit);
                    log_pagination_truncation(observed_total, offset, results.len());
                    let summary = crate::entities::adverse_event::summarize_search_results(
                        response.summary.total_reports,
                        &results,
                    );
                    if cli.json {
                        #[derive(serde::Serialize)]
                        struct SearchResponse {
                            total: Option<usize>,
                            count: usize,
                            summary: crate::entities::adverse_event::AdverseEventSearchSummary,
                            results: Vec<crate::entities::adverse_event::AdverseEventSearchResult>,
                        }

                        Ok(crate::render::json::to_pretty(&SearchResponse {
                            total: Some(summary.total_reports),
                            count: results.len(),
                            summary,
                            results,
                        })?)
                    } else {
                        Ok(crate::render::markdown::adverse_event_search_markdown(
                            &query_summary,
                            &results,
                            &summary,
                        )?)
                    }
                }
            },
            Commands::Disease { cmd } => match cmd {
                DiseaseCommand::Trials {
                    name,
                    limit,
                    offset,
                    source,
                } => {
                    let trial_source = crate::entities::trial::TrialSource::from_flag(&source)?;
                    let filters = crate::entities::trial::TrialSearchFilters {
                        condition: Some(name.clone()),
                        source: trial_source,
                        ..Default::default()
                    };
                    let (results, total) =
                        crate::entities::trial::search(&filters, limit, offset).await?;
                    if let Some(total) = total {
                        log_pagination_truncation(total as usize, offset, results.len());
                    }
                    if cli.json {
                        #[derive(serde::Serialize)]
                        struct SearchResponse {
                            count: usize,
                            total: Option<u32>,
                            results: Vec<crate::entities::trial::TrialSearchResult>,
                        }

                        Ok(crate::render::json::to_pretty(&SearchResponse {
                            count: results.len(),
                            total,
                            results,
                        })?)
                    } else {
                        let query = if offset > 0 {
                            format!("condition={name}, offset={offset}")
                        } else {
                            format!("condition={name}")
                        };
                        Ok(crate::render::markdown::trial_search_markdown(
                            &query, &results, total,
                        )?)
                    }
                }
                DiseaseCommand::Articles {
                    name,
                    limit,
                    offset,
                } => {
                    let filters = crate::entities::article::ArticleSearchFilters {
                        gene: None,
                        disease: Some(name.clone()),
                        drug: None,
                        author: None,
                        keyword: None,
                        date_from: None,
                        date_to: None,
                        article_type: None,
                        journal: None,
                        open_access: false,
                        no_preprints: true,
                        exclude_retracted: false,
                        sort: crate::entities::article::ArticleSort::Date,
                    };

                    let query = if offset > 0 {
                        format!("disease={name}, offset={offset}")
                    } else {
                        format!("disease={name}")
                    };
                    let fetch_limit = paged_fetch_limit(limit, offset, 50)?;
                    let rows = crate::entities::article::search(&filters, fetch_limit).await?;
                    let (results, total) = paginate_results(rows, offset, limit);
                    log_pagination_truncation(total, offset, results.len());
                    if cli.json {
                        #[derive(serde::Serialize)]
                        struct SearchResponse {
                            total: Option<usize>,
                            count: usize,
                            results: Vec<crate::entities::article::ArticleSearchResult>,
                        }

                        Ok(crate::render::json::to_pretty(&SearchResponse {
                            total: Some(total),
                            count: results.len(),
                            results,
                        })?)
                    } else {
                        Ok(crate::render::markdown::article_search_markdown(
                            &query, &results,
                        )?)
                    }
                }
                DiseaseCommand::Drugs {
                    name,
                    limit,
                    offset,
                } => {
                    let filters = crate::entities::drug::DrugSearchFilters {
                        indication: Some(name.clone()),
                        ..Default::default()
                    };
                    let mut query_summary = crate::entities::drug::search_query_summary(&filters);
                    if offset > 0 {
                        query_summary = format!("{query_summary}, offset={offset}");
                    }
                    let fetch_limit = paged_fetch_limit(limit, offset, 50)?;
                    let rows = crate::entities::drug::search(&filters, fetch_limit).await?;
                    let (results, total) = paginate_results(rows, offset, limit);
                    log_pagination_truncation(total, offset, results.len());
                    if cli.json {
                        #[derive(serde::Serialize)]
                        struct SearchResponse {
                            total: Option<usize>,
                            count: usize,
                            results: Vec<crate::entities::drug::DrugSearchResult>,
                        }

                        Ok(crate::render::json::to_pretty(&SearchResponse {
                            total: Some(total),
                            count: results.len(),
                            results,
                        })?)
                    } else {
                        Ok(crate::render::markdown::drug_search_markdown(
                            &query_summary,
                            &results,
                        )?)
                    }
                }
            },
            Commands::Article { cmd } => match cmd {
                ArticleCommand::Entities { pmid, limit } => {
                    let limit = paged_fetch_limit(limit, 0, 50)?;
                    let sections = vec!["annotations".to_string()];
                    let article = crate::entities::article::get(&pmid, &sections).await?;
                    let annotations = article
                        .annotations
                        .clone()
                        .map(|value| truncate_article_annotations(value, limit));
                    if cli.json {
                        #[derive(serde::Serialize)]
                        struct ArticleEntitiesResponse {
                            pmid: String,
                            annotations: Option<crate::entities::article::ArticleAnnotations>,
                        }
                        Ok(crate::render::json::to_pretty(&ArticleEntitiesResponse {
                            pmid,
                            annotations,
                        })?)
                    } else {
                        Ok(crate::render::markdown::article_entities_markdown(
                            article.pmid.as_deref().unwrap_or(&pmid),
                            annotations.as_ref(),
                            Some(limit),
                        )?)
                    }
                }
            },
            Commands::Gene { cmd } => match cmd {
                GeneCommand::Definition { symbol } => {
                    render_gene_card(&symbol, empty_sections(), cli.json).await
                }
                GeneCommand::Trials {
                    symbol,
                    limit,
                    offset,
                    source,
                } => {
                    let trial_source = crate::entities::trial::TrialSource::from_flag(&source)?;
                    let filters = crate::entities::trial::TrialSearchFilters {
                        mutation: Some(symbol.clone()),
                        source: trial_source,
                        ..Default::default()
                    };
                    let (results, total) =
                        crate::entities::trial::search(&filters, limit, offset).await?;
                    if let Some(total) = total {
                        log_pagination_truncation(total as usize, offset, results.len());
                    }
                    if cli.json {
                        #[derive(serde::Serialize)]
                        struct SearchResponse {
                            count: usize,
                            total: Option<u32>,
                            results: Vec<crate::entities::trial::TrialSearchResult>,
                        }

                        Ok(crate::render::json::to_pretty(&SearchResponse {
                            count: results.len(),
                            total,
                            results,
                        })?)
                    } else {
                        let query = if offset > 0 {
                            format!("mutation={symbol}, offset={offset}")
                        } else {
                            format!("mutation={symbol}")
                        };
                        Ok(crate::render::markdown::trial_search_markdown(
                            &query, &results, total,
                        )?)
                    }
                }
                GeneCommand::Drugs {
                    symbol,
                    limit,
                    offset,
                } => {
                    let filters = crate::entities::drug::DrugSearchFilters {
                        target: Some(symbol.clone()),
                        ..Default::default()
                    };
                    let mut query_summary = crate::entities::drug::search_query_summary(&filters);
                    if offset > 0 {
                        query_summary = format!("{query_summary}, offset={offset}");
                    }
                    let fetch_limit = paged_fetch_limit(limit, offset, 50)?;
                    let rows = crate::entities::drug::search(&filters, fetch_limit).await?;
                    let (results, total) = paginate_results(rows, offset, limit);
                    log_pagination_truncation(total, offset, results.len());
                    if cli.json {
                        #[derive(serde::Serialize)]
                        struct SearchResponse {
                            total: Option<usize>,
                            count: usize,
                            results: Vec<crate::entities::drug::DrugSearchResult>,
                        }

                        Ok(crate::render::json::to_pretty(&SearchResponse {
                            total: Some(total),
                            count: results.len(),
                            results,
                        })?)
                    } else {
                        Ok(crate::render::markdown::drug_search_markdown(
                            &query_summary,
                            &results,
                        )?)
                    }
                }
                GeneCommand::Articles {
                    symbol,
                    limit,
                    offset,
                } => {
                    let filters = crate::entities::article::ArticleSearchFilters {
                        gene: Some(symbol.clone()),
                        disease: None,
                        drug: None,
                        author: None,
                        keyword: None,
                        date_from: None,
                        date_to: None,
                        article_type: None,
                        journal: None,
                        open_access: false,
                        no_preprints: true,
                        exclude_retracted: false,
                        sort: crate::entities::article::ArticleSort::Date,
                    };
                    let query = if offset > 0 {
                        format!("gene={symbol}, offset={offset}")
                    } else {
                        format!("gene={symbol}")
                    };
                    let fetch_limit = paged_fetch_limit(limit, offset, 50)?;
                    let rows = crate::entities::article::search(&filters, fetch_limit).await?;
                    let (results, total) = paginate_results(rows, offset, limit);
                    log_pagination_truncation(total, offset, results.len());
                    if cli.json {
                        #[derive(serde::Serialize)]
                        struct SearchResponse {
                            total: Option<usize>,
                            count: usize,
                            results: Vec<crate::entities::article::ArticleSearchResult>,
                        }

                        Ok(crate::render::json::to_pretty(&SearchResponse {
                            total: Some(total),
                            count: results.len(),
                            results,
                        })?)
                    } else {
                        Ok(crate::render::markdown::article_search_markdown(
                            &query, &results,
                        )?)
                    }
                }
                GeneCommand::Pathways {
                    symbol,
                    limit,
                    offset,
                } => {
                    let fetch_limit = paged_fetch_limit(limit, offset, 25)?;
                    let sections = vec!["pathways".to_string()];
                    let mut gene = crate::entities::gene::get(&symbol, &sections).await?;
                    if let Some(pathways) = gene.pathways.take() {
                        let fetched = pathways.into_iter().take(fetch_limit).collect::<Vec<_>>();
                        let (results, observed_total) = paginate_results(fetched, offset, limit);
                        log_pagination_truncation(observed_total, offset, results.len());
                        gene.pathways = (!results.is_empty()).then_some(results);
                    }
                    if cli.json {
                        Ok(crate::render::json::to_pretty(&gene)?)
                    } else {
                        Ok(crate::render::markdown::gene_markdown(&gene, &sections)?)
                    }
                }
            },
            Commands::Pathway { cmd } => match cmd {
                PathwayCommand::Drugs { id, limit, offset } => {
                    let fetch_limit = paged_fetch_limit(limit, offset, 50)?;
                    let rows = pathway_drug_results(&id, fetch_limit).await?;
                    let (results, total) = paginate_results(rows, offset, limit);
                    log_pagination_truncation(total, offset, results.len());
                    if cli.json {
                        #[derive(serde::Serialize)]
                        struct SearchResponse {
                            total: Option<usize>,
                            count: usize,
                            results: Vec<crate::entities::drug::DrugSearchResult>,
                        }

                        Ok(crate::render::json::to_pretty(&SearchResponse {
                            total: Some(total),
                            count: results.len(),
                            results,
                        })?)
                    } else {
                        let query = if offset > 0 {
                            format!("pathway={id}, offset={offset}")
                        } else {
                            format!("pathway={id}")
                        };
                        Ok(crate::render::markdown::drug_search_markdown(&query, &results)?)
                    }
                }
                PathwayCommand::Articles { id, limit, offset } => {
                    let pathway = crate::entities::pathway::get(&id, empty_sections()).await?;
                    let pathway_name = pathway.name.trim();
                    let keyword = if pathway_name.is_empty() {
                        id.clone()
                    } else {
                        pathway_name.to_string()
                    };
                    let filters = crate::entities::article::ArticleSearchFilters {
                        gene: None,
                        disease: None,
                        drug: None,
                        author: None,
                        keyword: Some(keyword.clone()),
                        date_from: None,
                        date_to: None,
                        article_type: None,
                        journal: None,
                        open_access: false,
                        no_preprints: true,
                        exclude_retracted: false,
                        sort: crate::entities::article::ArticleSort::Date,
                    };
                    let query = if offset > 0 {
                        format!("keyword={keyword}, offset={offset}")
                    } else {
                        format!("keyword={keyword}")
                    };
                    let fetch_limit = paged_fetch_limit(limit, offset, 50)?;
                    let rows = crate::entities::article::search(&filters, fetch_limit).await?;
                    let (results, total) = paginate_results(rows, offset, limit);
                    log_pagination_truncation(total, offset, results.len());
                    if cli.json {
                        #[derive(serde::Serialize)]
                        struct SearchResponse {
                            total: Option<usize>,
                            count: usize,
                            results: Vec<crate::entities::article::ArticleSearchResult>,
                        }

                        Ok(crate::render::json::to_pretty(&SearchResponse {
                            total: Some(total),
                            count: results.len(),
                            results,
                        })?)
                    } else {
                        Ok(crate::render::markdown::article_search_markdown(
                            &query, &results,
                        )?)
                    }
                }
                PathwayCommand::Trials {
                    id,
                    limit,
                    offset,
                    source,
                } => {
                    let pathway = crate::entities::pathway::get(&id, empty_sections()).await?;
                    let pathway_name = pathway.name.trim();
                    let condition = if pathway_name.is_empty() {
                        id.clone()
                    } else {
                        pathway_name.to_string()
                    };
                    let trial_source = crate::entities::trial::TrialSource::from_flag(&source)?;
                    let filters = crate::entities::trial::TrialSearchFilters {
                        condition: Some(condition.clone()),
                        source: trial_source,
                        ..Default::default()
                    };
                    let (mut results, mut total) =
                        crate::entities::trial::search(&filters, limit, offset).await?;
                    let mut query = if offset > 0 {
                        format!("condition={condition}, offset={offset}")
                    } else {
                        format!("condition={condition}")
                    };

                    if should_try_pathway_trial_fallback(results.len(), offset, total) {
                        let pathway_with_genes =
                            crate::entities::pathway::get(&id, &["genes".to_string()]).await?;
                        let fallback_limit = limit.saturating_add(offset).clamp(1, 50);

                        for gene in pathway_with_genes.genes.into_iter().take(10) {
                            let gene = gene.trim().to_string();
                            if gene.is_empty() {
                                continue;
                            }

                            let fallback_filters = crate::entities::trial::TrialSearchFilters {
                                biomarker: Some(gene.clone()),
                                source: trial_source,
                                ..Default::default()
                            };

                            match crate::entities::trial::search(&fallback_filters, fallback_limit, 0)
                                .await
                            {
                                Ok((fallback_rows, fallback_total)) if !fallback_rows.is_empty() => {
                                    debug!(
                                        pathway_id = %id,
                                        fallback_gene = %gene,
                                        "Pathway trial condition search returned no rows; using biomarker fallback",
                                    );
                                    results =
                                        fallback_rows.into_iter().skip(offset).take(limit).collect();
                                    total = fallback_total;
                                    query = if offset > 0 {
                                        format!(
                                            "condition={condition}, fallback_biomarker={gene}, offset={offset}"
                                        )
                                    } else {
                                        format!("condition={condition}, fallback_biomarker={gene}")
                                    };
                                    break;
                                }
                                Ok(_) => {}
                                Err(err) => {
                                    warn!(pathway_id = %id, fallback_gene = %gene, "Pathway trial fallback failed: {err}");
                                }
                            }
                        }
                    }

                    if let Some(total) = total {
                        log_pagination_truncation(total as usize, offset, results.len());
                    }
                    if cli.json {
                        #[derive(serde::Serialize)]
                        struct SearchResponse {
                            count: usize,
                            total: Option<u32>,
                            results: Vec<crate::entities::trial::TrialSearchResult>,
                        }

                        Ok(crate::render::json::to_pretty(&SearchResponse {
                            count: results.len(),
                            total,
                            results,
                        })?)
                    } else {
                        Ok(crate::render::markdown::trial_search_markdown(
                            &query, &results, total,
                        )?)
                    }
                }
            },
            Commands::Protein { cmd } => match cmd {
                ProteinCommand::Structures {
                    accession,
                    limit,
                    offset,
                } => {
                    let sections = vec!["structures".to_string()];
                    let protein = crate::entities::protein::get_with_structure_limit(
                        &accession,
                        &sections,
                        Some(limit),
                        Some(offset),
                    )
                    .await?;
                    if cli.json {
                        Ok(crate::render::json::to_pretty(&protein)?)
                    } else {
                        Ok(crate::render::markdown::protein_markdown(&protein, &sections)?)
                    }
                }
            },
            Commands::Batch {
                entity,
                ids,
                sections,
                source,
            } => {
                let entity = entity.trim().to_ascii_lowercase();
                let parsed_ids = ids
                    .split(',')
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .collect::<Vec<_>>();
                let batch_sections = parse_batch_sections(sections.as_deref());

                if parsed_ids.is_empty() {
                    return Err(crate::error::BioMcpError::InvalidArgument(
                        "Batch IDs are required. Example: biomcp batch gene BRAF,TP53".into(),
                    )
                    .into());
                }
                if parsed_ids.len() > 10 {
                    return Err(crate::error::BioMcpError::InvalidArgument(
                        "Batch is limited to 10 IDs".into(),
                    )
                    .into());
                }

                match entity.as_str() {
                    "gene" => {
                        let futs = parsed_ids
                            .iter()
                            .map(|id| crate::entities::gene::get(id, &batch_sections));
                        let results = try_join_all(futs).await?;
                        if cli.json {
                            Ok(crate::render::json::to_pretty(&results)?)
                        } else {
                            let mut out = String::new();
                            out.push_str(&format!("# Batch: gene ({})\n\n", results.len()));
                            for (idx, item) in results.iter().enumerate() {
                                if idx > 0 {
                                    out.push_str("\n\n---\n\n");
                                }
                                out.push_str(&crate::render::markdown::gene_markdown(
                                    item,
                                    &batch_sections,
                                )?);
                            }
                            Ok(out)
                        }
                    }
                    "variant" => {
                        let futs = parsed_ids
                            .iter()
                            .map(|id| crate::entities::variant::get(id, &batch_sections));
                        let results = try_join_all(futs).await?;
                        if cli.json {
                            Ok(crate::render::json::to_pretty(&results)?)
                        } else {
                            let mut out = String::new();
                            out.push_str(&format!("# Batch: variant ({})\n\n", results.len()));
                            for (idx, item) in results.iter().enumerate() {
                                if idx > 0 {
                                    out.push_str("\n\n---\n\n");
                                }
                                out.push_str(&crate::render::markdown::variant_markdown(
                                    item,
                                    &batch_sections,
                                )?);
                            }
                            Ok(out)
                        }
                    }
                    "article" => {
                        let futs = parsed_ids
                            .iter()
                            .map(|id| crate::entities::article::get(id, &batch_sections));
                        let results = try_join_all(futs).await?;
                        if cli.json {
                            Ok(crate::render::json::to_pretty(&results)?)
                        } else {
                            let mut out = String::new();
                            out.push_str(&format!("# Batch: article ({})\n\n", results.len()));
                            for (idx, item) in results.iter().enumerate() {
                                if idx > 0 {
                                    out.push_str("\n\n---\n\n");
                                }
                                out.push_str(&crate::render::markdown::article_markdown(
                                    item,
                                    &batch_sections,
                                )?);
                            }
                            Ok(out)
                        }
                    }
                    "trial" => {
                        let trial_source = crate::entities::trial::TrialSource::from_flag(&source)?;
                        let futs = parsed_ids.iter().map(|id| {
                            crate::entities::trial::get(id, &batch_sections, trial_source)
                        });
                        let results = try_join_all(futs).await?;
                        if cli.json {
                            Ok(crate::render::json::to_pretty(&results)?)
                        } else {
                            let mut out = String::new();
                            out.push_str(&format!("# Batch: trial ({})\n\n", results.len()));
                            for (idx, item) in results.iter().enumerate() {
                                if idx > 0 {
                                    out.push_str("\n\n---\n\n");
                                }
                                out.push_str(&crate::render::markdown::trial_markdown(
                                    item,
                                    &batch_sections,
                                )?);
                            }
                            Ok(out)
                        }
                    }
                    "drug" => {
                        let futs = parsed_ids
                            .iter()
                            .map(|id| crate::entities::drug::get(id, &batch_sections));
                        let results = try_join_all(futs).await?;
                        if cli.json {
                            Ok(crate::render::json::to_pretty(&results)?)
                        } else {
                            let mut out = String::new();
                            out.push_str(&format!("# Batch: drug ({})\n\n", results.len()));
                            for (idx, item) in results.iter().enumerate() {
                                if idx > 0 {
                                    out.push_str("\n\n---\n\n");
                                }
                                out.push_str(&crate::render::markdown::drug_markdown(
                                    item,
                                    &batch_sections,
                                )?);
                            }
                            Ok(out)
                        }
                    }
                    "disease" => {
                        let futs = parsed_ids
                            .iter()
                            .map(|id| crate::entities::disease::get(id, &batch_sections));
                        let results = try_join_all(futs).await?;
                        if cli.json {
                            Ok(crate::render::json::to_pretty(&results)?)
                        } else {
                            let mut out = String::new();
                            out.push_str(&format!("# Batch: disease ({})\n\n", results.len()));
                            for (idx, item) in results.iter().enumerate() {
                                if idx > 0 {
                                    out.push_str("\n\n---\n\n");
                                }
                                out.push_str(&crate::render::markdown::disease_markdown(
                                    item,
                                    &batch_sections,
                                )?);
                            }
                            Ok(out)
                        }
                    }
                    "pgx" => {
                        let futs = parsed_ids
                            .iter()
                            .map(|id| crate::entities::pgx::get(id, &batch_sections));
                        let results = try_join_all(futs).await?;
                        if cli.json {
                            Ok(crate::render::json::to_pretty(&results)?)
                        } else {
                            let mut out = String::new();
                            out.push_str(&format!("# Batch: pgx ({})\n\n", results.len()));
                            for (idx, item) in results.iter().enumerate() {
                                if idx > 0 {
                                    out.push_str("\n\n---\n\n");
                                }
                                out.push_str(&crate::render::markdown::pgx_markdown(
                                    item,
                                    &batch_sections,
                                )?);
                            }
                            Ok(out)
                        }
                    }
                    "pathway" => {
                        let futs = parsed_ids
                            .iter()
                            .map(|id| crate::entities::pathway::get(id, &batch_sections));
                        let results = try_join_all(futs).await?;
                        if cli.json {
                            Ok(crate::render::json::to_pretty(&results)?)
                        } else {
                            let mut out = String::new();
                            out.push_str(&format!("# Batch: pathway ({})\n\n", results.len()));
                            for (idx, item) in results.iter().enumerate() {
                                if idx > 0 {
                                    out.push_str("\n\n---\n\n");
                                }
                                out.push_str(&crate::render::markdown::pathway_markdown(
                                    item,
                                    &batch_sections,
                                )?);
                            }
                            Ok(out)
                        }
                    }
                    "protein" => {
                        let futs = parsed_ids
                            .iter()
                            .map(|id| crate::entities::protein::get(id, &batch_sections));
                        let results = try_join_all(futs).await?;
                        if cli.json {
                            Ok(crate::render::json::to_pretty(&results)?)
                        } else {
                            let mut out = String::new();
                            out.push_str(&format!("# Batch: protein ({})\n\n", results.len()));
                            for (idx, item) in results.iter().enumerate() {
                                if idx > 0 {
                                    out.push_str("\n\n---\n\n");
                                }
                                out.push_str(&crate::render::markdown::protein_markdown(
                                    item,
                                    &batch_sections,
                                )?);
                            }
                            Ok(out)
                        }
                    }
                    "adverse-event" | "adverse_event" | "adverseevent" => {
                        if !batch_sections.is_empty() {
                            return Err(crate::error::BioMcpError::InvalidArgument(
                                "Batch sections are not supported for adverse-event".into(),
                            )
                            .into());
                        }
                        let futs = parsed_ids.iter().map(|id| crate::entities::adverse_event::get(id));
                        let results = try_join_all(futs).await?;
                        if cli.json {
                            Ok(crate::render::json::to_pretty(&results)?)
                        } else {
                            let mut out = String::new();
                            out.push_str(&format!("# Batch: adverse-event ({})\n\n", results.len()));
                            for (idx, item) in results.iter().enumerate() {
                                if idx > 0 {
                                    out.push_str("\n\n---\n\n");
                                }
                                match item {
                                    crate::entities::adverse_event::AdverseEventReport::Faers(r) => {
                                        out.push_str(
                                            &crate::render::markdown::adverse_event_markdown(
                                                r,
                                                empty_sections(),
                                            )?,
                                        );
                                    }
                                    crate::entities::adverse_event::AdverseEventReport::Device(r) => {
                                        out.push_str(
                                            &crate::render::markdown::device_event_markdown(r)?,
                                        );
                                    }
                                }
                            }
                            Ok(out)
                        }
                    }
                    other => Err(crate::error::BioMcpError::InvalidArgument(format!(
                        "Unknown batch entity '{other}'. Expected one of: gene, variant, article, trial, drug, disease, pgx, pathway, protein, adverse-event"
                    ))
                    .into()),
                }
            }
            Commands::Search { entity } => {
                match entity {
                SearchEntity::Gene {
                    query,
                    positional_query,
                    gene_type,
                    chromosome,
                    region,
                    pathway,
                    go_term,
                    limit,
                    offset,
                } => {
                    let query = resolve_query_input(query, positional_query, "--query")?;
                    let filters = crate::entities::gene::GeneSearchFilters {
                        query,
                        gene_type,
                        chromosome,
                        region,
                        pathway,
                        go_term,
                    };
                    let mut query_summary = crate::entities::gene::search_query_summary(&filters);
                    if offset > 0 {
                        query_summary = format!("{query_summary}, offset={offset}");
                    }
                    let page = crate::entities::gene::search_page(&filters, limit, offset).await?;
                    let results = page.results;
                    let pagination =
                        PaginationMeta::offset(offset, limit, results.len(), page.total);
                    if cli.json {
                        search_json(results, pagination)
                    } else {
                        let footer = pagination_footer_offset(&pagination);
                        Ok(crate::render::markdown::gene_search_markdown_with_footer(
                            &query_summary,
                            &results,
                            &footer,
                        )?)
                    }
                }
                SearchEntity::Disease {
                    query,
                    positional_query,
                    source,
                    inheritance,
                    phenotype,
                    onset,
                    limit,
                    offset,
                } => {
                    let query = resolve_query_input(query, positional_query, "--query")?;
                    let filters = crate::entities::disease::DiseaseSearchFilters {
                        query,
                        source,
                        inheritance,
                        phenotype,
                        onset,
                    };
                    let mut query_summary = crate::entities::disease::search_query_summary(&filters);
                    if offset > 0 {
                        query_summary = format!("{query_summary}, offset={offset}");
                    }
                    let page = crate::entities::disease::search_page(&filters, limit, offset).await?;
                    let results = page.results;
                    let pagination =
                        PaginationMeta::offset(offset, limit, results.len(), page.total);
                    if cli.json {
                        search_json(results, pagination)
                    } else {
                        let footer = pagination_footer_offset(&pagination);
                        Ok(crate::render::markdown::disease_search_markdown_with_footer(
                            &query_summary,
                            &results,
                            &footer,
                        )?)
                    }
                }
                SearchEntity::Pgx {
                    gene,
                    drug,
                    cpic_level,
                    pgx_testing,
                    evidence,
                    limit,
                    offset,
                } => {
                    let filters = crate::entities::pgx::PgxSearchFilters {
                        gene,
                        drug,
                        cpic_level,
                        pgx_testing,
                        evidence,
                    };
                    let mut query_summary = crate::entities::pgx::search_query_summary(&filters);
                    if offset > 0 {
                        query_summary = format!("{query_summary}, offset={offset}");
                    }
                    let page = crate::entities::pgx::search_page(&filters, limit, offset).await?;
                    let results = page.results;
                    let pagination =
                        PaginationMeta::offset(offset, limit, results.len(), page.total);
                    if cli.json {
                        search_json(results, pagination)
                    } else {
                        let footer = pagination_footer_offset(&pagination);
                        Ok(crate::render::markdown::pgx_search_markdown_with_footer(
                            &query_summary,
                            &results,
                            &footer,
                        )?)
                    }
                }
                SearchEntity::Phenotype {
                    terms,
                    limit,
                    offset,
                } => {
                    let mut query_summary = terms.trim().to_string();
                    if offset > 0 {
                        query_summary = format!("{query_summary}, offset={offset}");
                    }
                    let page =
                        crate::entities::disease::search_phenotype_page(&terms, limit, offset)
                            .await?;
                    let results = page.results;
                    let pagination =
                        PaginationMeta::offset(offset, limit, results.len(), page.total);
                    if cli.json {
                        search_json(results, pagination)
                    } else {
                        let footer = pagination_footer_offset(&pagination);
                        Ok(crate::render::markdown::phenotype_search_markdown_with_footer(
                            &query_summary,
                            &results,
                            &footer,
                        )?)
                    }
                }
                SearchEntity::Gwas {
                    gene,
                    trait_query,
                    region,
                    p_value,
                    limit,
                    offset,
                } => {
                    let filters = crate::entities::variant::GwasSearchFilters {
                        gene,
                        trait_query,
                        region,
                        p_value,
                    };
                    let mut query_summary = crate::entities::variant::gwas_search_query_summary(&filters);
                    if offset > 0 {
                        query_summary = format!("{query_summary}, offset={offset}");
                    }
                    let page =
                        crate::entities::variant::search_gwas_page(&filters, limit, offset)
                            .await?;
                    let results = page.results;
                    let pagination =
                        PaginationMeta::offset(offset, limit, results.len(), page.total);
                    if cli.json {
                        search_json(results, pagination)
                    } else {
                        let footer = pagination_footer_offset(&pagination);
                        Ok(crate::render::markdown::gwas_search_markdown_with_footer(
                            &query_summary,
                            &results,
                            &footer,
                        )?)
                    }
                }
                SearchEntity::Article {
                    gene,
                    disease,
                    drug,
                    author,
                    keyword,
                    positional_query,
                    date_from,
                    date_to,
                    article_type,
                    journal,
                    open_access,
                    no_preprints,
                    exclude_retracted,
                    include_retracted,
                    sort,
                    limit,
                    offset,
                } => {
                    let keyword =
                        resolve_query_input(keyword, positional_query, "--keyword/--query")?;
                    let sort = crate::entities::article::ArticleSort::from_flag(&sort)?;
                    let exclude_retracted = exclude_retracted || !include_retracted;
                    let filters = crate::entities::article::ArticleSearchFilters {
                        gene,
                        disease,
                        drug,
                        author,
                        keyword,
                        date_from,
                        date_to,
                        article_type,
                        journal,
                        open_access,
                        no_preprints,
                        exclude_retracted,
                        sort,
                    };

                    let query = vec![
                        filters.gene.as_deref().map(|v| format!("gene={v}")),
                        filters.disease.as_deref().map(|v| format!("disease={v}")),
                        filters.drug.as_deref().map(|v| format!("drug={v}")),
                        filters.author.as_deref().map(|v| format!("author={v}")),
                        filters.keyword.as_deref().map(|v| format!("keyword={v}")),
                        filters
                            .article_type
                            .as_deref()
                            .map(|v| format!("type={v}")),
                        filters.date_from.as_deref().map(|v| format!("date_from={v}")),
                        filters.date_to.as_deref().map(|v| format!("date_to={v}")),
                        filters.journal.as_deref().map(|v| format!("journal={v}")),
                        filters.open_access.then(|| "open_access=true".to_string()),
                        filters.no_preprints.then(|| "no_preprints=true".to_string()),
                        if include_retracted {
                            Some("include_retracted=true".to_string())
                        } else {
                            filters
                                .exclude_retracted
                                .then(|| "exclude_retracted=true".to_string())
                        },
                        Some(format!("sort={}", filters.sort.as_str())),
                        (offset > 0).then(|| format!("offset={offset}")),
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(", ");

                    let page = crate::entities::article::search_page(&filters, limit, offset).await?;
                    let results = page.results;
                    let pagination =
                        PaginationMeta::offset(offset, limit, results.len(), page.total);
                    if cli.json {
                        search_json(results, pagination)
                    } else {
                        let footer = pagination_footer_offset(&pagination);
                        Ok(crate::render::markdown::article_search_markdown_with_footer(
                            &query, &results, &footer,
                        )?)
                    }
                }
                SearchEntity::Trial {
                    condition,
                    intervention,
                    facility,
                    phase,
                    study_type,
                    age,
                    sex,
                    status,
                    mutation,
                    biomarker,
                    prior_therapies,
                    progression_on,
                    line_of_therapy,
                    sponsor,
                    sponsor_type,
                    date_from,
                    date_to,
                    lat,
                    lon,
                    distance,
                    results_available,
                    count_only,
                    source,
                    offset,
                    next_page,
                    limit,
                } => {
                    let trial_source = crate::entities::trial::TrialSource::from_flag(&source)?;
                    let filters = crate::entities::trial::TrialSearchFilters {
                        condition,
                        intervention,
                        facility,
                        status,
                        phase,
                        study_type,
                        age,
                        sex,
                        sponsor,
                        sponsor_type,
                        date_from,
                        date_to,
                        mutation,
                        biomarker,
                        prior_therapies,
                        progression_on,
                        line_of_therapy,
                        lat,
                        lon,
                        distance,
                        results_available,
                        source: trial_source,
                    };

                    if next_page
                        .as_deref()
                        .map(str::trim)
                        .is_some_and(|value| !value.is_empty())
                        && offset > 0
                    {
                        return Err(crate::error::BioMcpError::InvalidArgument(
                            "--next-page cannot be used together with --offset".into(),
                        )
                        .into());
                    }

                    let query =
                        trial_search_query_summary(&filters, offset, next_page.as_deref());
                    let page = crate::entities::trial::search_page(
                        &filters,
                        limit,
                        offset,
                        next_page.clone(),
                    )
                    .await?;
                    if count_only {
                        if cli.json {
                            #[derive(serde::Serialize)]
                            struct TrialCountOnlyJson {
                                total: Option<usize>,
                            }
                            return Ok(crate::render::json::to_pretty(&TrialCountOnlyJson {
                                total: page.total,
                            })?);
                        }
                        return Ok(match page.total {
                            Some(total) => format!("Total: {total}"),
                            None => "Total: unknown".to_string(),
                        });
                    }
                    let results = page.results;
                    let pagination = PaginationMeta::cursor(
                        offset,
                        limit,
                        results.len(),
                        page.total,
                        page.next_page_token,
                    );
                    if cli.json {
                        search_json(results, pagination)
                    } else {
                        let footer = if matches!(
                            trial_source,
                            crate::entities::trial::TrialSource::ClinicalTrialsGov
                        ) {
                            pagination_footer_cursor(&pagination)
                        } else {
                            pagination_footer_offset(&pagination)
                        };
                        let total = pagination.total.and_then(|value| u32::try_from(value).ok());
                        Ok(crate::render::markdown::trial_search_markdown_with_footer(
                            &query, &results, total, &footer,
                        )?)
                    }
                }
                SearchEntity::Variant {
                    gene,
                    positional_query,
                    hgvsp,
                    significance,
                    max_frequency,
                    min_cadd,
                    consequence,
                    review_status,
                    population,
                    revel_min,
                    gerp_min,
                    tumor_site,
                    condition,
                    impact,
                    lof,
                    has,
                    missing,
                    therapy,
                    limit,
                    offset,
                } => {
                    let gene = resolve_query_input(gene, positional_query, "--gene")?;
                    let filters = crate::entities::variant::VariantSearchFilters {
                        gene,
                        hgvsp,
                        significance,
                        max_frequency,
                        min_cadd,
                        consequence,
                        review_status,
                        population,
                        revel_min,
                        gerp_min,
                        tumor_site,
                        condition,
                        impact,
                        lof,
                        has,
                        missing,
                        therapy,
                    };

                    let mut query = crate::entities::variant::search_query_summary(&filters);
                    if offset > 0 {
                        query = if query.is_empty() {
                            format!("offset={offset}")
                        } else {
                            format!("{query}, offset={offset}")
                        };
                    }

                    let page = crate::entities::variant::search_page(&filters, limit, offset).await?;
                    let results = page.results;
                    let pagination =
                        PaginationMeta::offset(offset, limit, results.len(), page.total);
                    if cli.json {
                        search_json(results, pagination)
                    } else {
                        let footer = pagination_footer_offset(&pagination);
                        Ok(crate::render::markdown::variant_search_markdown_with_footer(
                            &query, &results, &footer,
                        )?)
                    }
                }
                SearchEntity::Drug {
                    query,
                    positional_query,
                    target,
                    indication,
                    mechanism,
                    drug_type,
                    atc,
                    pharm_class,
                    interactions,
                    limit,
                    offset,
                } => {
                    let query = resolve_query_input(query, positional_query, "--query")?;
                    let filters = crate::entities::drug::DrugSearchFilters {
                        query,
                        target,
                        indication,
                        mechanism,
                        drug_type,
                        atc,
                        pharm_class,
                        interactions,
                    };
                    let mut query_summary = crate::entities::drug::search_query_summary(&filters);
                    if offset > 0 {
                        query_summary = format!("{query_summary}, offset={offset}");
                    }
                    let page = crate::entities::drug::search_page(&filters, limit, offset).await?;
                    let results = page.results;
                    let pagination =
                        PaginationMeta::offset(offset, limit, results.len(), page.total);
                    if cli.json {
                        search_json(results, pagination)
                    } else {
                        let footer = pagination_footer_offset(&pagination);
                        Ok(crate::render::markdown::drug_search_markdown_with_footer(
                            &query_summary,
                            &results,
                            pagination.total,
                            &footer,
                        )?)
                    }
                }
                SearchEntity::Pathway {
                    query,
                    positional_query,
                    pathway_type,
                    top_level,
                    limit,
                    offset,
                } => {
                    let query = resolve_query_input(query, positional_query, "--query")?;
                    let filters = crate::entities::pathway::PathwaySearchFilters {
                        query,
                        pathway_type,
                        top_level,
                    };
                    let fetch_limit = paged_fetch_limit(limit, offset, 25)?;
                    let mut query_summary = crate::entities::pathway::search_query_summary(&filters);
                    if offset > 0 {
                        query_summary = if query_summary.is_empty() {
                            format!("offset={offset}")
                        } else {
                            format!("{query_summary}, offset={offset}")
                        };
                    }
                    let (rows, total) =
                        crate::entities::pathway::search_with_filters(&filters, fetch_limit).await?;
                    let (results, observed_total) = paginate_results(rows, offset, limit);
                    log_pagination_truncation(observed_total, offset, results.len());
                    let total = total.or(Some(observed_total));
                    let pagination =
                        PaginationMeta::offset(offset, limit, results.len(), total);
                    if cli.json {
                        search_json(results, pagination)
                    } else {
                        let footer = pagination_footer_offset(&pagination);
                        Ok(crate::render::markdown::pathway_search_markdown_with_footer(
                            &query_summary,
                            &results,
                            total,
                            &footer,
                        )?)
                    }
                }
                SearchEntity::Protein {
                    query,
                    positional_query,
                    all_species,
                    reviewed,
                    disease,
                    existence,
                    limit,
                    offset,
                    next_page,
                } => {
                    let query =
                        resolve_query_input(query, positional_query, "--query")?.unwrap_or_default();
                    if next_page
                        .as_deref()
                        .map(str::trim)
                        .is_some_and(|value| !value.is_empty())
                        && offset > 0
                    {
                        return Err(crate::error::BioMcpError::InvalidArgument(
                            "--next-page cannot be used together with --offset".into(),
                        )
                        .into());
                    }
                    let mut query_summary = crate::entities::protein::search_query_summary(
                        &query,
                        reviewed,
                        disease.as_deref(),
                        existence,
                        all_species,
                    );
                    if offset > 0 {
                        query_summary = if query_summary.is_empty() {
                            format!("offset={offset}")
                        } else {
                            format!("{query_summary}, offset={offset}")
                        };
                    }
                    let page = crate::entities::protein::search_page(
                        &query,
                        limit,
                        offset,
                        next_page.clone(),
                        all_species,
                        reviewed,
                        disease.as_deref(),
                        existence,
                    )
                    .await?;
                    let results = page.results;
                    let pagination = PaginationMeta::cursor(
                        offset,
                        limit,
                        results.len(),
                        page.total,
                        page.next_page_token,
                    );
                    if cli.json {
                        search_json(results, pagination)
                    } else {
                        let footer = pagination_footer_cursor(&pagination);
                        Ok(crate::render::markdown::protein_search_markdown_with_footer(
                            &query_summary,
                            &results,
                            &footer,
                        )?)
                    }
                }
                SearchEntity::AdverseEvent {
                    drug,
                    device,
                    manufacturer,
                    product_code,
                    reaction,
                    outcome,
                    serious,
                    date_from,
                    date_to,
                    suspect_only,
                    sex,
                    age_min,
                    age_max,
                    reporter,
                    count,
                    r#type,
                    classification,
                    limit,
                    offset,
                } => {
                    let query_type =
                        crate::entities::adverse_event::AdverseEventQueryType::from_flag(&r#type)?;

                    match query_type {
                        crate::entities::adverse_event::AdverseEventQueryType::Faers => {
                            if device.is_some() {
                                return Err(crate::error::BioMcpError::InvalidArgument(
                                    "--device can only be used with --type device".into(),
                                )
                                .into());
                            }
                            if manufacturer.is_some() {
                                return Err(crate::error::BioMcpError::InvalidArgument(
                                    "--manufacturer can only be used with --type device".into(),
                                )
                                .into());
                            }
                            if product_code.is_some() {
                                return Err(crate::error::BioMcpError::InvalidArgument(
                                    "--product-code can only be used with --type device".into(),
                                )
                                .into());
                            }
                            let filters = crate::entities::adverse_event::AdverseEventSearchFilters {
                                drug,
                                reaction,
                                outcome,
                                serious,
                                since: date_from,
                                date_to,
                                suspect_only,
                                sex,
                                age_min,
                                age_max,
                                reporter,
                            };
                            let mut query_summary =
                                crate::entities::adverse_event::search_query_summary(&filters);
                            if let Some(count_field) = count
                                .as_deref()
                                .map(str::trim)
                                .filter(|v| !v.is_empty())
                            {
                                if query_summary.is_empty() {
                                    query_summary = format!("count={count_field}");
                                } else {
                                    query_summary = format!("{query_summary}, count={count_field}");
                                }
                            }
                            if offset > 0 {
                                query_summary = format!("{query_summary}, offset={offset}");
                            }
                            if let Some(count_field) = count
                                .as_deref()
                                .map(str::trim)
                                .filter(|v| !v.is_empty())
                            {
                                let response = crate::entities::adverse_event::search_count(
                                    &filters,
                                    count_field,
                                    limit,
                                )
                                .await?;
                                if cli.json {
                                    #[derive(serde::Serialize)]
                                    struct CountResponse {
                                        query: String,
                                        count_field: String,
                                        buckets:
                                            Vec<crate::entities::adverse_event::AdverseEventCountBucket>,
                                    }

                                    return Ok(crate::render::json::to_pretty(&CountResponse {
                                        query: query_summary,
                                        count_field: response.count_field,
                                        buckets: response.buckets,
                                    })?);
                                }

                                return Ok(
                                    crate::render::markdown::adverse_event_count_markdown(
                                        &query_summary,
                                        &response.count_field,
                                        &response.buckets,
                                    )?,
                                );
                            }
                            let response =
                                crate::entities::adverse_event::search_with_summary(
                                    &filters,
                                    limit,
                                    offset,
                                )
                                .await?;
                            let summary = response.summary;
                            let results = response.results;
                            let pagination = PaginationMeta::offset(
                                offset,
                                limit,
                                results.len(),
                                Some(summary.total_reports),
                            );
                            if cli.json {
                                #[derive(serde::Serialize)]
                                struct SearchResponse {
                                    pagination: PaginationMeta,
                                    count: usize,
                                    summary:
                                        crate::entities::adverse_event::AdverseEventSearchSummary,
                                    results:
                                        Vec<crate::entities::adverse_event::AdverseEventSearchResult>,
                                }

                                Ok(crate::render::json::to_pretty(&SearchResponse {
                                    pagination,
                                    count: results.len(),
                                    summary,
                                    results,
                                })?)
                            } else {
                                let footer = pagination_footer_offset(&pagination);
                                Ok(crate::render::markdown::adverse_event_search_markdown_with_footer(
                                    &query_summary,
                                    &results,
                                    &summary,
                                    &footer,
                                )?)
                            }
                        }
                        crate::entities::adverse_event::AdverseEventQueryType::Recall => {
                            if date_from.is_some()
                                || date_to.is_some()
                                || suspect_only
                                || sex.is_some()
                                || age_min.is_some()
                                || age_max.is_some()
                                || reporter.is_some()
                                || count.is_some()
                            {
                                return Err(crate::error::BioMcpError::InvalidArgument(
                                    "--date-from/--date-to/--suspect-only/--sex/--age-min/--age-max/--reporter/--count are only valid for --type faers".into(),
                                )
                                .into());
                            }
                            if device.is_some() {
                                return Err(crate::error::BioMcpError::InvalidArgument(
                                    "--device can only be used with --type device".into(),
                                )
                                .into());
                            }
                            if manufacturer.is_some() {
                                return Err(crate::error::BioMcpError::InvalidArgument(
                                    "--manufacturer can only be used with --type device".into(),
                                )
                                .into());
                            }
                            if product_code.is_some() {
                                return Err(crate::error::BioMcpError::InvalidArgument(
                                    "--product-code can only be used with --type device".into(),
                                )
                                .into());
                            }
                            if outcome.is_some() {
                                return Err(crate::error::BioMcpError::InvalidArgument(
                                    "--outcome is only valid for --type faers".into(),
                                )
                                .into());
                            }
                            let filters = crate::entities::adverse_event::RecallSearchFilters {
                                drug,
                                classification,
                            };
                            let mut query_summary =
                                crate::entities::adverse_event::recall_query_summary(&filters);
                            if offset > 0 {
                                query_summary = format!("{query_summary}, offset={offset}");
                            }
                            let page = crate::entities::adverse_event::search_recalls_page(
                                &filters,
                                limit,
                                offset,
                            )
                            .await?;
                            let results = page.results;
                            let pagination =
                                PaginationMeta::offset(offset, limit, results.len(), page.total);
                            if cli.json {
                                search_json(results, pagination)
                            } else {
                                let footer = pagination_footer_offset(&pagination);
                                Ok(crate::render::markdown::recall_search_markdown_with_footer(
                                    &query_summary,
                                    &results,
                                    &footer,
                                )?)
                            }
                        }
                        crate::entities::adverse_event::AdverseEventQueryType::Device => {
                            if drug.is_some() {
                                return Err(crate::error::BioMcpError::InvalidArgument(
                                    "--drug cannot be used with --type device (use --device)".into(),
                                )
                                .into());
                            }
                            if reaction.is_some() {
                                return Err(crate::error::BioMcpError::InvalidArgument(
                                    "--reaction is not supported with --type device".into(),
                                )
                                .into());
                            }
                            if outcome.is_some() {
                                return Err(crate::error::BioMcpError::InvalidArgument(
                                    "--outcome is only valid for --type faers".into(),
                                )
                                .into());
                            }
                            if classification.is_some() {
                                return Err(crate::error::BioMcpError::InvalidArgument(
                                    "--classification is only valid for --type recall".into(),
                                )
                                .into());
                            }
                            if date_to.is_some()
                                || suspect_only
                                || sex.is_some()
                                || age_min.is_some()
                                || age_max.is_some()
                                || reporter.is_some()
                                || count.is_some()
                            {
                                return Err(crate::error::BioMcpError::InvalidArgument(
                                    "--date-to/--suspect-only/--sex/--age-min/--age-max/--reporter/--count are only valid for --type faers".into(),
                                )
                                .into());
                            }

                            let filters = crate::entities::adverse_event::DeviceEventSearchFilters {
                                device,
                                manufacturer,
                                product_code,
                                serious: serious.is_some(),
                                since: date_from,
                            };
                            let mut query_summary =
                                crate::entities::adverse_event::device_query_summary(&filters);
                            if offset > 0 {
                                query_summary = format!("{query_summary}, offset={offset}");
                            }
                            let page = crate::entities::adverse_event::search_device_page(
                                &filters,
                                limit,
                                offset,
                            )
                            .await?;
                            let results = page.results;
                            let pagination =
                                PaginationMeta::offset(offset, limit, results.len(), page.total);
                            if cli.json {
                                search_json(results, pagination)
                            } else {
                                let footer = pagination_footer_offset(&pagination);
                                Ok(crate::render::markdown::device_event_search_markdown_with_footer(
                                    &query_summary,
                                    &results,
                                    &footer,
                                )?)
                            }
                        }
                    }
                }
                }
            }
            Commands::Health { apis_only } => {
                let report = crate::cli::health::check(apis_only).await?;
                if cli.json {
                    Ok(crate::render::json::to_pretty(&report)?)
                } else {
                    Ok(report.to_markdown())
                }
            }
            Commands::Skill { command } => match command {
                None => Ok(crate::cli::skill::show_overview()?),
                Some(crate::cli::skill::SkillCommand::List) => Ok(crate::cli::skill::list_use_cases()?),
                Some(crate::cli::skill::SkillCommand::Install { dir, force }) => {
                    Ok(crate::cli::skill::install_skills(dir.as_deref(), force)?)
                }
                Some(crate::cli::skill::SkillCommand::Show(args)) => {
                    let key = if args.is_empty() {
                        String::new()
                    } else if args.len() == 1 {
                        args[0].clone()
                    } else {
                        args.join("-")
                    };
                    Ok(crate::cli::skill::show_use_case(&key)?)
                }
            },
            Commands::Update { check } => Ok(crate::cli::update::run(check).await?),
            Commands::Uninstall => Ok(uninstall_self()?),
            Commands::Enrich { genes, limit } => {
                const MAX_ENRICH_LIMIT: usize = 50;
                if limit == 0 || limit > MAX_ENRICH_LIMIT {
                    return Err(crate::error::BioMcpError::InvalidArgument(format!(
                        "--limit must be between 1 and {MAX_ENRICH_LIMIT}"
                    ))
                    .into());
                }
                let genes = genes
                    .split(',')
                    .map(str::trim)
                    .filter(|g| !g.is_empty())
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                if genes.is_empty() {
                    return Err(crate::error::BioMcpError::InvalidArgument(
                        "At least one gene is required. Example: biomcp enrich BRAF,KRAS".into(),
                    )
                    .into());
                }
                let terms = crate::sources::gprofiler::GProfilerClient::new()?
                    .enrich_genes(&genes, limit)
                    .await?;
                if cli.json {
                    #[derive(serde::Serialize)]
                    struct EnrichResponse {
                        genes: Vec<String>,
                        count: usize,
                        results: Vec<crate::sources::gprofiler::GProfilerTerm>,
                    }
                    Ok(crate::render::json::to_pretty(&EnrichResponse {
                        genes,
                        count: terms.len(),
                        results: terms,
                    })?)
                } else {
                    Ok(enrich_markdown(&genes, &terms))
                }
            }
            Commands::List { entity } => {
                crate::cli::list::render(entity.as_deref()).map_err(Into::into)
            }
            Commands::Mcp | Commands::Serve | Commands::ServeHttp { .. } => {
                anyhow::bail!("MCP/serve commands should not go through CLI run()")
            }
            Commands::Version => Ok(version_output()),
        }
    })
    .await
}

/// Main CLI execution - called by MCP `shell` tool.
///
/// # Errors
///
/// Returns an error when CLI args cannot be parsed or when command execution fails.
pub async fn execute(mut args: Vec<String>) -> anyhow::Result<String> {
    if args.is_empty() {
        args.push("biomcp".to_string());
    }
    let cli = Cli::try_parse_from(args)?;
    run(cli).await
}

#[cfg(test)]
mod tests {
    use super::{
        ArticleCommand, Cli, Commands, GeneCommand, ProteinCommand, VariantCommand, execute,
        extract_json_from_sections, parse_trial_location_paging, resolve_query_input,
        should_try_pathway_trial_fallback, trial_search_query_summary,
        truncate_article_annotations,
    };
    use clap::Parser;

    #[test]
    fn extract_json_from_sections_detects_trailing_long_flag() {
        let sections = vec!["all".to_string(), "--json".to_string()];
        let (cleaned, json_override) = extract_json_from_sections(&sections);
        assert_eq!(cleaned, vec!["all".to_string()]);
        assert!(json_override);
    }

    #[test]
    fn extract_json_from_sections_detects_trailing_short_flag() {
        let sections = vec!["clinvar".to_string(), "-j".to_string()];
        let (cleaned, json_override) = extract_json_from_sections(&sections);
        assert_eq!(cleaned, vec!["clinvar".to_string()]);
        assert!(json_override);
    }

    #[test]
    fn extract_json_from_sections_keeps_regular_sections() {
        let sections = vec!["eligibility".to_string(), "locations".to_string()];
        let (cleaned, json_override) = extract_json_from_sections(&sections);
        assert_eq!(cleaned, sections);
        assert!(!json_override);
    }

    #[test]
    fn parse_trial_location_paging_extracts_offset_limit_flags() {
        let sections = vec![
            "locations".to_string(),
            "--offset".to_string(),
            "20".to_string(),
            "--limit=10".to_string(),
        ];
        let (cleaned, offset, limit) =
            parse_trial_location_paging(&sections).expect("valid pagination flags");
        assert_eq!(cleaned, vec!["locations".to_string()]);
        assert_eq!(offset, Some(20));
        assert_eq!(limit, Some(10));
    }

    #[test]
    fn pathway_trial_fallback_allows_no_match_on_first_page() {
        assert!(should_try_pathway_trial_fallback(0, 0, Some(0)));
        assert!(should_try_pathway_trial_fallback(0, 0, None));
    }

    #[test]
    fn pathway_trial_fallback_skips_offset_or_known_matches() {
        assert!(!should_try_pathway_trial_fallback(0, 5, Some(2)));
        assert!(!should_try_pathway_trial_fallback(0, 0, Some(7)));
        assert!(!should_try_pathway_trial_fallback(1, 0, Some(1)));
    }

    #[test]
    fn trial_search_query_summary_includes_geo_filters() {
        let summary = trial_search_query_summary(
            &crate::entities::trial::TrialSearchFilters {
                condition: Some("melanoma".into()),
                facility: Some("MD Anderson".into()),
                age: Some(67),
                sex: Some("female".into()),
                sponsor_type: Some("nih".into()),
                lat: Some(40.7128),
                lon: Some(-74.006),
                distance: Some(50),
                ..Default::default()
            },
            0,
            None,
        );
        assert!(summary.contains("condition=melanoma"));
        assert!(summary.contains("facility=MD Anderson"));
        assert!(summary.contains("age=67"));
        assert!(summary.contains("sex=female"));
        assert!(summary.contains("sponsor_type=nih"));
        assert!(summary.contains("lat=40.7128"));
        assert!(summary.contains("lon=-74.006"));
        assert!(summary.contains("distance=50"));
    }

    #[test]
    fn resolve_query_input_accepts_flag_or_positional() {
        let from_flag = resolve_query_input(Some("BRAF".into()), None, "--query").unwrap();
        assert_eq!(from_flag.as_deref(), Some("BRAF"));

        let from_positional =
            resolve_query_input(None, Some("melanoma".into()), "--query").unwrap();
        assert_eq!(from_positional.as_deref(), Some("melanoma"));
    }

    #[test]
    fn resolve_query_input_rejects_dual_values() {
        let err =
            resolve_query_input(Some("BRAF".into()), Some("TP53".into()), "--query").unwrap_err();
        assert!(format!("{err}").contains("Use either positional QUERY or --query, not both"));

        let err_gene =
            resolve_query_input(Some("TP53".into()), Some("BRAF".into()), "--gene").unwrap_err();
        assert!(format!("{err_gene}").contains("Use either positional QUERY or --gene, not both"));
    }

    #[test]
    fn gene_get_alias_parses_as_definition_subcommand() {
        let cli = Cli::try_parse_from(["biomcp", "gene", "get", "BRAF"])
            .expect("gene get alias should parse");
        match cli.command {
            Commands::Gene {
                cmd: GeneCommand::Definition { symbol },
            } => assert_eq!(symbol, "BRAF"),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn variant_trials_parses_source_flag() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "variant",
            "trials",
            "BRAF V600E",
            "--source",
            "nci",
            "--limit",
            "3",
        ])
        .expect("variant trials with --source should parse");
        match cli.command {
            Commands::Variant {
                cmd:
                    VariantCommand::Trials {
                        source,
                        limit,
                        offset,
                        ..
                    },
            } => {
                assert_eq!(source, "nci");
                assert_eq!(limit, 3);
                assert_eq!(offset, 0);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn search_trial_parses_new_filter_flags() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "search",
            "trial",
            "-c",
            "melanoma",
            "--facility",
            "MD Anderson",
            "--age",
            "67",
            "--sex",
            "female",
            "--sponsor-type",
            "nih",
            "--count-only",
            "--limit",
            "3",
        ])
        .expect("search trial new flags should parse");

        match cli.command {
            Commands::Search {
                entity:
                    super::SearchEntity::Trial {
                        facility,
                        age,
                        sex,
                        sponsor_type,
                        count_only,
                        limit,
                        ..
                    },
            } => {
                assert_eq!(facility.as_deref(), Some("MD Anderson"));
                assert_eq!(age, Some(67));
                assert_eq!(sex.as_deref(), Some("female"));
                assert_eq!(sponsor_type.as_deref(), Some("nih"));
                assert!(count_only);
                assert_eq!(limit, 3);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn article_entities_parses_limit_flag() {
        let cli =
            Cli::try_parse_from(["biomcp", "article", "entities", "22663011", "--limit", "2"])
                .expect("article entities with --limit should parse");
        match cli.command {
            Commands::Article {
                cmd: ArticleCommand::Entities { pmid, limit },
            } => {
                assert_eq!(pmid, "22663011");
                assert_eq!(limit, 2);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn gene_pathways_parses_limit_and_offset() {
        let cli = Cli::try_parse_from([
            "biomcp", "gene", "pathways", "BRAF", "--limit", "5", "--offset", "1",
        ])
        .expect("gene pathways pagination flags should parse");
        match cli.command {
            Commands::Gene {
                cmd:
                    GeneCommand::Pathways {
                        symbol,
                        limit,
                        offset,
                    },
            } => {
                assert_eq!(symbol, "BRAF");
                assert_eq!(limit, 5);
                assert_eq!(offset, 1);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn protein_structures_parses_offset_flag() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "protein",
            "structures",
            "P15056",
            "--limit",
            "5",
            "--offset",
            "5",
        ])
        .expect("protein structures pagination flags should parse");
        match cli.command {
            Commands::Protein {
                cmd:
                    ProteinCommand::Structures {
                        accession,
                        limit,
                        offset,
                    },
            } => {
                assert_eq!(accession, "P15056");
                assert_eq!(limit, 5);
                assert_eq!(offset, 5);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn search_variant_parses_positional_query() {
        let cli = Cli::try_parse_from(["biomcp", "search", "variant", "BRAF", "--limit", "2"])
            .expect("search variant positional query should parse");
        match cli.command {
            Commands::Search {
                entity:
                    super::SearchEntity::Variant {
                        gene,
                        positional_query,
                        limit,
                        ..
                    },
            } => {
                assert!(gene.is_none());
                assert_eq!(positional_query.as_deref(), Some("BRAF"));
                assert_eq!(limit, 2);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn truncate_article_annotations_applies_limit_per_bucket() {
        let annotations = crate::entities::article::ArticleAnnotations {
            genes: vec![
                crate::entities::article::AnnotationCount {
                    text: "BRAF".into(),
                    count: 2,
                },
                crate::entities::article::AnnotationCount {
                    text: "TP53".into(),
                    count: 1,
                },
            ],
            diseases: vec![
                crate::entities::article::AnnotationCount {
                    text: "melanoma".into(),
                    count: 2,
                },
                crate::entities::article::AnnotationCount {
                    text: "glioma".into(),
                    count: 1,
                },
            ],
            chemicals: vec![
                crate::entities::article::AnnotationCount {
                    text: "vemurafenib".into(),
                    count: 1,
                },
                crate::entities::article::AnnotationCount {
                    text: "dabrafenib".into(),
                    count: 1,
                },
            ],
            mutations: vec![
                crate::entities::article::AnnotationCount {
                    text: "V600E".into(),
                    count: 1,
                },
                crate::entities::article::AnnotationCount {
                    text: "L858R".into(),
                    count: 1,
                },
            ],
        };
        let truncated = truncate_article_annotations(annotations, 1);
        assert_eq!(truncated.genes.len(), 1);
        assert_eq!(truncated.diseases.len(), 1);
        assert_eq!(truncated.chemicals.len(), 1);
        assert_eq!(truncated.mutations.len(), 1);
    }

    #[tokio::test]
    async fn enrich_rejects_zero_limit_before_api_call() {
        let err = execute(vec![
            "biomcp".to_string(),
            "enrich".to_string(),
            "BRCA1,TP53".to_string(),
            "--limit".to_string(),
            "0".to_string(),
        ])
        .await
        .expect_err("enrich should reject --limit 0");
        assert!(err.to_string().contains("--limit must be between 1 and 50"));
    }

    #[tokio::test]
    async fn enrich_rejects_limit_above_max_before_api_call() {
        let err = execute(vec![
            "biomcp".to_string(),
            "enrich".to_string(),
            "BRCA1,TP53".to_string(),
            "--limit".to_string(),
            "51".to_string(),
        ])
        .await
        .expect_err("enrich should reject --limit > 50");
        assert!(err.to_string().contains("--limit must be between 1 and 50"));
    }
}
