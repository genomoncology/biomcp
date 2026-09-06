//! Top-level CLI routing enums composed from per-family payload modules.

use clap::{Args, Subcommand};

use super::{
    adverse_event, article, author, cache, chart, diagnostic, disease, drug, gene, gwas, pathway,
    pgx, phenotype, protein, search_all_command, skill, study, system, trial, variant,
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

INPUT:
  --gene accepts one nonempty symbol without whitespace.
  --keyword is provider-neutral; use --gene/--disease/--drug instead of gene:/disease:/drug: syntax.

See also: biomcp list search-all")]
    All(search_all_command::SearchAllArgs),
    /// Search exact Semantic Scholar author records by name
    #[command(
        after_help = "EXAMPLES:\n  biomcp search author -q \"Louis Williams\" --source semanticscholar --limit 5\n\nSee also: biomcp list author"
    )]
    Author(author::AuthorSearchArgs),
    /// Search genes by symbol, name, type, or chromosome (MyGene.info)
    #[command(after_help = "\
EXAMPLES:
  biomcp search gene BRAF
  biomcp search gene -q kinase --type protein-coding --region chr7:140424943-140624564 --limit 5

See also: biomcp list gene")]
    Gene(gene::GeneSearchArgs),
    /// Search diseases by name or ontology (Monarch/MONDO)
    #[command(after_help = "\
EXAMPLES:
  biomcp search disease \"lung cancer\"
  biomcp search disease -q melanoma --inheritance \"autosomal dominant\" --phenotype HP:0001250 --onset adult --limit 5

See also: biomcp list disease")]
    Disease(disease::DiseaseSearchArgs),
    /// Search source-native diagnostic tests from local GTR and WHO IVD data
    #[command(after_help = "\
EXAMPLES:
  biomcp search diagnostic --gene BRCA1 --limit 5
  biomcp search diagnostic --disease HIV --source who-ivd --limit 5
  biomcp search diagnostic --gene EGFR --type Clinical --source gtr --limit 5
  biomcp search diagnostic --manufacturer InTec --source who-ivd --limit 5

Diagnostic search is filter-only. At least one of --gene, --disease, --type, or --manufacturer is required.
Disease filters require at least 3 alphanumeric characters and match full words or phrases at boundaries.
Use --limit and --offset to page broader diagnostic result sets.
`--source` accepts gtr, who-ivd, or all. WHO IVD is disease/type/manufacturer-oriented; GTR remains the gene-capable source.
See also: biomcp list diagnostic")]
    Diagnostic(diagnostic::DiagnosticSearchArgs),
    /// Search pharmacogenomic interactions
    #[command(after_help = "\
EXAMPLES:
  biomcp search pgx -g CYP2D6
  biomcp search pgx -d warfarin --cpic-level A

See also: biomcp list pgx")]
    Pgx(pgx::PgxSearchArgs),
    /// Search disease matches from HPO IDs or symptom phrases (Monarch semsim)
    #[command(after_help = "\
EXAMPLES:
  biomcp search phenotype \"HP:0001250 HP:0001263\"
  biomcp search phenotype \"HP:0001250,HP:0001263\" --limit 5
  biomcp search phenotype \"seizure, developmental delay\" --limit 5

See also: biomcp list phenotype")]
    Phenotype(phenotype::PhenotypeSearchArgs),
    /// Search GWAS associations by gene or trait
    #[command(after_help = "\
EXAMPLES:
  biomcp search gwas -g TCF7L2
  biomcp search gwas --trait EFO_0000305 --p-value 5e-8

See also: biomcp list gwas")]
    Gwas(gwas::GwasSearchArgs),
    /// Search articles by gene, disease, drug, keyword, or author (author candidate search uses compatible author-capable sources)
    #[command(after_help = "\
When to use: use keyword search to scan a topic before you know the entities. Add -g/--gene when you already know the molecular anchor. Prefer --type review for synthesis questions.

