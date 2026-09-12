//! Drug metadata normalization, approvals, shortage, and FAERS helpers.

use std::collections::HashSet;

use crate::error::BioMcpError;
use crate::sources::mychem::{MyChemHit, MyChemNdcField, MyChemUniiField};
use crate::sources::openfda::{DrugsFdaResult, OpenFdaClient, OpenFdaResponse};

use super::label::extract_openfda_values;
use super::{Drug, DrugApproval, DrugApprovalProduct, DrugApprovalSubmission, DrugShortageEntry};

fn same_populated_id(left: Option<&str>, right: Option<&str>) -> (bool, bool) {
    match (
        left.map(str::trim).filter(|value| !value.is_empty()),
        right.map(str::trim).filter(|value| !value.is_empty()),
    ) {
        (Some(a), Some(b)) => (a.eq_ignore_ascii_case(b), !a.eq_ignore_ascii_case(b)),
        _ => (false, false),
    }
}

fn unii_anchor(resolved: Option<&str>, field: Option<&MyChemUniiField>) -> (bool, bool) {
    let Some(resolved) = resolved.map(str::trim).filter(|value| !value.is_empty()) else {
        return (false, false);
    };
    let values = match field {
        Some(MyChemUniiField::One(value)) => vec![value.unii.as_deref()],
        Some(MyChemUniiField::Many(values)) => {
            values.iter().map(|value| value.unii.as_deref()).collect()
        }
        None => Vec::new(),
    };
    values
        .into_iter()
        .flatten()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .fold((false, false), |(matched, conflict), value| {
            (
                matched || resolved.eq_ignore_ascii_case(value),
                conflict || !resolved.eq_ignore_ascii_case(value),
            )
        })
}

pub(super) fn orphan_aliases(requested_name: &str, drug: &Drug, hits: &[MyChemHit]) -> Vec<String> {
    let mut values = vec![requested_name.to_string(), drug.name.clone()];
    let anchored = drug.drugbank_id.is_some() || drug.chembl_id.is_some() || drug.unii.is_some();
    if anchored {
        for hit in hits {
            let (db_match, db_conflict) = same_populated_id(
                drug.drugbank_id.as_deref(),
                hit.drugbank.as_ref().and_then(|value| value.id.as_deref()),
            );
            let (chembl_match, chembl_conflict) = same_populated_id(
                drug.chembl_id.as_deref(),
                hit.chembl
                    .as_ref()
                    .and_then(|value| value.molecule_chembl_id.as_deref()),
            );
            let (unii_match, unii_conflict) = unii_anchor(drug.unii.as_deref(), hit.unii.as_ref());
            if !(db_match || chembl_match || unii_match)
                || db_conflict
                || chembl_conflict
                || unii_conflict
            {
                continue;
            }
            if let Some(value) = hit.unii.as_ref().and_then(|value| value.display_name()) {
                values.push(value.into());
            }
            if let Some(drugbank) = &hit.drugbank {
                if let Some(value) = &drugbank.name {
                    values.push(value.clone());
                }
                values.extend(drugbank.synonyms.iter().cloned());
            }
            if let Some(value) = hit
                .chembl
                .as_ref()
                .and_then(|value| value.pref_name.as_ref())
            {
                values.push(value.clone());
            }
            if let Some(ndc) = &hit.ndc {
                match ndc {
                    MyChemNdcField::One(row) => {
                        if let Some(value) = &row.nonproprietaryname {
                            values.push(value.clone());
                        }
                    }
                    MyChemNdcField::Many(rows) => {
                        values.extend(rows.iter().filter_map(|row| row.nonproprietaryname.clone()))
                    }
                }
            }
            if let Some(openfda) = &hit.openfda {
                values.extend(openfda.generic_name.clone().into_vec());
                values.extend(openfda.brand_name.clone().into_vec());
            }
        }
    }
    let mut seen = HashSet::new();
    values
        .into_iter()
        .map(|value| value.split_ascii_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|value| !value.is_empty() && seen.insert(value.to_ascii_lowercase()))
        .take(6)
        .collect()
}

fn normalize_date_yyyymmdd(value: Option<&str>) -> Option<String> {
    let v = value?.trim();
    if v.len() != 8 || !v.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!("{}-{}-{}", &v[0..4], &v[4..6], &v[6..8]))
}

fn normalize_route(route: &str) -> String {
    let route = route.trim().to_ascii_lowercase();
    if route.is_empty() {
        return String::new();
    }
    if matches!(
        route.as_str(),
        "iv" | "intravenous" | "intravenous injection" | "intravenous infusion"
    ) {
        return "IV".to_string();
    }
    if matches!(route.as_str(), "subcutaneous" | "sub-cutaneous") {
        return "subcutaneous".to_string();
    }
    route
}

