//! Study CLI subcommands.

use clap::Subcommand;

use crate::cli::ChartArgs;

#[derive(Subcommand, Debug)]
pub enum StudyCommand {
    /// List locally available cBioPortal studies
    #[command(after_help = "\
EXAMPLES:
  biomcp study list

See also: biomcp list study")]
    List,
    /// List downloadable study IDs or install a study locally
    #[command(after_help = "\
EXAMPLES:
  biomcp study download --list
  biomcp study download msk_impact_2017
  biomcp study download brca_tcga_pan_can_atlas_2018

See also: biomcp list study")]
    Download {
        /// List available remote study IDs instead of downloading a study
        #[arg(long, conflicts_with = "study_id")]
        list: bool,
        /// Study ID to download (required unless --list; for example, msk_impact_2017)
        #[arg(value_name = "STUDY_ID", required_unless_present = "list")]
        study_id: Option<String>,
    },
    /// Run per-study gene query
    #[command(after_help = "\
EXAMPLES:
  biomcp study query --study msk_impact_2017 --gene TP53 --type mutations
  biomcp study query --study brca_tcga_pan_can_atlas_2018 --gene ERBB2 --type cna
  biomcp study query --study paad_qcmg_uq_2016 --gene KRAS --type expression
  biomcp study query --study msk_impact_2017 --gene RET --type sv

See also: biomcp list study")]
    Query {
        /// cBioPortal study ID (for example, msk_impact_2017)
        #[arg(short, long)]
        study: String,
        /// HGNC gene symbol to summarize (for example, TP53 or ERBB2)
        #[arg(short, long)]
        gene: String,
        /// Query type. Canonical values: mutations, cna, expression, sv.
        /// Accepted aliases: mutation, copy_number, copy-number, expr, fusion, fusions, structural_variants.
        #[arg(short = 't', long = "type")]
        query_type: String,
        #[command(flatten)]
        chart: ChartArgs,
    },
    /// Rank the most frequently mutated genes in a study
    #[command(after_help = "\
EXAMPLES:
  biomcp study top-mutated --study msk_impact_2017
  biomcp study top-mutated --study cll_broad_2022 --limit 10

See also: biomcp list study")]
    TopMutated {
        /// cBioPortal study ID (for example, msk_impact_2017)
        #[arg(short, long)]
        study: String,
        /// Maximum number of genes to display, 1-50 (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Intersect sample filters across mutation, CNA, expression, and clinical data
    #[command(after_help = "\
EXAMPLES:
  biomcp study filter --study msk_impact_2017 --mutated TP53
  biomcp study filter --study brca_tcga_pan_can_atlas_2018 --mutated TP53 --amplified ERBB2
  biomcp study filter --study brca_tcga_pan_can_atlas_2018 --mutated TP53 --expression-above ERBB2:1.5 --cancer-type \"Breast Cancer\"

See also: biomcp list study")]
    Filter {
        /// cBioPortal study ID (for example, msk_impact_2017)
        #[arg(short, long)]
        study: String,
        /// Keep samples with a mutation in GENE (repeatable)
        #[arg(long)]
        mutated: Vec<String>,
        /// Keep samples with high-level copy-number amplification in GENE (repeatable)
        #[arg(long)]
        amplified: Vec<String>,
        /// Keep samples with a deep copy-number deletion in GENE (repeatable)
        #[arg(long)]
        deleted: Vec<String>,
        /// Keep samples where GENE expression exceeds THRESHOLD (format: GENE:THRESHOLD; repeatable)
        #[arg(long = "expression-above")]
        expression_above: Vec<String>,
        /// Keep samples where GENE expression is below THRESHOLD (format: GENE:THRESHOLD; repeatable)
        #[arg(long = "expression-below")]
        expression_below: Vec<String>,
        /// Keep samples matching this cancer type label (repeatable)
        #[arg(long = "cancer-type")]
        cancer_type: Vec<String>,
    },
    /// Split a cohort into mutant vs wildtype groups
    #[command(after_help = "\
EXAMPLES:
  biomcp study cohort --study brca_tcga_pan_can_atlas_2018 --gene TP53

See also: biomcp list study")]
    Cohort {
        /// cBioPortal study ID (for example, msk_impact_2017)
        #[arg(short, long)]
        study: String,
        /// HGNC gene symbol used to split mutant vs wildtype groups
        #[arg(short, long)]
        gene: String,
    },
    /// Summarize KM survival and log-rank statistics by mutation group
    #[command(after_help = "\
EXAMPLES:
  biomcp study survival --study brca_tcga_pan_can_atlas_2018 --gene TP53
  biomcp study survival --study brca_tcga_pan_can_atlas_2018 --gene TP53 --endpoint DFS

See also: biomcp list study")]
    Survival {
        /// cBioPortal study ID (for example, msk_impact_2017)
        #[arg(short, long)]
        study: String,
        /// HGNC gene symbol used to define mutant vs wildtype groups
        #[arg(short, long)]
        gene: String,
        /// Survival endpoint. Canonical values: os, dfs, pfs, dss.
        /// Accepted aliases: overall, overall_survival, disease_free, progression_free, disease_specific.
        #[arg(short, long, default_value = "os")]
        endpoint: String,
        #[command(flatten)]
        chart: ChartArgs,
    },
    /// Compare expression or mutation rate across mutation groups
    #[command(after_help = "\
EXAMPLES:
  biomcp study compare --study brca_tcga_pan_can_atlas_2018 --gene TP53 --type expression --target ERBB2
  biomcp study compare --study brca_tcga_pan_can_atlas_2018 --gene TP53 --type mutations --target PIK3CA

See also: biomcp list study")]
    Compare {
        /// cBioPortal study ID (for example, msk_impact_2017)
        #[arg(short, long)]
        study: String,
        /// HGNC gene symbol used to define mutant vs wildtype groups
        #[arg(short, long)]
        gene: String,
        /// Comparison type. Canonical values: expression, mutations.
        /// Accepted aliases: expr, mutation.
        #[arg(short = 't', long = "type")]
        compare_type: String,
        /// Target gene symbol to compare across mutation groups
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
        /// cBioPortal study ID (for example, msk_impact_2017)
        #[arg(short, long)]
        study: String,
        /// Comma-separated HGNC gene symbols (2-10 genes, for example TP53,KRAS,PIK3CA)
        #[arg(short, long)]
        genes: String,
        #[command(flatten)]
        chart: ChartArgs,
    },
}

mod dispatch;
pub(crate) use self::dispatch::handle_command;

#[cfg(test)]
mod tests;
