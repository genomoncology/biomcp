//! Drugs@FDA lookups use the resolved card's own names and verify each result.

use super::label::{
    card_own_label_brands, label_result_generic_names, label_result_ingredient_names,
};
use super::metadata::map_drugsfda_approvals;
use super::{Drug, DrugApproval};
use crate::error::BioMcpError;
use crate::sources::openfda::{DrugsFdaResult, OpenFdaClient};

pub(super) async fn fetch(
    drug: &Drug,
    label_response: Option<&serde_json::Value>,
) -> Result<Vec<DrugApproval>, BioMcpError> {
    let names = std::iter::once(drug.name.as_str())
        .chain(drug.brand_names.iter().map(String::as_str))
        .filter(|name| !name.trim().is_empty())
        .take(11)
        .map(OpenFdaClient::escape_query_value)
        .collect::<Vec<_>>();
    let query = names
        .iter()
        .flat_map(|name| {
            [
                "openfda.generic_name",
                "openfda.brand_name",
                "products.brand_name",
                "products.active_ingredients.name",
            ]
            .map(|field| format!("{field}:\"{name}\""))
        })
        .collect::<Vec<_>>()
        .join(" OR ");
    let Some(mut response) = OpenFdaClient::new()?.drugsfda_search(&query, 8, 0).await? else {
        return Ok(Vec::new());
    };
    let linked_labels = linked_label_identities(drug, label_response);
    response
        .results
        .retain(|row| matches_card(row, drug, &linked_labels));
    Ok(map_drugsfda_approvals(response))
}

struct LabelIdentity {
    brands: Vec<String>,
    ingredient_names: Vec<String>,
}

fn linked_label_identities(
    drug: &Drug,
    response: Option<&serde_json::Value>,
) -> Vec<LabelIdentity> {
    response
        .and_then(|value| value.get("results"))
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|record| {
            let mut ingredients = label_result_ingredient_names(record);
            ingredients.sort_by_key(|name| name.to_ascii_lowercase());
            ingredients.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
            // Additional active ingredients make a separate combination product.
            if ingredients.len() != 1 {
                return None;
            }
            let single = serde_json::json!({"results": [record]});
            let brands = card_own_label_brands(&single, &drug.name)
                .into_iter()
                .filter(|brand| {
                    drug.brand_names
                        .iter()
                        .any(|known| brand.eq_ignore_ascii_case(known))
                })
                .collect::<Vec<_>>();
            if brands.is_empty() {
                return None;
            }
            ingredients.extend(label_result_generic_names(record));
            Some(LabelIdentity {
                brands,
                ingredient_names: ingredients,
            })
        })
        .collect()
}

fn equals(left: &str, right: &str) -> bool {
    left.trim().eq_ignore_ascii_case(right.trim())
}

fn matches_card(row: &DrugsFdaResult, drug: &Drug, labels: &[LabelIdentity]) -> bool {
    (!row.products.is_empty()
        && row.products.iter().all(|product| {
            let ingredients = product
                .active_ingredients
                .iter()
                .filter_map(|ingredient| ingredient.name.as_deref())
                .collect::<Vec<_>>();
            if product.active_ingredients.len() > 1 {
                return false;
            }
            if let Some(ingredient) = ingredients.first() {
                if equals(ingredient, &drug.name) {
                    return true;
                }
                // A shared name core or suffix never proves product identity.
                // A selected source label must link the exact brand and full ingredient.
                return product.brand_name.as_deref().is_some_and(|brand| {
                    labels.iter().any(|label| {
                        label.brands.iter().any(|known| equals(brand, known))
                            && label
                                .ingredient_names
                                .iter()
                                .any(|name| equals(ingredient, name))
                    })
                });
            }
            // Brand-only rescue is allowed only without contradictory generic metadata.
            let generic_agrees = row.openfda.as_ref().is_none_or(|meta| {
                meta.generic_name
                    .iter()
                    .all(|name| equals(name, &drug.name))
            });
            generic_agrees
                && product.brand_name.as_deref().is_some_and(|brand| {
                    std::iter::once(&drug.name)
                        .chain(&drug.brand_names)
                        .any(|known| equals(brand, known))
                })
        }))
        || (row.products.is_empty()
            && row.openfda.as_ref().is_some_and(|meta| {
                !meta.generic_name.is_empty()
                    && meta
                        .generic_name
                        .iter()
                        .all(|name| equals(name, &drug.name))
            }))
}