EXAMPLES:
  biomcp search article \"BRAF resistance\"
  biomcp search article -q \"immunotherapy resistance\" --limit 5
  biomcp search article -g BRAF --date-from 2024-01-01
  biomcp search article -d melanoma --type review --journal Nature --limit 5
  biomcp search article -k \"Kartagener syndrome ciliopathy\" --limit 50 --max-per-source 10
  biomcp search article -g BRAF --source pubtator --limit 20
  biomcp search article -k \"BRAF melanoma\" --source semanticscholar --limit 5
  biomcp search article -k \"Hirschsprung disease ganglion cells\" --source litsense2 --limit 5
  biomcp search article -k \"Hirschsprung disease ganglion cells\" --ranking-mode hybrid --weight-semantic 0.5 --weight-lexical 0.2 --limit 5
  biomcp search article -g BRAF --source pubmed --limit 5
  biomcp search article -g BRAF --debug-plan --limit 5
  biomcp --json search article -k \"Oncotype DX review\" --session lit-review-1 --limit 5

OUTPUT:
  - JSON search rows are compact by default; use `--full` for abstracts, complete source provenance, and ranking diagnostics.
  - `--sort date` replaces relevance ranking and emits an in-band warning in JSON and Markdown.

RANKING:
  - `--sort relevance` accepts `--ranking-mode lexical|semantic|hybrid`.
  - Omit `--ranking-mode` to use `hybrid` when `--keyword` is present and `lexical` otherwise.
  - `semantic` sorts by the LitSense2-derived semantic signal and falls back to lexical ties.
  - Hybrid score = `0.4*semantic + 0.3*lexical + 0.2*citations + 0.1*position` by default, using the same LitSense2-derived semantic signal and `semantic=0` when LitSense2 did not match.
  - Use `--weight-semantic`, `--weight-lexical`, `--weight-citations`, and `--weight-position` to retune hybrid ranking.

CAPPING:
  - Cap each federated source's contribution after deduplication and before ranking.
  - Default: 40% of `--limit` on federated pools with at least three surviving primary sources.
  - `0` uses the default cap.
  - Setting it equal to `--limit` disables capping.
  - Rows count against their primary source after deduplication.

SESSION LOOP BREAKER:
  - `--session <TOKEN>` is an optional local caller label for consecutive article keyword searches.
  - Tokens are not secrets; use a short non-identifying label such as `lit-review-1`.
  - In JSON mode, overlapping same-session keyword searches can add `_meta.suggestions[]` fallbacks: prior `batch article --mode compact`, `discover`, then a date-narrowed retry when available.
  - Markdown output is unchanged.

QUERY FORMULATION:
  - Known gene/disease/drug anchors belong in `-g/--gene`, `-d/--disease`, or `--drug`.
  - Article `--gene` accepts one nonempty symbol without whitespace; put additional concepts in `--keyword`.
  - Use provider-neutral `-k/--keyword` for mechanisms, phenotypes, datasets, outcomes, and other free-text concepts; use `--author` or `--journal` instead of provider field syntax.
  - Do not put `gene:`, `disease:`, or `drug:` field expressions in keyword. A caller needing that literal phrase can include literal quote bytes; shell/JSON delimiters alone are not bytes in the runtime value.
  - `-a/--author` limits default candidate search to author-capable sources (Europe PMC + PubMed when compatible); other filters may narrow further.
  - PubMed ESearch cleans question-format gene/disease/drug/keyword terms provider-locally; query echoes and non-PubMed sources keep the original wording.
  - Unknown-entity questions should stay keyword-first or start with `discover`.
  - Keyword-only result pages can suggest typed `get gene`, `get drug`, or `get disease` follow-ups when the whole `-k/--keyword` exactly matches a vocabulary label or alias.
  - Multi-concept phrases and searches that already use `-g/--gene`, `-d/--disease`, or `--drug` do not get direct entity suggestions.
  - Adding `-k/--keyword` keeps the default route on PubTator3 + Europe PMC + PubMed + Semantic Scholar and selects default `hybrid` relevance. Use `--source semanticscholar` or `--source litsense2` explicitly when you want one of those sources alone.
  - Prefer `--type review` for synthesis or list-style questions; it can narrow the compatible default backend set.
  - Avoid: `biomcp search article \"TP53 apoptosis gene regulation\"`
    Prefer: `biomcp search article -g TP53 -k \"apoptosis gene regulation\" --limit 5`
  - Avoid: `biomcp search article -d neurofibromatosis -k \"cafe-au-lait spots neurofibromas\"`
    Prefer: `biomcp search article -k '\"cafe-au-lait spots\" neurofibromas disease' --type review --limit 5`

