//! Cell-line CLI payloads.

use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct CellLineSearchArgs {
    /// Cell line name, synonym, or CVCL accession
    #[arg(short, long)]
    pub query: Option<String>,
    /// Positional alias for -q/--query; multi-word queries must be quoted
    #[arg(value_name = "QUERY")]
    pub positional_query: Option<String>,
    /// Maximum results, 1-25 (default: 10)
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
    /// Skip the first N results
    #[arg(long, default_value = "0")]
    pub offset: usize,
}

#[derive(Args, Debug)]
pub struct CellLineGetArgs {
    /// Cellosaurus accession (CVCL_2119) or a DepMap, Cell Model Passports, ChEMBL, or PharmacoDB ID
    pub id: String,
    /// Sections to include (variants, xrefs, chembl, drug_response, all)
    #[arg(trailing_var_arg = true)]
    pub sections: Vec<String>,
}

/// Cell-line cross-entity helpers.
#[derive(Subcommand, Debug)]
pub enum CellLineCommand {
    /// Show published PharmacoDB drug-response rows for this cell line in one dataset
    #[command(after_help = "\
EXAMPLES:
  biomcp cell-line drug-response CVCL_2119 --dataset GDSC1
  biomcp cell-line drug-response CVCL_2119 --dataset CTRPv2 --limit 50
  biomcp cell-line drug-response ACH-000362 --dataset GDSC2 --offset 25

Note: `--dataset` is required. One cell line can hold tens of thousands of
experiments, so BioMCP never lists a whole cell line. Use
`biomcp get cell-line <accession> drug_response` for the counts per dataset.
Values are PharmacoDB metrics as published; BioMCP adds no units and no
sensitivity labels.
See also: biomcp list cell-line")]
    DrugResponse {
        /// Cellosaurus accession (CVCL_2119) or any source ID `get cell-line` accepts
        id: String,
        /// PharmacoDB dataset name (e.g., GDSC1); required
        #[arg(long)]
        dataset: String,
        /// Maximum rows, 1-100 (default: 25)
        #[arg(short, long, default_value = "25")]
        limit: usize,
        /// Skip the first N rows
        #[arg(long, default_value = "0")]
        offset: usize,
    },
}

mod dispatch;
pub(super) use self::dispatch::{handle_command, handle_get, handle_search};

/// Help text for `biomcp search cell-line`, kept with the entity.
pub(super) const SEARCH_AFTER_HELP: &str = "\
EXAMPLES:
  biomcp search cell-line MOLM13
  biomcp search cell-line \"MV4;11\"
  biomcp search cell-line KG1 --limit 5
  biomcp search cell-line CVCL_2119

Note: HL-60 (CVCL_0002) and HL-60(TB) (CVCL_A794) are separate lines.
See also: biomcp list cell-line";

/// Help text for `biomcp get cell-line`, kept with the entity.
pub(super) const GET_AFTER_HELP: &str = "\
EXAMPLES:
  biomcp get cell-line CVCL_2119
  biomcp get cell-line CVCL_2119 xrefs
  biomcp get cell-line CVCL_1844 variants
  biomcp get cell-line CVCL_2119 chembl
  biomcp get cell-line CVCL_2119 drug_response
  biomcp get cell-line ACH-000362

See also: biomcp list cell-line";
