use super::{VariantSearchParams, field_presence_expression, normalize_filter_key};
use crate::error::BioMcpError;

fn has_non_empty(value: Option<&str>) -> bool {
    value.is_some_and(|value| !value.trim().is_empty())
}

pub(super) fn validate_missing_filter_conflicts(
    params: &VariantSearchParams,
) -> Result<(), BioMcpError> {
    let Some(missing) = params.missing.as_deref() else {
        return Ok(());
    };
    field_presence_expression("--missing", missing)?;
    let missing = normalize_filter_key(missing);
    let same_presence_filter = params.has.as_deref().is_some_and(|has| {
        field_presence_expression("--has", has).is_ok() && normalize_filter_key(has) == missing
    });
    let conflicting = same_presence_filter
        || match missing.as_str() {
            "cadd" => params.min_cadd.is_some(),
            "revel" => params.revel_min.is_some(),
            "gerp" => params.gerp_min.is_some(),
            "gnomad" => {
                params.max_frequency.is_some() || has_non_empty(params.population.as_deref())
            }
            "clinvar" => {
                has_non_empty(params.significance.as_deref())
                    || has_non_empty(params.review_status.as_deref())
                    || has_non_empty(params.condition.as_deref())
            }
            "snpeff" => {
                has_non_empty(params.consequence.as_deref())
                    || has_non_empty(params.impact.as_deref())
                    || params.lof
            }
            "civic" => has_non_empty(params.therapy.as_deref()),
            "cosmic" => has_non_empty(params.tumor_site.as_deref()),
            _ => false,
        };
    if conflicting {
        return Err(BioMcpError::InvalidArgument(format!(
            "Filters requiring {missing} cannot be combined with --missing {missing}"
        )));
    }
    Ok(())
}
