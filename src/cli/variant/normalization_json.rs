use serde::Serialize;

#[derive(Debug, Serialize)]
struct VariantNormalizationJsonResponse<'a> {
    input: &'a str,
    status: &'static str,
    message: String,
    results: Vec<&'a str>,
    services: &'a [crate::entities::variant::VariantNormalizationAggregate],
    _meta: VariantNormalizationJsonMeta,
}

#[derive(Debug, Serialize)]
struct VariantNormalizationJsonMeta {
    next_commands: Vec<String>,
}

pub(super) fn render(
    result: &crate::entities::variant::VariantNormalizationResponse,
) -> Result<String, crate::error::BioMcpError> {
    let mut results = Vec::new();
    for value in result.services.iter().flat_map(|service| match service {
        crate::entities::variant::VariantNormalizationAggregate::Legacy(service) => service
            .genomic_descriptions
            .iter()
            .map(|value| value.coordinate.as_str())
            .chain(service.normalized_description.as_deref())
            .collect::<Vec<_>>(),
        crate::entities::variant::VariantNormalizationAggregate::Car(car) => {
            car.item.caid.as_deref().into_iter().collect::<Vec<_>>()
        }
    }) {
        if !value.trim().is_empty() && !results.contains(&value) {
            results.push(value);
        }
    }
    let status = if results.is_empty() {
        "no_result"
    } else {
        "success"
    };
    let message = if results.is_empty() {
        "No normalized variant result was available from the selected service(s).".to_string()
    } else {
        format!("Found {} normalized variant result(s).", results.len())
    };
    let quoted_input = crate::render::markdown::quote_arg(&result.input);
    crate::render::json::to_pretty(&VariantNormalizationJsonResponse {
        input: &result.input,
        status,
        message,
        results,
        services: &result.services,
        _meta: VariantNormalizationJsonMeta {
            next_commands: vec![
                format!("biomcp variant normalize all {quoted_input}"),
                format!("biomcp get variant {quoted_input}"),
            ],
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::variant::{
        VariantNormalizationAggregate, VariantNormalizationResponse,
        VariantNormalizationServiceResult, VariantNormalizationStatus,
    };

    #[test]
    fn render_deduplicates_aggregate_results() {
        let response = VariantNormalizationResponse {
            input: "NM_000000.0:c.1A>T".to_string(),
            services: vec![
                VariantNormalizationAggregate::Legacy(VariantNormalizationServiceResult {
                    service: "mutalyzer".to_string(),
                    status: VariantNormalizationStatus::Success,
                    input_description: None,
                    normalized_description: Some("NM_000000.0:c.1A>T".to_string()),
                    corrected_description: None,
                    transcript_description: None,
                    protein: None,
                    genomic_descriptions: Vec::new(),
                    warnings: Vec::new(),
                    message: None,
                }),
                VariantNormalizationAggregate::Legacy(VariantNormalizationServiceResult {
                    service: "variantvalidator".to_string(),
                    status: VariantNormalizationStatus::Success,
                    input_description: None,
                    normalized_description: Some("NM_000000.0:c.1A>T".to_string()),
                    corrected_description: None,
                    transcript_description: None,
                    protein: None,
                    genomic_descriptions: vec![crate::entities::GenomicCoordinate {
                        coordinate: "NC_000001.11:g.1A>T".into(),
                        genome_build: "GRCh38".into(),
                        source: "test".into(),
                        provenance: None,
                    }],
                    warnings: Vec::new(),
                    message: None,
                }),
            ],
        };

        let rendered = render(&response).expect("render normalization JSON");
        let payload: serde_json::Value = serde_json::from_str(&rendered).expect("parse JSON");

        assert_eq!(
            payload["results"].as_array().expect("results array").len(),
            2
        );
        assert_eq!(payload["message"], "Found 2 normalized variant result(s).");
    }
}
