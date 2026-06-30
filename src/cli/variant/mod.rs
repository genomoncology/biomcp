//! Variant CLI payloads and subcommands.

use clap::{Args, Subcommand};

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
    /// Min CADD score (>=0)
    #[arg(long)]
    pub min_cadd: Option<f64>,
    /// Functional consequence filter (e.g., missense_variant)
    #[arg(long)]
    pub consequence: Option<String>,
    /// ClinVar review status filter (e.g., 2, expert_panel)
    #[arg(long = "review-status")]
    pub review_status: Option<String>,
    /// Population AF scope (afr, amr, eas, fin, nfe, sas)
    #[arg(long)]
    pub population: Option<String>,
    /// Minimum REVEL score
    #[arg(long = "revel-min")]
    pub revel_min: Option<f64>,
    /// Minimum GERP score
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
    /// Require presence of a field
    #[arg(long)]
    pub has: Option<String>,
    /// Require missing field
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
    /// Exact rsID, HGVS, or "GENE CHANGE" (e.g., rs113488022, "BRAF V600E", "BRAF p.Val600Glu")
    pub id: String,
    /// Sections to include (predict, predictions, clinvar, population, conservation, cosmic, cgi, civic, cbioportal, gwas, all)
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
    /// Search articles using variant annotations, with labeled best-effort fallback
    #[command(after_help = "\
EXAMPLES:
  biomcp variant articles \"BRAF V600E\" --limit 5
  biomcp variant articles rs113488022 --limit 5

Note: Uses PubTator variant annotations when a confident entity match is available, and labels best-effort free-text fallback when annotation recall is unavailable.
See also: biomcp list variant")]
    Articles {
        /// Variant identifier (rsID, HGVS, or "GENE CHANGE")
        id: String,
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
    /// Normalize explicit transcript HGVS with Mutalyzer and/or VariantValidator
    #[command(after_help = "\
EXAMPLES:
  biomcp variant normalize all NM_000248.3:c.135del
  biomcp variant normalize mutalyzer NM_000248.3:c.135del
  biomcp variant normalize variantvalidator 'NM_004448.2:c.829G>T'

SERVICES:
  all
  mutalyzer
  variantvalidator

Note: This proxy accepts explicit transcript HGVS input and does not parse reports, choose transcripts, classify variants, or assign clinical meaning.
See also: biomcp list variant")]
    Normalize {
        /// Service selector: all, mutalyzer, or variantvalidator
        service: String,
        /// Transcript HGVS, e.g. NM_000248.3:c.135del
        #[arg(value_name = "transcript_hgvs")]
        variant: String,
    },
    #[command(external_subcommand)]
    External(Vec<String>),
}

mod dispatch;
mod guidance;
mod normalization_json;
pub(crate) use self::dispatch::{handle_command, handle_get, handle_search};

#[cfg(test)]
mod tests;
