//! Top-level CLI routing enums composed from per-family payload modules.

use clap::{Args, Subcommand};

use super::{
    article, author, cache, cell_line, chart, disease, drug, gene, pathway, protein, skill, study,
    system, variant,
};

#[derive(Subcommand, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Commands {
    /// Search for entities
    #[command(after_help = "\
EXAMPLES:
  biomcp search gene BRAF")]
    Search {
        #[command(subcommand)]
        entity: SearchEntity,
    },
    /// Get entity by ID
    #[command(after_help = "\
EXAMPLES:
  biomcp get gene BRAF")]
    Get {
        #[command(subcommand)]
        entity: GetEntity,
    },
    /// Variant cross-entity helpers
    #[command(after_help = "\
EXAMPLES:
  biomcp variant normalize all NM_000248.3:c.135del
  biomcp variant normalize variantvalidator 'NM_004448.2:c.829G>T'
  biomcp variant trials \"BRAF V600E\" --limit 5")]
    Variant {
        #[command(subcommand)]
        cmd: variant::VariantCommand,
    },
    /// Drug cross-entity helpers
    Drug {
        #[command(subcommand)]
        cmd: drug::DrugCommand,
    },
    /// Cell-line cross-entity helpers
    #[command(
        name = "cell-line",
        after_help = "\
EXAMPLES:
  biomcp cell-line drug-response CVCL_2119 --dataset GDSC1"
    )]
    CellLine {
        #[command(subcommand)]
        cmd: cell_line::CellLineCommand,
    },
    /// Disease cross-entity helpers
    Disease {
        #[command(subcommand)]
        cmd: disease::DiseaseCommand,
    },
    /// Article cross-entity helpers
    Article {
        #[command(subcommand)]
        cmd: article::ArticleCommand,
    },
    /// Provider-exact author helpers
    Author {
        #[command(subcommand)]
        cmd: author::AuthorCommand,
    },
    /// Gene cross-entity helpers
    Gene {
        #[command(subcommand)]
        cmd: gene::GeneCommand,
    },
    /// Pathway cross-entity helpers
    Pathway {
        #[command(subcommand)]
        cmd: pathway::PathwayCommand,
    },
    /// Protein cross-entity helpers
    Protein {
        #[command(subcommand)]
        cmd: protein::ProteinCommand,
    },
    /// Local cBioPortal study analytics
    #[command(after_help = "\
EXAMPLES:
  biomcp study list")]
    Study {
        #[command(subcommand)]
        cmd: study::StudyCommand,
    },
    /// Check external API connectivity
    Health(system::HealthArgs),
    /// Inspect the managed HTTP cache (CLI-only; cache commands reveal workstation-local filesystem paths)
    #[command(after_help = "\
EXAMPLES:
  biomcp cache stats")]
    Cache {
        #[command(subcommand)]
        cmd: cache::CacheCommand,
    },
    /// EMA (European Medicines Agency) local data management
    #[command(after_help = "\
EXAMPLES:
  biomcp ema sync    # force refresh the EMA local data feeds")]
    Ema {
        #[command(subcommand)]
        cmd: system::EmaCommand,
    },
    /// WHO Prequalification local data management
    #[command(after_help = "\
EXAMPLES:
  biomcp who sync    # force refresh the WHO finished-pharma, API, and vaccine exports")]
    Who {
        #[command(subcommand)]
        cmd: system::WhoCommand,
    },
    /// CDC CVX/MVX vaccine identity local data management
    #[command(after_help = "\
EXAMPLES:
  biomcp cvx sync    # force refresh the CDC CVX/MVX vaccine identity bundle")]
    Cvx {
        #[command(subcommand)]
        cmd: system::CvxCommand,
    },
    /// DDInter local interaction data management
    #[command(after_help = "\
EXAMPLES:
  biomcp ddinter sync    # force refresh the eight DDInter CSV files")]
    Ddinter {
        #[command(subcommand)]
        cmd: system::DdinterCommand,
    },
    /// NCBI GTR local data management
    #[command(after_help = "\
EXAMPLES:
  biomcp gtr sync    # force refresh the local GTR diagnostic bundle")]
    Gtr {
        #[command(subcommand)]
        cmd: system::GtrCommand,
    },
    /// GenCC local gene-disease validity data management
    #[command(after_help = "EXAMPLES:\n  biomcp gencc sync    # revalidate the GenCC dataset")]
    Gencc {
        #[command(subcommand)]
        cmd: system::GenCcCommand,
    },
    /// WHO Prequalified IVD local data management
    #[command(after_help = "\
