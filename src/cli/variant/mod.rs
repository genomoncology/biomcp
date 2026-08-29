//! Variant CLI payloads and subcommands.

use clap::{Args, Subcommand};

pub use crate::entities::article::VariantArticleStrategy;

#[derive(Args, Debug)]
pub struct VariantSearchArgs {
    /// Filter by gene symbol
    #[arg(short = 'g', long)]
    pub gene: Option<String>,
    /// Optional positional query tokens
    #[arg(value_name = "QUERY", num_args = 0..)]
    pub positional_query: Vec<String>,
    /// Filter by protein change (e.g., V600E, p.V600E, or p.Val600Glu)
    #[arg(long)]
    pub hgvsp: Option<String>,
    /// ClinVar significance (e.g., pathogenic, benign, uncertain)
    #[arg(long)]
    pub significance: Option<String>,
    /// Max gnomAD allele frequency (0-1)
    #[arg(long)]
    pub max_frequency: Option<f64>,
    /// Minimum finite CADD score (>=0)
    #[arg(long)]
    pub min_cadd: Option<f64>,
    /// Functional consequence filter (e.g., missense_variant)
    #[arg(long)]
    pub consequence: Option<String>,
    /// ClinVar review status (0-4, N_star/N_stars, none, expert_panel, criteria_provided)
    #[arg(long = "review-status")]
    pub review_status: Option<String>,
    /// Population AF scope (afr, amr, eas, fin, nfe, sas)
    #[arg(long)]
    pub population: Option<String>,
    /// Minimum REVEL score
    #[arg(long = "revel-min")]
    pub revel_min: Option<f64>,
    /// Minimum finite GERP score
    #[arg(long = "gerp-min")]
    pub gerp_min: Option<f64>,
    /// Filter by COSMIC tumor site
    #[arg(long = "tumor-site")]
    pub tumor_site: Option<String>,
    /// Filter by ClinVar condition
    #[arg(long)]
    pub condition: Option<String>,
    /// Filter by SnpEff impact (HIGH/MODERATE/LOW/MODIFIER)
    #[arg(long)]
    pub impact: Option<String>,
    /// Restrict to loss-of-function variants
    #[arg(long)]
    pub lof: bool,
    /// Require a field (cadd, revel, gerp, clinvar, gnomad, dbsnp, snpeff, civic, cosmic)
    #[arg(long)]
    pub has: Option<String>,
    /// Require a missing field (same values as --has)
    #[arg(long)]
    pub missing: Option<String>,
    /// Filter CIViC therapy name
    #[arg(long)]
    pub therapy: Option<String>,
    /// Maximum results, 1-50 (default: 10)
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
    /// Skip the first N results
    #[arg(long, default_value = "0")]
    pub offset: usize,
}

#[derive(Args, Debug)]
pub struct VariantGetArgs {
    /// Declare the genome build for a genomic coordinate (hg19/hg38; GRCh37/GRCh38 aliases)
    #[arg(long, value_name = "hg19|hg38")]
    pub assembly: Option<crate::entities::variant::GenomeBuild>,
    /// Exact rsID, genomic coordinate, or "GENE CHANGE" (e.g., rs113488022, GRCh38:chr7:g.140753336A>T, NC_000010.11:g.87925512G>A, chr10:87925512:G:A, NC_000010.11:87925511:G:A, NC_000010.11:g.87925512del, "BRAF V600E")
    pub id: String,
    /// Sections to include (predict, predictions, clinvar, population, population-details, conservation, cosmic, cgi, civic, cbioportal, gwas, all)
    #[arg(trailing_var_arg = true)]
    pub sections: Vec<String>,
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
        /// Maximum results, 1-50 (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
        /// Trial data source (ctgov or nci)
        #[arg(long, default_value = "ctgov")]
        source: String,
    },
    /// Search articles by unioning exact variant evidence routes
    #[command(after_help = "\
EXAMPLES:
  biomcp variant articles \"BRAF V600E\" --limit 5
  biomcp variant articles rs113488022 --strategy annotation --limit 5
  biomcp --json variant articles --input variants.json --debug-plan
  cat variants.json | biomcp --json variant articles --input -