See also: biomcp list article")]
    Article(article::ArticleSearchArgs),
    /// Search trials by condition, intervention, mutation, or location (CTGov by default; NCI with --source nci)
    #[command(after_help = "\
EXAMPLES:
  biomcp search trial -c melanoma -s recruiting
  biomcp search trial -c \"Rett Syndrome\" --limit 20
  biomcp search trial -p 3 -i pembrolizumab
  biomcp search trial -i daraxonrasib --limit 20
  biomcp search trial -i daraxonrasib --no-alias-expand --limit 20
  biomcp search trial -c melanoma --facility \"MD Anderson\" --age 67 --limit 5
  biomcp search trial --age 0.5 --count-only          # infants eligible (6 months)
  biomcp search trial --mutation \"BRAF V600E\" --status recruiting --study-type interventional --has-results --limit 5
  biomcp search trial -c \"endometrial cancer\" --criteria \"mismatch repair deficient\" -s recruiting
  biomcp search trial -c melanoma --source nci --status recruiting --limit 5

Trial search is filter-based (no free-text query).

Source-specific notes:
  - CTGov: `--condition` sends the supplied condition literally.
  - CTGov: every `--intervention` worker is one quoted literal; expansion uses plausible trade names and investigational codes while excluding systematic chemical synonyms.
  - CTGov: expanded rows expose `matched_intervention_label` / `Matched Intervention` when an alternate alias matched first.
  - CTGov: a rejected expanded alias preserves successful requested-name results and makes the exact total unknown.
  - CTGov: `--no-alias-expand` sends one literal request.
  - CTGov: `--next-page` is not supported when intervention alias expansion fans out to multiple queries; use `--offset` or `--no-alias-expand`.
  - CTGov: `--mutation` broadly searches eligibility, title, summary, and keywords. After broad discovery, simple mutation text is checked against registry eligibility to remove exclusion-only matches; trials where the term is absent remain discoverable, while boolean expressions are discovery-only.
  - CTGov: `--biomarker` is a phrase search over keyword, intervention, and condition; try it for gene-level broadening when a specific `--mutation` returns zero rows.
  - CTGov: `--phase 1/2` and `--phase 2/3` keep combined-label semantics, not an OR search.
  - NCI: `--condition` grounds to an NCI disease ID when available and otherwise falls back to CTS `keyword`.
  - NCI: `--status` accepts one mapped status at a time; comma-separated status lists are rejected.
  - NCI: `--phase 1/2` / `2/3` map to CTS `I_II` / `II_III`; `early_phase1` is not supported on `--source nci`.
  - NCI: `--lat`/`--lon`/`--distance` use direct `sites.org_coordinates_*` CTS filters.
  - NCI: use one quoted value total across `--biomarker`, `--mutation`, and `--criteria`.
  - NCI: `--study-type`, `--sponsor`, and update-date filters are rejected rather than ignored.
  - NCI: there is no separate NCI keyword flag in this ticket.
See also: biomcp list trial")]
    Trial(trial::TrialSearchArgs),
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

