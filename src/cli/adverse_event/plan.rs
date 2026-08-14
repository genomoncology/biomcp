use super::AdverseEventSearchArgs;
use crate::entities::adverse_event::{
    AdverseEventQueryType, AdverseEventSourceFilter, DeviceEventSeriousness,
};
use crate::error::BioMcpError;

#[derive(Debug)]
pub(super) struct AdverseEventSearchPlan {
    pub drug: Option<String>,
    pub query_type: AdverseEventQueryType,
    pub source_filter: AdverseEventSourceFilter,
    pub device_seriousness: Option<DeviceEventSeriousness>,
}

fn reject_inapplicable_filters(route: &str, filters: Vec<&'static str>) -> Result<(), BioMcpError> {
    if filters.is_empty() {
        Ok(())
    } else {
        Err(BioMcpError::InvalidArgument(format!(
            "{route} does not support: {}",
            filters.join(", ")
        )))
    }
}

fn faers_invalid_filters(args: &AdverseEventSearchArgs) -> Vec<&'static str> {
    let mut invalid = Vec::new();
    if args.classification.is_some() {
        invalid.push("--classification");
    }
    if args.device.is_some() {
        invalid.push("--device");
    }
    if args.manufacturer.is_some() {
        invalid.push("--manufacturer");
    }
    if args.product_code.is_some() {
        invalid.push("--product-code");
    }
    invalid
}

fn vaers_invalid_filters(args: &AdverseEventSearchArgs) -> Vec<&'static str> {
    let mut invalid = Vec::new();
    if args.reaction.is_some() {
        invalid.push("--reaction");
    }
    if args.outcome.is_some() {
        invalid.push("--outcome");
    }
    if args.serious.is_some() {
        invalid.push("--serious");
    }
    if args.date_from.is_some() {
        invalid.push("--date-from");
    }
    if args.date_to.is_some() {
        invalid.push("--date-to");
    }
    if args.suspect_only {
        invalid.push("--suspect-only");
    }
    if args.sex.is_some() {
        invalid.push("--sex");
    }
    if args.age_min.is_some() {
        invalid.push("--age-min");
    }
    if args.age_max.is_some() {
        invalid.push("--age-max");
    }
    if args.reporter.is_some() {
        invalid.push("--reporter");
    }
    if args.count.is_some() {
        invalid.push("--count");
    }
    if args.offset > 0 {
        invalid.push("--offset");
    }
    invalid
}

fn recall_invalid_filters(args: &AdverseEventSearchArgs) -> Vec<&'static str> {
    let mut invalid = vaers_invalid_filters(args);
    if args.device.is_some() {
        invalid.push("--device");
    }
    if args.manufacturer.is_some() {
        invalid.push("--manufacturer");
    }
    if args.product_code.is_some() {
        invalid.push("--product-code");
    }
    invalid.retain(|flag| *flag != "--offset");
    invalid
}

fn device_invalid_filters(args: &AdverseEventSearchArgs) -> Vec<&'static str> {
    let mut invalid = Vec::new();
    if args.reaction.is_some() {
        invalid.push("--reaction");
    }
    if args.outcome.is_some() {
        invalid.push("--outcome");
    }
    if args.classification.is_some() {
        invalid.push("--classification");
    }
    if args.date_to.is_some() {
        invalid.push("--date-to");
    }
    if args.suspect_only {
        invalid.push("--suspect-only");
    }
    if args.sex.is_some() {
        invalid.push("--sex");
    }
    if args.age_min.is_some() {
        invalid.push("--age-min");
    }
    if args.age_max.is_some() {
        invalid.push("--age-max");
    }
    if args.reporter.is_some() {
        invalid.push("--reporter");
    }
    if args.count.is_some() {
        invalid.push("--count");
    }
    invalid
}

pub(super) fn search_plan_from_args(
    args: &AdverseEventSearchArgs,
) -> Result<AdverseEventSearchPlan, BioMcpError> {
    let drug = super::super::resolve_query_input(
        args.drug.clone(),
        args.positional_query.clone(),
        "--drug",
    )?;
    let query_type = AdverseEventQueryType::from_flag(&args.r#type)?;
    let source_filter = AdverseEventSourceFilter::from_flag(&args.source)?;

    if args.limit == 0 || args.limit > 50 {
        return Err(BioMcpError::InvalidArgument(
            "--limit must be between 1 and 50".into(),
        ));
    }

    let device_seriousness = match query_type {
        AdverseEventQueryType::Faers => {
            reject_inapplicable_filters("--type faers", faers_invalid_filters(args))?;
            if args.count.is_some() && source_filter != AdverseEventSourceFilter::Faers {
                return Err(BioMcpError::InvalidArgument(
                    "--count requires explicit --source faers".into(),
                ));
            }
            if args.count.is_some() && args.offset > 0 {
                return Err(BioMcpError::InvalidArgument(
                    "--count requires --offset 0".into(),
                ));
            }
            if source_filter == AdverseEventSourceFilter::Vaers {
                reject_inapplicable_filters("--source vaers", vaers_invalid_filters(args))?;
            }
            None
        }
        AdverseEventQueryType::Recall => {
            if source_filter != AdverseEventSourceFilter::All {
                return Err(BioMcpError::InvalidArgument(
                    "--source is only supported for --type faers adverse-event search".into(),
                ));
            }
            reject_inapplicable_filters("--type recall", recall_invalid_filters(args))?;
            None
        }
        AdverseEventQueryType::Device => {
            if source_filter != AdverseEventSourceFilter::All {
                return Err(BioMcpError::InvalidArgument(
                    "--source is only supported for --type faers adverse-event search".into(),
                ));
            }
            if drug.is_some() {
                return Err(BioMcpError::InvalidArgument(
                    "--drug cannot be used with --type device (use --device)".into(),
                ));
            }
            reject_inapplicable_filters("--type device", device_invalid_filters(args))?;
            args.serious
                .as_deref()
                .map(DeviceEventSeriousness::from_flag)
                .transpose()?
        }
    };

    Ok(AdverseEventSearchPlan {
        drug,
        query_type,
        source_filter,
        device_seriousness,
    })
}
