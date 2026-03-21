//! Top-level CLI parsing and command execution.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand, ValueEnum};
use futures::{StreamExt, future::try_join_all};
use tracing::{debug, warn};

use crate::cli::debug_plan::{DebugPlan, DebugPlanLeg};

pub mod chart;
pub mod debug_plan;
pub mod discover;
pub mod health;
pub mod list;
pub mod search_all;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ChartType {
    Bar,
    Pie,
    Histogram,
    Density,
    Box,
    Violin,
    Ridgeline,
    Survival,
}

impl ChartType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bar => "bar",
            Self::Pie => "pie",
            Self::Histogram => "histogram",
            Self::Density => "density",
            Self::Box => "box",
            Self::Violin => "violin",
            Self::Ridgeline => "ridgeline",
            Self::Survival => "survival",
        }
    }
}

impl std::fmt::Display for ChartType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Args, Debug, Clone, PartialEq, Eq, Default)]
pub struct ChartArgs {
    #[arg(
        long,
        value_enum,
        hide_short_help = true,
        help_heading = "Chart Output"
    )]
    pub chart: Option<ChartType>,

    #[arg(
        long,
        requires = "chart",
        conflicts_with = "output",
        hide_short_help = true,
        help_heading = "Chart Output"
    )]
    pub terminal: bool,

    #[arg(
        short = 'o',
        long = "output",
        value_name = "FILE",
        requires = "chart",
        hide_short_help = true,
        help_heading = "Chart Output"
    )]
    pub output: Option<PathBuf>,

    #[arg(
        long,
        requires = "chart",
        hide_short_help = true,
        help_heading = "Chart Styling"
    )]
    pub title: Option<String>,

    #[arg(
        long,
        requires = "chart",
        hide_short_help = true,
        help_heading = "Chart Styling"
    )]
    pub theme: Option<String>,

    #[arg(
        long,
        requires = "chart",
        hide_short_help = true,
        help_heading = "Chart Styling"
    )]
    pub palette: Option<String>,

    #[arg(long, hide = true, requires = "chart")]
    pub mcp_inline: bool,
}

pub struct CliOutput {
    pub text: String,
    pub svg: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone)]
pub struct CommandOutcome {
    pub text: String,
    pub stream: OutputStream,
    pub exit_code: u8,
}

impl CommandOutcome {
    fn stdout(text: String) -> Self {
        Self {
            text,
            stream: OutputStream::Stdout,
            exit_code: 0,
        }
    }

    fn stdout_with_exit(text: String, exit_code: u8) -> Self {
        Self {
            text,
            stream: OutputStream::Stdout,
            exit_code,
        }
    }

    fn stderr_with_exit(text: String, exit_code: u8) -> Self {
        Self {
            text,
            stream: OutputStream::Stderr,
            exit_code,
        }
    }
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
    /// Local cBioPortal study analytics
    Study {
        #[command(subcommand)]
        cmd: StudyCommand,
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
    #[command(
        about = "Run the MCP Streamable HTTP server at /mcp",
        long_about = "Run the MCP Streamable HTTP server at /mcp.\n\nThis is the canonical remote/server deployment mode.\nHealth routes: GET /health, GET /readyz, GET /."
    )]
    ServeHttp {
        /// Host address to bind
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Port to listen on
        #[arg(long, default_value = "8080")]
        port: u16,
    },
    #[command(
        about = "removed legacy SSE compatibility command; use `serve-http`",
        long_about = "removed legacy SSE compatibility command.\n\ndeprecated users should run `biomcp serve-http` and connect remote clients to `/mcp` instead."
    )]
    ServeSse,
    /// BioMCP skill overview and installer for agents
    #[command(after_help = "\
EXAMPLES:
  biomcp skill            # show skill overview
  biomcp skill install    # install skill to your agent config")]
    Skill {
        #[command(subcommand)]
        command: Option<skill::SkillCommand>,
    },
    /// Chart type documentation for study visualizations
    #[command(after_help = "\
EXAMPLES:
  biomcp chart
  biomcp chart bar
  biomcp chart violin")]
    Chart {
        #[command(subcommand)]
        command: Option<chart::ChartCommand>,
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
        /// Optional entity name (gene, variant, article, trial, drug, disease, pgx, gwas, pathway, protein, study, adverse-event, search-all)
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
    /// Resolve free-text biomedical text into typed concepts and suggested commands
    #[command(after_help = "\
EXAMPLES:
  biomcp discover ERBB1
  biomcp discover Keytruda
  biomcp discover \"chest pain\"
  biomcp --json discover diabetes

See also: biomcp list discover")]
    Discover {
        /// Free-text biomedical query
        query: String,
    },
    /// Show version
    Version {
        /// Include executable provenance and PATH diagnostics
        #[arg(long)]
        verbose: bool,
    },
}

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug)]
pub enum SearchEntity {
    /// Cross-entity counts-first search card
    #[command(after_help = "\
EXAMPLES:
  biomcp search all --gene BRAF --disease melanoma
  biomcp search all --keyword resistance
  biomcp search all --gene BRAF --counts-only
  biomcp search all --gene BRAF --debug-plan

See also: biomcp list search-all")]
    All {
        /// Gene slot (e.g., BRAF)
        #[arg(short = 'g', long)]
        gene: Option<String>,
        /// Variant slot (e.g., \"BRAF V600E\")
        #[arg(short = 'v', long)]
        variant: Option<String>,
        /// Disease slot (e.g., melanoma)
        #[arg(short = 'd', long)]
        disease: Option<String>,
        /// Drug slot (e.g., dabrafenib)
        #[arg(long)]
        drug: Option<String>,
        /// Keyword slot
        #[arg(short = 'k', long)]
        keyword: Option<String>,
        /// Optional positional query alias for -k/--keyword
        #[arg(value_name = "QUERY")]
        positional_query: Option<String>,
        /// Date lower bound for date-capable sections (YYYY, YYYY-MM, or YYYY-MM-DD)
        #[arg(long)]
        since: Option<String>,
        /// Maximum rows per section (default: 3)
        #[arg(short, long, default_value = "3")]
        limit: usize,
        /// Render counts per section only (skip section rows)
        #[arg(long = "counts-only")]
        counts_only: bool,
        /// Include the executed multi-leg routing plan in markdown or JSON output
        #[arg(long = "debug-plan")]
        debug_plan: bool,
    },
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
        /// Optional positional query alias for -g/--gene
        #[arg(value_name = "QUERY")]
        positional_query: Option<String>,
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
        /// Optional positional query alias for -g/--gene
        #[arg(value_name = "QUERY")]
        positional_query: Option<String>,
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
    /// Search articles by gene, disease, drug, keyword, or author (PubTator3 + Europe PMC, optional Semantic Scholar)
    #[command(after_help = "\
EXAMPLES:
  biomcp search article \"BRAF resistance\"
  biomcp search article -q \"immunotherapy resistance\" --limit 5
  biomcp search article -g BRAF --date-from 2024-01-01
  biomcp search article -d melanoma --type review --journal Nature --limit 5
  biomcp search article -g BRAF --source pubtator --limit 20
  biomcp search article -g BRAF --debug-plan --limit 5

See also: biomcp list article")]
    Article {
        /// Filter by gene symbol
        #[arg(short, long)]
        gene: Option<String>,

        /// Filter by disease name
        #[arg(short, long, num_args = 1..)]
        disease: Vec<String>,

        /// Filter by drug/chemical name
        #[arg(long, num_args = 1..)]
        drug: Vec<String>,

        /// Filter by author name
        #[arg(short = 'a', long, num_args = 1..)]
        author: Vec<String>,

        /// Free text keyword search (alias: -q, --query)
        #[arg(
            short = 'k',
            long = "keyword",
            visible_short_alias = 'q',
            visible_alias = "query",
            num_args = 1..
        )]
        keyword: Vec<String>,
        /// Optional positional query alias for -k/--keyword/--query
        #[arg(value_name = "QUERY")]
        positional_query: Option<String>,

        /// Published after date (YYYY-MM-DD)
        #[arg(long = "date-from", alias = "since")]
        date_from: Option<String>,
        /// Published before date (YYYY-MM-DD)
        #[arg(long = "date-to", alias = "until")]
        date_to: Option<String>,

        // `long = "type"` is used instead of deriving from the field name because
        // `type` is a Rust reserved keyword. Internally we use `article_type`.
        /// Filter by publication type [values: research-article, review, case-reports, meta-analysis]
        #[arg(long = "type")]
        article_type: Option<String>,
        /// Filter by journal title
        #[arg(long, num_args = 1..)]
        journal: Vec<String>,

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

        /// Sort order [values: date, citations, relevance] (default: relevance)
        #[arg(long, default_value = "relevance", value_parser = ["date", "citations", "relevance"])]
        sort: String,

        /// Article source [values: all, pubtator, europepmc] (default: all)
        #[arg(long, default_value = "all", value_parser = ["all", "pubtator", "europepmc"])]
        source: String,

        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
        /// Include the executed search planner output in markdown or JSON output
        #[arg(long = "debug-plan")]
        debug_plan: bool,
    },
    /// Search trials by condition, intervention, mutation, or location (ClinicalTrials.gov)
    #[command(after_help = "\
EXAMPLES:
  biomcp search trial -c melanoma -s recruiting
  biomcp search trial -p 3 -i pembrolizumab
  biomcp search trial -c melanoma --facility \"MD Anderson\" --age 67 --limit 5
  biomcp search trial --age 0.5 --count-only          # infants eligible (6 months)
  biomcp search trial --mutation \"BRAF V600E\" --status recruiting --study-type interventional --has-results --limit 5
  biomcp search trial -c \"endometrial cancer\" --criteria \"mismatch repair deficient\" -s recruiting