Exact protein, coding-HGVS, and rsID searches reject contradictory source identities and report structured resolution in JSON. Gene-only and discovery-filter searches remain broad.
For variant mentions in trials: biomcp variant trials \"BRAF V600E\"
See also: biomcp list variant")]
    Variant(variant::VariantSearchArgs),
    /// Search drugs by name, target, indication, or mechanism (MyChem.info)
    #[command(after_help = "\
When to use: use this when you know the drug or brand name, or switch to --indication, --target, or --mechanism for structured drug discovery.

EXAMPLES:
  biomcp search drug pembrolizumab
  biomcp search drug trastuzumab --region who --limit 5
  biomcp search drug artesunate --region who --product-type api --limit 5
  biomcp search drug BCG --region who --product-type vaccine --limit 5
  biomcp search drug Keytruda --limit 5
  biomcp search drug Keytruda --region eu --limit 5
  biomcp search drug \"influenza vaccine\" --region ema --limit 5
  biomcp search drug --indication malaria --region who --limit 5
  biomcp search drug -q \"kinase inhibitor\" --target EGFR --atc L01 --pharm-class kinase --limit 5

Note: Interaction lookups are not part of `search drug`; use `biomcp drug interactions <name>` instead.
Omitting --region on a plain name/alias search checks U.S., EU, and WHO data.
If you omit --region while using structured filters such as --target or --indication, BioMCP stays on the U.S. MyChem path.
Explicit --region who filters structured U.S. hits through WHO Prequalification.
Use --region ema as an accepted alias for the canonical --region eu value.
WHO-only --product-type <finished_pharma|api|vaccine> requires explicit --region who.
WHO vaccine search is plain name/brand only; structured WHO filters reject `--product-type vaccine`.
Default WHO search excludes vaccines unless you explicitly request `--product-type vaccine`.
CDC CVX/MVX can also expand explicit WHO vaccine name/brand searches after MyChem identity misses.
Explicit --region eu|all with structured filters still errors.

See also: biomcp list drug")]
    Drug(drug::DrugSearchArgs),
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
    Pathway(pathway::PathwaySearchArgs),
    /// Search proteins by name or accession (UniProt)
    #[command(after_help = "\
EXAMPLES:
  biomcp search protein kinase
  biomcp search protein -q \"BRAF\" --reviewed --disease melanoma --existence 1 --limit 5

See also: biomcp list protein")]
    Protein(protein::ProteinSearchArgs),
    /// Search adverse event reports (OpenFDA FAERS / CDC VAERS / recalls / devices)
    #[command(after_help = "\
EXAMPLES:
  biomcp search adverse-event -d pembrolizumab --reaction rash
  biomcp search adverse-event \"COVID-19 vaccine\" --source all --limit 5
  biomcp search adverse-event \"MMR vaccine\" --source vaers --limit 5
  biomcp search adverse-event --type recall -d nivolumab

Vaccine queries default to combined OpenFDA FAERS + CDC VAERS when the query
resolves to a vaccine and the active filters are VAERS-compatible. `--source
vaers` is aggregate-only, and some FAERS filters are intentionally unsupported
on the VAERS path.

See also: biomcp list adverse-event")]
    AdverseEvent(adverse_event::AdverseEventSearchArgs),
}

#[derive(Subcommand, Debug)]
pub enum GetEntity {
    /// Get one exact Semantic Scholar author record
    #[command(
        after_help = "EXAMPLES:\n  biomcp get author semanticscholar:1716151\n\nSee also: biomcp list author"
    )]
    Author(author::AuthorGetArgs),
    /// Get gene by symbol or known single-gene alias
    #[command(after_help = "\
When to use: use this for the default card, then add protein, hpa, expression, diseases, diagnostics, or funding when you need deeper biology, localization, diagnostic-test, or NIH grant context. Known aliases that map to one canonical human gene also resolve here.

EXAMPLES:
  biomcp get gene BRAF
  biomcp get gene PD-L1
  biomcp get gene BRAF pathways
  biomcp get gene BRCA1 diagnostics
  biomcp get gene BRAF hpa
  biomcp get gene ERBB2 funding