fn maybe_brand_alias(name: &str) -> Option<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() || !trimmed.contains(' ') {
        return None;
    }
    let first = trimmed.split_whitespace().next()?;
    if first.len() < 4 {
        return None;
    }
    if first
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
    {
        return Some(first.to_string());
    }
    None
}

fn route_rank(route: &str) -> usize {
    if route == "IV" {
        0
    } else if route == "subcutaneous" {
        1
    } else if route == "oral" {
        2
    } else {
        3
    }
}

pub(super) fn apply_openfda_metadata(drug: &mut Drug, label_response: &serde_json::Value) {
    let mut brand_names: Vec<String> = extract_openfda_values(label_response, "brand_name");
    brand_names.extend(
        brand_names
            .iter()
            .filter_map(|name| maybe_brand_alias(name))
            .collect::<Vec<_>>(),
    );
    brand_names.extend(drug.brand_names.clone());
    let mut seen: HashSet<String> = HashSet::new();
    let mut merged: Vec<String> = Vec::new();
    for name in brand_names {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            continue;
        }
        let key = trimmed.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        merged.push(trimmed.to_string());
        if merged.len() >= 5 {
            break;
        }
    }
    if !merged.is_empty() {
        drug.brand_names = merged;
    }

    let mut routes = extract_openfda_values(label_response, "route")
        .into_iter()
        .map(|v| normalize_route(&v))
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>();
    if let Some(existing) = drug.route.as_deref() {
        let normalized = normalize_route(existing);
        if !normalized.is_empty() {
            routes.push(normalized);
        }
    }
    routes.sort_by(|a, b| route_rank(a).cmp(&route_rank(b)).then_with(|| a.cmp(b)));
    routes.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
    if !routes.is_empty() {
        drug.route = Some(routes.join(", "));
    }
}

fn trim_nonempty(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn dedupe_trimmed_casefold(values: impl IntoIterator<Item = String>, max: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for value in values {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        let key = value.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        out.push(value.to_string());
        if out.len() >= max {
            break;
        }
    }
    out
}

pub(super) fn map_drugsfda_approvals(resp: OpenFdaResponse<DrugsFdaResult>) -> Vec<DrugApproval> {
    let mut out: Vec<DrugApproval> = Vec::new();
    let mut seen_apps: HashSet<String> = HashSet::new();

    for row in resp.results {
        let Some(application_number) = row
            .application_number
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
        else {
            continue;
        };
        if !seen_apps.insert(application_number.to_ascii_lowercase()) {
            continue;
        }

        let sponsor_name = row
            .sponsor_name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string);

        let (openfda_brand_names, openfda_generic_names) = row
            .openfda
            .map(|meta| {
                (
                    dedupe_trimmed_casefold(meta.brand_name, 10),
                    dedupe_trimmed_casefold(meta.generic_name, 10),
                )
            })
            .unwrap_or_default();

        let mut products: Vec<DrugApprovalProduct> = row
            .products
            .into_iter()
            .filter_map(|product| {
                let brand_name = trim_nonempty(product.brand_name);
                let dosage_form = trim_nonempty(product.dosage_form);
                let route = trim_nonempty(product.route).map(|v| normalize_route(&v));
                let marketing_status = trim_nonempty(product.marketing_status);
                let active_ingredients = dedupe_trimmed_casefold(
                    product.active_ingredients.into_iter().filter_map(|ai| {
                        let name = ai.name.as_deref().map(str::trim).filter(|v| !v.is_empty());
                        let strength = ai
                            .strength
                            .as_deref()
                            .map(str::trim)
                            .filter(|v| !v.is_empty());
                        match (name, strength) {
                            (Some(name), Some(strength)) => Some(format!("{name} ({strength})")),
                            (Some(name), None) => Some(name.to_string()),
                            _ => None,
                        }
                    }),
                    6,
                );

                if brand_name.is_none()
                    && dosage_form.is_none()
                    && route.is_none()
                    && marketing_status.is_none()
                    && active_ingredients.is_empty()
                {
                    return None;
                }

                Some(DrugApprovalProduct {
                    brand_name,
                    dosage_form,
                    route,
                    marketing_status,
                    active_ingredients,
                })
            })
            .collect();

        products.truncate(6);

        let mut submissions: Vec<DrugApprovalSubmission> = row
            .submissions
            .into_iter()
            .filter_map(|submission| {
                let submission_type = trim_nonempty(submission.submission_type);
                let submission_number = trim_nonempty(submission.submission_number);
                let status = trim_nonempty(submission.submission_status);
                let status_date =
                    normalize_date_yyyymmdd(submission.submission_status_date.as_deref());

                if submission_type.is_none()
                    && submission_number.is_none()
                    && status.is_none()
                    && status_date.is_none()
                {
                    return None;
                }

                Some(DrugApprovalSubmission {
                    submission_type,
                    submission_number,
                    status,
                    status_date,
                })
            })
            .collect();

        submissions.sort_by(|a, b| b.status_date.cmp(&a.status_date));
        submissions.truncate(8);

        out.push(DrugApproval {
            application_number,
            sponsor_name,
            openfda_brand_names,
            openfda_generic_names,
            products,
            submissions,
        });
        if out.len() >= 8 {
            break;
        }
    }

    out
}