Trial search is filter-based (no free-text query).
See also: biomcp list trial")]
    Trial {
        /// Filter by condition/disease
        #[arg(short = 'c', long, num_args = 1..)]
        condition: Vec<String>,
        /// Optional positional query alias for -c/--condition
        #[arg(value_name = "QUERY")]
        positional_query: Option<String>,

        /// Filter by intervention/drug
        #[arg(short = 'i', long, num_args = 1..)]
        intervention: Vec<String>,

        /// Filter by institution/facility name (text-search mode by default).
        ///
        /// Without `--lat`/`--lon`/`--distance`, this uses cheap CTGov
        /// `query.locn` text-search mode. With all three geo flags, it enters
        /// geo-verify mode and performs extra per-study location fetches to
        /// confirm the facility match within the requested distance. Geo-verify
        /// mode is materially more expensive, especially with `--count-only`.
        #[arg(long, num_args = 1..)]
        facility: Vec<String>,

        /// Filter by phase [values: NA, 1, 1/2, 2, 3, 4, EARLY_PHASE1, PHASE1, PHASE2, PHASE3, PHASE4].
        ///
        /// `1/2` matches the ClinicalTrials.gov combined Phase 1/Phase 2 label
        /// (studies tagged as both phases), not Phase 1 OR Phase 2.
        #[arg(short = 'p', long)]
        phase: Option<String>,
        /// Study type (e.g., interventional, observational)
        #[arg(long = "study-type")]
        study_type: Option<String>,

        /// Patient age in years for eligibility matching (decimals accepted, e.g. 0.5 for 6 months).
        ///
        /// With `--count-only`, age-only CTGov searches report an approximate
        /// upstream total because BioMCP applies the age filter during full
        /// search, not the fast count path.
        #[arg(long)]
        age: Option<f32>,

        /// Eligible sex filter [values: female, male, all].
        ///
        /// `all` (also `any`/`both`) resolves to no sex restriction, so no sex
        /// filter is sent to ClinicalTrials.gov. Use `female` or `male` to
        /// apply an actual restriction.
        #[arg(long)]
        sex: Option<String>,

        /// Filter by trial status [values: recruiting, not_yet_recruiting, enrolling_by_invitation, active_not_recruiting, completed, suspended, terminated, withdrawn]
        #[arg(short = 's', long)]
        status: Option<String>,

        /// Search mutation-related ClinicalTrials.gov text fields (best-effort)
        #[arg(long, num_args = 1..)]
        mutation: Vec<String>,

        /// Search eligibility criteria with free-text terms (best-effort)
        #[arg(long, num_args = 1..)]
        criteria: Vec<String>,

        /// Biomarker filter (NCI CTS; best-effort for ctgov)
        #[arg(long, num_args = 1..)]
        biomarker: Vec<String>,

        /// Prior therapy mentioned in eligibility
        #[arg(long, num_args = 1..)]
        prior_therapies: Vec<String>,

        /// Drug/therapy patient progressed on
        #[arg(long, num_args = 1..)]
        progression_on: Vec<String>,

        /// Line of therapy: 1L, 2L, 3L+
        #[arg(long)]
        line_of_therapy: Option<String>,

        /// Filter by sponsor (best-effort)
        #[arg(long, num_args = 1..)]
        sponsor: Vec<String>,

        /// Sponsor/funder category [values: nih, industry, fed, other]
        #[arg(long = "sponsor-type")]
        sponsor_type: Option<String>,

        /// Trials updated after date (YYYY-MM-DD)
        #[arg(long = "date-from", alias = "since")]
        date_from: Option<String>,
        /// Trials updated before date (YYYY-MM-DD)
        #[arg(long = "date-to", alias = "until")]
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
    /// Search variants by gene, shorthand alias, significance, frequency, or consequence (ClinVar/gnomAD)
    #[command(after_help = "\
EXAMPLES:
  biomcp search variant BRAF --limit 5
  biomcp search variant \"PTPN22 620W\" --limit 5
  biomcp search variant -g PTPN22 R620W --limit 5
  biomcp search variant BRAF p.Val600Glu --limit 5
  biomcp search variant -g BRAF --significance pathogenic
  biomcp search variant -g BRCA1 --review-status 2 --revel-min 0.7 --consequence missense_variant --limit 5
  biomcp search variant --hgvsp p.Val600Glu -g BRAF --limit 5

For variant mentions in trials: biomcp variant trials \"BRAF V600E\"
See also: biomcp list variant")]
    Variant {
        /// Filter by gene symbol
        #[arg(short = 'g', long)]
        gene: Option<String>,
        /// Optional positional query tokens
        #[arg(value_name = "QUERY", num_args = 0..)]
        positional_query: Vec<String>,

        /// Filter by protein change (e.g., V600E, p.V600E, or p.Val600Glu)
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
  biomcp search drug -q \"kinase inhibitor\" --target EGFR --atc L01 --pharm-class kinase --limit 5

Note: --interactions is currently unavailable from the public data sources BioMCP uses.

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
        /// Filter by interaction partner drug name (currently unavailable from public data sources)
        #[arg(long)]
        interactions: Option<String>,

        /// Maximum results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Search pathways by name or keyword
    #[command(
        override_usage = "biomcp search pathway [OPTIONS] <QUERY>\n       biomcp search pathway [OPTIONS] --top-level [QUERY]",
        after_help = "\
EXAMPLES:
  biomcp search pathway \"MAPK signaling\"
  biomcp search pathway \"Pathways in cancer\" --limit 5
  biomcp search pathway -q \"DNA repair\" --limit 5
  biomcp search pathway --top-level --limit 5

See also: biomcp list pathway"
    )]
    Pathway {
        /// Free text query (pathway name, process, keyword)
        #[arg(short, long)]
        query: Option<String>,
        /// Positional alias for -q/--query; required unless --top-level is present, and multi-word queries must be quoted
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
        /// Optional positional query alias for -d/--drug
        #[arg(value_name = "QUERY")]
        positional_query: Option<String>,

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
        #[arg(long = "date-to", alias = "until")]
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
  biomcp get gene BRAF hpa

See also: biomcp list gene")]
    Gene {
        /// Gene symbol (e.g., BRAF, TP53, EGFR)
        symbol: String,
        /// Sections to include (pathways, ontology, diseases, protein, go, interactions, civic, expression, hpa, druggability, clingen, constraint, disgenet, all)
        #[arg(trailing_var_arg = true)]
        sections: Vec<String>,
    },
    /// Get article by PMID, PMCID, or DOI
    #[command(after_help = "\
EXAMPLES:
  biomcp get article 22663011
  biomcp get article 22663011 annotations
  biomcp get article 22663011 tldr

See also: biomcp list article")]
    Article {
        /// PMID (e.g., 22663011), PMCID (e.g., PMC9984800), or DOI (e.g., 10.1056/NEJMoa1203421)
        id: String,
        /// Sections to include (annotations, fulltext, tldr, all)
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
        /// Sections to include (genes, pathways, phenotypes, variants, models, prevalence, civic, disgenet, all)
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
    /// Get variant by exact rsID, HGVS, or "GENE CHANGE" (e.g., "BRAF V600E" or "BRAF p.Val600Glu")
    #[command(after_help = "\
EXAMPLES:
  biomcp get variant rs113488022
  biomcp get variant \"BRAF V600E\" clinvar
  biomcp get variant \"BRAF p.Val600Glu\"

Shorthand like \"PTPN22 620W\" or \"R620W\" should go through `biomcp search variant`.

See also: biomcp list variant")]
    Variant {
        /// Exact rsID, HGVS, or "GENE CHANGE" (e.g., rs113488022, "BRAF V600E", "BRAF p.Val600Glu")
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
  biomcp get drug pembrolizumab approvals

See also: biomcp list drug")]
    Drug {
        /// Drug name (e.g., pembrolizumab, carboplatin)
        name: String,
        /// Sections to include (label, shortage, targets, indications, interactions, civic, approvals, all)
        #[arg(trailing_var_arg = true)]
        sections: Vec<String>,
    },
    /// Get pathway by ID
    #[command(after_help = "\
EXAMPLES:
  biomcp get pathway R-HSA-5673001
  biomcp get pathway hsa05200
  biomcp get pathway R-HSA-5673001 genes
  biomcp get pathway R-HSA-5673001 events

See also: biomcp list pathway")]
    Pathway {
        /// Pathway ID (e.g., R-HSA-5673001, hsa05200)
        id: String,
        /// Sections to include (genes, events (Reactome only), enrichment (Reactome only), all = all sections available for the resolved source)
        #[arg(trailing_var_arg = true)]
        sections: Vec<String>,
    },
    /// Get protein by UniProt accession or gene symbol
    #[command(after_help = "\
EXAMPLES:
  biomcp get protein P15056
  biomcp get protein P15056 complexes
  biomcp get protein P15056 structures

See also: biomcp list protein")]
    Protein {
        /// UniProt accession or HGNC symbol (e.g., P15056 or BRAF)
        accession: String,
        /// Sections to include (domains, interactions, complexes, structures, all)
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
    /// Search trials mentioning the variant in mutation-related text fields (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp variant trials \"BRAF V600E\" --limit 5
  biomcp variant trials \"BRAF V600E\" --source nci --limit 5
  biomcp variant trials rs113488022 --limit 5

Note: Searches ClinicalTrials.gov mutation-related free-text fields, including eligibility, title, summary, and keywords. Results depend on source document wording.
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
    #[command(external_subcommand)]
    External(Vec<String>),
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
    #[command(external_subcommand)]
    External(Vec<String>),
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
    /// Fetch compact summary cards for multiple known article IDs
    #[command(after_help = "\
EXAMPLES:
  biomcp article batch 22663011 24200969
  biomcp article batch 22663011 10.1056/NEJMoa1203421 --json

Returns compact multi-article summary cards for anchor selection.
S2_API_KEY adds optional TLDR and citation metadata when available.
See also: biomcp list article")]
    Batch {
        /// PMIDs, PMCIDs, or DOIs
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
    },
    /// Traverse citing papers with Semantic Scholar contexts and intents
    #[command(after_help = "\
EXAMPLES:
  biomcp article citations 22663011 --limit 5
  biomcp article citations PMC9984800 --limit 5

Requires: S2_API_KEY
See also: biomcp list article")]
    Citations {
        /// PMID, PMCID, DOI, arXiv ID, or Semantic Scholar paper ID
        id: String,
        /// Maximum rows (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Traverse referenced papers with Semantic Scholar contexts and intents
    #[command(after_help = "\
EXAMPLES:
  biomcp article references 22663011 --limit 5
  biomcp article references 10.1056/NEJMoa1203421 --limit 5

Requires: S2_API_KEY
See also: biomcp list article")]
    References {
        /// PMID, PMCID, DOI, arXiv ID, or Semantic Scholar paper ID
        id: String,
        /// Maximum rows (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Find related papers from one or more positive seeds
    #[command(after_help = "\
EXAMPLES:
  biomcp article recommendations 22663011 --limit 5
  biomcp article recommendations 22663011 24200969 --negative 39073865 --limit 5

Requires: S2_API_KEY
See also: biomcp list article")]
    Recommendations {
        /// One or more positive seeds (PMID, PMCID, DOI, arXiv ID, or Semantic Scholar paper ID)
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
        /// One or more negative seeds
        #[arg(long = "negative")]
        negative: Vec<String>,
        /// Maximum rows (default: 10)
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
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Subcommand, Debug)]
pub enum PathwayCommand {
    /// Search drugs linked to genes in this pathway (best-effort)
    #[command(after_help = "\
EXAMPLES:
  biomcp pathway drugs R-HSA-5673001 --limit 5
  biomcp pathway drugs hsa05200 --limit 5
  biomcp pathway drugs R-HSA-6802957 --limit 5

Note: Searches free-text fields (e.g., eligibility criteria). Results depend on source document wording.
See also: biomcp list pathway")]
    Drugs {
        /// Pathway ID (e.g., R-HSA-5673001, hsa05200)
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
  biomcp pathway articles hsa05200 --limit 5
  biomcp pathway articles R-HSA-6802957 --limit 5

Note: Searches free-text fields (e.g., eligibility criteria). Results depend on source document wording.
See also: biomcp list pathway")]
    Articles {
        /// Pathway ID (e.g., R-HSA-5673001, hsa05200)
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
  biomcp pathway trials hsa05200 --limit 5
  biomcp pathway trials R-HSA-5673001 --source nci --limit 5

Note: Searches free-text fields (e.g., eligibility criteria). Results depend on source document wording.
See also: biomcp list pathway")]
    Trials {
        /// Pathway ID (e.g., R-HSA-5673001, hsa05200)
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

#[derive(Subcommand, Debug)]
pub enum StudyCommand {
    /// List locally available cBioPortal studies
    #[command(after_help = "\
EXAMPLES:
  biomcp study list

See also: biomcp list study")]
    List,
    /// Download a cBioPortal study into the local study directory
    #[command(after_help = "\
EXAMPLES:
  biomcp study download --list
  biomcp study download msk_impact_2017
  biomcp study download brca_tcga_pan_can_atlas_2018

See also: biomcp list study")]
    Download {
        /// List downloadable remote study IDs
        #[arg(long, conflicts_with = "study_id")]
        list: bool,
        /// Study identifier (e.g., msk_impact_2017)
        #[arg(value_name = "STUDY_ID", required_unless_present = "list")]
        study_id: Option<String>,
    },
    /// Run a study-scoped query for one gene
    #[command(after_help = "\
EXAMPLES:
  biomcp study query --study msk_impact_2017 --gene TP53 --type mutations
  biomcp study query --study brca_tcga_pan_can_atlas_2018 --gene ERBB2 --type cna
  biomcp study query --study paad_qcmg_uq_2016 --gene KRAS --type expression

See also: biomcp list study")]
    Query {
        /// Study identifier (e.g., msk_impact_2017)
        #[arg(short, long)]
        study: String,
        /// HGNC gene symbol (e.g., TP53)
        #[arg(short, long)]
        gene: String,
        /// Query type (mutations, cna, expression)
        #[arg(short = 't', long = "type")]
        query_type: String,
        #[command(flatten)]
        chart: ChartArgs,
    },
    /// Filter samples across mutation, CNA, expression, and clinical criteria
    #[command(after_help = "\
EXAMPLES:
  biomcp study filter --study msk_impact_2017 --mutated TP53
  biomcp study filter --study brca_tcga_pan_can_atlas_2018 --mutated TP53 --amplified ERBB2
  biomcp study filter --study brca_tcga_pan_can_atlas_2018 --mutated TP53 --expression-above ERBB2:1.5 --cancer-type \"Breast Cancer\"

See also: biomcp list study")]
    Filter {
        /// Study identifier (e.g., brca_tcga_pan_can_atlas_2018)
        #[arg(short, long)]
        study: String,
        /// Gene with at least one mutation (repeatable)
        #[arg(long)]
        mutated: Vec<String>,
        /// Gene with CNA amplification, value == 2 (repeatable)
        #[arg(long)]
        amplified: Vec<String>,
        /// Gene with CNA deep deletion, value == -2 (repeatable)
        #[arg(long)]
        deleted: Vec<String>,
        /// Gene with expression above threshold, GENE:THRESHOLD (repeatable)
        #[arg(long = "expression-above")]
        expression_above: Vec<String>,
        /// Gene with expression below threshold, GENE:THRESHOLD (repeatable)
        #[arg(long = "expression-below")]
        expression_below: Vec<String>,
        /// Cancer type filter, case-insensitive exact match (repeatable)
        #[arg(long = "cancer-type")]
        cancer_type: Vec<String>,
    },
    /// Define a cohort split by mutation status
    #[command(after_help = "\
EXAMPLES:
  biomcp study cohort --study brca_tcga_pan_can_atlas_2018 --gene TP53

See also: biomcp list study")]
    Cohort {
        /// Study identifier (e.g., brca_tcga_pan_can_atlas_2018)
        #[arg(short, long)]
        study: String,
        /// HGNC gene symbol (e.g., TP53)
        #[arg(short, long)]
        gene: String,
    },
    /// Compare mutation-stratified groups on survival outcomes
    #[command(after_help = "\
EXAMPLES:
  biomcp study survival --study brca_tcga_pan_can_atlas_2018 --gene TP53
  biomcp study survival --study brca_tcga_pan_can_atlas_2018 --gene TP53 --endpoint DFS

See also: biomcp list study")]
    Survival {
        /// Study identifier (e.g., brca_tcga_pan_can_atlas_2018)
        #[arg(short, long)]
        study: String,
        /// HGNC gene symbol (e.g., TP53)
        #[arg(short, long)]
        gene: String,
        /// Survival endpoint (os, dfs, pfs, dss). Default: os
        #[arg(short, long, default_value = "os")]
        endpoint: String,
        #[command(flatten)]
        chart: ChartArgs,
    },
    /// Compare mutation-stratified groups on expression or mutation rates
    #[command(after_help = "\
EXAMPLES:
  biomcp study compare --study brca_tcga_pan_can_atlas_2018 --gene TP53 --type expression --target ERBB2
  biomcp study compare --study brca_tcga_pan_can_atlas_2018 --gene TP53 --type mutations --target PIK3CA

See also: biomcp list study")]
    Compare {
        /// Study identifier (e.g., brca_tcga_pan_can_atlas_2018)
        #[arg(short, long)]
        study: String,
        /// Gene for cohort stratification (e.g., TP53)
        #[arg(short, long)]
        gene: String,
        /// Comparison type (expression or mutations)
        #[arg(short = 't', long = "type")]
        compare_type: String,
        /// Target gene to compare across groups
        #[arg(long)]
        target: String,
        #[command(flatten)]
        chart: ChartArgs,
    },
    /// Compute pairwise mutation co-occurrence across genes
    #[command(after_help = "\
EXAMPLES:
  biomcp study co-occurrence --study msk_impact_2017 --genes TP53,KRAS
  biomcp study co-occurrence --study brca_tcga_pan_can_atlas_2018 --genes TP53,PIK3CA,GATA3

See also: biomcp list study")]
    CoOccurrence {
        /// Study identifier (e.g., msk_impact_2017)
        #[arg(short, long)]
        study: String,
        /// Comma-separated gene symbols (2..=10)
        #[arg(short, long)]
        genes: String,
        #[command(flatten)]
        chart: ChartArgs,
    },
}

fn empty_sections() -> &'static [String] {
    &[]
}

fn related_article_filters() -> crate::entities::article::ArticleSearchFilters {
    crate::entities::article::ArticleSearchFilters {
        gene: None,
        gene_anchored: false,
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
        exclude_retracted: true,
        sort: crate::entities::article::ArticleSort::Relevance,
    }
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

fn parse_expression_filter(
    value: &str,
    flag: &str,
    make_criterion: impl FnOnce(String, f64) -> crate::entities::study::FilterCriterion,
) -> Result<crate::entities::study::FilterCriterion, crate::error::BioMcpError> {
    let trimmed = value.trim();
    let invalid = || {
        crate::error::BioMcpError::InvalidArgument(format!(
            "Invalid value '{trimmed}' for {flag}. Expected GENE:THRESHOLD."
        ))
    };

    let (gene, threshold) = trimmed.split_once(':').ok_or_else(invalid)?;
    let gene = gene.trim();
    let threshold = threshold.trim();
    if gene.is_empty() || threshold.is_empty() {
        return Err(invalid());
    }
    let threshold = threshold.parse::<f64>().map_err(|_| invalid())?;
    Ok(make_criterion(gene.to_string(), threshold))
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

fn chart_json_conflict(
    chart: &ChartArgs,
    json_output: bool,
) -> Result<(), crate::error::BioMcpError> {
    if json_output && chart.chart.is_some() {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--json cannot be combined with --chart. Use standard study output for JSON, or remove --json for chart rendering.".into(),
        ));
    }
    Ok(())
}

fn mcp_output_flag_error() -> crate::error::BioMcpError {
    crate::error::BioMcpError::InvalidArgument(
        "MCP chart responses do not support --output/-o. Omit file output and consume the inline SVG image content instead.".into(),
    )
}

fn is_charted_mcp_study_command(cli: &Cli) -> Result<bool, crate::error::BioMcpError> {
    let chart = match &cli.command {
        Commands::Study {
            cmd:
                StudyCommand::Query { chart, .. }
                | StudyCommand::Survival { chart, .. }
                | StudyCommand::Compare { chart, .. }
                | StudyCommand::CoOccurrence { chart, .. },
        } => chart,
        _ => return Ok(false),
    };

    if chart.chart.is_none() || cli.json {
        return Ok(false);
    }
    if chart.output.is_some() {
        return Err(mcp_output_flag_error());
    }
    Ok(true)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum McpChartPass {
    Text,
    Svg,
}

fn require_flag_value(
    args: &[String],
    index: usize,
    flag: &str,
) -> Result<String, crate::error::BioMcpError> {
    args.get(index + 1).cloned().ok_or_else(|| {
        crate::error::BioMcpError::InvalidArgument(format!("{flag} requires a value"))
    })
}

fn rewrite_mcp_chart_args(
    args: &[String],
    pass: McpChartPass,
) -> Result<Vec<String>, crate::error::BioMcpError> {
    let mut rewritten = Vec::with_capacity(args.len() + 1);
    rewritten.push(
        args.first()
            .cloned()
            .unwrap_or_else(|| "biomcp".to_string()),
    );

    let mut i = 1usize;
    let mut saw_inline_flag = false;
    while i < args.len() {
        let token = &args[i];
        match token.as_str() {
            "--chart" => {
                let value = require_flag_value(args, i, "--chart")?;
                if pass == McpChartPass::Svg {
                    rewritten.push(token.clone());
                    rewritten.push(value);
                }
                i += 2;
            }
            "--terminal" => {
                i += 1;
            }
            "--output" => {
                if pass == McpChartPass::Svg {
                    return Err(mcp_output_flag_error());
                }
                let _ = require_flag_value(args, i, "--output")?;
                i += 2;
            }
            "-o" => {
                if pass == McpChartPass::Svg {
                    return Err(mcp_output_flag_error());
                }
                let _ = require_flag_value(args, i, "-o")?;
                i += 2;
            }
            "--title" | "--theme" | "--palette" => {
                let value = require_flag_value(args, i, token)?;
                if pass == McpChartPass::Svg {
                    rewritten.push(token.clone());
                    rewritten.push(value);
                }
                i += 2;
            }
            "--mcp-inline" => {
                if pass == McpChartPass::Svg {
                    rewritten.push(token.clone());
                }
                saw_inline_flag = true;
                i += 1;
            }
            _ => {
                if token.starts_with("--chart=") {
                    if pass == McpChartPass::Svg {
                        rewritten.push(token.clone());
                    }
                    i += 1;
                    continue;
                }
                if token.starts_with("--output=") || token.starts_with("-o=") {
                    if pass == McpChartPass::Svg {
                        return Err(mcp_output_flag_error());
                    }
                    i += 1;
                    continue;
                }
                if token.starts_with("-o") && token.len() > 2 {
                    if pass == McpChartPass::Svg {
                        return Err(mcp_output_flag_error());
                    }
                    i += 1;
                    continue;
                }
                if token.starts_with("--title=")
                    || token.starts_with("--theme=")
                    || token.starts_with("--palette=")
                {
                    if pass == McpChartPass::Svg {
                        rewritten.push(token.clone());
                    }
                    i += 1;
                    continue;
                }
                rewritten.push(token.clone());
                i += 1;
            }
        }
    }

    if pass == McpChartPass::Svg && !saw_inline_flag {
        rewritten.push("--mcp-inline".to_string());
    }
    Ok(rewritten)
}

fn normalize_cli_tokens(values: Vec<String>) -> Option<String> {
    let joined = values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    normalize_cli_query(Some(joined))
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

fn parse_simple_gene_change(query: &str) -> Option<(String, String)> {
    let parts = query.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 2 {
        return None;
    }

    let gene = parts[0].trim();
    let change = parts[1]
        .trim()
        .trim_start_matches("p.")
        .trim_start_matches("P.");
    if gene.is_empty() || change.is_empty() {
        return None;
    }

    let candidate = format!("{gene} {change}");
    match crate::entities::variant::parse_variant_id(&candidate).ok()? {
        crate::entities::variant::VariantIdFormat::GeneProteinChange { gene, change } => {
            Some((gene, change))
        }
        _ => None,
    }
}

fn parse_gene_c_hgvs(query: &str) -> Option<(String, String)> {
    let parts = query.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 2 {
        return None;
    }

    let gene = parts[0].trim();
    let change = parts[1].trim();
    if gene.is_empty() || change.is_empty() || !crate::sources::is_valid_gene_symbol(gene) {
        return None;
    }
    if !change.starts_with("c.") && !change.starts_with("C.") {
        return None;
    }
    Some((gene.to_string(), format!("c.{}", change[2..].trim())))
}

fn parse_exon_deletion_phrase(query: &str) -> Option<(String, String)> {
    let parts = query.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 4 {
        return None;
    }

    let gene = parts[0].trim();
    if !crate::sources::is_valid_gene_symbol(gene)
        || !parts[1].eq_ignore_ascii_case("exon")
        || parts[2].parse::<u32>().ok().is_none()
        || !parts[3].eq_ignore_ascii_case("deletion")
    {
        return None;
    }

    Some((gene.to_string(), "inframe_deletion".to_string()))
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ResolvedVariantQuery {
    gene: Option<String>,
    hgvsp: Option<String>,
    hgvsc: Option<String>,
    rsid: Option<String>,
    protein_alias: Option<crate::entities::variant::VariantProteinAlias>,
    consequence: Option<String>,
    condition: Option<String>,
}

#[derive(Debug, Clone)]
struct VariantSearchRequest {
    gene: Option<String>,
    positional_query: Vec<String>,
    hgvsp: Option<String>,
    significance: Option<String>,
    max_frequency: Option<f64>,
    min_cadd: Option<f64>,
    consequence: Option<String>,
    review_status: Option<String>,
    population: Option<String>,
    revel_min: Option<f64>,
    gerp_min: Option<f64>,
    tumor_site: Option<String>,
    condition: Option<String>,
    impact: Option<String>,
    lof: bool,
    has: Option<String>,
    missing: Option<String>,
    therapy: Option<String>,
    limit: usize,
    offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum VariantSearchPlan {
    Standard(ResolvedVariantQuery),
    Guidance(crate::entities::variant::VariantGuidance),
}

fn resolve_variant_query(
    gene_flag: Option<String>,
    hgvsp_flag: Option<String>,
    consequence_flag: Option<String>,
    condition_flag: Option<String>,
    positional_tokens: Vec<String>,
) -> Result<VariantSearchPlan, crate::error::BioMcpError> {
    let gene_flag = normalize_cli_query(gene_flag);
    let hgvsp_flag = normalize_cli_query(hgvsp_flag)
        .map(|value| crate::entities::variant::normalize_protein_change(&value).unwrap_or(value));
    let consequence_flag = normalize_cli_query(consequence_flag);
    let condition_flag = normalize_cli_query(condition_flag);

    let positional = positional_tokens
        .iter()
        .map(|token| token.trim())
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let positional = normalize_cli_query(Some(positional));

    let Some(query) = positional else {
        return Ok(VariantSearchPlan::Standard(ResolvedVariantQuery {
            gene: gene_flag,
            hgvsp: hgvsp_flag,
            consequence: consequence_flag,
            condition: condition_flag,
            ..Default::default()
        }));
    };

    let token_count = query.split_whitespace().count();
    if token_count <= 1 {
        if let Ok(crate::entities::variant::VariantIdFormat::RsId(rsid)) =
            crate::entities::variant::parse_variant_id(&query)
        {
            if gene_flag.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "Use either positional QUERY or --gene, not both".into(),
                ));
            }
            return Ok(VariantSearchPlan::Standard(ResolvedVariantQuery {
                rsid: Some(rsid),
                hgvsp: hgvsp_flag,
                consequence: consequence_flag,
                condition: condition_flag,
                ..Default::default()
            }));
        }

        if let Some(gene) = gene_flag.clone() {
            if let Some(protein_alias) =
                crate::entities::variant::parse_variant_protein_alias(&query)
            {
                if hgvsp_flag.is_some() {
                    return Err(crate::error::BioMcpError::InvalidArgument(
                        "Positional residue alias conflicts with --hgvsp".into(),
                    ));
                }
                return Ok(VariantSearchPlan::Standard(ResolvedVariantQuery {
                    gene: Some(gene),
                    protein_alias: Some(protein_alias),
                    consequence: consequence_flag,
                    condition: condition_flag,
                    ..Default::default()
                }));
            }
            if let crate::entities::variant::VariantInputKind::Shorthand(
                crate::entities::variant::VariantShorthand::ProteinChangeOnly { change },
            ) = crate::entities::variant::classify_variant_input(&query)
            {
                if hgvsp_flag.is_some() {
                    return Err(crate::error::BioMcpError::InvalidArgument(
                        "Positional protein change conflicts with --hgvsp".into(),
                    ));
                }
                return Ok(VariantSearchPlan::Standard(ResolvedVariantQuery {
                    gene: Some(gene),
                    hgvsp: Some(change),
                    consequence: consequence_flag,
                    condition: condition_flag,
                    ..Default::default()
                }));
            }
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Use either positional QUERY or --gene, not both".into(),
            ));
        }

        if let Some(guidance) = crate::entities::variant::variant_guidance(&query) {
            return Ok(VariantSearchPlan::Guidance(guidance));
        }
        return Ok(VariantSearchPlan::Standard(ResolvedVariantQuery {
            gene: Some(query),
            hgvsp: hgvsp_flag,
            consequence: consequence_flag,
            condition: condition_flag,
            ..Default::default()
        }));
    }

    if let Some((gene, change)) = parse_simple_gene_change(&query) {
        if gene_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional \"GENE CHANGE\" conflicts with --gene".into(),
            ));
        }
        if hgvsp_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional \"GENE CHANGE\" conflicts with --hgvsp".into(),
            ));
        }
        return Ok(VariantSearchPlan::Standard(ResolvedVariantQuery {
            gene: Some(gene),
            hgvsp: Some(change),
            consequence: consequence_flag,
            condition: condition_flag,
            ..Default::default()
        }));
    }

    if let crate::entities::variant::VariantInputKind::Shorthand(
        crate::entities::variant::VariantShorthand::GeneResidueAlias {
            gene,
            position,
            residue,
            ..
        },
    ) = crate::entities::variant::classify_variant_input(&query)
    {
        if gene_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional residue alias conflicts with --gene".into(),
            ));
        }
        if hgvsp_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional residue alias conflicts with --hgvsp".into(),
            ));
        }
        return Ok(VariantSearchPlan::Standard(ResolvedVariantQuery {
            gene: Some(gene),
            protein_alias: Some(crate::entities::variant::VariantProteinAlias {
                position,
                residue,
            }),
            consequence: consequence_flag,
            condition: condition_flag,
            ..Default::default()
        }));
    }

    if let Some((gene, hgvsc)) = parse_gene_c_hgvs(&query) {
        if gene_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional \"GENE c.HGVS\" conflicts with --gene".into(),
            ));
        }
        return Ok(VariantSearchPlan::Standard(ResolvedVariantQuery {
            gene: Some(gene),
            hgvsp: hgvsp_flag,
            hgvsc: Some(hgvsc),
            consequence: consequence_flag,
            condition: condition_flag,
            ..Default::default()
        }));
    }

    if let Some((gene, consequence)) = parse_exon_deletion_phrase(&query) {
        if gene_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional exon-deletion query conflicts with --gene".into(),
            ));
        }
        if consequence_flag.is_some() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "Positional exon-deletion query conflicts with --consequence".into(),
            ));
        }
        return Ok(VariantSearchPlan::Standard(ResolvedVariantQuery {
            gene: Some(gene),
            hgvsp: hgvsp_flag,
            consequence: Some(consequence),
            condition: condition_flag,
            ..Default::default()
        }));
    }

    if condition_flag.is_some() {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "Use either positional QUERY or --condition, not both".into(),
        ));
    }
    Ok(VariantSearchPlan::Standard(ResolvedVariantQuery {
        gene: gene_flag,
        hgvsp: hgvsp_flag,
        consequence: consequence_flag,
        condition: Some(query),
        ..Default::default()
    }))
}