See also: biomcp list gene")]
    Gene(gene::GeneGetArgs),
    /// Get article by PMID, PMCID, or DOI
    #[command(after_help = "\
EXAMPLES:
  biomcp get article 22663011
  biomcp get article 22663011 annotations
  biomcp get article 22663011 fulltext
  biomcp get article 22663011 fulltext --out ./articles
  biomcp get article 22663011 fulltext --pdf
  biomcp --json get article <id> assets
  biomcp get article <id> asset <asset-key>
  biomcp get article <id> asset <asset-key> --out ./assets
  biomcp get article 22663011 tldr

Full text defaults to the XML -> PMC HTML ladder. Abstract-only and metadata-only responses are partial results, so later rungs continue until an article body wins.
Requested fulltext JSON includes `full_text_coverage` and sanitized per-rung attempts.
Use `assets` for the JSON-only merged article asset manifest (PMC OA, Europe PMC, recognized JATS/PMC HTML links, and eligible Figshare siblings).
Use `asset <asset-key>` to return one advertised asset as raw bytes with no conversion; handles stay as BioMCP commands, not provider URLs.
Binary or unknown-type assets are refused when standard output is a terminal.
Pipe standard output to preserve exact asset bytes, or use `--output FILE` for an exact destination.
Use `--out DIR` to export resolved full text with a findable name or an asset under its advertised filename. The directory must already exist.
Asset keywords:
assets
asset <asset-key>
raw bytes
Add `--pdf` only with `fulltext` to allow Semantic Scholar PDF as the final fallback.
`--pdf` requires the fulltext section.

See also: biomcp list article")]
    Article(article::ArticleGetArgs),
    /// Get disease by name or ID (e.g., MONDO:0005105)
    #[command(after_help = "\
When to use: use this for the normalized disease card, then add diagnostics, funding, survival, or clinical_features when you need diagnostic tests, NIH grant context, cancer outcomes, or Monarch/HPO phenotype rows framed as clinical features. The clinical_features section is opt-in, remains excluded from all, and unsupported diseases return a truthful Monarch/HPO empty state; pivot to search article -d when you need broader review literature.

EXAMPLES:
  biomcp get disease melanoma
  biomcp get disease MONDO:0005105 genes
  biomcp get disease tuberculosis diagnostics
  biomcp get disease melanoma clinical_features
  biomcp get disease \"chronic myeloid leukemia\" funding
  biomcp get disease \"chronic myeloid leukemia\" survival
  biomcp get disease --name \"chronic myeloid leukemia\" survival

Use --name when a multi-word disease name would otherwise be confused with section tokens.
clinical_features is a Monarch/HPO-backed opt-in view over disease phenotype annotations.

See also: biomcp list disease")]
    Disease(disease::DiseaseGetArgs),
    /// Get diagnostic test detail by exact GTR accession or WHO IVD product code
    #[command(after_help = "\
EXAMPLES:
  biomcp get diagnostic GTR000006692.3
  biomcp get diagnostic GTR000006692.3 genes
  biomcp get diagnostic GTR000006692.3 regulatory
  biomcp get diagnostic \"ITPW02232- TC40\"
  biomcp get diagnostic \"ITPW02232- TC40\" conditions
  biomcp get diagnostic \"ITPW02232- TC40\" regulatory

Supported section tokens: genes, conditions, methods, regulatory, all
`regulatory` is opt-in and is not expanded by `all`.

See also: biomcp list diagnostic")]
    Diagnostic(diagnostic::DiagnosticGetArgs),
    /// Get pharmacogenomics card by gene or drug (e.g., CYP2D6, warfarin)
    #[command(after_help = "\
EXAMPLES:
  biomcp get pgx CYP2D6
  biomcp get pgx warfarin recommendations

See also: biomcp list pgx")]
    Pgx(pgx::PgxGetArgs),
    /// Get trial by NCT ID (e.g., NCT02576665)
    #[command(after_help = "\
