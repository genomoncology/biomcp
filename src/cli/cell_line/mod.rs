//! Cell-line CLI payloads.

use clap::Args;

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
    /// Sections to include (variants, xrefs, chembl, all)
    #[arg(trailing_var_arg = true)]
    pub sections: Vec<String>,
}

mod dispatch;
pub(super) use self::dispatch::{handle_get, handle_search};

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
  biomcp get cell-line ACH-000362

See also: biomcp list cell-line";