async fn render_gene_card(
    symbol: &str,
    sections: &[String],
    json_output: bool,
) -> anyhow::Result<String> {
    let gene = crate::entities::gene::get(symbol, sections).await?;
    if json_output {
        Ok(crate::render::json::to_entity_json(
            &gene,
            crate::render::markdown::gene_evidence_urls(&gene),
            crate::render::markdown::related_gene(&gene),
            crate::render::provenance::gene_section_sources(&gene),
        )?)
    } else {
        Ok(crate::render::markdown::gene_markdown(&gene, sections)?)
    }
}

fn alias_suggestion_markdown(
    query: &str,
    requested_entity: crate::entities::discover::DiscoverType,
    decision: &crate::entities::discover::AliasFallbackDecision,
) -> String {
    let err = crate::error::BioMcpError::NotFound {
        entity: requested_entity.cli_name().to_string(),
        id: query.trim().to_string(),
        suggestion: crate::render::markdown::alias_fallback_suggestion(decision),
    };
    format!("Error: {err}")
}

fn alias_suggestion_outcome(
    query: &str,
    requested_entity: crate::entities::discover::DiscoverType,
    decision: &crate::entities::discover::AliasFallbackDecision,
    json_output: bool,
) -> anyhow::Result<CommandOutcome> {
    if json_output {
        return Ok(CommandOutcome::stdout_with_exit(
            crate::render::json::to_alias_suggestion_json(decision)?,
            1,
        ));
    }
    Ok(CommandOutcome::stderr_with_exit(
        alias_suggestion_markdown(query, requested_entity, decision),
        1,
    ))
}

fn variant_guidance_markdown(guidance: &crate::entities::variant::VariantGuidance) -> String {
    let err = crate::error::BioMcpError::NotFound {
        entity: "variant".into(),
        id: guidance.query.clone(),
        suggestion: crate::render::markdown::variant_guidance_suggestion(guidance),
    };
    format!("Error: {err}")
}

fn variant_guidance_outcome(
    guidance: &crate::entities::variant::VariantGuidance,
    json_output: bool,
) -> anyhow::Result<CommandOutcome> {
    if json_output {
        return Ok(CommandOutcome::stdout_with_exit(
            crate::render::json::to_variant_guidance_json(guidance)?,
            1,
        ));
    }
    Ok(CommandOutcome::stderr_with_exit(
        variant_guidance_markdown(guidance),
        1,
    ))
}

async fn try_alias_fallback_outcome(
    query: &str,
    requested_entity: crate::entities::discover::DiscoverType,
    json_output: bool,
) -> anyhow::Result<Option<CommandOutcome>> {
    match crate::cli::discover::resolve_query(
        query,
        crate::cli::discover::DiscoverMode::AliasFallback,
    )
    .await
    {
        Ok(result) => {
            let decision =
                crate::entities::discover::classify_alias_fallback(&result, requested_entity);
            match decision {
                crate::entities::discover::AliasFallbackDecision::None => Ok(None),
                other => Ok(Some(alias_suggestion_outcome(
                    query,
                    requested_entity,
                    &other,
                    json_output,
                )?)),
            }
        }
        Err(err) => {
            warn!(
                query = query.trim(),
                entity = requested_entity.cli_name(),
                "alias fallback discovery unavailable: {err}"
            );
            Ok(None)
        }
    }
}

async fn render_gene_card_outcome(
    symbol: &str,
    sections: &[String],
    json_output: bool,
    alias_suggestions_as_json: bool,
) -> anyhow::Result<CommandOutcome> {
    match crate::entities::gene::get(symbol, sections).await {
        Ok(gene) => {
            let text = if json_output {
                crate::render::json::to_entity_json(
                    &gene,
                    crate::render::markdown::gene_evidence_urls(&gene),
                    crate::render::markdown::related_gene(&gene),
                    crate::render::provenance::gene_section_sources(&gene),
                )?
            } else {
                crate::render::markdown::gene_markdown(&gene, sections)?
            };
            Ok(CommandOutcome::stdout(text))
        }
        Err(err @ crate::error::BioMcpError::NotFound { .. }) => {
            if let Some(outcome) = try_alias_fallback_outcome(
                symbol,
                crate::entities::discover::DiscoverType::Gene,
                json_output || alias_suggestions_as_json,
            )
            .await?
            {
                Ok(outcome)
            } else {
                Err(err.into())
            }
        }
        Err(err) => Err(err.into()),
    }
}

async fn render_variant_card_outcome(
    id: &str,
    sections: &[String],
    json_output: bool,
    guidance_as_json: bool,
) -> anyhow::Result<CommandOutcome> {
    if let Some(guidance) = crate::entities::variant::variant_guidance(id) {
        return variant_guidance_outcome(&guidance, json_output || guidance_as_json);
    }

    match crate::entities::variant::get(id, sections).await {
        Ok(variant) => {
            let text = if json_output {
                crate::render::json::to_entity_json(
                    &variant,
                    crate::render::markdown::variant_evidence_urls(&variant),
                    crate::render::markdown::related_variant(&variant),
                    crate::render::provenance::variant_section_sources(&variant),
                )?
            } else {
                crate::render::markdown::variant_markdown(&variant, sections)?
            };
            Ok(CommandOutcome::stdout(text))
        }
        Err(err) => Err(err.into()),
    }
}