Note: The default union combines exact PubTator annotations, normalized aliases, and source citations before ranking and pagination. Structured JSON input accepts 1-10 items. Use annotation or lexical only for route diagnosis. Unresolved input is labeled as best-effort free text.
See also: biomcp list variant")]
    Articles {
        /// Variant identifier (rsID, HGVS, or "GENE CHANGE")
        #[arg(required_unless_present = "input")]
        id: Option<String>,
        /// JSON request file, or - for stdin (1-10 structured variants)
        #[arg(long, value_name = "PATH")]
        input: Option<String>,
        /// Include normalized route, provider, ranking, and work facts (JSON only)
        #[arg(long)]
        debug_plan: bool,
        /// Verify article identity from captured provider evidence
        #[arg(long)]
        verify_identity: bool,
        /// Return only captured-evidence confirmations (requires --verify-identity)
        #[arg(long, requires = "verify_identity")]
        confirmed_only: bool,
        /// Retrieval strategy (union, annotation, or lexical)
        #[arg(long, value_enum, default_value_t = VariantArticleStrategy::Union)]
        strategy: VariantArticleStrategy,
        /// Maximum results, 1-50 (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Skip the first N results
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Join a variant to residue, domain, PDB, AlphaFold, and Cancerhotspots context (opt-in)
    #[command(after_help = "\
EXAMPLES:
  biomcp variant structure \"BRAF V600E\"
  biomcp --json variant structure \"BRAF V600E\"

Note: Resolves the exact variant, selects the requested residue when possible, then joins InterPro domains, UniProt PDB/AlphaFold structures, and Cancerhotspots recurrence. This network-heavy helper is opt-in and does not change default get variant output.
See also: biomcp list variant")]
    Structure {
        /// Variant identifier (rsID, HGVS, or "GENE CHANGE")
        id: String,
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
    /// Retrieve versioned ClinGen ERepo expert assertions by CAid
    #[command(after_help = "\
EXAMPLES:
  biomcp --json variant erepo CA015543
  biomcp --json variant erepo CA015543 --detail
  biomcp --json variant erepo --input caids.json
  biomcp --json variant erepo --gene PTEN --limit 25 --offset 0

ClinGen ERepo contains germline variant interpretations. For somatic tumor questions, use CIViC: get gene <symbol> civic or get variant <id> civic.

Note: Batch input accepts 1-50 CAids and returns summaries only. Gene search is bounded and paged. Detail requires one CAid and, when multiple assertions exist, an explicit assertion UUID.")]
    Erepo {
        /// ClinGen Allele identifier (for example CA015543)
        #[arg(required_unless_present_any = ["input", "gene"], conflicts_with = "gene")]
        caid: Option<String>,
        /// JSON input file, or - for stdin, containing 1-50 CAids
        #[arg(long, value_name = "PATH", conflicts_with = "gene")]
        input: Option<String>,
        /// Search compact assertions for one HGNC gene symbol
        #[arg(long, conflicts_with_all = ["detail", "assertion", "version"])]
        gene: Option<String>,
        /// Maximum gene-search results, 1-100 (default: 25)
        #[arg(long, default_value = "25", value_parser = parse_erepo_limit)]
        limit: usize,
        /// Skip the first N gene-search results
        #[arg(long, default_value = "0")]
        offset: usize,
        /// Fetch one selected versioned SEPIO detail document
        #[arg(long)]
        detail: bool,
        /// Assertion UUID required when the summary has multiple assertions
        #[arg(long, requires = "detail")]
        assertion: Option<String>,
        /// Exact document version, available only with --detail
        #[arg(long, requires = "detail")]
        version: Option<String>,
    },
    /// Normalize explicit transcript HGVS with Mutalyzer and/or VariantValidator
    #[command(after_help = "\
EXAMPLES:
  biomcp variant normalize all NM_000248.3:c.135del
  biomcp variant normalize mutalyzer NM_000248.3:c.135del
  biomcp variant normalize variantvalidator 'NM_004448.2:c.829G>T'
  biomcp variant normalize car 'NM_000546.6:c.215C>G'

SERVICES:
  all
  mutalyzer
  variantvalidator
  car

Note: This proxy accepts explicit transcript HGVS input and does not parse reports, choose transcripts, classify variants, or assign clinical meaning.
See also: biomcp list variant")]
    Normalize {
        /// Service selector: all, mutalyzer, or variantvalidator; CAR is available as car
        service: String,
        /// Transcript HGVS, or CAR-supported versioned RefSeq genomic HGVS
        #[arg(
            value_name = "hgvs",
            required_unless_present = "input",
            conflicts_with = "input"
        )]
        variant: Option<String>,
        /// JSON file, or - for stdin, containing 1-50 CAR HGVS strings
        #[arg(long, value_name = "PATH", conflicts_with = "variant")]
        input: Option<String>,
    },
    #[command(external_subcommand)]
    External(Vec<String>),
}

fn parse_erepo_limit(value: &str) -> Result<usize, String> {
    let limit = value
        .parse::<usize>()
        .map_err(|_| "limit must be an integer from 1 to 100".to_owned())?;
    if (1..=100).contains(&limit) {
        Ok(limit)
    } else {
        Err("limit must be between 1 and 100".to_owned())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ResolvedVariantQuery {
    pub(super) gene: Option<String>,
    pub(super) hgvsp: Option<String>,
    pub(super) hgvsc: Option<String>,
    pub(super) rsid: Option<String>,
    pub(super) protein_alias: Option<crate::entities::variant::VariantProteinAlias>,
    pub(super) consequence: Option<String>,
    pub(super) condition: Option<String>,
    pub(super) requested_identity: Option<Box<crate::entities::variant::RequestedVariantIdentity>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum VariantSearchPlan {
    Standard(ResolvedVariantQuery),
    Guidance(crate::entities::variant::VariantGuidance),
}

impl VariantSearchPlan {
    fn standard(mut query: ResolvedVariantQuery) -> Self {
        let exact = query.hgvsp.is_some() || query.hgvsc.is_some() || query.rsid.is_some();
        query.requested_identity = exact.then(|| {
            Box::new(
                crate::entities::variant::RequestedVariantIdentity::for_search(
                    query.gene.clone(),
                    query.hgvsp.clone(),
                    query.hgvsc.clone(),
                    query.rsid.clone(),
                ),
            )
        });
        query.hgvsp = query.hgvsp.as_deref().map(dispatch::normalize_search_hgvsp);
        Self::Standard(query)
    }
}

mod articles;
mod car;
mod dispatch;
mod erepo;
mod guidance;
mod normalization_json;
mod trial;
#[cfg(test)]
pub(crate) use self::dispatch::render_loaded_card;
pub(crate) use self::dispatch::{handle_command, handle_get, handle_search};

#[cfg(test)]
#[path = "../../../tests/unit/cli/variant.rs"]
mod tests;
