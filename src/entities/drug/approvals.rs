//! Drugs@FDA lookups use the resolved card's own names and verify each result.

use super::label::{
    card_own_label_brands, label_result_generic_names, label_result_ingredient_names,
};
use super::metadata::map_drugsfda_approvals;
use super::{Drug, DrugApproval};
use crate::error::BioMcpError;
use crate::sources::openfda::{DrugsFdaOpenFda, DrugsFdaProduct, DrugsFdaResult, OpenFdaClient};

pub(super) async fn fetch(
    drug: &Drug,
    label_response: Option<&serde_json::Value>,
    identity_brands: &[String],
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
    let linked_labels = linked_label_identities(drug, label_response, identity_brands);
    response
        .results
        .retain_mut(|row| filter_row(row, drug, &linked_labels, identity_brands));
    Ok(map_drugsfda_approvals(response))
}

struct LabelIdentity {
    brands: Vec<String>,
    ingredient_names: Vec<String>,
    ingredients: Vec<String>,
}

fn linked_label_identities(
    drug: &Drug,
    response: Option<&serde_json::Value>,
    identity_brands: &[String],
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
            let generics = label_result_generic_names(record);
            let combination = crate::transform::drug::card_names_combination(&drug.name);
            if ingredients.is_empty()
                || (combination
                    && (ingredients.len() < 2
                        || !generics.iter().any(|name| equals(name, &drug.name))))
                || (!combination && ingredients.len() != 1)
            {
                return None;
            }
            if combination {
                let requested = drug
                    .name
                    .to_ascii_lowercase()
                    .replace(", and ", ",")
                    .replace(" and ", ",")
                    .split(',')
                    .map(|name| name.trim().to_string())
                    .collect::<Vec<_>>();
                if requested.len() < 2
                    || requested.iter().any(String::is_empty)
                    || !same_ingredient_set(&requested, &ingredients)
                {
                    return None;
                }
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
            let mut ingredient_names = ingredients.clone();
            if !combination {
                // Metadata added from this label may use a qualified name.
                // Only the original card's brand or an exact generic links
                // that label to this product; a shared name core cannot.
                if !generics.iter().any(|name| equals(name, &drug.name))
                    && !brands
                        .iter()
                        .any(|brand| identity_brands.iter().any(|known| equals(brand, known)))
                {
                    return None;
                }
                ingredient_names.extend(generics);
            }
            Some(LabelIdentity {
                brands,
                ingredient_names,
                ingredients,
            })
        })
        .collect()
}

fn equals(left: &str, right: &str) -> bool {
    left.trim().eq_ignore_ascii_case(right.trim())
}

fn same_ingredient_set(left: &[String], right: &[String]) -> bool {
    let normalize = |value: &str| value.trim().to_ascii_lowercase();
    let mut left = left
        .iter()
        .map(|value| normalize(value))
        .collect::<Vec<_>>();
    let mut right = right
        .iter()
        .map(|value| normalize(value))
        .collect::<Vec<_>>();
    left.sort();
    right.sort();
    left == right
}

fn product_matches(
    product: &DrugsFdaProduct,
    meta: Option<&DrugsFdaOpenFda>,
    drug: &Drug,
    labels: &[LabelIdentity],
    identity_brands: &[String],
) -> bool {
    let ingredients = product
        .active_ingredients
        .iter()
        .filter_map(|ingredient| ingredient.name.as_deref())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if crate::transform::drug::card_names_combination(&drug.name) {
        return ingredients.len() == product.active_ingredients.len()
            && product.brand_name.as_deref().is_some_and(|brand| {
                labels.iter().any(|label| {
                    label.ingredients.len() > 1
                        && label.brands.iter().any(|known| equals(brand, known))
                        && same_ingredient_set(&ingredients, &label.ingredients)
                })
            });
    }
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
    let generic_agrees = meta.is_none_or(|meta| {
        meta.generic_name
            .iter()
            .all(|name| equals(name, &drug.name))
    });
    generic_agrees
        && product.brand_name.as_deref().is_some_and(|brand| {
            std::iter::once(&drug.name)
                .chain(identity_brands)
                .chain(labels.iter().flat_map(|label| &label.brands))
                .any(|known| equals(brand, known))
        })
}

fn filter_row(
    row: &mut DrugsFdaResult,
    drug: &Drug,
    labels: &[LabelIdentity],
    identity_brands: &[String],
) -> bool {
    if row.products.is_empty() {
        return !crate::transform::drug::card_names_combination(&drug.name)
            && row.openfda.as_ref().is_some_and(|meta| {
                !meta.generic_name.is_empty()
                    && meta
                        .generic_name
                        .iter()
                        .all(|name| equals(name, &drug.name))
            });
    }
    let metadata = row.openfda.as_ref();
    row.products
        .retain(|product| product_matches(product, metadata, drug, labels, identity_brands));
    !row.products.is_empty()
}
