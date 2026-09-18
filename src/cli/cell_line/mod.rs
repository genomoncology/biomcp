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
    /// Sections to include (variants, xrefs, all)
    #[arg(trailing_var_arg = true)]
    pub sections: Vec<String>,
}

mod dispatch;
pub(super) use self::dispatch::{handle_get, handle_search};