EXAMPLES:
  biomcp get trial NCT02576665
  biomcp get trial NCT02576665 --source ctgov eligibility
  biomcp get trial NCT02576665 contacts eligibility locations
  biomcp get trial NCT02576665 --offset 20 --limit 20 locations
  biomcp --json get trial NCT03361748 documents
  biomcp get trial NCT03361748 document Prot_SAP_000.pdf

`documents` is a standalone JSON-only CTGov manifest. `document <filename>` returns an exact advertised file as raw bytes without PDF conversion, up to 32 MiB.
Supported ordinary section tokens: eligibility, contacts, locations, outcomes, arms, references, all

See also: biomcp list trial")]
    Trial(trial::TrialGetArgs),
    /// Get variant by exact rsID, genomic/transcript HGVS, or "GENE CHANGE" (e.g., "BRAF V600E" or "BRAF p.Val600Glu")
    #[command(after_help = "\
EXAMPLES:
  biomcp get variant rs113488022
  biomcp get variant --assembly hg38 'chr7:g.140753336A>T'
  biomcp get variant \"BRAF V600E\" clinvar
  biomcp get variant \"BRAF p.Val600Glu\"
  biomcp get variant 'NM_004333.6:c.1799T>A'

Chromosome HGVS accepts exact-copy repeats, range deletions,
sequence-qualified deletions, duplications, insertions, inversions, and delins.

Shorthand like \"PTPN22 620W\" or \"R620W\" should go through `biomcp search variant`.

See also: biomcp list variant")]
    Variant(variant::VariantGetArgs),
    /// Get drug by name
    #[command(after_help = "\
EXAMPLES:
  biomcp get drug pembrolizumab
  biomcp get drug pembrolizumab label --raw
  biomcp get drug trastuzumab regulatory --region who
  biomcp get drug Keytruda regulatory --region eu
  biomcp get drug Dupixent regulatory --region ema
  biomcp get drug Ozempic safety --region eu
  biomcp get drug pembrolizumab targets
  biomcp get drug pembrolizumab approvals
  biomcp get drug --name \"tepotinib hydrochloride\" label

Use --name when a multi-word drug name would otherwise be confused with section tokens.
Note: `--region ema` is accepted as an alias for the canonical `eu` region value.
If you omit `--region` on `biomcp get drug <name> regulatory`, BioMCP checks U.S. and EU regulatory data.

See also: biomcp list drug")]
    Drug(drug::DrugGetArgs),
    /// Get pathway by ID
    #[command(after_help = "\
EXAMPLES:
  biomcp get pathway R-HSA-5673001
  biomcp get pathway hsa05200
  biomcp get pathway R-HSA-5673001 genes
  biomcp get pathway R-HSA-5673001 events
  biomcp get pathway P21964-2        # returns a hint to use `biomcp get protein P21964-2`
  biomcp get pathway ENSG00000157764 # returns a hint to use `biomcp get gene ENSG00000157764`
  biomcp get pathway BRAF            # returns a hint to use `biomcp get gene BRAF`
  biomcp get pathway rs113488022     # returns a hint to use `biomcp get variant rs113488022`

See also: biomcp list pathway")]
    Pathway(pathway::PathwayGetArgs),
    /// Get protein by UniProt accession or gene symbol
    #[command(after_help = "\
EXAMPLES:
  biomcp get protein P15056
  biomcp get protein P15056 complexes
  biomcp get protein P15056 structures

See also: biomcp list protein")]
    Protein(protein::ProteinGetArgs),
    /// Get adverse event report by FAERS safetyreportid or MAUDE mdr_report_key
    #[command(after_help = "\
EXAMPLES:
  biomcp get adverse-event 10222779
  biomcp get adverse-event 10222779 reactions

See also: biomcp list adverse-event")]
    AdverseEvent(adverse_event::AdverseEventGetArgs),
}