EXAMPLES:
  biomcp who-ivd sync    # force refresh the local WHO IVD diagnostic CSV")]
    WhoIvd {
        #[command(subcommand)]
        cmd: system::WhoIvdCommand,
    },
    /// Run MCP server over stdio
    #[command(after_help = "\
EXAMPLES:
  biomcp mcp")]
    Mcp(McpArgs),
    /// Alias for `mcp` (Claude Desktop friendly)
    #[command(after_help = "\
EXAMPLES:
  biomcp serve")]
    Serve,
    /// Print MCP client configuration for local stdio BioMCP
    #[command(after_help = "\
EXAMPLES:
  biomcp mcp-config
  biomcp mcp-config --client claude-desktop
  biomcp mcp-config --client codex
  biomcp mcp-config --client json --absolute-path")]
    McpConfig(system::McpConfigArgs),
    #[command(
        about = "Run the MCP Streamable HTTP server at /mcp",
        long_about = "Run the MCP Streamable HTTP server at /mcp.\n\nThis is the canonical remote/server deployment mode.\nHealth routes: GET /health, GET /readyz, GET /."
    )]
    ServeHttp(system::ServeHttpArgs),
    #[command(
        hide = true,
        about = "removed legacy SSE compatibility command; use `serve-http`",
        long_about = "removed legacy SSE compatibility command.\n\ndeprecated users should run `biomcp serve-http` and connect remote clients to `/mcp` instead."
    )]
    ServeSse,
    /// BioMCP skill overview and installer for agents
    #[command(after_help = "\
EXAMPLES:
  biomcp skill            # show skill overview
  biomcp skill 01         # open a worked example by number
  biomcp skill article-follow-up
  biomcp skill render     # print canonical agent prompt
  biomcp skill status     # compare installed guidance with this binary
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
    /// Update the biomcp binary from GitHub releases with SHA256 checksum verification
    #[command(
        long_about = "Update a standalone-installer-owned biomcp binary from GitHub releases.\n\nRelease archives always require SHA256 checksum verification. Windows users should rerun the verified standalone installer."
    )]
    Update(system::UpdateArgs),
    /// Uninstall biomcp from the current location
    Uninstall,
    /// Command reference for entities and flags
    #[command(after_help = "\
EXAMPLES:
  biomcp list gene")]
    List(system::ListArgs),
    /// Parallel get operations (article supports compact or detail mode)
    #[command(after_help = "\
EXAMPLES:
  biomcp batch article 22663011,24200969 --mode compact
  biomcp batch article 22663011,24200969 --mode detail --sections tldr
  biomcp batch gene BRAF,TP53 --sections pathways,interactions
  biomcp batch trial NCT02576665,NCT03715933 --source nci
  biomcp batch variant \"BRAF V600E\",\"KRAS G12D\" --json

NOTES:
  - Article compact mode accepts up to 20 IDs; detail and other batches accept up to 10.
  - Each call must use a single entity type.

See also: biomcp list batch")]
    Batch(system::BatchArgs),
    /// Gene set enrichment against g:Profiler
    Enrich(system::EnrichArgs),
    /// Resolve free-text biomedical text into a typed concept and suggested commands
    #[command(after_help = "\
When to use: use discover when you only have a free-text biomedical phrase and need BioMCP to resolve the first entity or alias before choosing a typed command.
Discover is primarily a single-entity resolver. Existing routed exceptions still cover symptom-of-disease prompts, HPO symptom bridging, treatment prompts, gene+disease orientation, and unambiguous gene-plus-topic follow-ups.
The trimmed query may contain at most 4,096 UTF-8 bytes.
Relational or multi-entity questions may redirect to `biomcp search all --keyword \"<query>\"` instead of surfacing weak collocation matches.
When discover cannot resolve a canonical biomedical concept, it suggests article search instead of leaving an empty dead end.

EXAMPLES:
  biomcp discover ERBB1
  biomcp discover Keytruda
  biomcp discover \"chest pain\"
  biomcp discover \"drug classes that interact with warfarin\"
  biomcp discover \"CTCF cohesin\"
  biomcp --json discover diabetes

See also: biomcp list discover")]
    Discover(system::DiscoverArgs),
    /// Show version
    Version(system::VersionArgs),
}

#[derive(Args, Debug)]
pub struct McpArgs {
    #[command(subcommand)]
    pub command: Option<McpCommand>,
}

impl McpArgs {
    pub(crate) const fn is_tools(&self) -> bool {
        matches!(self.command, Some(McpCommand::Tools))
    }
}

#[derive(Subcommand, Debug)]
pub enum McpCommand {
    /// Print the MCP tool catalog as a JSON array
    Tools,
}

mod selectors;
pub use self::selectors::{GetEntity, SearchEntity};