async fn render_variant_search_outcome(
    json_output: bool,
    guidance_as_json: bool,
    request: VariantSearchRequest,
) -> anyhow::Result<CommandOutcome> {
    let VariantSearchRequest {
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
    } = request;

    let resolved =
        match resolve_variant_query(gene, hgvsp, consequence, condition, positional_query)? {
            VariantSearchPlan::Standard(resolved) => resolved,
            VariantSearchPlan::Guidance(guidance) => {
                return variant_guidance_outcome(&guidance, json_output || guidance_as_json);
            }
        };

    let filters = crate::entities::variant::VariantSearchFilters {
        gene: resolved.gene,
        hgvsp: resolved.hgvsp,
        hgvsc: resolved.hgvsc,
        rsid: resolved.rsid,
        protein_alias: resolved.protein_alias,
        significance,
        max_frequency,
        min_cadd,
        consequence: resolved.consequence,
        review_status,
        population,
        revel_min,
        gerp_min,
        tumor_site,
        condition: resolved.condition,
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
    let pagination = PaginationMeta::offset(offset, limit, results.len(), page.total);
    if json_output {
        return Ok(CommandOutcome::stdout(search_json(results, pagination)?));
    }

    let footer = pagination_footer_offset(&pagination);
    Ok(CommandOutcome::stdout(
        crate::render::markdown::variant_search_markdown_with_footer(&query, &results, &footer)?,
    ))
}

async fn render_drug_card_outcome(
    name: &str,
    sections: &[String],
    json_output: bool,
    alias_suggestions_as_json: bool,
) -> anyhow::Result<CommandOutcome> {
    match crate::entities::drug::get(name, sections).await {
        Ok(drug) => {
            let text = if json_output {
                crate::render::json::to_entity_json(
                    &drug,
                    crate::render::markdown::drug_evidence_urls(&drug),
                    crate::render::markdown::related_drug(&drug),
                    crate::render::provenance::drug_section_sources(&drug),
                )?
            } else {
                crate::render::markdown::drug_markdown(&drug, sections)?
            };
            Ok(CommandOutcome::stdout(text))
        }
        Err(err @ crate::error::BioMcpError::NotFound { .. }) => {
            if let Some(outcome) = try_alias_fallback_outcome(
                name,
                crate::entities::discover::DiscoverType::Drug,
                json_output || alias_suggestions_as_json,
            )
            .await?
            {
                Ok(outcome)
            } else {
                Err(err.into())
            }
        }
        Err(err) => Err(err.into()),
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct LocationPaginationMeta {
    total: usize,
    offset: usize,
    limit: usize,
    has_more: bool,
}

fn trial_locations_json(
    trial: &crate::entities::trial::Trial,
    location_pagination: LocationPaginationMeta,
) -> anyhow::Result<String> {
    #[derive(serde::Serialize)]
    struct TrialWithLocationPagination<'a> {
        #[serde(flatten)]
        trial: &'a crate::entities::trial::Trial,
        location_pagination: LocationPaginationMeta,
    }

    crate::render::json::to_entity_json(
        &TrialWithLocationPagination {
            trial,
            location_pagination,
        },
        crate::render::markdown::trial_evidence_urls(trial),
        crate::render::markdown::related_trial(trial),
        crate::render::provenance::trial_section_sources(trial),
    )
    .map_err(Into::into)
}

fn paginate_trial_locations(
    trial: &mut crate::entities::trial::Trial,
    offset: usize,
    limit: usize,
) -> LocationPaginationMeta {
    let locations = trial.locations.take().unwrap_or_default();
    let total = locations.len();
    let paged: Vec<_> = locations.into_iter().skip(offset).take(limit).collect();
    let has_more = offset.saturating_add(paged.len()) < total;
    trial.locations = Some(paged);
    LocationPaginationMeta {
        total,
        offset,
        limit,
        has_more,
    }
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

fn article_query_summary(
    filters: &crate::entities::article::ArticleSearchFilters,
    source_filter: crate::entities::article::ArticleSourceFilter,
    include_retracted: bool,
    offset: usize,
) -> String {
    vec![
        filters.gene.as_deref().map(|v| format!("gene={v}")),
        filters.disease.as_deref().map(|v| format!("disease={v}")),
        filters.drug.as_deref().map(|v| format!("drug={v}")),
        filters.author.as_deref().map(|v| format!("author={v}")),
        filters.keyword.as_deref().map(|v| format!("keyword={v}")),
        filters.article_type.as_deref().map(|v| format!("type={v}")),
        filters
            .date_from
            .as_deref()
            .map(|v| format!("date_from={v}")),
        filters.date_to.as_deref().map(|v| format!("date_to={v}")),
        filters.journal.as_deref().map(|v| format!("journal={v}")),
        filters.open_access.then(|| "open_access=true".to_string()),
        filters
            .no_preprints
            .then(|| "no_preprints=true".to_string()),
        if include_retracted {
            Some("include_retracted=true".to_string())
        } else {
            filters
                .exclude_retracted
                .then(|| "exclude_retracted=true".to_string())
        },
        Some(format!("sort={}", filters.sort.as_str())),
        (source_filter != crate::entities::article::ArticleSourceFilter::All)
            .then(|| format!("source={}", source_filter.as_str())),
        (offset > 0).then(|| format!("offset={offset}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(", ")
}

fn article_debug_filters(
    filters: &crate::entities::article::ArticleSearchFilters,
    source_filter: crate::entities::article::ArticleSourceFilter,
) -> Vec<String> {
    vec![
        filters.gene.as_deref().map(|v| format!("gene={v}")),
        filters.disease.as_deref().map(|v| format!("disease={v}")),
        filters.drug.as_deref().map(|v| format!("drug={v}")),
        filters.author.as_deref().map(|v| format!("author={v}")),
        filters.keyword.as_deref().map(|v| format!("keyword={v}")),
        filters
            .date_from
            .as_deref()
            .map(|v| format!("date_from={v}")),
        filters.date_to.as_deref().map(|v| format!("date_to={v}")),
        filters.article_type.as_deref().map(|v| format!("type={v}")),
        filters.journal.as_deref().map(|v| format!("journal={v}")),
        filters.open_access.then(|| "open_access=true".to_string()),
        filters
            .no_preprints
            .then(|| "no_preprints=true".to_string()),
        Some(format!("exclude_retracted={}", filters.exclude_retracted)),
        Some(format!("sort={}", filters.sort.as_str())),
        Some(format!("source={}", source_filter.as_str())),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn build_article_debug_plan(
    query: &str,
    filters: &crate::entities::article::ArticleSearchFilters,
    source_filter: crate::entities::article::ArticleSourceFilter,
    results: &[crate::entities::article::ArticleSearchResult],
    pagination: &PaginationMeta,
) -> Result<DebugPlan, crate::error::BioMcpError> {
    let summary = crate::entities::article::summarize_debug_plan(filters, source_filter, results)?;
    Ok(DebugPlan {
        surface: "search_article",
        query: query.to_string(),
        anchor: None,
        legs: vec![DebugPlanLeg {
            leg: "article".to_string(),
            entity: "article".to_string(),
            filters: article_debug_filters(filters, source_filter),
            routing: summary.routing,
            sources: summary.sources,
            matched_sources: summary.matched_sources,
            count: results.len(),
            total: pagination.total,
            note: None,
            error: None,
        }],
    })
}

fn article_search_json(
    query: &str,
    sort: crate::entities::article::ArticleSort,
    semantic_scholar_enabled: bool,
    debug_plan: Option<DebugPlan>,
    results: Vec<crate::entities::article::ArticleSearchResult>,
    pagination: PaginationMeta,
) -> anyhow::Result<String> {
    #[derive(serde::Serialize)]
    struct ArticleSearchResponse {
        query: String,
        sort: String,
        semantic_scholar_enabled: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        ranking_policy: Option<&'static str>,
        pagination: PaginationMeta,
        count: usize,
        results: Vec<crate::entities::article::ArticleSearchResult>,
        #[serde(skip_serializing_if = "Option::is_none")]
        debug_plan: Option<DebugPlan>,
    }

    let count = results.len();
    crate::render::json::to_pretty(&ArticleSearchResponse {
        query: query.to_string(),
        sort: sort.as_str().to_string(),
        semantic_scholar_enabled,
        ranking_policy: (sort == crate::entities::article::ArticleSort::Relevance).then_some(
            "directness-first (title coverage > title+abstract coverage > study/review cue > citation support)",
        ),
        pagination,
        count,
        results,
        debug_plan,
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

fn version_output(verbose: bool) -> String {
    let cargo_version = env!("CARGO_PKG_VERSION");
    let git_tag = option_env!("BIOMCP_BUILD_GIT_TAG");
    let git = option_env!("BIOMCP_BUILD_GIT_SHA").unwrap_or("unknown");
    let build = option_env!("BIOMCP_BUILD_DATE").unwrap_or("unknown");
    let version = git_tag
        .filter(|t| t.starts_with('v') && !t.contains('-'))
        .map(|t| &t[1..])
        .unwrap_or(cargo_version);
    let base = format!("biomcp {version} (git {git}, build {build})");
    if !verbose {
        return base;
    }

    let executable = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let path_hits = find_biomcp_on_path();
    let active = std::env::current_exe()
        .ok()
        .as_deref()
        .and_then(canonical_for_compare);
    let mut out = Vec::new();
    out.push(base);
    out.push(format!("Executable: {executable}"));
    out.push(format!("Build: version={version}, git={git}, date={build}"));
    out.push("PATH:".to_string());
    if path_hits.is_empty() {
        out.push("- (no biomcp binaries found on PATH)".to_string());
    } else {
        for hit in &path_hits {
            let canonical = canonical_for_compare(hit);
            let marker = if active.is_some() && active == canonical {
                " (active)"
            } else {
                ""
            };
            out.push(format!("- {}{}", hit.display(), marker));
        }
    }
    if executable.contains("/.venv/") || executable.contains("\\.venv\\") {
        out.push("Warning: active executable appears to come from a virtualenv path.".to_string());
    }
    if path_hits.len() > 1 {
        out.push(format!(
            "Warning: multiple biomcp binaries found on PATH ({}).",
            path_hits.len()
        ));
    }
    out.join("\n")
}

fn find_biomcp_on_path() -> Vec<PathBuf> {
    #[cfg(windows)]
    let binary_name = "biomcp.exe";
    #[cfg(not(windows))]
    let binary_name = "biomcp";

    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let Some(path_var) = std::env::var_os("PATH") else {
        return out;
    };
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(binary_name);
        if !candidate.is_file() {
            continue;
        }
        let canonical = canonical_for_compare(&candidate);
        let key = canonical
            .as_deref()
            .unwrap_or(candidate.as_path())
            .display()
            .to_string();
        if seen.insert(key) {
            out.push(candidate);
        }
    }
    out
}

fn canonical_for_compare(path: &Path) -> Option<PathBuf> {
    std::fs::canonicalize(path).ok()
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
        filters.criteria.as_deref().map(|v| format!("criteria={v}")),
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

fn trim_protein_change_prefix(value: &str) -> &str {
    value
        .trim()
        .trim_start_matches("p.")
        .trim_start_matches("P.")
}

async fn variant_trial_mutation_query(id: &str) -> String {
    let id = id.trim();
    if id.is_empty() {
        return String::new();
    }

    if let Ok(crate::entities::variant::VariantIdFormat::GeneProteinChange { gene, change }) =
        crate::entities::variant::parse_variant_id(id)
    {
        let normalized = crate::entities::variant::normalize_protein_change(&change)
            .unwrap_or_else(|| trim_protein_change_prefix(&change).to_string());
        if !normalized.is_empty() {
            return format!("{gene} {normalized}");
        }
    }

    if let Ok(variant) = crate::entities::variant::get(id, empty_sections()).await {
        let gene = variant.gene.trim();
        let protein = variant
            .hgvs_p
            .as_deref()
            .map(|value| {
                crate::entities::variant::normalize_protein_change(value)
                    .unwrap_or_else(|| trim_protein_change_prefix(value).to_string())
            })
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
                    Ok(crate::render::json::to_entity_json(
                        &article,
                        crate::render::markdown::article_evidence_urls(&article),
                        crate::render::markdown::related_article(&article),
                        crate::render::provenance::article_section_sources(&article),
                    )?)
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
                    Ok(crate::render::json::to_entity_json(
                        &disease,
                        crate::render::markdown::disease_evidence_urls(&disease),
                        crate::render::markdown::related_disease(&disease),
                        crate::render::provenance::disease_section_sources(&disease),
                    )?)
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
                    Ok(crate::render::json::to_entity_json(
                        &pgx,
                        crate::render::markdown::pgx_evidence_urls(&pgx),
                        crate::render::markdown::related_pgx(&pgx),
                        crate::render::provenance::pgx_section_sources(&pgx),
                    )?)
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
                    location_pagination = Some(paginate_trial_locations(&mut trial, offset, limit));
                }
                if json_output {
                    if let Some(loc_page) = location_pagination {
                        trial_locations_json(&trial, loc_page)
                    } else {
                        Ok(crate::render::json::to_entity_json(
                            &trial,
                            crate::render::markdown::trial_evidence_urls(&trial),
                            crate::render::markdown::related_trial(&trial),
                            crate::render::provenance::trial_section_sources(&trial),
                        )?)
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
                    Ok(crate::render::json::to_entity_json(
                        &variant,
                        crate::render::markdown::variant_evidence_urls(&variant),
                        crate::render::markdown::related_variant(&variant),
                        crate::render::provenance::variant_section_sources(&variant),
                    )?)
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
                    Ok(crate::render::json::to_entity_json(
                        &drug,
                        crate::render::markdown::drug_evidence_urls(&drug),
                        crate::render::markdown::related_drug(&drug),
                        crate::render::provenance::drug_section_sources(&drug),
                    )?)
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
                    Ok(crate::render::json::to_entity_json(
                        &pathway,
                        crate::render::markdown::pathway_evidence_urls(&pathway),
                        crate::render::markdown::related_pathway(&pathway),
                        crate::render::provenance::pathway_section_sources(&pathway),
                    )?)
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
                    Ok(crate::render::json::to_entity_json(
                        &protein,
                        crate::render::markdown::protein_evidence_urls(&protein),
                        crate::render::markdown::related_protein(&protein, &sections),
                        crate::render::provenance::protein_section_sources(&protein),
                    )?)
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
                    return match &event {
                        crate::entities::adverse_event::AdverseEventReport::Faers(r) => {
                            Ok(crate::render::json::to_entity_json(
                                &event,
                                crate::render::markdown::adverse_event_evidence_urls(r),
                                crate::render::markdown::related_adverse_event(r),
                                crate::render::provenance::adverse_event_report_section_sources(
                                    &event,
                                ),
                            )?)
                        }
                        crate::entities::adverse_event::AdverseEventReport::Device(r) => {
                            Ok(crate::render::json::to_entity_json(
                                &event,
                                crate::render::markdown::device_event_evidence_urls(r),
                                crate::render::markdown::related_device_event(r),
                                crate::render::provenance::adverse_event_report_section_sources(
                                    &event,
                                ),
                            )?)
                        }
                    };
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
                        gene_anchored: true,
                        keyword,
                        ..related_article_filters()
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
                        Ok(crate::render::markdown::article_search_markdown_with_footer_and_context(
                            &query,
                            &results,
                            "",
                            filters.sort,
                            crate::entities::article::semantic_scholar_search_enabled(
                                &filters,
                                crate::entities::article::ArticleSourceFilter::All,
                            ),
                            None,
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
                VariantCommand::External(args) => {
                    let id = args.join(" ");
                    let variant = crate::entities::variant::get(&id, empty_sections()).await?;
                    if cli.json {
                        Ok(crate::render::json::to_entity_json(
                            &variant,
                            crate::render::markdown::variant_evidence_urls(&variant),
                            crate::render::markdown::related_variant(&variant),
                            crate::render::provenance::variant_section_sources(&variant),
                        )?)
                    } else {
                        Ok(crate::render::markdown::variant_markdown(
                            &variant,
                            empty_sections(),
                        )?)
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
                DrugCommand::External(args) => {
                    let name = args.join(" ");
                    let drug = crate::entities::drug::get(&name, empty_sections()).await?;
                    if cli.json {
                        Ok(crate::render::json::to_entity_json(
                            &drug,
                            crate::render::markdown::drug_evidence_urls(&drug),
                            crate::render::markdown::related_drug(&drug),
                            crate::render::provenance::drug_section_sources(&drug),
                        )?)
                    } else {
                        Ok(crate::render::markdown::drug_markdown(&drug, empty_sections())?)
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
                        disease: Some(name.clone()),
                        ..related_article_filters()
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
                        Ok(crate::render::markdown::article_search_markdown_with_footer_and_context(
                            &query,
                            &results,
                            "",
                            filters.sort,
                            crate::entities::article::semantic_scholar_search_enabled(
                                &filters,
                                crate::entities::article::ArticleSourceFilter::All,
                            ),
                            None,
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
                ArticleCommand::Batch { ids } => {
                    let results = crate::entities::article::get_batch_compact(&ids).await?;
                    if cli.json {
                        Ok(crate::render::json::to_pretty(&results)?)
                    } else {
                        Ok(crate::render::markdown::article_batch_markdown(&results)?)
                    }
                }
                ArticleCommand::Citations { id, limit } => {
                    let limit = paged_fetch_limit(limit, 0, 100)?;
                    let graph = crate::entities::article::citations(&id, limit).await?;
                    if cli.json {
                        Ok(crate::render::json::to_pretty(&graph)?)
                    } else {
                        Ok(crate::render::markdown::article_graph_markdown(
                            "Citations",
                            &graph,
                        )?)
                    }
                }
                ArticleCommand::References { id, limit } => {
                    let limit = paged_fetch_limit(limit, 0, 100)?;
                    let graph = crate::entities::article::references(&id, limit).await?;
                    if cli.json {
                        Ok(crate::render::json::to_pretty(&graph)?)
                    } else {
                        Ok(crate::render::markdown::article_graph_markdown(
                            "References",
                            &graph,
                        )?)
                    }
                }
                ArticleCommand::Recommendations {
                    ids,
                    negative,
                    limit,
                } => {
                    let limit = paged_fetch_limit(limit, 0, 100)?;
                    let recommendations =
                        crate::entities::article::recommendations(&ids, &negative, limit).await?;
                    if cli.json {
                        Ok(crate::render::json::to_pretty(&recommendations)?)
                    } else {
                        Ok(crate::render::markdown::article_recommendations_markdown(
                            &recommendations,
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
                        biomarker: Some(symbol.clone()),
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
                            format!("biomarker={symbol}, offset={offset}")
                        } else {
                            format!("biomarker={symbol}")
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
                        gene_anchored: true,
                        ..related_article_filters()
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
                        Ok(crate::render::markdown::article_search_markdown_with_footer_and_context(
                            &query,
                            &results,
                            "",
                            filters.sort,
                            crate::entities::article::semantic_scholar_search_enabled(
                                &filters,
                                crate::entities::article::ArticleSourceFilter::All,
                            ),
                            None,
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
                GeneCommand::External(args) => {
                    let symbol = args.join(" ");
                    render_gene_card(&symbol, empty_sections(), cli.json).await
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
                        keyword: Some(keyword.clone()),
                        ..related_article_filters()
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
                        Ok(crate::render::markdown::article_search_markdown_with_footer_and_context(
                            &query,
                            &results,
                            "",
                            filters.sort,
                            crate::entities::article::semantic_scholar_search_enabled(
                                &filters,
                                crate::entities::article::ArticleSourceFilter::All,
                            ),
                            None,
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
            Commands::Study { cmd } => match cmd {
                StudyCommand::List => {
                    let studies = crate::entities::study::list_studies().await?;
                    if cli.json {
                        Ok(crate::render::json::to_pretty(&studies)?)
                    } else {
                        Ok(crate::render::markdown::study_list_markdown(&studies))
                    }
                }
                StudyCommand::Download { list, study_id } => {
                    if list {
                        let result = crate::entities::study::list_downloadable_studies().await?;
                        if cli.json {
                            Ok(crate::render::json::to_pretty(&result)?)
                        } else {
                            Ok(crate::render::markdown::study_download_catalog_markdown(
                                &result,
                            ))
                        }
                    } else {
                        let study_id = study_id.expect("clap should require study_id");
                        let result = crate::entities::study::download_study(&study_id).await?;
                        if cli.json {
                            Ok(crate::render::json::to_pretty(&result)?)
                        } else {
                            Ok(crate::render::markdown::study_download_markdown(&result))
                        }
                    }
                }
                StudyCommand::Query {
                    study,
                    gene,
                    query_type,
                    chart,
                } => {
                    let query_type = crate::entities::study::StudyQueryType::from_flag(&query_type)?;
                    chart_json_conflict(&chart, cli.json)?;
                    if let Some(chart_type) = chart.chart {
                        crate::render::chart::validate_query_chart_type(query_type, chart_type)?;
                        let options = crate::render::chart::ChartRenderOptions::from_args(
                            chart.terminal,
                            chart.mcp_inline,
                            chart.output,
                            chart.title,
                            chart.theme,
                            chart.palette,
                        );
                        match query_type {
                            crate::entities::study::StudyQueryType::Mutations => {
                                let result =
                                    crate::entities::study::query_study(&study, &gene, query_type)
                                        .await?;
                                let crate::entities::study::StudyQueryResult::MutationFrequency(
                                    result,
                                ) = result
                                else {
                                    unreachable!("mutation query should return mutation result");
                                };
                                Ok(crate::render::chart::render_mutation_frequency_chart(
                                    &result,
                                    chart_type,
                                    &options,
                                )?)
                            }
                            crate::entities::study::StudyQueryType::Cna => {
                                let result =
                                    crate::entities::study::query_study(&study, &gene, query_type)
                                        .await?;
                                let crate::entities::study::StudyQueryResult::CnaDistribution(
                                    result,
                                ) = result
                                else {
                                    unreachable!("cna query should return cna result");
                                };
                                Ok(crate::render::chart::render_cna_chart(
                                    &result,
                                    chart_type,
                                    &options,
                                )?)
                            }
                            crate::entities::study::StudyQueryType::Expression => Ok(
                                match chart_type {
                                    ChartType::Histogram => {
                                        let values =
                                            crate::entities::study::expression_values(&study, &gene)
                                                .await?;
                                        crate::render::chart::render_expression_histogram_chart(
                                            &study, &gene, &values, &options,
                                        )?
                                    }
                                    ChartType::Density => {
                                        let values =
                                            crate::entities::study::expression_values(&study, &gene)
                                                .await?;
                                        crate::render::chart::render_expression_density_chart(
                                            &study, &gene, &values, &options,
                                        )?
                                    }
                                    other => {
                                        return Err(crate::error::BioMcpError::InvalidArgument(
                                            format!("Invalid chart type: {other}"),
                                        )
                                        .into());
                                    }
                                },
                            ),
                        }
                    } else {
                        let result =
                            crate::entities::study::query_study(&study, &gene, query_type).await?;
                        if cli.json {
                            Ok(crate::render::json::to_pretty(&result)?)
                        } else {
                            Ok(crate::render::markdown::study_query_markdown(&result))
                        }
                    }
                }
                StudyCommand::Filter {
                    study,
                    mutated,
                    amplified,
                    deleted,
                    expression_above,
                    expression_below,
                    cancer_type,
                } => {
                    let mut criteria = Vec::new();
                    for gene in mutated {
                        criteria.push(crate::entities::study::FilterCriterion::Mutated(gene));
                    }
                    for gene in amplified {
                        criteria.push(crate::entities::study::FilterCriterion::Amplified(gene));
                    }
                    for gene in deleted {
                        criteria.push(crate::entities::study::FilterCriterion::Deleted(gene));
                    }
                    for value in expression_above {
                        criteria.push(parse_expression_filter(
                            &value,
                            "--expression-above",
                            crate::entities::study::FilterCriterion::ExpressionAbove,
                        )?);
                    }
                    for value in expression_below {
                        criteria.push(parse_expression_filter(
                            &value,
                            "--expression-below",
                            crate::entities::study::FilterCriterion::ExpressionBelow,
                        )?);
                    }
                    for value in cancer_type {
                        criteria.push(crate::entities::study::FilterCriterion::CancerType(value));
                    }
                    if criteria.is_empty() {
                        return Err(crate::error::BioMcpError::InvalidArgument(
                            crate::entities::study::filter_required_message().to_string(),
                        )
                        .into());
                    }

                    let result = crate::entities::study::filter(&study, criteria).await?;
                    if cli.json {
                        Ok(crate::render::json::to_pretty(&result)?)
                    } else {
                        Ok(crate::render::markdown::study_filter_markdown(&result))
                    }
                }
                StudyCommand::Cohort { study, gene } => {
                    let result = crate::entities::study::cohort(&study, &gene).await?;
                    if cli.json {
                        Ok(crate::render::json::to_pretty(&result)?)
                    } else {
                        Ok(crate::render::markdown::study_cohort_markdown(&result))
                    }
                }
                StudyCommand::Survival {
                    study,
                    gene,
                    endpoint,
                    chart,
                } => {
                    let endpoint = crate::entities::study::SurvivalEndpoint::from_flag(&endpoint)?;
                    chart_json_conflict(&chart, cli.json)?;
                    if let Some(chart_type) = chart.chart {
                        crate::render::chart::validate_standalone_chart_type(
                            "study survival",
                            chart_type,
                            &[ChartType::Bar, ChartType::Survival],
                        )?;
                        let result = crate::entities::study::survival(&study, &gene, endpoint).await?;
                        let options = crate::render::chart::ChartRenderOptions::from_args(
                            chart.terminal,
                            chart.mcp_inline,
                            chart.output,
                            chart.title,
                            chart.theme,
                            chart.palette,
                        );
                        Ok(crate::render::chart::render_survival_chart(
                            &result,
                            chart_type,
                            &options,
                        )?)
                    } else {
                        let result = crate::entities::study::survival(&study, &gene, endpoint).await?;
                        if cli.json {
                            Ok(crate::render::json::to_pretty(&result)?)
                        } else {
                            Ok(crate::render::markdown::study_survival_markdown(&result))
                        }
                    }
                }
                StudyCommand::Compare {
                    study,
                    gene,
                    compare_type,
                    target,
                    chart,
                } => {
                    chart_json_conflict(&chart, cli.json)?;
                    match compare_type.trim().to_ascii_lowercase().as_str() {
                        "expression" | "expr" => {
                            if let Some(chart_type) = chart.chart {
                                crate::render::chart::validate_compare_chart_type(
                                    "expression",
                                    chart_type,
                                )?;
                                let groups = crate::entities::study::compare_expression_values(
                                    &study, &gene, &target,
                                )
                                .await?;
                                let options = crate::render::chart::ChartRenderOptions::from_args(
                                    chart.terminal,
                                    chart.mcp_inline,
                                    chart.output,
                                    chart.title,
                                    chart.theme,
                                    chart.palette,
                                );
                                Ok(crate::render::chart::render_expression_compare_chart(
                                    &study,
                                    &gene,
                                    &target,
                                    &groups,
                                    chart_type,
                                    &options,
                                )?)
                            } else {
                                let result =
                                    crate::entities::study::compare_expression(&study, &gene, &target)
                                        .await?;
                                if cli.json {
                                    Ok(crate::render::json::to_pretty(&result)?)
                                } else {
                                    Ok(crate::render::markdown::study_compare_expression_markdown(
                                        &result,
                                    ))
                                }
                            }
                        }
                        "mutations" | "mutation" => {
                            if let Some(chart_type) = chart.chart {
                                crate::render::chart::validate_compare_chart_type(
                                    "mutations",
                                    chart_type,
                                )?;
                                let result =
                                    crate::entities::study::compare_mutations(&study, &gene, &target)
                                        .await?;
                                let options = crate::render::chart::ChartRenderOptions::from_args(
                                    chart.terminal,
                                    chart.mcp_inline,
                                    chart.output,
                                    chart.title,
                                    chart.theme,
                                    chart.palette,
                                );
                                Ok(crate::render::chart::render_mutation_compare_chart(
                                    &result,
                                    chart_type,
                                    &options,
                                )?)
                            } else {
                                let result =
                                    crate::entities::study::compare_mutations(&study, &gene, &target)
                                        .await?;
                                if cli.json {
                                    Ok(crate::render::json::to_pretty(&result)?)
                                } else {
                                    Ok(crate::render::markdown::study_compare_mutations_markdown(
                                        &result,
                                    ))
                                }
                            }
                        }
                        other => Err(crate::error::BioMcpError::InvalidArgument(format!(
                            "Unknown comparison type '{other}'. Expected: expression, mutations."
                        ))
                        .into()),
                    }
                }
                StudyCommand::CoOccurrence { study, genes, chart } => {
                    chart_json_conflict(&chart, cli.json)?;
                    let genes = genes
                        .split(',')
                        .map(str::trim)
                        .filter(|gene| !gene.is_empty())
                        .map(str::to_string)
                        .collect::<Vec<_>>();
                    if genes.len() < 2 || genes.len() > 10 {
                        return Err(crate::error::BioMcpError::InvalidArgument(
                            "--genes must contain 2 to 10 comma-separated symbols".into(),
                        )
                        .into());
                    }
                    if let Some(chart_type) = chart.chart {
                        crate::render::chart::validate_standalone_chart_type(
                            "study co-occurrence",
                            chart_type,
                            &[ChartType::Bar, ChartType::Pie],
                        )?;
                        let result = crate::entities::study::co_occurrence(&study, &genes).await?;
                        let options = crate::render::chart::ChartRenderOptions::from_args(
                            chart.terminal,
                            chart.mcp_inline,
                            chart.output,
                            chart.title,
                            chart.theme,
                            chart.palette,
                        );
                        Ok(crate::render::chart::render_co_occurrence_chart(
                            &result,
                            chart_type,
                            &options,
                        )?)
                    } else {
                        let result = crate::entities::study::co_occurrence(&study, &genes).await?;
                        if cli.json {
                            Ok(crate::render::json::to_pretty(&result)?)
                        } else {
                            Ok(crate::render::markdown::study_co_occurrence_markdown(&result))
                        }
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
                SearchEntity::All {
                    gene,
                    variant,
                    disease,
                    drug,
                    keyword,
                    positional_query,
                    since,
                    limit,
                    counts_only,
                    debug_plan,
                } => {
                    let keyword = resolve_query_input(keyword, positional_query, "--keyword")?;
                    let input = crate::cli::search_all::SearchAllInput {
                        gene,
                        variant,
                        disease,
                        drug,
                        keyword,
                        since,
                        limit,
                        counts_only,
                        debug_plan,
                    };
                    let results = crate::cli::search_all::dispatch(&input).await?;
                    if cli.json {
                        Ok(crate::render::json::to_pretty(&results)?)
                    } else {
                        Ok(crate::render::markdown::search_all_markdown(
                            &results,
                            input.counts_only,
                        )?)
                    }
                }
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
                    positional_query,
                    drug,
                    cpic_level,
                    pgx_testing,
                    evidence,
                    limit,
                    offset,
                } => {
                    let gene = resolve_query_input(gene, positional_query, "--gene")?;
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
                    positional_query,
                    trait_query,
                    region,
                    p_value,
                    limit,
                    offset,
                } => {
                    let gene = resolve_query_input(gene, positional_query, "--gene")?;
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
                    source,
                    limit,
                    offset,
                    debug_plan,
                } => {
                    let disease = normalize_cli_tokens(disease);
                    let drug = normalize_cli_tokens(drug);
                    let author = normalize_cli_tokens(author);
                    let keyword = resolve_query_input(
                        normalize_cli_tokens(keyword),
                        positional_query,
                        "--keyword/--query",
                    )?;
                    let journal = normalize_cli_tokens(journal);
                    let sort = crate::entities::article::ArticleSort::from_flag(&sort)?;
                    let source_filter =
                        crate::entities::article::ArticleSourceFilter::from_flag(&source)?;
                    let exclude_retracted = exclude_retracted || !include_retracted;
                    let gene_anchored = gene
                        .as_deref()
                        .map(str::trim)
                        .is_some_and(|value| !value.is_empty())
                        && disease
                            .as_deref()
                            .map(str::trim)
                            .is_none_or(str::is_empty)
                        && drug
                            .as_deref()
                            .map(str::trim)
                            .is_none_or(str::is_empty)
                        && author
                            .as_deref()
                            .map(str::trim)
                            .is_none_or(str::is_empty)
                        && keyword
                            .as_deref()
                            .map(str::trim)
                            .is_none_or(str::is_empty);
                    let filters = crate::entities::article::ArticleSearchFilters {
                        gene,
                        gene_anchored,
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

                    let query = article_query_summary(
                        &filters,
                        source_filter,
                        include_retracted,
                        offset,
                    );

                    let page =
                        crate::entities::article::search_page(&filters, limit, offset, source_filter)
                            .await?;
                    let results = page.results;
                    let pagination =
                        PaginationMeta::offset(offset, limit, results.len(), page.total);
                    let semantic_scholar_enabled =
                        crate::entities::article::semantic_scholar_search_enabled(
                            &filters,
                            source_filter,
                        );
                    let debug_plan = if debug_plan {
                        Some(build_article_debug_plan(
                            &query,
                            &filters,
                            source_filter,
                            &results,
                            &pagination,
                        )?)
                    } else {
                        None
                    };
                    if cli.json {
                        article_search_json(
                            &query,
                            filters.sort,
                            semantic_scholar_enabled,
                            debug_plan,
                            results,
                            pagination,
                        )
                    } else {
                        let footer = pagination_footer_offset(&pagination);
                        Ok(crate::render::markdown::article_search_markdown_with_footer_and_context(
                            &query,
                            &results,
                            &footer,
                            filters.sort,
                            semantic_scholar_enabled,
                            debug_plan.as_ref(),
                        )?)
                    }
                }
                SearchEntity::Trial {
                    condition,
                    positional_query,
                    intervention,
                    facility,
                    phase,
                    study_type,
                    age,
                    sex,
                    status,
                    mutation,
                    criteria,
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
                    let condition = resolve_query_input(
                        normalize_cli_tokens(condition),
                        positional_query,
                        "--condition",
                    )?;
                    let intervention = normalize_cli_tokens(intervention);
                    let facility = normalize_cli_tokens(facility);
                    let mutation = normalize_cli_tokens(mutation);
                    let criteria = normalize_cli_tokens(criteria);
                    let biomarker = normalize_cli_tokens(biomarker);
                    let prior_therapies = normalize_cli_tokens(prior_therapies);
                    let progression_on = normalize_cli_tokens(progression_on);
                    let sponsor = normalize_cli_tokens(sponsor);
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
                        criteria,
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
                    if count_only {
                        let count = crate::entities::trial::count_all(&filters).await?;
                        if cli.json {
                            use crate::entities::trial::TrialCount;

                            #[derive(serde::Serialize)]
                            struct TrialCountOnlyJson {
                                total: Option<usize>,
                                #[serde(skip_serializing_if = "Option::is_none")]
                                approximate: Option<bool>,
                            }
                            let (total, approximate) = match count {
                                TrialCount::Exact(total) => (Some(total), None),
                                TrialCount::Approximate(total) => (Some(total), Some(true)),
                                TrialCount::Unknown => (None, None),
                            };
                            return Ok(crate::render::json::to_pretty(&TrialCountOnlyJson {
                                total,
                                approximate,
                            })?);
                        }
                        return Ok(match count {
                            crate::entities::trial::TrialCount::Exact(total) => {
                                format!("Total: {total}")
                            }
                            crate::entities::trial::TrialCount::Approximate(total) => {
                                format!("Total: {total} (approximate, age post-filtered)")
                            }
                            crate::entities::trial::TrialCount::Unknown => {
                                "Total: unknown (traversal limit reached)".to_string()
                            }
                        });
                    }
                    let page = crate::entities::trial::search_page(
                        &filters,
                        limit,
                        offset,
                        next_page.clone(),
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
                    let outcome = render_variant_search_outcome(
                        cli.json,
                        false,
                        VariantSearchRequest {
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
                        },
                    )
                    .await?;
                    if outcome.exit_code == 0 {
                        Ok(outcome.text)
                    } else {
                        anyhow::bail!("{}", outcome.text)
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
                    positional_query,
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
                    let drug = resolve_query_input(drug, positional_query, "--drug")?;
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
            Commands::Chart { command } => Ok(crate::cli::chart::show(command.as_ref())?),
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
            Commands::Discover { query } => {
                crate::cli::discover::run(crate::cli::discover::DiscoverArgs { query }, cli.json)
                    .await
            }
            Commands::List { entity } => {
                crate::cli::list::render(entity.as_deref()).map_err(Into::into)
            }
            Commands::Mcp
            | Commands::Serve
            | Commands::ServeHttp { .. }
            | Commands::ServeSse => {
                anyhow::bail!("MCP/serve commands should not go through CLI run()")
            }
            Commands::Version { verbose } => Ok(version_output(verbose)),
        }
    })
    .await
}

async fn run_outcome_inner(
    cli: Cli,
    alias_suggestions_as_json: bool,
) -> anyhow::Result<CommandOutcome> {
    match cli.command {
        Commands::Get {
            entity: GetEntity::Gene { symbol, sections },
        } => {
            let json = cli.json;
            let no_cache = cli.no_cache;
            crate::sources::with_no_cache(no_cache, async move {
                let (sections, json_override) = extract_json_from_sections(&sections);
                let json_output = json || json_override;
                render_gene_card_outcome(&symbol, &sections, json_output, alias_suggestions_as_json)
                    .await
            })
            .await
        }
        Commands::Get {
            entity: GetEntity::Drug { name, sections },
        } => {
            let json = cli.json;
            let no_cache = cli.no_cache;
            crate::sources::with_no_cache(no_cache, async move {
                let (sections, json_override) = extract_json_from_sections(&sections);
                let json_output = json || json_override;
                render_drug_card_outcome(&name, &sections, json_output, alias_suggestions_as_json)
                    .await
            })
            .await
        }
        Commands::Get {
            entity: GetEntity::Variant { id, sections },
        } => {
            let json = cli.json;
            let no_cache = cli.no_cache;
            crate::sources::with_no_cache(no_cache, async move {
                let (sections, json_override) = extract_json_from_sections(&sections);
                let json_output = json || json_override;
                render_variant_card_outcome(&id, &sections, json_output, alias_suggestions_as_json)
                    .await
            })
            .await
        }
        Commands::Search {
            entity:
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
                },
        } => {
            let json = cli.json;
            let no_cache = cli.no_cache;
            crate::sources::with_no_cache(no_cache, async move {
                render_variant_search_outcome(
                    json,
                    alias_suggestions_as_json,
                    VariantSearchRequest {
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
                    },
                )
                .await
            })
            .await
        }
        Commands::Gene {
            cmd: GeneCommand::Definition { symbol },
        } => {
            let json = cli.json;
            let no_cache = cli.no_cache;
            crate::sources::with_no_cache(no_cache, async move {
                render_gene_card_outcome(&symbol, empty_sections(), json, alias_suggestions_as_json)
                    .await
            })
            .await
        }
        Commands::Gene {
            cmd: GeneCommand::External(args),
        } => {
            let symbol = args.join(" ");
            let json = cli.json;
            let no_cache = cli.no_cache;
            crate::sources::with_no_cache(no_cache, async move {
                render_gene_card_outcome(&symbol, empty_sections(), json, alias_suggestions_as_json)
                    .await
            })
            .await
        }
        Commands::Drug {
            cmd: DrugCommand::External(args),
        } => {
            let name = args.join(" ");
            let json = cli.json;
            let no_cache = cli.no_cache;
            crate::sources::with_no_cache(no_cache, async move {
                render_drug_card_outcome(&name, empty_sections(), json, alias_suggestions_as_json)
                    .await
            })
            .await
        }
        command => Ok(CommandOutcome::stdout(
            run(Cli {
                command,
                json: cli.json,
                no_cache: cli.no_cache,
            })
            .await?,
        )),
    }
}

pub async fn run_outcome(cli: Cli) -> anyhow::Result<CommandOutcome> {
    run_outcome_inner(cli, false).await
}

/// Main CLI execution - called by the MCP `biomcp` tool.
///
/// # Errors
///
/// Returns an error when CLI args cannot be parsed or when command execution fails.
pub async fn execute(mut args: Vec<String>) -> anyhow::Result<String> {
    if args.is_empty() {
        args.push("biomcp".to_string());
    }
    let cli = Cli::try_parse_from(args)?;
    let outcome = run_outcome(cli).await?;
    if outcome.exit_code == 0 {
        Ok(outcome.text)
    } else {
        anyhow::bail!("{}", outcome.text)
    }
}

pub async fn execute_mcp(mut args: Vec<String>) -> anyhow::Result<CliOutput> {
    if args.is_empty() {
        args.push("biomcp".to_string());
    }

    let cli = Cli::try_parse_from(args.clone())?;
    if !is_charted_mcp_study_command(&cli)? {
        let outcome = run_outcome_inner(cli, true).await?;
        return Ok(CliOutput {
            text: outcome.text,
            svg: None,
        });
    }

    let text = execute(rewrite_mcp_chart_args(&args, McpChartPass::Text)?).await?;
    let svg = execute(rewrite_mcp_chart_args(&args, McpChartPass::Svg)?).await?;
    Ok(CliOutput {
        text,
        svg: Some(svg),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        ArticleCommand, ChartArgs, ChartType, Cli, Commands, DrugCommand, GeneCommand, GetEntity,
        OutputStream, PaginationMeta, ProteinCommand, StudyCommand, VariantCommand,
        VariantSearchPlan, article_search_json, execute, execute_mcp, extract_json_from_sections,
        paginate_trial_locations, parse_simple_gene_change, parse_trial_location_paging,
        resolve_query_input, resolve_variant_query, run_outcome, should_try_pathway_trial_fallback,
        trial_locations_json, trial_search_query_summary, truncate_article_annotations,
    };
    use clap::{CommandFactory, Parser};
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn lock_env() -> tokio::sync::MutexGuard<'static, ()> {
        crate::test_support::env_lock().lock().await
    }

    struct EnvVarGuard {
        name: &'static str,
        previous: Option<String>,
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            // Safety: tests serialize environment mutation with `lock_env()`.
            unsafe {
                match &self.previous {
                    Some(value) => std::env::set_var(self.name, value),
                    None => std::env::remove_var(self.name),
                }
            }
        }
    }

    fn set_env_var(name: &'static str, value: Option<&str>) -> EnvVarGuard {
        let previous = std::env::var(name).ok();
        // Safety: tests serialize environment mutation with `lock_env()`.
        unsafe {
            match value {
                Some(value) => std::env::set_var(name, value),
                None => std::env::remove_var(name),
            }
        }
        EnvVarGuard { name, previous }
    }

    async fn mount_gene_lookup_miss(server: &MockServer, symbol: &str) {
        Mock::given(method("GET"))
            .and(path("/v3/query"))
            .and(query_param("q", format!("symbol:\"{symbol}\"")))
            .and(query_param("species", "human"))
            .and(query_param(
                "fields",
                "symbol,name,summary,alias,type_of_gene,ensembl.gene,entrezgene,genomic_pos.chr,genomic_pos.start,genomic_pos.end,genomic_pos.strand,MIM,uniprot,pathway.kegg",
            ))
            .and(query_param("size", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{"total":0,"hits":[]}"#,
                "application/json",
            ))
            .expect(1)
            .mount(server)
            .await;
    }

    async fn mount_gene_lookup_hit(server: &MockServer, symbol: &str, name: &str, entrez: &str) {
        Mock::given(method("GET"))
            .and(path("/v3/query"))
            .and(query_param("q", format!("symbol:\"{symbol}\"")))
            .and(query_param("species", "human"))
            .and(query_param(
                "fields",
                "symbol,name,summary,alias,type_of_gene,ensembl.gene,entrezgene,genomic_pos.chr,genomic_pos.start,genomic_pos.end,genomic_pos.strand,MIM,uniprot,pathway.kegg",
            ))
            .and(query_param("size", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                format!(
                    r#"{{
                        "total": 1,
                        "hits": [{{
                            "symbol": "{symbol}",
                            "name": "{name}",
                            "entrezgene": "{entrez}"
                        }}]
                    }}"#
                ),
                "application/json",
            ))
            .expect(1)
            .mount(server)
            .await;
    }

    async fn mount_drug_lookup_miss(server: &MockServer, query: &str) {
        Mock::given(method("GET"))
            .and(path("/v1/query"))
            .and(query_param("q", query))
            .and(query_param("size", "25"))
            .and(query_param("from", "0"))
            .and(query_param(
                "fields",
                crate::sources::mychem::MYCHEM_FIELDS_GET,
            ))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(r#"{"total":0,"hits":[]}"#, "application/json"),
            )
            .expect(1)
            .mount(server)
            .await;
    }

    async fn mount_ols_alias(
        server: &MockServer,
        query: &str,
        ontology_prefix: &str,
        obo_id: &str,
        label: &str,
        synonyms: &[&str],
        expected_calls: u64,
    ) {
        Mock::given(method("GET"))
            .and(path("/api/search"))
            .and(query_param("q", query))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "response": {
                    "docs": [{
                        "iri": format!("http://example.org/{ontology_prefix}/{}", obo_id.replace(':', "_")),
                        "ontology_name": ontology_prefix,
                        "ontology_prefix": ontology_prefix,
                        "short_form": obo_id.to_ascii_lowercase(),
                        "obo_id": obo_id,
                        "label": label,
                        "description": [],
                        "exact_synonyms": synonyms,
                        "type": "class"
                    }]
                }
            })))
            .expect(expected_calls)
            .mount(server)
            .await;
    }

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
    fn skill_help_examples_match_installed_surface() {
        let mut command = Cli::command();
        let skill = command
            .find_subcommand_mut("skill")
            .expect("skill subcommand should exist");
        let mut help = Vec::new();
        skill
            .write_long_help(&mut help)
            .expect("skill help should render");
        let help = String::from_utf8(help).expect("help should be utf-8");

        assert!(help.contains("biomcp skill            # show skill overview"));
        assert!(help.contains("biomcp skill install    # install skill to your agent config"));
        assert!(!help.contains("biomcp skill list"));
        assert!(!help.contains("biomcp skill 03"));
        assert!(!help.contains("variant-to-treatment"));
        assert!(!help.contains("Commands:\n  list"));
    }

    #[test]
    fn serve_http_help_describes_streamable_http() {
        let mut command = Cli::command();
        let serve_http = command
            .find_subcommand_mut("serve-http")
            .expect("serve-http subcommand should exist");
        let mut help = Vec::new();
        serve_http
            .write_long_help(&mut help)
            .expect("serve-http help should render");
        let help = String::from_utf8(help).expect("help should be utf-8");

        assert!(help.contains("Streamable HTTP"));
        assert!(help.contains("/mcp"));
        assert!(!help.contains("SSE transport"));
    }

    #[test]
    fn serve_sse_help_stays_visible_and_deprecated() {
        let mut command = Cli::command();
        let serve_sse = command
            .find_subcommand_mut("serve-sse")
            .expect("serve-sse subcommand should exist");
        let mut help = Vec::new();
        serve_sse
            .write_long_help(&mut help)
            .expect("serve-sse help should render");
        let help = String::from_utf8(help).expect("help should be utf-8");

        assert!(help.contains("serve-sse"));
        assert!(help.contains("removed"));
        assert!(help.contains("serve-http"));
    }

    fn render_trial_search_long_help() -> String {
        let mut command = Cli::command();
        let search = command
            .find_subcommand_mut("search")
            .expect("search subcommand should exist");
        let trial = search
            .find_subcommand_mut("trial")
            .expect("trial subcommand should exist");
        let mut help = Vec::new();
        trial
            .write_long_help(&mut help)
            .expect("trial help should render");
        String::from_utf8(help).expect("help should be utf-8")
    }

    fn render_pathway_search_long_help() -> String {
        let mut command = Cli::command();
        let search = command
            .find_subcommand_mut("search")
            .expect("search subcommand should exist");
        let pathway = search
            .find_subcommand_mut("pathway")
            .expect("pathway subcommand should exist");
        let mut help = Vec::new();
        pathway
            .write_long_help(&mut help)
            .expect("pathway help should render");
        String::from_utf8(help).expect("help should be utf-8")
    }

    #[test]
    fn trial_facility_help_names_text_search_and_geo_verify_modes() {
        let help = render_trial_search_long_help();

        assert!(help.contains("text-search mode"));
        assert!(help.contains("geo-verify mode"));
        assert!(help.contains("materially more expensive"));
    }

    #[test]
    fn trial_phase_help_explains_combined_phase_label() {
        let help = render_trial_search_long_help();

        assert!(help.contains("1/2"));
        assert!(help.contains("combined Phase 1/Phase 2 label"));
        assert!(help.contains("not Phase 1 OR Phase 2"));
    }

    #[test]
    fn trial_sex_help_explains_all_means_no_restriction() {
        let help = render_trial_search_long_help();

        assert!(help.contains("all"));
        assert!(help.contains("no sex restriction"));
    }

    #[test]
    fn trial_age_help_explains_age_only_count_is_approximate() {
        let help = render_trial_search_long_help();

        assert!(help.contains("age-only CTGov searches report an approximate upstream total"));
    }

    #[test]
    fn search_pathway_help_describes_conditional_query_contract() {
        let help = render_pathway_search_long_help();

        assert!(help.contains("biomcp search pathway [OPTIONS] <QUERY>"));
        assert!(help.contains("biomcp search pathway [OPTIONS] --top-level [QUERY]"));
        assert!(help.contains("required unless --top-level is present"));
        assert!(help.contains("multi-word queries must be quoted"));
        assert!(help.contains("biomcp search pathway --top-level --limit 5"));
    }

    #[test]
    fn pathway_help_describes_source_aware_section_contract() {
        let mut command = Cli::command();
        let get = command
            .find_subcommand_mut("get")
            .expect("get subcommand should exist");
        let pathway = get
            .find_subcommand_mut("pathway")
            .expect("pathway subcommand should exist");
        let mut help = Vec::new();
        pathway
            .write_long_help(&mut help)
            .expect("pathway help should render");
        let help = String::from_utf8(help).expect("help should be utf-8");

        assert!(help.contains("events (Reactome only)"));
        assert!(help.contains("enrichment (Reactome only)"));
        assert!(help.contains("all = all sections available for the resolved source"));
        assert!(help.contains("biomcp get pathway R-HSA-5673001 events"));
        assert!(!help.contains("biomcp get pathway hsa05200 enrichment"));
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
    fn trial_locations_json_preserves_location_pagination_and_section_sources() {
        let trial = crate::entities::trial::Trial {
            nct_id: "NCT00000001".to_string(),
            source: Some("ctgov".to_string()),
            title: "Example trial".to_string(),
            status: "Recruiting".to_string(),
            phase: Some("Phase 2".to_string()),
            study_type: Some("Interventional".to_string()),
            age_range: Some("18 Years and older".to_string()),
            conditions: vec!["melanoma".to_string()],
            interventions: vec!["osimertinib".to_string()],
            sponsor: Some("Example Sponsor".to_string()),
            enrollment: Some(100),
            summary: Some("Example summary".to_string()),
            start_date: Some("2024-01-01".to_string()),
            completion_date: None,
            eligibility_text: None,
            locations: Some(vec![crate::entities::trial::TrialLocation {
                facility: "Example Hospital".to_string(),
                city: "Boston".to_string(),
                state: Some("MA".to_string()),
                country: "United States".to_string(),
                status: Some("Recruiting".to_string()),
                contact_name: None,
                contact_phone: None,
            }]),
            outcomes: None,
            arms: None,
            references: None,
        };

        let json = trial_locations_json(
            &trial,
            super::LocationPaginationMeta {
                total: 42,
                offset: 20,
                limit: 10,
                has_more: true,
            },
        )
        .expect("trial locations json");

        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["nct_id"], "NCT00000001");
        assert_eq!(value["location_pagination"]["total"], 42);
        assert_eq!(value["location_pagination"]["offset"], 20);
        assert_eq!(value["location_pagination"]["limit"], 10);
        assert_eq!(value["location_pagination"]["has_more"], true);
        assert!(value.get("_meta").is_some());
        assert_eq!(value["_meta"]["section_sources"][0]["key"], "overview");
        assert_eq!(
            value["_meta"]["section_sources"][0]["sources"][0],
            "ClinicalTrials.gov"
        );
        assert!(
            value["_meta"]["section_sources"]
                .as_array()
                .expect("section sources array")
                .iter()
                .any(|entry| entry["key"] == "locations")
        );
    }

    #[test]
    fn article_search_json_includes_query_and_ranking_context() {
        let pagination = PaginationMeta::offset(0, 3, 1, Some(1));
        let json = article_search_json(
            "gene=BRAF, sort=relevance",
            crate::entities::article::ArticleSort::Relevance,
            true,
            None,
            vec![crate::entities::article::ArticleSearchResult {
                pmid: "22663011".into(),
                pmcid: Some("PMC9984800".into()),
                doi: Some("10.1056/NEJMoa1203421".into()),
                title: "BRAF melanoma review".into(),
                journal: Some("Journal".into()),
                date: Some("2025-01-01".into()),
                citation_count: Some(12),
                influential_citation_count: Some(4),
                source: crate::entities::article::ArticleSource::EuropePmc,
                matched_sources: vec![
                    crate::entities::article::ArticleSource::EuropePmc,
                    crate::entities::article::ArticleSource::SemanticScholar,
                ],
                score: None,
                is_retracted: Some(false),
                abstract_snippet: Some("Abstract".into()),
                ranking: Some(crate::entities::article::ArticleRankingMetadata {
                    directness_tier: 3,
                    anchor_count: 2,
                    title_anchor_hits: 2,
                    abstract_anchor_hits: 0,
                    combined_anchor_hits: 2,
                    all_anchors_in_title: true,
                    all_anchors_in_text: true,
                    study_or_review_cue: true,
                }),
                normalized_title: "braf melanoma review".into(),
                normalized_abstract: "abstract".into(),
                publication_type: Some("Review".into()),
                insertion_index: 0,
            }],
            pagination,
        )
        .expect("article search json should render");

        let value: serde_json::Value =
            serde_json::from_str(&json).expect("json should parse successfully");
        assert_eq!(value["query"], "gene=BRAF, sort=relevance");
        assert_eq!(value["sort"], "relevance");
        assert_eq!(value["semantic_scholar_enabled"], true);
        assert!(value["ranking_policy"].as_str().is_some());
        assert_eq!(value["results"][0]["ranking"]["directness_tier"], 3);
        assert_eq!(
            value["results"][0]["matched_sources"][1],
            serde_json::Value::String("semanticscholar".into())
        );
    }

    #[test]
    fn paginate_trial_locations_handles_missing_locations() {
        let mut trial = crate::entities::trial::Trial {
            nct_id: "NCT00000001".to_string(),
            source: Some("ctgov".to_string()),
            title: "Example trial".to_string(),
            status: "Recruiting".to_string(),
            phase: Some("Phase 2".to_string()),
            study_type: Some("Interventional".to_string()),
            age_range: Some("18 Years and older".to_string()),
            conditions: vec!["melanoma".to_string()],
            interventions: vec!["osimertinib".to_string()],
            sponsor: Some("Example Sponsor".to_string()),
            enrollment: Some(100),
            summary: Some("Example summary".to_string()),
            start_date: Some("2024-01-01".to_string()),
            completion_date: None,
            eligibility_text: None,
            locations: None,
            outcomes: None,
            arms: None,
            references: None,
        };

        let meta = paginate_trial_locations(&mut trial, 20, 10);
        assert_eq!(meta.total, 0);
        assert_eq!(meta.offset, 20);
        assert_eq!(meta.limit, 10);
        assert!(!meta.has_more);
        assert!(trial.locations.is_some());
        assert_eq!(trial.locations.as_ref().map_or(usize::MAX, Vec::len), 0);
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
                age: Some(67.0),
                sex: Some("female".into()),
                criteria: Some("mismatch repair deficient".into()),
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
        assert!(summary.contains("criteria=mismatch repair deficient"));
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
    fn parse_simple_gene_change_detects_supported_forms() {
        assert_eq!(
            parse_simple_gene_change("BRAF V600E"),
            Some(("BRAF".into(), "V600E".into()))
        );
        assert_eq!(
            parse_simple_gene_change("EGFR T790M"),
            Some(("EGFR".into(), "T790M".into()))
        );
        assert_eq!(
            parse_simple_gene_change("BRAF p.V600E"),
            Some(("BRAF".into(), "V600E".into()))
        );
        assert_eq!(
            parse_simple_gene_change("BRAF p.Val600Glu"),
            Some(("BRAF".into(), "V600E".into()))
        );
    }

    #[test]
    fn parse_simple_gene_change_rejects_non_simple_forms() {
        assert_eq!(parse_simple_gene_change("BRAF"), None);
        assert_eq!(parse_simple_gene_change("EGFR Exon 19 Deletion"), None);
        assert_eq!(parse_simple_gene_change("EGFR Exon19"), None);
        assert_eq!(parse_simple_gene_change("braf V600E"), None);
    }

    #[test]
    fn resolve_variant_query_maps_single_token_to_gene() {
        let resolved = resolve_variant_query(None, None, None, None, vec!["BRAF".into()]).unwrap();
        let VariantSearchPlan::Standard(resolved) = resolved else {
            panic!("expected standard search plan");
        };
        assert_eq!(resolved.gene.as_deref(), Some("BRAF"));
        assert!(resolved.hgvsp.is_none());
        assert!(resolved.hgvsc.is_none());
        assert!(resolved.rsid.is_none());
        assert!(resolved.condition.is_none());
    }

    #[test]
    fn resolve_variant_query_maps_simple_gene_change_to_gene_and_hgvsp() {
        let resolved =
            resolve_variant_query(None, None, None, None, vec!["BRAF".into(), "V600E".into()])
                .unwrap();
        let VariantSearchPlan::Standard(resolved) = resolved else {
            panic!("expected standard search plan");
        };
        assert_eq!(resolved.gene.as_deref(), Some("BRAF"));
        assert_eq!(resolved.hgvsp.as_deref(), Some("V600E"));
        assert!(resolved.hgvsc.is_none());
        assert!(resolved.rsid.is_none());
        assert!(resolved.condition.is_none());
    }

    #[test]
    fn resolve_variant_query_maps_long_form_positional_gene_change_to_gene_and_hgvsp() {
        let resolved = resolve_variant_query(
            None,
            None,
            None,
            None,
            vec!["BRAF".into(), "p.Val600Glu".into()],
        )
        .unwrap();
        let VariantSearchPlan::Standard(resolved) = resolved else {
            panic!("expected standard search plan");
        };
        assert_eq!(resolved.gene.as_deref(), Some("BRAF"));
        assert_eq!(resolved.hgvsp.as_deref(), Some("V600E"));
        assert!(resolved.hgvsc.is_none());
        assert!(resolved.rsid.is_none());
        assert!(resolved.condition.is_none());
    }

    #[test]
    fn resolve_variant_query_maps_rsid_to_rsid_filter() {
        let resolved =
            resolve_variant_query(None, None, None, None, vec!["rs113488022".into()]).unwrap();
        let VariantSearchPlan::Standard(resolved) = resolved else {
            panic!("expected standard search plan");
        };
        assert_eq!(resolved.rsid.as_deref(), Some("rs113488022"));
        assert!(resolved.gene.is_none());
        assert!(resolved.hgvsp.is_none());
        assert!(resolved.hgvsc.is_none());
        assert!(resolved.condition.is_none());
    }

    #[test]
    fn resolve_variant_query_maps_gene_hgvsc_text_to_gene_and_hgvsc() {
        let resolved = resolve_variant_query(
            None,
            None,
            None,
            None,
            vec!["BRAF".into(), "c.1799T>A".into()],
        )
        .unwrap();
        let VariantSearchPlan::Standard(resolved) = resolved else {
            panic!("expected standard search plan");
        };
        assert_eq!(resolved.gene.as_deref(), Some("BRAF"));
        assert_eq!(resolved.hgvsc.as_deref(), Some("c.1799T>A"));
        assert!(resolved.hgvsp.is_none());
        assert!(resolved.rsid.is_none());
        assert!(resolved.condition.is_none());
    }

    #[test]
    fn resolve_variant_query_maps_exon_deletion_phrase_to_gene_and_consequence() {
        let resolved = resolve_variant_query(
            None,
            None,
            None,
            None,
            vec!["EGFR".into(), "Exon".into(), "19".into(), "Deletion".into()],
        )
        .unwrap();
        let VariantSearchPlan::Standard(resolved) = resolved else {
            panic!("expected standard search plan");
        };
        assert_eq!(resolved.gene.as_deref(), Some("EGFR"));
        assert_eq!(resolved.consequence.as_deref(), Some("inframe_deletion"));
        assert!(resolved.hgvsp.is_none());
        assert!(resolved.hgvsc.is_none());
        assert!(resolved.rsid.is_none());
        assert!(resolved.condition.is_none());
    }

    #[test]
    fn resolve_variant_query_maps_gene_residue_alias_to_residue_alias_search() {
        let resolved =
            resolve_variant_query(None, None, None, None, vec!["PTPN22".into(), "620W".into()])
                .unwrap();
        let VariantSearchPlan::Standard(resolved) = resolved else {
            panic!("expected standard search plan");
        };
        assert_eq!(resolved.gene.as_deref(), Some("PTPN22"));
        assert_eq!(
            resolved.protein_alias,
            Some(crate::entities::variant::VariantProteinAlias {
                position: 620,
                residue: 'W',
            })
        );
        assert!(resolved.hgvsp.is_none());
        assert!(resolved.condition.is_none());
    }

    #[test]
    fn resolve_variant_query_maps_gene_flag_residue_alias_to_residue_alias_search() {
        let resolved =
            resolve_variant_query(Some("PTPN22".into()), None, None, None, vec!["620W".into()])
                .unwrap();
        let VariantSearchPlan::Standard(resolved) = resolved else {
            panic!("expected standard search plan");
        };
        assert_eq!(resolved.gene.as_deref(), Some("PTPN22"));
        assert_eq!(
            resolved.protein_alias,
            Some(crate::entities::variant::VariantProteinAlias {
                position: 620,
                residue: 'W',
            })
        );
        assert!(resolved.hgvsp.is_none());
        assert!(resolved.condition.is_none());
    }

    #[test]
    fn resolve_variant_query_uses_gene_context_for_standalone_protein_change() {
        let resolved = resolve_variant_query(
            Some("PTPN22".into()),
            None,
            None,
            None,
            vec!["R620W".into()],
        )
        .unwrap();
        let VariantSearchPlan::Standard(resolved) = resolved else {
            panic!("expected standard search plan");
        };
        assert_eq!(resolved.gene.as_deref(), Some("PTPN22"));
        assert_eq!(resolved.hgvsp.as_deref(), Some("R620W"));
        assert!(resolved.protein_alias.is_none());
    }

    #[test]
    fn resolve_variant_query_uses_gene_context_for_long_form_single_token_change() {
        let resolved = resolve_variant_query(
            Some("BRAF".into()),
            None,
            None,
            None,
            vec!["p.Val600Glu".into()],
        )
        .unwrap();
        let VariantSearchPlan::Standard(resolved) = resolved else {
            panic!("expected standard search plan");
        };
        assert_eq!(resolved.gene.as_deref(), Some("BRAF"));
        assert_eq!(resolved.hgvsp.as_deref(), Some("V600E"));
        assert!(resolved.protein_alias.is_none());
    }

    #[test]
    fn resolve_variant_query_returns_guidance_for_standalone_protein_change() {
        let resolved = resolve_variant_query(None, None, None, None, vec!["R620W".into()]).unwrap();
        let VariantSearchPlan::Guidance(guidance) = resolved else {
            panic!("expected guidance plan");
        };
        assert_eq!(guidance.query, "R620W");
        assert!(matches!(
            guidance.kind,
            crate::entities::variant::VariantGuidanceKind::ProteinChangeOnly { .. }
        ));
    }

    #[test]
    fn resolve_variant_query_returns_guidance_for_long_form_single_token_change() {
        let resolved =
            resolve_variant_query(None, None, None, None, vec!["p.Val600Glu".into()]).unwrap();
        let VariantSearchPlan::Guidance(guidance) = resolved else {
            panic!("expected guidance plan");
        };
        assert_eq!(guidance.query, "p.Val600Glu");
        assert!(matches!(
            guidance.kind,
            crate::entities::variant::VariantGuidanceKind::ProteinChangeOnly { .. }
        ));
        assert_eq!(
            guidance.next_commands.first().map(String::as_str),
            Some("biomcp search variant --hgvsp V600E --limit 10")
        );
    }

    #[test]
    fn resolve_variant_query_normalizes_long_form_hgvsp_flag() {
        let resolved = resolve_variant_query(
            Some("BRAF".into()),
            Some("p.Val600Glu".into()),
            None,
            None,
            Vec::new(),
        )
        .unwrap();
        let VariantSearchPlan::Standard(resolved) = resolved else {
            panic!("expected standard search plan");
        };
        assert_eq!(resolved.gene.as_deref(), Some("BRAF"));
        assert_eq!(resolved.hgvsp.as_deref(), Some("V600E"));
        assert!(resolved.hgvsc.is_none());
        assert!(resolved.rsid.is_none());
        assert!(resolved.condition.is_none());
    }

    #[test]
    fn resolve_variant_query_rejects_conflicts_with_positional_mapping() {
        let gene_conflict = resolve_variant_query(
            Some("TP53".into()),
            None,
            None,
            None,
            vec!["BRAF".into(), "V600E".into()],
        )
        .unwrap_err();
        assert!(format!("{gene_conflict}").contains("conflicts with --gene"));

        let hgvsp_conflict = resolve_variant_query(
            None,
            Some("G12D".into()),
            None,
            None,
            vec!["KRAS".into(), "G12C".into()],
        )
        .unwrap_err();
        assert!(format!("{hgvsp_conflict}").contains("conflicts with --hgvsp"));

        let consequence_conflict = resolve_variant_query(
            None,
            None,
            Some("missense_variant".into()),
            None,
            vec!["EGFR".into(), "Exon".into(), "19".into(), "Deletion".into()],
        )
        .unwrap_err();
        assert!(
            format!("{consequence_conflict}")
                .contains("Positional exon-deletion query conflicts with --consequence")
        );
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
    fn gene_bare_symbol_parses_as_external_subcommand() {
        let cli =
            Cli::try_parse_from(["biomcp", "gene", "BRAF"]).expect("bare gene symbol should parse");
        match cli.command {
            Commands::Gene {
                cmd: GeneCommand::External(args),
            } => assert_eq!(args, vec!["BRAF"]),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn drug_bare_name_parses_as_external_subcommand() {
        let cli = Cli::try_parse_from(["biomcp", "drug", "imatinib"])
            .expect("bare drug name should parse");
        match cli.command {
            Commands::Drug {
                cmd: DrugCommand::External(args),
            } => assert_eq!(args, vec!["imatinib"]),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn variant_bare_id_parses_as_external_subcommand() {
        let cli = Cli::try_parse_from(["biomcp", "variant", "BRAF V600E"])
            .expect("bare variant id should parse");
        match cli.command {
            Commands::Variant {
                cmd: VariantCommand::External(args),
            } => assert_eq!(args, vec!["BRAF V600E"]),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn discover_top_level_command_parses_query() {
        let cli =
            Cli::try_parse_from(["biomcp", "discover", "ERBB1"]).expect("discover should parse");
        match cli.command {
            Commands::Discover { query } => assert_eq!(query, "ERBB1"),
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
    fn search_article_parses_source_flag() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "search",
            "article",
            "-g",
            "BRAF",
            "--source",
            "pubtator",
            "--debug-plan",
            "--limit",
            "5",
        ])
        .expect("search article with --source should parse");

        match cli.command {
            Commands::Search {
                entity:
                    super::SearchEntity::Article {
                        gene,
                        source,
                        debug_plan,
                        limit,
                        offset,
                        ..
                    },
            } => {
                assert_eq!(gene.as_deref(), Some("BRAF"));
                assert_eq!(source, "pubtator");
                assert!(debug_plan);
                assert_eq!(limit, 5);
                assert_eq!(offset, 0);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn search_article_defaults_to_relevance_sort() {
        let cli = Cli::try_parse_from(["biomcp", "search", "article", "-k", "melanoma"])
            .expect("search article without --sort should parse");

        match cli.command {
            Commands::Search {
                entity: super::SearchEntity::Article { sort, .. },
            } => assert_eq!(sort, "relevance"),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn search_article_parses_multi_token_keyword_and_until_alias() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "search",
            "article",
            "-k",
            "vemurafenib",
            "resistance",
            "melanoma",
            "--sort",
            "date",
            "--since",
            "2010-01-01",
            "--until",
            "2015-12-31",
            "--limit",
            "10",
        ])
        .expect("search article multi-token keyword with --until should parse");

        match cli.command {
            Commands::Search {
                entity:
                    super::SearchEntity::Article {
                        keyword,
                        date_from,
                        date_to,
                        limit,
                        ..
                    },
            } => {
                assert_eq!(
                    keyword,
                    vec![
                        "vemurafenib".to_string(),
                        "resistance".to_string(),
                        "melanoma".to_string()
                    ]
                );
                assert_eq!(date_from.as_deref(), Some("2010-01-01"));
                assert_eq!(date_to.as_deref(), Some("2015-12-31"));
                assert_eq!(limit, 10);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn search_article_parses_keyword_with_extra_free_text() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "search",
            "article",
            "-k",
            "EGFR resistance mechanism",
            "non-small cell lung cancer",
            "--sort",
            "citations",
            "--limit",
            "5",
        ])
        .expect("search article keyword plus extra free text should parse");

        match cli.command {
            Commands::Search {
                entity: super::SearchEntity::Article { keyword, limit, .. },
            } => {
                assert_eq!(
                    keyword,
                    vec![
                        "EGFR resistance mechanism".to_string(),
                        "non-small cell lung cancer".to_string()
                    ]
                );
                assert_eq!(limit, 5);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn related_article_filters_default_to_relevance_and_safety_flags() {
        let filters = super::related_article_filters();

        assert_eq!(
            filters.sort,
            crate::entities::article::ArticleSort::Relevance
        );
        assert!(!filters.open_access);
        assert!(filters.no_preprints);
        assert!(filters.exclude_retracted);
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
            "0.5",
            "--sex",
            "female",
            "--criteria",
            "mismatch repair deficient",
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
                        criteria,
                        sponsor_type,
                        count_only,
                        limit,
                        ..
                    },
            } => {
                assert_eq!(facility, vec!["MD Anderson".to_string()]);
                assert_eq!(age, Some(0.5));
                assert_eq!(sex.as_deref(), Some("female"));
                assert_eq!(criteria, vec!["mismatch repair deficient".to_string()]);
                assert_eq!(sponsor_type.as_deref(), Some("nih"));
                assert!(count_only);
                assert_eq!(limit, 3);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn search_trial_rejects_non_numeric_age() {
        let err =
            Cli::try_parse_from(["biomcp", "search", "trial", "--age", "abc", "--count-only"])
                .expect_err("non-numeric age should fail to parse");
        let rendered = err.to_string();

        assert!(rendered.contains("invalid value 'abc' for '--age <AGE>'"));
        assert!(rendered.contains("invalid float literal"));
    }

    #[test]
    fn search_trial_parses_unquoted_multi_token_mutation() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "search",
            "trial",
            "-c",
            "melanoma",
            "--mutation",
            "BRAF",
            "V600E",
            "--intervention",
            "vemurafenib",
            "--status",
            "recruiting",
            "--limit",
            "3",
        ])
        .expect("search trial unquoted multi-token mutation should parse");

        match cli.command {
            Commands::Search {
                entity:
                    super::SearchEntity::Trial {
                        condition,
                        mutation,
                        intervention,
                        status,
                        limit,
                        ..
                    },
            } => {
                assert_eq!(condition, vec!["melanoma".to_string()]);
                assert_eq!(mutation, vec!["BRAF".to_string(), "V600E".to_string()]);
                assert_eq!(intervention, vec!["vemurafenib".to_string()]);
                assert_eq!(status.as_deref(), Some("recruiting"));
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
    fn get_article_parses_tldr_section() {
        let cli = Cli::try_parse_from(["biomcp", "get", "article", "22663011", "tldr"])
            .expect("get article tldr should parse");

        match cli.command {
            Commands::Get {
                entity: GetEntity::Article { id, sections },
            } => {
                assert_eq!(id, "22663011");
                assert_eq!(sections, vec!["tldr".to_string()]);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn article_citations_parses_limit_flag() {
        let cli =
            Cli::try_parse_from(["biomcp", "article", "citations", "22663011", "--limit", "3"])
                .expect("article citations with --limit should parse");

        match cli.command {
            Commands::Article {
                cmd: ArticleCommand::Citations { id, limit },
            } => {
                assert_eq!(id, "22663011");
                assert_eq!(limit, 3);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn article_batch_parses_multiple_ids() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "article",
            "batch",
            "22663011",
            "10.1056/NEJMoa1203421",
        ])
        .expect("article batch should parse");

        match cli.command {
            Commands::Article {
                cmd: ArticleCommand::Batch { ids },
            } => {
                assert_eq!(
                    ids,
                    vec!["22663011".to_string(), "10.1056/NEJMoa1203421".to_string()]
                );
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn article_recommendations_parse_positive_and_negative_ids() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "article",
            "recommendations",
            "22663011",
            "24200969",
            "--negative",
            "39073865",
            "--negative",
            "31452104",
            "--limit",
            "4",
        ])
        .expect("article recommendations should parse");

        match cli.command {
            Commands::Article {
                cmd:
                    ArticleCommand::Recommendations {
                        ids,
                        negative,
                        limit,
                    },
            } => {
                assert_eq!(ids, vec!["22663011".to_string(), "24200969".to_string()]);
                assert_eq!(
                    negative,
                    vec!["39073865".to_string(), "31452104".to_string()]
                );
                assert_eq!(limit, 4);
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
    fn study_list_parses_subcommand() {
        let cli =
            Cli::try_parse_from(["biomcp", "study", "list"]).expect("study list should parse");
        match cli.command {
            Commands::Study {
                cmd: StudyCommand::List,
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn study_download_parses_positional_study_id() {
        let cli = Cli::try_parse_from(["biomcp", "study", "download", "msk_impact_2017"])
            .expect("study download should parse");
        match cli.command {
            Commands::Study {
                cmd: StudyCommand::Download { list, study_id },
            } => {
                assert!(!list);
                assert_eq!(study_id.as_deref(), Some("msk_impact_2017"));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn study_download_parses_list_flag() {
        let cli = Cli::try_parse_from(["biomcp", "study", "download", "--list"])
            .expect("study download list should parse");
        match cli.command {
            Commands::Study {
                cmd: StudyCommand::Download { list, study_id },
            } => {
                assert!(list);
                assert_eq!(study_id, None);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn study_cohort_parses_required_flags() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "study",
            "cohort",
            "--study",
            "brca_tcga_pan_can_atlas_2018",
            "--gene",
            "TP53",
        ])
        .expect("study cohort should parse");
        match cli.command {
            Commands::Study {
                cmd: StudyCommand::Cohort { study, gene },
            } => {
                assert_eq!(study, "brca_tcga_pan_can_atlas_2018");
                assert_eq!(gene, "TP53");
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn study_query_parses_required_flags() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "study",
            "query",
            "--study",
            "msk_impact_2017",
            "--gene",
            "TP53",
            "--type",
            "mutations",
        ])
        .expect("study query should parse");
        match cli.command {
            Commands::Study {
                cmd:
                    StudyCommand::Query {
                        study,
                        gene,
                        query_type,
                        ..
                    },
            } => {
                assert_eq!(study, "msk_impact_2017");
                assert_eq!(gene, "TP53");
                assert_eq!(query_type, "mutations");
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn study_query_parses_chart_flags() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "study",
            "query",
            "--study",
            "msk_impact_2017",
            "--gene",
            "TP53",
            "--type",
            "mutations",
            "--chart",
            "bar",
            "--terminal",
            "--title",
            "TP53 mutations",
            "--theme",
            "dark",
            "--palette",
            "wong",
        ])
        .expect("study query chart flags should parse");
        match cli.command {
            Commands::Study {
                cmd: StudyCommand::Query { chart, .. },
            } => {
                assert_eq!(chart.chart, Some(ChartType::Bar));
                assert!(chart.terminal);
                assert_eq!(chart.title.as_deref(), Some("TP53 mutations"));
                assert_eq!(chart.theme.as_deref(), Some("dark"));
                assert_eq!(chart.palette.as_deref(), Some("wong"));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn study_chart_subcommand_parses_specific_topic() {
        let cli =
            Cli::try_parse_from(["biomcp", "chart", "violin"]).expect("chart docs should parse");
        match cli.command {
            Commands::Chart { command } => {
                assert_eq!(format!("{command:?}"), "Some(Violin)");
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn study_survival_parses_survival_chart_flag() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "study",
            "survival",
            "--study",
            "brca_tcga_pan_can_atlas_2018",
            "--gene",
            "TP53",
            "--chart",
            "survival",
            "--terminal",
        ])
        .expect("study survival chart flags should parse");
        match cli.command {
            Commands::Study {
                cmd: StudyCommand::Survival { chart, .. },
            } => {
                assert_eq!(chart.chart, Some(ChartType::Survival));
                assert!(chart.terminal);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn study_chart_subcommand_parses_survival_topic() {
        let cli = Cli::try_parse_from(["biomcp", "chart", "survival"])
            .expect("survival chart docs should parse");
        match cli.command {
            Commands::Chart { command } => {
                assert_eq!(format!("{command:?}"), "Some(Survival)");
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn chart_auxiliary_flags_require_chart() {
        let err = Cli::try_parse_from([
            "biomcp",
            "study",
            "query",
            "--study",
            "msk_impact_2017",
            "--gene",
            "TP53",
            "--type",
            "mutations",
            "--terminal",
        ])
        .expect_err("--terminal without --chart should fail");
        let msg = err.to_string();
        assert!(msg.contains("--chart"));
    }

    #[test]
    fn short_help_hides_chart_flags_but_long_help_shows_them() {
        let mut cmd = Cli::command();
        let study = cmd.find_subcommand_mut("study").expect("study command");
        let query = study
            .find_subcommand_mut("query")
            .expect("study query command");

        let mut short_help = Vec::new();
        query
            .write_help(&mut short_help)
            .expect("short help should render");
        let short_help = String::from_utf8(short_help).expect("utf8 short help");
        assert!(!short_help.contains("--chart"));

        let mut long_help = Vec::new();
        query
            .write_long_help(&mut long_help)
            .expect("long help should render");
        let long_help = String::from_utf8(long_help).expect("utf8 long help");
        assert!(long_help.contains("--chart"));
        assert!(long_help.contains("Chart Output"));
    }

    #[test]
    fn chart_args_default_to_no_chart() {
        let args = ChartArgs {
            chart: None,
            terminal: false,
            output: None,
            title: None,
            theme: None,
            palette: None,
            mcp_inline: false,
        };
        assert_eq!(args.chart, None);
        assert!(!args.terminal);
        assert!(!args.mcp_inline);
    }

    #[test]
    fn study_survival_parses_endpoint_flag() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "study",
            "survival",
            "--study",
            "brca_tcga_pan_can_atlas_2018",
            "--gene",
            "TP53",
            "--endpoint",
            "dfs",
        ])
        .expect("study survival should parse");
        match cli.command {
            Commands::Study {
                cmd:
                    StudyCommand::Survival {
                        study,
                        gene,
                        endpoint,
                        ..
                    },
            } => {
                assert_eq!(study, "brca_tcga_pan_can_atlas_2018");
                assert_eq!(gene, "TP53");
                assert_eq!(endpoint, "dfs");
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn study_compare_parses_type_and_target() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "study",
            "compare",
            "--study",
            "brca_tcga_pan_can_atlas_2018",
            "--gene",
            "TP53",
            "--type",
            "expression",
            "--target",
            "ERBB2",
        ])
        .expect("study compare should parse");
        match cli.command {
            Commands::Study {
                cmd:
                    StudyCommand::Compare {
                        study,
                        gene,
                        compare_type,
                        target,
                        ..
                    },
            } => {
                assert_eq!(study, "brca_tcga_pan_can_atlas_2018");
                assert_eq!(gene, "TP53");
                assert_eq!(compare_type, "expression");
                assert_eq!(target, "ERBB2");
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn study_filter_parses_all_flags_and_repeated_values() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "study",
            "filter",
            "--study",
            "brca_tcga_pan_can_atlas_2018",
            "--mutated",
            "TP53",
            "--mutated",
            "PIK3CA",
            "--amplified",
            "ERBB2",
            "--deleted",
            "PTEN",
            "--expression-above",
            "MYC:1.5",
            "--expression-above",
            "ERBB2:-0.5",
            "--expression-below",
            "ESR1:0.5",
            "--cancer-type",
            "Breast Cancer",
            "--cancer-type",
            "Lung Cancer",
        ])
        .expect("study filter should parse");
        match cli.command {
            Commands::Study {
                cmd:
                    StudyCommand::Filter {
                        study,
                        mutated,
                        amplified,
                        deleted,
                        expression_above,
                        expression_below,
                        cancer_type,
                    },
            } => {
                assert_eq!(study, "brca_tcga_pan_can_atlas_2018");
                assert_eq!(mutated, vec!["TP53", "PIK3CA"]);
                assert_eq!(amplified, vec!["ERBB2"]);
                assert_eq!(deleted, vec!["PTEN"]);
                assert_eq!(expression_above, vec!["MYC:1.5", "ERBB2:-0.5"]);
                assert_eq!(expression_below, vec!["ESR1:0.5"]);
                assert_eq!(cancer_type, vec!["Breast Cancer", "Lung Cancer"]);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn study_co_occurrence_parses_gene_list() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "study",
            "co-occurrence",
            "--study",
            "brca_tcga_pan_can_atlas_2018",
            "--genes",
            "TP53,PIK3CA,GATA3",
        ])
        .expect("study co-occurrence should parse");
        match cli.command {
            Commands::Study {
                cmd: StudyCommand::CoOccurrence { study, genes, .. },
            } => {
                assert_eq!(study, "brca_tcga_pan_can_atlas_2018");
                assert_eq!(genes, "TP53,PIK3CA,GATA3");
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn search_variant_parses_single_token_positional_query() {
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
                assert_eq!(positional_query, vec!["BRAF".to_string()]);
                assert_eq!(limit, 2);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn search_variant_parses_multi_token_positional_query_and_flag() {
        let cli = Cli::try_parse_from([
            "biomcp", "search", "variant", "BRAF", "V600E", "--limit", "5",
        ])
        .expect("search variant positional+flag should parse");
        match cli.command {
            Commands::Search {
                entity:
                    super::SearchEntity::Variant {
                        positional_query,
                        limit,
                        ..
                    },
            } => {
                assert_eq!(
                    positional_query,
                    vec!["BRAF".to_string(), "V600E".to_string()]
                );
                assert_eq!(limit, 5);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn search_variant_parses_quoted_gene_change_positional_query() {
        let cli =
            Cli::try_parse_from(["biomcp", "search", "variant", "BRAF V600E", "--limit", "5"])
                .expect("search variant quoted positional should parse");
        match cli.command {
            Commands::Search {
                entity:
                    super::SearchEntity::Variant {
                        positional_query,
                        limit,
                        ..
                    },
            } => {
                assert_eq!(positional_query, vec!["BRAF V600E".to_string()]);
                assert_eq!(limit, 5);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn search_trial_parses_positional_query() {
        let cli = Cli::try_parse_from(["biomcp", "search", "trial", "melanoma", "--limit", "2"])
            .expect("search trial positional query should parse");
        match cli.command {
            Commands::Search {
                entity:
                    super::SearchEntity::Trial {
                        condition,
                        positional_query,
                        limit,
                        ..
                    },
            } => {
                assert!(condition.is_empty());
                assert_eq!(positional_query.as_deref(), Some("melanoma"));
                assert_eq!(limit, 2);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn search_trial_parses_multi_word_positional_query() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "search",
            "trial",
            "non-small cell lung cancer",
            "--limit",
            "2",
        ])
        .expect("search trial multi-word positional query should parse");
        match cli.command {
            Commands::Search {
                entity:
                    super::SearchEntity::Trial {
                        positional_query,
                        limit,
                        ..
                    },
            } => {
                assert_eq!(
                    positional_query.as_deref(),
                    Some("non-small cell lung cancer")
                );
                assert_eq!(limit, 2);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn search_trial_parses_positional_query_with_status_flag() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "search",
            "trial",
            "melanoma",
            "--status",
            "recruiting",
        ])
        .expect("search trial positional query with status should parse");
        match cli.command {
            Commands::Search {
                entity:
                    super::SearchEntity::Trial {
                        positional_query,
                        status,
                        ..
                    },
            } => {
                assert_eq!(positional_query.as_deref(), Some("melanoma"));
                assert_eq!(status.as_deref(), Some("recruiting"));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn search_pgx_parses_positional_query() {
        let cli = Cli::try_parse_from(["biomcp", "search", "pgx", "CYP2D6", "--limit", "2"])
            .expect("search pgx positional query should parse");
        match cli.command {
            Commands::Search {
                entity:
                    super::SearchEntity::Pgx {
                        gene,
                        positional_query,
                        limit,
                        ..
                    },
            } => {
                assert!(gene.is_none());
                assert_eq!(positional_query.as_deref(), Some("CYP2D6"));
                assert_eq!(limit, 2);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn search_gwas_parses_positional_query() {
        let cli = Cli::try_parse_from(["biomcp", "search", "gwas", "BRAF", "--limit", "2"])
            .expect("search gwas positional query should parse");
        match cli.command {
            Commands::Search {
                entity:
                    super::SearchEntity::Gwas {
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
    fn search_pathway_parses_multi_word_positional_query() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "search",
            "pathway",
            "MAPK signaling",
            "--limit",
            "2",
        ])
        .expect("search pathway positional query should parse");
        match cli.command {
            Commands::Search {
                entity:
                    super::SearchEntity::Pathway {
                        query,
                        positional_query,
                        limit,
                        ..
                    },
            } => {
                assert!(query.is_none());
                assert_eq!(positional_query.as_deref(), Some("MAPK signaling"));
                assert_eq!(limit, 2);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn search_pathway_parses_quoted_flag_query() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "search",
            "pathway",
            "-q",
            "DNA repair",
            "--limit",
            "2",
        ])
        .expect("search pathway -q query should parse");
        match cli.command {
            Commands::Search {
                entity:
                    super::SearchEntity::Pathway {
                        query,
                        positional_query,
                        limit,
                        ..
                    },
            } => {
                assert_eq!(query.as_deref(), Some("DNA repair"));
                assert!(positional_query.is_none());
                assert_eq!(limit, 2);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn search_adverse_event_parses_positional_query() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "search",
            "adverse-event",
            "pembrolizumab",
            "--limit",
            "2",
        ])
        .expect("search adverse-event positional query should parse");
        match cli.command {
            Commands::Search {
                entity:
                    super::SearchEntity::AdverseEvent {
                        drug,
                        positional_query,
                        limit,
                        ..
                    },
            } => {
                assert!(drug.is_none());
                assert_eq!(positional_query.as_deref(), Some("pembrolizumab"));
                assert_eq!(limit, 2);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn search_all_parses_slot_flags() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "search",
            "all",
            "--gene",
            "BRAF",
            "--disease",
            "melanoma",
            "--keyword",
            "resistance",
            "--since",
            "2024-01-01",
            "--counts-only",
            "--debug-plan",
            "--limit",
            "4",
        ])
        .expect("search all flags should parse");

        match cli.command {
            Commands::Search {
                entity:
                    super::SearchEntity::All {
                        gene,
                        disease,
                        keyword,
                        since,
                        counts_only,
                        debug_plan,
                        limit,
                        ..
                    },
            } => {
                assert_eq!(gene.as_deref(), Some("BRAF"));
                assert_eq!(disease.as_deref(), Some("melanoma"));
                assert_eq!(keyword.as_deref(), Some("resistance"));
                assert_eq!(since.as_deref(), Some("2024-01-01"));
                assert!(counts_only);
                assert!(debug_plan);
                assert_eq!(limit, 4);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn search_all_parses_positional_keyword() {
        let cli = Cli::try_parse_from(["biomcp", "search", "all", "BRAF", "--limit", "2"])
            .expect("search all positional query should parse");
        match cli.command {
            Commands::Search {
                entity:
                    super::SearchEntity::All {
                        keyword,
                        positional_query,
                        limit,
                        ..
                    },
            } => {
                assert!(keyword.is_none());
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

    #[tokio::test]
    async fn search_adverse_event_device_rejects_positional_drug_alias() {
        let err = execute(vec![
            "biomcp".to_string(),
            "search".to_string(),
            "adverse-event".to_string(),
            "pembrolizumab".to_string(),
            "--type".to_string(),
            "device".to_string(),
        ])
        .await
        .expect_err("device query should reject positional drug alias");
        assert!(
            err.to_string()
                .contains("--drug cannot be used with --type device")
        );
    }

    #[tokio::test]
    async fn search_all_requires_at_least_one_typed_slot() {
        let err = execute(vec![
            "biomcp".to_string(),
            "search".to_string(),
            "all".to_string(),
        ])
        .await
        .expect_err("search all should require typed slots");
        assert!(err.to_string().contains("at least one typed slot"));
        assert!(err.to_string().contains("--gene"));
    }

    #[tokio::test]
    async fn search_pathway_requires_query_unless_top_level() {
        let err = execute(vec![
            "biomcp".to_string(),
            "search".to_string(),
            "pathway".to_string(),
        ])
        .await
        .expect_err("search pathway should require query unless --top-level");
        assert!(
            err.to_string().contains(
                "Query is required. Example: biomcp search pathway -q \"MAPK signaling\""
            )
        );
    }

    #[tokio::test]
    async fn study_co_occurrence_requires_2_to_10_genes() {
        let err = execute(vec![
            "biomcp".to_string(),
            "study".to_string(),
            "co-occurrence".to_string(),
            "--study".to_string(),
            "msk_impact_2017".to_string(),
            "--genes".to_string(),
            "TP53".to_string(),
        ])
        .await
        .expect_err("study co-occurrence should validate gene count");
        assert!(err.to_string().contains("--genes must contain 2 to 10"));
    }

    #[tokio::test]
    async fn study_filter_requires_at_least_one_criterion() {
        let err = execute(vec![
            "biomcp".to_string(),
            "study".to_string(),
            "filter".to_string(),
            "--study".to_string(),
            "brca_tcga_pan_can_atlas_2018".to_string(),
        ])
        .await
        .expect_err("study filter should require criteria");
        assert!(
            err.to_string()
                .contains("At least one filter criterion is required")
        );
    }

    #[tokio::test]
    async fn study_filter_rejects_malformed_expression_threshold() {
        let err = execute(vec![
            "biomcp".to_string(),
            "study".to_string(),
            "filter".to_string(),
            "--study".to_string(),
            "brca_tcga_pan_can_atlas_2018".to_string(),
            "--expression-above".to_string(),
            "MYC:not-a-number".to_string(),
        ])
        .await
        .expect_err("study filter should validate threshold format");
        assert!(err.to_string().contains("--expression-above"));
        assert!(err.to_string().contains("GENE:THRESHOLD"));
    }

    #[tokio::test]
    async fn study_survival_rejects_unknown_endpoint() {
        let err = execute(vec![
            "biomcp".to_string(),
            "study".to_string(),
            "survival".to_string(),
            "--study".to_string(),
            "msk_impact_2017".to_string(),
            "--gene".to_string(),
            "TP53".to_string(),
            "--endpoint".to_string(),
            "foo".to_string(),
        ])
        .await
        .expect_err("study survival should validate endpoint");
        assert!(err.to_string().contains("Unknown survival endpoint"));
    }

    #[tokio::test]
    async fn study_compare_rejects_unknown_type() {
        let err = execute(vec![
            "biomcp".to_string(),
            "study".to_string(),
            "compare".to_string(),
            "--study".to_string(),
            "msk_impact_2017".to_string(),
            "--gene".to_string(),
            "TP53".to_string(),
            "--type".to_string(),
            "foo".to_string(),
            "--target".to_string(),
            "ERBB2".to_string(),
        ])
        .await
        .expect_err("study compare should validate type");
        assert!(err.to_string().contains("Unknown comparison type"));
    }

    #[tokio::test]
    async fn gene_alias_fallback_returns_exit_1_markdown_suggestion() {
        let _guard = lock_env().await;
        let mygene = MockServer::start().await;
        let ols = MockServer::start().await;
        let _mygene_base = set_env_var("BIOMCP_MYGENE_BASE", Some(&format!("{}/v3", mygene.uri())));
        let _ols_base = set_env_var("BIOMCP_OLS4_BASE", Some(&ols.uri()));
        let _umls_base = set_env_var("BIOMCP_UMLS_BASE", None);
        let _umls_key = set_env_var("UMLS_API_KEY", None);

        mount_gene_lookup_miss(&mygene, "ERBB1").await;
        mount_ols_alias(&ols, "ERBB1", "hgnc", "HGNC:3236", "EGFR", &["ERBB1"], 1).await;

        let cli = Cli::try_parse_from(["biomcp", "get", "gene", "ERBB1"]).expect("parse");
        let outcome = run_outcome(cli).await.expect("alias outcome");

        assert_eq!(outcome.stream, OutputStream::Stderr);
        assert_eq!(outcome.exit_code, 1);
        assert!(outcome.text.contains("Error: gene 'ERBB1' not found."));
        assert!(
            outcome
                .text
                .contains("Did you mean: `biomcp get gene EGFR`")
        );
    }

    #[tokio::test]
    async fn gene_alias_fallback_json_writes_stdout_and_exit_1() {
        let _guard = lock_env().await;
        let mygene = MockServer::start().await;
        let ols = MockServer::start().await;
        let _mygene_base = set_env_var("BIOMCP_MYGENE_BASE", Some(&format!("{}/v3", mygene.uri())));
        let _ols_base = set_env_var("BIOMCP_OLS4_BASE", Some(&ols.uri()));
        let _umls_base = set_env_var("BIOMCP_UMLS_BASE", None);
        let _umls_key = set_env_var("UMLS_API_KEY", None);

        mount_gene_lookup_miss(&mygene, "ERBB1").await;
        mount_ols_alias(&ols, "ERBB1", "hgnc", "HGNC:3236", "EGFR", &["ERBB1"], 1).await;

        let cli = Cli::try_parse_from(["biomcp", "--json", "get", "gene", "ERBB1"]).expect("parse");
        let outcome = run_outcome(cli).await.expect("alias json outcome");

        assert_eq!(outcome.stream, OutputStream::Stdout);
        assert_eq!(outcome.exit_code, 1);
        let value: serde_json::Value =
            serde_json::from_str(&outcome.text).expect("valid alias json");
        assert_eq!(
            value["_meta"]["alias_resolution"]["canonical"], "EGFR",
            "json={value}"
        );
        assert_eq!(value["_meta"]["next_commands"][0], "biomcp get gene EGFR");
    }

    #[tokio::test]
    async fn variant_get_shorthand_json_returns_variant_guidance_metadata() {
        let cli =
            Cli::try_parse_from(["biomcp", "--json", "get", "variant", "R620W"]).expect("parse");
        let outcome = run_outcome(cli).await.expect("variant guidance outcome");

        assert_eq!(outcome.stream, OutputStream::Stdout);
        assert_eq!(outcome.exit_code, 1);

        let value: serde_json::Value =
            serde_json::from_str(&outcome.text).expect("valid variant guidance json");
        assert_eq!(
            value["_meta"]["alias_resolution"]["requested_entity"],
            "variant"
        );
        assert_eq!(
            value["_meta"]["alias_resolution"]["kind"],
            "protein_change_only"
        );
        assert_eq!(value["_meta"]["alias_resolution"]["query"], "R620W");
        assert_eq!(value["_meta"]["alias_resolution"]["change"], "R620W");
        assert_eq!(
            value["_meta"]["next_commands"][0],
            "biomcp search variant --hgvsp R620W --limit 10"
        );
    }

    #[tokio::test]
    async fn variant_search_shorthand_json_returns_variant_guidance_metadata() {
        let cli =
            Cli::try_parse_from(["biomcp", "--json", "search", "variant", "R620W"]).expect("parse");
        let outcome = run_outcome(cli)
            .await
            .expect("variant search guidance outcome");

        assert_eq!(outcome.stream, OutputStream::Stdout);
        assert_eq!(outcome.exit_code, 1);

        let value: serde_json::Value =
            serde_json::from_str(&outcome.text).expect("valid variant guidance json");
        assert_eq!(
            value["_meta"]["alias_resolution"]["requested_entity"],
            "variant"
        );
        assert_eq!(
            value["_meta"]["alias_resolution"]["kind"],
            "protein_change_only"
        );
        assert_eq!(value["_meta"]["next_commands"][1], "biomcp discover R620W");
    }

    #[tokio::test]
    async fn canonical_gene_lookup_skips_discovery() {
        let _guard = lock_env().await;
        let mygene = MockServer::start().await;
        let ols = MockServer::start().await;
        let _mygene_base = set_env_var("BIOMCP_MYGENE_BASE", Some(&format!("{}/v3", mygene.uri())));
        let _ols_base = set_env_var("BIOMCP_OLS4_BASE", Some(&ols.uri()));
        let _umls_base = set_env_var("BIOMCP_UMLS_BASE", None);
        let _umls_key = set_env_var("UMLS_API_KEY", None);

        mount_gene_lookup_hit(&mygene, "TP53", "tumor protein p53", "7157").await;
        mount_ols_alias(&ols, "TP53", "hgnc", "HGNC:11998", "TP53", &["P53"], 0).await;

        let cli = Cli::try_parse_from(["biomcp", "get", "gene", "TP53"]).expect("parse");
        let outcome = run_outcome(cli).await.expect("success outcome");

        assert_eq!(outcome.stream, OutputStream::Stdout);
        assert_eq!(outcome.exit_code, 0);
        assert!(outcome.text.contains("# TP53"));
    }

    #[tokio::test]
    async fn ambiguous_gene_miss_points_to_discover() {
        let _guard = lock_env().await;
        let mygene = MockServer::start().await;
        let ols = MockServer::start().await;
        let _mygene_base = set_env_var("BIOMCP_MYGENE_BASE", Some(&format!("{}/v3", mygene.uri())));
        let _ols_base = set_env_var("BIOMCP_OLS4_BASE", Some(&ols.uri()));
        let _umls_base = set_env_var("BIOMCP_UMLS_BASE", None);
        let _umls_key = set_env_var("UMLS_API_KEY", None);

        mount_gene_lookup_miss(&mygene, "V600E").await;
        mount_ols_alias(&ols, "V600E", "so", "SO:0001583", "V600E", &["V600E"], 1).await;

        let cli = Cli::try_parse_from(["biomcp", "get", "gene", "V600E"]).expect("parse");
        let outcome = run_outcome(cli).await.expect("ambiguous outcome");

        assert_eq!(outcome.stream, OutputStream::Stderr);
        assert_eq!(outcome.exit_code, 1);
        assert!(
            outcome
                .text
                .contains("BioMCP could not map 'V600E' to a single gene.")
        );
        assert!(outcome.text.contains("1. biomcp discover V600E"));
        assert!(outcome.text.contains("2. biomcp search gene -q V600E"));
    }

    #[tokio::test]
    async fn alias_fallback_ols_failure_preserves_original_not_found() {
        let _guard = lock_env().await;
        let mygene = MockServer::start().await;
        let ols = MockServer::start().await;
        let _mygene_base = set_env_var("BIOMCP_MYGENE_BASE", Some(&format!("{}/v3", mygene.uri())));
        let _ols_base = set_env_var("BIOMCP_OLS4_BASE", Some(&ols.uri()));
        let _umls_base = set_env_var("BIOMCP_UMLS_BASE", None);
        let _umls_key = set_env_var("UMLS_API_KEY", None);

        mount_gene_lookup_miss(&mygene, "ERBB1").await;
        Mock::given(method("GET"))
            .and(path("/api/search"))
            .and(query_param("q", "ERBB1"))
            .respond_with(ResponseTemplate::new(500).set_body_raw("upstream down", "text/plain"))
            .expect(1u64..)
            .mount(&ols)
            .await;

        let cli = Cli::try_parse_from(["biomcp", "get", "gene", "ERBB1"]).expect("parse");
        let err = run_outcome(cli)
            .await
            .expect_err("should preserve not found");
        let rendered = err.to_string();

        assert!(rendered.contains("gene 'ERBB1' not found"));
        assert!(rendered.contains("Try searching: biomcp search gene -q ERBB1"));
    }

    #[tokio::test]
    async fn drug_alias_fallback_returns_exit_1_markdown_suggestion() {
        let _guard = lock_env().await;
        let mychem = MockServer::start().await;
        let ols = MockServer::start().await;
        let _mychem_base = set_env_var("BIOMCP_MYCHEM_BASE", Some(&format!("{}/v1", mychem.uri())));
        let _ols_base = set_env_var("BIOMCP_OLS4_BASE", Some(&ols.uri()));
        let _umls_base = set_env_var("BIOMCP_UMLS_BASE", None);
        let _umls_key = set_env_var("UMLS_API_KEY", None);

        mount_drug_lookup_miss(&mychem, "Keytruda").await;
        mount_ols_alias(
            &ols,
            "Keytruda",
            "mesh",
            "MESH:C582435",
            "pembrolizumab",
            &["Keytruda"],
            1,
        )
        .await;

        let cli = Cli::try_parse_from(["biomcp", "get", "drug", "Keytruda"]).expect("parse");
        let outcome = run_outcome(cli).await.expect("drug alias outcome");

        assert_eq!(outcome.stream, OutputStream::Stderr);
        assert_eq!(outcome.exit_code, 1);
        assert!(outcome.text.contains("Error: drug 'Keytruda' not found."));
        assert!(
            outcome
                .text
                .contains("Did you mean: `biomcp get drug pembrolizumab`")
        );
    }

    #[tokio::test]
    async fn execute_mcp_alias_suggestion_returns_structured_json_text() {
        let _guard = lock_env().await;
        let mygene = MockServer::start().await;
        let ols = MockServer::start().await;
        let _mygene_base = set_env_var("BIOMCP_MYGENE_BASE", Some(&format!("{}/v3", mygene.uri())));
        let _ols_base = set_env_var("BIOMCP_OLS4_BASE", Some(&ols.uri()));
        let _umls_base = set_env_var("BIOMCP_UMLS_BASE", None);
        let _umls_key = set_env_var("UMLS_API_KEY", None);

        mount_gene_lookup_miss(&mygene, "ERBB1").await;
        mount_ols_alias(&ols, "ERBB1", "hgnc", "HGNC:3236", "EGFR", &["ERBB1"], 1).await;

        let output = execute_mcp(vec![
            "biomcp".to_string(),
            "get".to_string(),
            "gene".to_string(),
            "ERBB1".to_string(),
        ])
        .await
        .expect("mcp alias outcome");

        let value: serde_json::Value =
            serde_json::from_str(&output.text).expect("valid mcp alias json");
        assert_eq!(value["_meta"]["alias_resolution"]["kind"], "canonical");
        assert_eq!(value["_meta"]["alias_resolution"]["canonical"], "EGFR");
    }
}

#[cfg(test)]
mod next_commands_validity {
    use super::Cli;
    use clap::Parser;

    fn parse_cmd(cmd: &str) -> Vec<String> {
        shlex::split(cmd).unwrap_or_else(|| panic!("shlex failed on: {cmd}"))
    }

    fn assert_parses(cmd: &str) {
        Cli::try_parse_from(parse_cmd(cmd))
            .unwrap_or_else(|e| panic!("failed to parse '{cmd}': {e}"));
    }

    #[test]
    fn gene_next_commands_parse() {
        assert_parses("biomcp search variant -g BRAF");
        assert_parses("biomcp search article -g BRAF");
        assert_parses("biomcp search drug --target BRAF");
        assert_parses("biomcp gene trials BRAF");
    }

    #[test]
    fn variant_next_commands_parse() {
        assert_parses("biomcp get gene BRAF");
        assert_parses("biomcp search drug --target BRAF");
        assert_parses(r#"biomcp variant trials "rs113488022""#);
        assert_parses(r#"biomcp variant articles "rs113488022""#);
        assert_parses(r#"biomcp variant oncokb "rs113488022""#);
    }

    #[test]
    fn article_next_commands_parse() {
        assert_parses("biomcp get gene EGFR");
        assert_parses("biomcp search disease --query melanoma");
        assert_parses("biomcp get drug osimertinib");
        assert_parses("biomcp article entities 12345");
        assert_parses("biomcp article citations 12345 --limit 3");
        assert_parses("biomcp article references 12345 --limit 3");
        assert_parses("biomcp article recommendations 12345 67890 --negative 11111 --limit 3");
    }

    #[test]
    fn trial_next_commands_parse() {
        assert_parses("biomcp search disease --query melanoma");
        assert_parses("biomcp search article -d melanoma");
        assert_parses("biomcp search trial -c melanoma");
        assert_parses("biomcp get drug dabrafenib");
        assert_parses("biomcp drug trials dabrafenib");
    }

    #[test]
    fn disease_next_commands_parse() {
        assert_parses("biomcp search trial -c melanoma");
        assert_parses("biomcp search article -d melanoma");
        assert_parses("biomcp search drug melanoma");
    }

    #[test]
    fn pgx_next_commands_parse() {
        assert_parses("biomcp search pgx -g CYP2D6");
        assert_parses("biomcp search pgx -d warfarin");
    }

    #[test]
    fn drug_next_commands_parse() {
        assert_parses("biomcp drug trials osimertinib");
        assert_parses("biomcp drug adverse-events osimertinib");
        assert_parses("biomcp get gene EGFR");
    }

    #[test]
    fn pathway_next_commands_parse() {
        assert_parses("biomcp pathway drugs R-HSA-5673001");
    }

    #[test]
    fn protein_next_commands_parse() {
        assert_parses("biomcp get protein P00533 structures");
        assert_parses("biomcp get protein P00533 complexes");
        assert_parses("biomcp get gene EGFR");
    }

    #[test]
    fn adverse_event_next_commands_parse() {
        assert_parses("biomcp get drug osimertinib");
        assert_parses("biomcp drug adverse-events osimertinib");
        assert_parses("biomcp drug trials osimertinib");
    }

    #[test]
    fn device_event_next_commands_parse() {
        assert_parses("biomcp search adverse-event --type device --device HeartValve");
        assert_parses(r#"biomcp search adverse-event --type recall --classification "Class I""#);
    }

    #[test]
    fn discover_next_commands_parse() {
        // gene — unambiguous and ambiguous
        assert_parses("biomcp get gene EGFR");
        assert_parses(r#"biomcp search gene -q "ERBB1" --limit 10"#);
        // drug
        assert_parses(r#"biomcp get drug "pembrolizumab""#);
        // disease — unambiguous helpers and ambiguous fallback
        assert_parses(r#"biomcp get disease "cystic fibrosis""#);
        assert_parses(r#"biomcp disease trials "cystic fibrosis""#);
        assert_parses(r#"biomcp search article -k "cystic fibrosis" --limit 5"#);
        assert_parses(r#"biomcp search disease -q "diabetes" --limit 10"#);
        // symptom
        assert_parses(r#"biomcp search disease -q "chest pain" --limit 10"#);
        assert_parses(r#"biomcp search trial -c "chest pain" --limit 5"#);
        assert_parses(r#"biomcp search article -k "chest pain" --limit 5"#);
        // pathway
        assert_parses(r#"biomcp search pathway -q "MAPK signaling" --limit 5"#);
        // variant with and without gene inference
        assert_parses(r#"biomcp get variant "BRAF V600E""#);
        assert_parses(r#"biomcp search article -k "V600E" --limit 5"#);
        // trial intent
        assert_parses(r#"biomcp search trial -c "Breast Cancer" --limit 5"#);
        assert_parses(r#"biomcp search article -k "Breast Cancer" --limit 5"#);
    }
}

#[cfg(test)]
mod next_commands_json_property {
    use super::Cli;
    use clap::Parser;
    use serde::Serialize;

    use crate::entities::adverse_event::{AdverseEvent, AdverseEventReport, DeviceEvent};
    use crate::entities::article::{AnnotationCount, Article, ArticleAnnotations};
    use crate::entities::disease::Disease;
    use crate::entities::drug::Drug;
    use crate::entities::gene::Gene;
    use crate::entities::pathway::Pathway;
    use crate::entities::pgx::Pgx;
    use crate::entities::protein::Protein;
    use crate::entities::trial::Trial;
    use crate::entities::variant::Variant;

    fn assert_json_next_commands_parse(label: &str, json: &str) {
        let value: serde_json::Value =
            serde_json::from_str(json).unwrap_or_else(|e| panic!("{label}: invalid json: {e}"));
        let cmds = value["_meta"]["next_commands"]
            .as_array()
            .unwrap_or_else(|| panic!("{label}: missing _meta.next_commands"));
        assert!(
            !cmds.is_empty(),
            "{label}: expected at least one next_command"
        );
        for cmd in cmds {
            let cmd = cmd
                .as_str()
                .unwrap_or_else(|| panic!("{label}: next_command was not a string"));
            let argv =
                shlex::split(cmd).unwrap_or_else(|| panic!("{label}: shlex failed on: {cmd}"));
            Cli::try_parse_from(argv)
                .unwrap_or_else(|e| panic!("{label}: failed to parse '{cmd}': {e}"));
        }
    }

    fn assert_entity_json_next_commands<T: Serialize>(
        label: &str,
        entity: &T,
        evidence_urls: Vec<(&'static str, String)>,
        next_commands: Vec<String>,
        section_sources: Vec<crate::render::provenance::SectionSource>,
    ) {
        let json = crate::render::json::to_entity_json(
            entity,
            evidence_urls,
            next_commands,
            section_sources,
        )
        .unwrap_or_else(|e| panic!("{label}: failed to render entity json: {e}"));
        assert_json_next_commands_parse(label, &json);
    }

    #[test]
    fn gene_json_next_commands_parse() {
        let gene = Gene {
            symbol: "BRAF".to_string(),
            name: "B-Raf proto-oncogene".to_string(),
            entrez_id: "673".to_string(),
            ensembl_id: Some("ENSG00000157764".to_string()),
            location: Some("7q34".to_string()),
            genomic_coordinates: None,
            omim_id: Some("164757".to_string()),
            uniprot_id: Some("P15056".to_string()),
            summary: None,
            gene_type: None,
            aliases: Vec::new(),
            clinical_diseases: Vec::new(),
            clinical_drugs: Vec::new(),
            pathways: None,
            ontology: None,
            diseases: None,
            protein: None,
            go: None,
            interactions: None,
            civic: None,
            expression: None,
            hpa: None,
            druggability: None,
            clingen: None,
            constraint: None,
            disgenet: None,
        };

        assert_entity_json_next_commands(
            "gene",
            &gene,
            crate::render::markdown::gene_evidence_urls(&gene),
            crate::render::markdown::related_gene(&gene),
            crate::render::provenance::gene_section_sources(&gene),
        );
    }

    #[test]
    fn article_json_next_commands_parse() {
        let article = Article {
            pmid: Some("22663011".to_string()),
            pmcid: Some("PMC9984800".to_string()),
            doi: Some("10.1056/NEJMoa1203421".to_string()),
            title: "Example".to_string(),
            authors: Vec::new(),
            journal: None,
            date: None,
            citation_count: None,
            publication_type: None,
            open_access: None,
            abstract_text: None,
            full_text_path: None,
            full_text_note: None,
            annotations: Some(ArticleAnnotations {
                genes: vec![AnnotationCount {
                    text: "EGFR".to_string(),
                    count: 1,
                }],
                diseases: vec![AnnotationCount {
                    text: "melanoma".to_string(),
                    count: 1,
                }],
                chemicals: vec![AnnotationCount {
                    text: "osimertinib".to_string(),
                    count: 1,
                }],
                mutations: Vec::new(),
            }),
            semantic_scholar: None,
            pubtator_fallback: false,
        };

        assert_entity_json_next_commands(
            "article",
            &article,
            crate::render::markdown::article_evidence_urls(&article),
            crate::render::markdown::related_article(&article),
            crate::render::provenance::article_section_sources(&article),
        );
    }

    #[test]
    fn disease_json_next_commands_parse() {
        let disease = Disease {
            id: "MONDO:0004992".to_string(),
            name: "melanoma".to_string(),
            definition: None,
            synonyms: Vec::new(),
            parents: Vec::new(),
            associated_genes: Vec::new(),
            gene_associations: Vec::new(),
            top_genes: Vec::new(),
            top_gene_scores: Vec::new(),
            treatment_landscape: Vec::new(),
            recruiting_trial_count: None,
            pathways: Vec::new(),
            phenotypes: Vec::new(),
            variants: Vec::new(),
            top_variant: None,
            models: Vec::new(),
            prevalence: Vec::new(),
            prevalence_note: None,
            civic: None,
            disgenet: None,
            xrefs: std::collections::HashMap::new(),
        };

        assert_entity_json_next_commands(
            "disease",
            &disease,
            crate::render::markdown::disease_evidence_urls(&disease),
            crate::render::markdown::related_disease(&disease),
            crate::render::provenance::disease_section_sources(&disease),
        );
    }

    #[test]
    fn pgx_json_next_commands_parse() {
        let pgx = Pgx {
            query: "CYP2D6".to_string(),
            gene: Some("CYP2D6".to_string()),
            drug: Some("warfarin sodium".to_string()),
            interactions: Vec::new(),
            recommendations: Vec::new(),
            frequencies: Vec::new(),
            guidelines: Vec::new(),
            annotations: Vec::new(),
            annotations_note: None,
        };

        assert_entity_json_next_commands(
            "pgx",
            &pgx,
            crate::render::markdown::pgx_evidence_urls(&pgx),
            crate::render::markdown::related_pgx(&pgx),
            crate::render::provenance::pgx_section_sources(&pgx),
        );
    }

    #[test]
    fn trial_json_next_commands_parse() {
        let trial = Trial {
            nct_id: "NCT01234567".to_string(),
            source: None,
            title: "Example trial".to_string(),
            status: "Recruiting".to_string(),
            phase: None,
            study_type: None,
            age_range: None,
            conditions: vec!["melanoma".to_string()],
            interventions: vec!["dabrafenib".to_string()],
            sponsor: None,
            enrollment: None,
            summary: None,
            start_date: None,
            completion_date: None,
            eligibility_text: None,
            locations: None,
            outcomes: None,
            arms: None,
            references: None,
        };

        assert_entity_json_next_commands(
            "trial",
            &trial,
            crate::render::markdown::trial_evidence_urls(&trial),
            crate::render::markdown::related_trial(&trial),
            crate::render::provenance::trial_section_sources(&trial),
        );
    }

    #[test]
    fn variant_json_next_commands_parse() {
        let variant: Variant = serde_json::from_value(serde_json::json!({
            "id": "rs113488022",
            "gene": "BRAF",
            "hgvs_p": "p.V600E",
            "rsid": "rs113488022"
        }))
        .expect("variant should deserialize");

        assert_entity_json_next_commands(
            "variant",
            &variant,
            crate::render::markdown::variant_evidence_urls(&variant),
            crate::render::markdown::related_variant(&variant),
            crate::render::provenance::variant_section_sources(&variant),
        );
    }

    #[test]
    fn drug_json_next_commands_parse() {
        let drug = Drug {
            name: "osimertinib".to_string(),
            drugbank_id: Some("DB09330".to_string()),
            chembl_id: Some("CHEMBL3353410".to_string()),
            unii: None,
            drug_type: None,
            mechanism: None,
            mechanisms: Vec::new(),
            approval_date: None,
            approval_date_raw: None,
            approval_date_display: None,
            approval_summary: None,
            brand_names: Vec::new(),
            route: None,
            targets: vec!["EGFR".to_string()],
            indications: Vec::new(),
            interactions: Vec::new(),
            interaction_text: None,
            pharm_classes: Vec::new(),
            top_adverse_events: Vec::new(),
            faers_query: None,
            label: None,
            label_set_id: None,
            shortage: None,
            approvals: None,
            civic: None,
        };

        assert_entity_json_next_commands(
            "drug",
            &drug,
            crate::render::markdown::drug_evidence_urls(&drug),
            crate::render::markdown::related_drug(&drug),
            crate::render::provenance::drug_section_sources(&drug),
        );
    }

    #[test]
    fn pathway_json_next_commands_parse() {
        let pathway = Pathway {
            source: "KEGG".to_string(),
            id: "hsa05200".to_string(),
            name: "Pathways in cancer".to_string(),
            species: None,
            summary: None,
            genes: Vec::new(),
            events: Vec::new(),
            enrichment: Vec::new(),
        };

        let next_commands = crate::render::markdown::related_pathway(&pathway);
        assert_eq!(
            next_commands,
            vec!["biomcp pathway drugs hsa05200".to_string()]
        );
        assert!(
            next_commands
                .iter()
                .all(|cmd| !cmd.contains("get pathway hsa05200")),
            "pathway next_commands should not repeat the current flow"
        );
        assert!(
            next_commands
                .iter()
                .all(|cmd| !cmd.contains("events") && !cmd.contains("enrichment")),
            "pathway next_commands should not suggest unsupported sections"
        );

        assert_entity_json_next_commands(
            "pathway",
            &pathway,
            crate::render::markdown::pathway_evidence_urls(&pathway),
            next_commands,
            crate::render::provenance::pathway_section_sources(&pathway),
        );
    }

    #[test]
    fn protein_json_next_commands_parse() {
        let protein = Protein {
            accession: "P00533".to_string(),
            entry_id: Some("EGFR_HUMAN".to_string()),
            name: "Epidermal growth factor receptor".to_string(),
            gene_symbol: Some("EGFR".to_string()),
            organism: None,
            length: None,
            function: None,
            structures: Vec::new(),
            structure_count: None,
            domains: Vec::new(),
            interactions: Vec::new(),
            complexes: Vec::new(),
        };

        let base_next_commands = crate::render::markdown::related_protein(&protein, &[]);
        assert!(base_next_commands.contains(&"biomcp get protein P00533 structures".to_string()));
        assert!(base_next_commands.contains(&"biomcp get protein P00533 complexes".to_string()));

        let section_next_commands =
            crate::render::markdown::related_protein(&protein, &["complexes".to_string()]);
        assert!(
            !section_next_commands.contains(&"biomcp get protein P00533 complexes".to_string())
        );
        assert!(
            section_next_commands.contains(&"biomcp get protein P00533 structures".to_string())
        );
        assert!(section_next_commands.contains(&"biomcp get gene EGFR".to_string()));

        assert_entity_json_next_commands(
            "protein",
            &protein,
            crate::render::markdown::protein_evidence_urls(&protein),
            section_next_commands,
            crate::render::provenance::protein_section_sources(&protein),
        );
    }

    #[test]
    fn faers_json_next_commands_parse() {
        let faers = AdverseEvent {
            report_id: "1001".to_string(),
            drug: "osimertinib".to_string(),
            reactions: Vec::new(),
            outcomes: Vec::new(),
            patient: None,
            concomitant_medications: Vec::new(),
            reporter_type: None,
            reporter_country: None,
            indication: None,
            serious: true,
            date: None,
        };
        let report = AdverseEventReport::Faers(faers.clone());

        assert_entity_json_next_commands(
            "adverse-event-faers",
            &report,
            crate::render::markdown::adverse_event_evidence_urls(&faers),
            crate::render::markdown::related_adverse_event(&faers),
            crate::render::provenance::adverse_event_report_section_sources(&report),
        );
    }

    #[test]
    fn device_event_json_next_commands_parse() {
        let device = DeviceEvent {
            report_id: "MDR-123".to_string(),
            report_number: None,
            device: "HeartValve".to_string(),
            manufacturer: None,
            event_type: None,
            date: None,
            description: None,
        };
        let report = AdverseEventReport::Device(device.clone());

        assert_entity_json_next_commands(
            "adverse-event-device",
            &report,
            crate::render::markdown::device_event_evidence_urls(&device),
            crate::render::markdown::related_device_event(&device),
            crate::render::provenance::adverse_event_report_section_sources(&report),
        );
    }
}
