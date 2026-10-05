//! Exact gene plus protein admission and bounded checked interval query spellings.
use super::{
    IntervalSearchAssertion, is_exact_gene_token, normalize_protein_change,
    protein_interval_search, split_gene_change_tokens,
};
use crate::entities::variant::VariantIdFormat;
use crate::error::BioMcpError;
use biodata::HgvsProteinStyle;
use std::fmt;

const INPUT_LIMIT: usize = 512;
const LIMIT_MESSAGE: &str = "Protein interval lookup exceeds its input limit.";

pub(in crate::entities::variant) struct ProteinGet<'a> {
    source: &'a str,
    pub(in crate::entities::variant) gene: &'a str,
    change: &'a str,
    assertion: IntervalSearchAssertion<'a>,
    pub(in crate::entities::variant) spellings: Vec<String>,
}
impl fmt::Debug for ProteinGet<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProteinGet")
            .field("source_bytes", &self.source.len())
            .field("gene_bytes", &self.gene.len())
            .field("change_bytes", &self.change.len())
            .field(
                "query_bytes",
                &self.spellings.iter().map(String::len).collect::<Vec<_>>(),
            )
            .field("disposition", &self.assertion.disposition())
            .finish()
    }
}
impl ProteinGet<'_> {
    pub(in crate::entities::variant) fn format(&self) -> VariantIdFormat {
        VariantIdFormat::GeneProteinChange {
            gene: self.gene.to_string(),
            change: self.spellings[0][2..].to_string(),
        }
    }
}

pub(in crate::entities::variant) fn selects(input: &str) -> bool {
    split_gene_change_tokens(input).is_some_and(|(gene, change)| {
        is_exact_gene_token(gene) && IntervalSearchAssertion::selects(change)
    })
}

pub(in crate::entities::variant) fn prepare(
    input: &str,
) -> Result<Option<ProteinGet<'_>>, BioMcpError> {
    if !selects(input) {
        return Ok(None);
    }
    if input.len() > INPUT_LIMIT {
        return Err(BioMcpError::InvalidArgument(LIMIT_MESSAGE.into()));
    }
    let Some((gene, change)) = split_gene_change_tokens(input) else {
        return Ok(None);
    };
    let assertion = protein_interval_search(change);
    let Some(parsed) = assertion.parsed() else {
        return Ok(None);
    };
    let mut spellings = Vec::new();
    for style in [HgvsProteinStyle::OneLetter, HgvsProteinStyle::ThreeLetter] {
        let spelling = parsed.render_constructed(style).map_err(|_| {
            BioMcpError::InvalidArgument(
                "Protein interval lookup could not prepare its query.".into(),
            )
        })?;
        if spelling.len() > INPUT_LIMIT {
            return Err(BioMcpError::InvalidArgument(LIMIT_MESSAGE.into()));
        }
        if !spellings.contains(&spelling) {
            spellings.push(spelling);
        }
    }
    Ok(Some(ProteinGet {
        source: input,
        gene,
        change,
        assertion,
        spellings,
    }))
}

pub(super) fn parse_exact_gene_protein_change(input: &str) -> Option<VariantIdFormat> {
    let (gene, change) = split_gene_change_tokens(input)?;
    if !is_exact_gene_token(gene) {
        return None;
    }
    if let Some(change) = normalize_protein_change(change) {
        return Some(VariantIdFormat::GeneProteinChange {
            gene: gene.to_string(),
            change,
        });
    }
    prepare(input)
        .ok()
        .flatten()
        .map(|prepared| prepared.format())
}
