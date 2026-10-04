//! Borrow original operands before the retained positional trim/join preparation.
use super::{ResolvedVariantQuery, VariantSearchPlan};
use crate::entities::variant::{
    IntervalSearchAssertion, IntervalSearchDisposition, VariantGuidance, VariantGuidanceKind,
    is_exact_gene_token, protein_interval_search,
};
use crate::error::BioMcpError;

pub(super) fn retained_preparation(tokens: &[String]) -> Option<String> {
    let positional = tokens
        .iter()
        .map(|token| token.trim())
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    crate::cli::normalize_cli_query(Some(positional))
}

fn operands(tokens: &[String]) -> Option<(Option<&str>, &str)> {
    let mut nonempty = tokens.iter().filter(|token| !token.trim().is_empty());
    let first = nonempty.next()?;
    match (nonempty.next(), nonempty.next()) {
        (None, None) => {
            let mut words = first.split_whitespace();
            let gene = words.next()?;
            if is_exact_gene_token(gene) {
                let change = words.next()?;
                if words.next().is_some() {
                    return None;
                }
                let offset = change.as_ptr() as usize - first.as_ptr() as usize;
                Some((Some(gene), &first[offset..]))
            } else {
                Some((None, first))
            }
        }
        (Some(change), None)
            if first.split_whitespace().count() == 1
                && is_exact_gene_token(first.trim())
                && change.split_whitespace().count() == 1 =>
        {
            Some((Some(first.trim()), change))
        }
        _ => None,
    }
}

fn invalid(message: &str) -> BioMcpError {
    BioMcpError::InvalidArgument(message.to_owned())
}

pub(super) fn resolve(
    tokens: &[String],
    flags: [&Option<String>; 4],
) -> Result<Option<VariantSearchPlan>, BioMcpError> {
    let [gene_flag, hgvsp_flag, consequence, condition] = flags.map(|flag| flag.as_deref());
    let Some((pair_gene, source)) = operands(tokens) else {
        return Ok(None);
    };
    if !IntervalSearchAssertion::selects(source) {
        return Ok(None);
    }
    if pair_gene.is_some() && gene_flag.is_some() {
        return Err(invalid("Positional \"GENE CHANGE\" conflicts with --gene"));
    }
    if hgvsp_flag.is_some() {
        return Err(invalid(if pair_gene.is_some() {
            "Positional \"GENE CHANGE\" conflicts with --hgvsp"
        } else {
            "Positional protein change conflicts with --hgvsp"
        }));
    }
    let assertion = protein_interval_search(source);
    if assertion.disposition() != IntervalSearchDisposition::Checked {
        return Err(invalid("Positional protein interval cannot be searched"));
    }
    let Some(spelling) = assertion.query_spelling() else {
        return Err(invalid("Positional protein interval cannot be searched"));
    };
    let Some(gene) = pair_gene.or(gene_flag) else {
        return Ok(Some(VariantSearchPlan::Guidance(guidance(spelling))));
    };
    VariantSearchPlan::checked_interval(
        ResolvedVariantQuery {
            gene: Some(gene.to_owned()),
            hgvsp: Some(spelling.to_owned()),
            consequence: consequence.map(str::to_owned),
            condition: condition.map(str::to_owned),
            ..Default::default()
        },
        &assertion,
    )
    .map(Some)
}

fn guidance(spelling: &str) -> VariantGuidance {
    let next_commands = if spelling.ends_with('*') {
        vec!["biomcp search variant --help".to_owned()]
    } else {
        vec![
            format!("biomcp search variant --hgvsp '{spelling}' --limit 10"),
            format!("biomcp discover '{spelling}'"),
        ]
    };
    VariantGuidance {
        query: spelling.to_owned(),
        kind: VariantGuidanceKind::ProteinChangeOnly {
            change: spelling.to_owned(),
        },
        next_commands,
    }
}