pub(super) async fn fetch_shortage_entries(
    drug_name: &str,
) -> Result<Vec<DrugShortageEntry>, BioMcpError> {
    let drug_name = drug_name.trim();
    if drug_name.is_empty() {
        return Ok(Vec::new());
    }

    let escaped = OpenFdaClient::escape_query_value(drug_name);
    let q = if drug_name.chars().any(|c| c.is_whitespace()) {
        format!(
            "generic_name:\"{escaped}\" OR openfda.generic_name:\"{escaped}\" OR openfda.brand_name:\"{escaped}\""
        )
    } else {
        format!(
            "generic_name:*{escaped}* OR openfda.generic_name:*{escaped}* OR openfda.brand_name:*{escaped}*"
        )
    };

    let client = OpenFdaClient::new()?;
    let resp = client.shortage_search(&q, 5, 0).await?;
    let Some(resp) = resp else {
        return Ok(Vec::new());
    };

    let out = resp
        .results
        .into_iter()
        .map(|r| DrugShortageEntry {
            status: r
                .status
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
            availability: r
                .availability
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
            company_name: r
                .company_name
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
            generic_name: r
                .generic_name
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
            related_info: r
                .related_info
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
            update_date: normalize_date_yyyymmdd(r.update_date.as_deref()),
            initial_posting_date: normalize_date_yyyymmdd(r.initial_posting_date.as_deref()),
        })
        .collect::<Vec<_>>();

    Ok(out)
}

fn extract_top_adverse_events(resp: &crate::sources::openfda::OpenFdaCountResponse) -> Vec<String> {
    let mut ranked: Vec<(String, usize)> = resp
        .results
        .iter()
        .filter_map(|bucket| {
            let term = bucket.term.trim();
            if term.is_empty() {
                return None;
            }
            Some((term.to_string(), bucket.count))
        })
        .collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked.truncate(3);
    ranked.into_iter().map(|(label, _)| label).collect()
}

fn faers_adverse_event_query(drug_name: &str) -> Option<String> {
    let drug_name = drug_name.trim();
    if drug_name.is_empty() {
        return None;
    }

    let escaped = OpenFdaClient::escape_query_value(drug_name);
    Some(format!(
        "(patient.drug.openfda.generic_name:\"{escaped}\" OR patient.drug.openfda.brand_name:\"{escaped}\" OR patient.drug.medicinalproduct:\"{escaped}\") AND patient.drug.drugcharacterization:1"
    ))
}

pub(super) async fn fetch_top_adverse_events(
    drug_name: &str,
) -> Result<(Vec<String>, Option<String>), BioMcpError> {
    let Some(q) = faers_adverse_event_query(drug_name) else {
        return Ok((Vec::new(), None));
    };

    let client = OpenFdaClient::new()?;
    let resp = client
        .faers_count(&q, "patient.reaction.reactionmeddrapt.exact", 50)
        .await?;
    let Some(resp) = resp else {
        return Ok((Vec::new(), Some(q)));
    };
    Ok((extract_top_adverse_events(&resp), Some(q)))
}

pub(super) async fn populate_top_adverse_event_preview(drug: &mut Drug) -> bool {
    match tokio::time::timeout(
        super::OPTIONAL_SAFETY_TIMEOUT,
        fetch_top_adverse_events(&drug.name),
    )
    .await
    {
        Ok(Ok((events, faers_query))) => {
            drug.top_adverse_events = events;
            drug.faers_query = faers_query;
            false
        }
        Ok(Err(_)) | Err(_) => true,
    }
}

pub(super) fn merge_unique_casefold(
    dst: &mut Vec<String>,
    values: impl IntoIterator<Item = String>,
) {
    let mut seen: HashSet<String> = dst.iter().map(|v| v.to_ascii_lowercase()).collect();
    for value in values {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        let key = value.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        dst.push(value.to_string());
    }
}

#[cfg(test)]
mod tests;
