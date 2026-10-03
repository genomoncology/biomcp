//! Drug base resolution retains admitted pages and fallback custody.
use super::*;

pub(in crate::entities::drug) struct ResolvedDrugBase {
    pub(in crate::entities::drug) drug: Drug,
    pub(in crate::entities::drug) label_response: Option<serde_json::Value>,
    pub(in crate::entities::drug) label_attempt_failed: bool,
    pub(in crate::entities::drug) trial_alias_candidates: Vec<TrialAlias>,
    pub(in crate::entities::drug) selected_hits: Vec<MyChemHit>,
    pub(in crate::entities::drug) source_pages: Vec<crate::sources::mychem::MyChemQueryResponse>,
    pub(in crate::entities::drug) fallbacks: Vec<DrugFallback>,
    pub(in crate::entities::drug) label_signals:
        Vec<crate::sources::mychem::conversion::DrugConversion>,
}

#[derive(Debug, serde::Serialize)]
pub(in crate::entities::drug) struct DrugFallback {
    pub(in crate::entities::drug) from: String,
    pub(in crate::entities::drug) to: String,
    pub(in crate::entities::drug) reason: &'static str,
    pub(in crate::entities::drug) candidate_origin: Option<(String, usize)>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(in crate::entities::drug) struct DrugDiscoveryContribution {
    pub(in crate::entities::drug) response_digest: Option<String>,
    pub(in crate::entities::drug) origin: crate::sources::mychem::conversion::ConversionOrigin,
    pub(in crate::entities::drug) namespace: Option<String>,
    pub(in crate::entities::drug) source_term_type:
        crate::sources::mychem::conversion::ConversionTermType,
    pub(in crate::entities::drug) ordinal: usize,
    pub(in crate::entities::drug) source_pointer: String,
    pub(in crate::entities::drug) lexical_text: String,
    pub(in crate::entities::drug) action: &'static str,
    pub(in crate::entities::drug) reason: &'static str,
    pub(in crate::entities::drug) target: Option<String>,
}
#[derive(Default)]
pub(in crate::entities::drug) struct DrugResolverReport {
    pub(in crate::entities::drug) discovery: Vec<DrugDiscoveryContribution>,
    pub(in crate::entities::drug) discarded_hits: Vec<MyChemHit>,
}
enum SparseDrugDiscoverDecision {
    Canonical(String),
    AliasFallback,
    None,
}
pub(in crate::entities::drug) struct SparseDrugDiscoverRescue {
    decision: SparseDrugDiscoverDecision,
    pub(in crate::entities::drug) contributions: Vec<DrugDiscoveryContribution>,
}
impl SparseDrugDiscoverRescue {
    pub(in crate::entities::drug) fn none() -> Self {
        Self {
            decision: SparseDrugDiscoverDecision::None,
            contributions: Vec::new(),
        }
    }
}

fn normalized_discover_drug_label(value: &str) -> String {
    value.trim().trim_matches('.').to_ascii_lowercase()
}

async fn discover_sparse_drug_rescue(name: &str) -> SparseDrugDiscoverRescue {
    let Ok(result) = crate::entities::discover::resolve_query(
        name,
        crate::entities::discover::DiscoverMode::AliasFallback,
    )
    .await
    else {
        return SparseDrugDiscoverRescue::none();
    };

    classify_sparse_drug_rescue(&result)
}
pub(in crate::entities::drug) fn classify_sparse_drug_rescue(
    result: &crate::entities::discover::DiscoverResult,
) -> SparseDrugDiscoverRescue {
    let Some(top) = result.concepts.first() else {
        return SparseDrugDiscoverRescue::none();
    };

    let has_drug_signal = result
        .concepts
        .iter()
        .any(|concept| concept.primary_type == crate::entities::discover::DiscoverType::Drug);
    if !has_drug_signal {
        return SparseDrugDiscoverRescue::none();
    }

    if top.primary_type == crate::entities::discover::DiscoverType::Drug
        && top.match_tier == crate::entities::discover::MatchTier::Exact
        && top.confidence == crate::entities::discover::DiscoverConfidence::CanonicalId
    {
        let top_label = normalized_discover_drug_label(&top.label);
        let competing_exact_drug = result.concepts.iter().any(|concept| {
            concept.primary_type == crate::entities::discover::DiscoverType::Drug
                && concept.match_tier == crate::entities::discover::MatchTier::Exact
                && concept.confidence == crate::entities::discover::DiscoverConfidence::CanonicalId
                && normalized_discover_drug_label(&concept.label) != top_label
        });
        if !top_label.is_empty() && !competing_exact_drug {
            return SparseDrugDiscoverRescue {
                decision: SparseDrugDiscoverDecision::Canonical(top.label.clone()),
                contributions: vec![DrugDiscoveryContribution {
                    response_digest: None,
                    origin: crate::sources::mychem::conversion::ConversionOrigin {
                        section: "concepts".into(),
                        field: "label".into(),
                        section_index: Some(0),
                        value_index: None,
                    },
                    namespace: None,
                    source_term_type: crate::sources::mychem::conversion::ConversionTermType {
                        namespace: "Discover".into(),
                        label: "label".into(),
                    },
                    ordinal: 0,
                    source_pointer: "/concepts/0/label".into(),
                    lexical_text: top.label.clone(),
                    action: "select_canonical_retry_label",
                    reason: "Exact and CanonicalId top Drug has no competing exact canonical Drug label",
                    target: Some(top.label.clone()),
                }],
            };
        }
    }

    let contributions = result
        .concepts
        .iter()
        .enumerate()
        .filter(|(_, concept)| {
            concept.primary_type == crate::entities::discover::DiscoverType::Drug
                && concept.match_tier == crate::entities::discover::MatchTier::Exact
                && concept.confidence == crate::entities::discover::DiscoverConfidence::CanonicalId
        })
        .map(|(ordinal, concept)| DrugDiscoveryContribution {
            response_digest: None,
            origin: crate::sources::mychem::conversion::ConversionOrigin {
                section: "concepts".into(),
                field: "label".into(),
                section_index: Some(ordinal),
                value_index: None,
            },
            namespace: None,
            source_term_type: crate::sources::mychem::conversion::ConversionTermType {
                namespace: "Discover".into(),
                label: "label".into(),
            },
            ordinal,
            source_pointer: format!("/concepts/{ordinal}/label"),
            lexical_text: concept.label.clone(),
            action: "refuse_ambiguous_canonical",
            reason: "competing Exact CanonicalId Drug with different normalized label",
            target: None,
        })
        .collect();
    SparseDrugDiscoverRescue {
        decision: SparseDrugDiscoverDecision::AliasFallback,
        contributions,
    }
}

pub(in crate::entities::drug) async fn resolve_drug_base(
    name: &str,
    fetch_label_response: bool,
    label_required: bool,
) -> Result<ResolvedDrugBase, BioMcpError> {
    resolve_drug_base_with_discover(
        name,
        fetch_label_response,
        label_required,
        discover_sparse_drug_rescue(name),
    )
    .await
}
pub(in crate::entities::drug) async fn resolve_drug_base_with_discover(
    name: &str,
    fetch_label_response: bool,
    label_required: bool,
    discover: impl std::future::Future<Output = SparseDrugDiscoverRescue>,
) -> Result<ResolvedDrugBase, BioMcpError> {
    resolve_drug_base_with_discover_report(
        name,
        fetch_label_response,
        label_required,
        discover,
        &mut DrugResolverReport::default(),
    )
    .await
}

pub(in crate::entities::drug) async fn resolve_drug_base_with_discover_report(
    name: &str,
    fetch_label_response: bool,
    label_required: bool,
    discover: impl std::future::Future<Output = SparseDrugDiscoverRescue>,
    report: &mut DrugResolverReport,
) -> Result<ResolvedDrugBase, BioMcpError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Drug name is required. Example: biomcp get drug pembrolizumab".into(),
        ));
    }
    if name.len() > 256 {
        return Err(BioMcpError::InvalidArgument(
            "Drug name is too long.".into(),
        ));
    }

    let original_not_found = || BioMcpError::NotFound {
        entity: "drug".into(),
        id: name.to_string(),
        suggestion: format!("Try searching: biomcp search drug -q \"{name}\""),
    };

    let mut lookup_name = name.to_string();
    let mut resp = direct_drug_lookup(name).await?;
    let mut source_pages = vec![resp.clone()];
    let mut fallbacks = Vec::new();
    let mut label_signals = Vec::new();

    if resp.hits.is_empty() {
        let fallback_filters = DrugSearchFilters {
            query: Some(name.to_string()),
            ..Default::default()
        };
        let mut candidate_origin = None;
        let fallback_name = crate::entities::drug::search::search_page_with_custody(&fallback_filters, 2, 0)
            .await
            .map(|(page, custody)| {
                if page.results.len() == 1 {
                    let candidate = &page.results[0].name;
                    for hit in &custody.hits {
                        let events = crate::utils::sync::recover_poison(hit.conversion.lock()).clone();
                        if events.iter().any(|event| event.action == "select_display" && event.target.as_ref().and_then(serde_json::Value::as_str) == Some(candidate)) {
                            candidate_origin = Some((hit.page.digest().into(), hit.row.source().ordinal()));
                            for event in events.iter().filter(|event| event.action == "select_display") {
                                hit.record(event.claim_index, "get fallback", "ascii_lowercase_retry_label", "one different nonblank search name selects detail request", Some(candidate.clone()));
                            }
                            for (index, claim) in hit.row.identity().claims().iter().enumerate() {
                                if matches!(claim.value(), biodata::DrugClaimValue::Code(_)) {
                                    hit.record(Some(index), "get fallback", "omit_candidate_identifier", "search result establishes lookup label; final GET alone supplies product code", None);
                                }
                            }
                        }
                    }
                }
                source_pages.push(custody);
                Some(page)
            })
            .or_else(|error| {
                if crate::sources::mychem::optional_failure(&error) {
                    Ok(None)
                } else {
                    Err(error)
                }
            })?
            .and_then(|page| {
                if page.results.len() != 1 {
                    return None;
                }
                let candidate = page.results[0].name.trim();
                if candidate.is_empty() || candidate.eq_ignore_ascii_case(name) {
                    None
                } else {
                    Some(candidate.to_string())
                }
            });

        if let Some(candidate) = fallback_name {
            if let Some(fallback_resp) = optional_lookup(&candidate).await?
                && !fallback_resp.hits.is_empty()
            {
                fallbacks.push(DrugFallback {
                    from: lookup_name.clone(),
                    to: candidate.clone(),
                    reason: "one different nonblank search result",
                    candidate_origin,
                });
                lookup_name = candidate;
                source_pages.push(fallback_resp.clone());
                resp = fallback_resp;
            } else {
                return Err(original_not_found());
            }
        } else {
            return Err(original_not_found());
        }
    }

    let mut selected = transform::drug::select_hits_for_name(&resp.hits, &lookup_name);
    let mut drug = transform::drug::merge_mychem_hits(&selected, &lookup_name);
    let needs_canonical_fallback =
        drug.drugbank_id.is_none() && drug.chembl_id.is_none() && drug.unii.is_none();
    if needs_canonical_fallback
        && let Ok(client) = OpenFdaClient::new()
        && let Ok(Some((label_response, label_bytes))) = client.label_search_with_bytes(name).await
        && let Some((candidate, label_origin)) =
            crate::entities::drug::search::search_results_from_openfda_label_response_with_origin(
                &label_response,
                name,
                1,
            )
            .into_iter()
            .next()
        && !candidate.name.eq_ignore_ascii_case(name)
        && let Some(fallback_resp) = optional_lookup(&candidate.name).await?
        && !fallback_resp.hits.is_empty()
    {
        for hit in &resp.hits {
            record_replaced_display(hit);
        }
        use crate::sources::mychem::conversion::{
            ConversionOrigin, ConversionTermType, DrugConversion,
        };
        use sha2::Digest;
        label_signals.push(DrugConversion {
            response_digest: format!("sha256:{:x}", sha2::Sha256::digest(&label_bytes)),
            ordinal: label_origin.result_index,
            claim_index: None,
            origin: Some(ConversionOrigin {
                section: "results.openfda".into(),
                field: label_origin.field.into(),
                section_index: Some(label_origin.result_index),
                value_index: label_origin.value_index,
            }),
            lexical_text: Some(label_origin.lexical_text),
            namespace: None,
            source_term_type: Some(ConversionTermType {
                namespace: "OpenFDA".into(),
                label: label_origin.field.into(),
            }),
            source_only: false,
            source_pointer: None,
            stage: "get fallback",
            action: "select_retry_label",
            reason: "first generic label supplies different canonical lookup",
            target: Some(serde_json::json!(candidate.name)),
        });
        fallbacks.push(DrugFallback {
            from: drug.name.clone(),
            to: candidate.name.clone(),
            reason: "label canonical signal",
            candidate_origin: None,
        });
        lookup_name = candidate.name;
        source_pages.push(fallback_resp.clone());
        resp = fallback_resp;
        selected = transform::drug::select_hits_for_name(&resp.hits, &lookup_name);
        drug = transform::drug::merge_mychem_hits(&selected, &lookup_name);
    }

    if drug.drugbank_id.is_none() && drug.chembl_id.is_none() && drug.unii.is_none() {
        let rescue = discover.await;
        report.discovery.extend(rescue.contributions);
        match rescue.decision {
            SparseDrugDiscoverDecision::Canonical(candidate) => {
                if let Some(fallback_resp) = optional_lookup(&candidate).await?
                    && !fallback_resp.hits.is_empty()
                {
                    for hit in &resp.hits {
                        record_replaced_display(hit);
                    }
                    fallbacks.push(DrugFallback {
                        from: drug.name.clone(),
                        to: normalized_discover_drug_label(&candidate),
                        reason: "unique discover canonical",
                        candidate_origin: None,
                    });
                    lookup_name = candidate;
                    source_pages.push(fallback_resp.clone());
                    resp = fallback_resp;
                    selected = transform::drug::select_hits_for_name(&resp.hits, &lookup_name);
                    drug = transform::drug::merge_mychem_hits(&selected, &lookup_name);
                }
            }
            SparseDrugDiscoverDecision::AliasFallback => {
                for hit in &resp.hits {
                    hit.record_claims(
                        "get fallback",
                        "discard_sparse_product",
                        "ambiguous canonical discovery refuses successful sparse result",
                    );
                }
                report.discarded_hits.extend(resp.hits.clone());
                return Err(original_not_found());
            }
            SparseDrugDiscoverDecision::None => {}
        }
    }

    let mut label_response_opt: Option<serde_json::Value> = None;
    let mut label_attempt_failed = false;
    if fetch_label_response {
        match OpenFdaClient::new() {
            Ok(client) => match client.label_search(&drug.name).await {
                Ok(v) => label_response_opt = v,
                Err(err) => {
                    if label_required {
                        return Err(err);
                    }
                    label_attempt_failed = true;
                }
            },
            Err(err) => {
                if label_required {
                    return Err(err);
                }
                label_attempt_failed = true;
            }
        }
    }

    if let Some(label_response) = label_response_opt.as_ref() {
        apply_openfda_metadata(&mut drug, label_response);
        drug.label_set_id = extract_label_set_id(label_response);
    }

    let trial_alias_candidates = trial_alias_candidates_from_hits(&selected);
    Ok(ResolvedDrugBase {
        drug,
        label_response: label_response_opt,
        label_attempt_failed,
        trial_alias_candidates,
        selected_hits: selected.into_iter().cloned().collect(),
        source_pages,
        fallbacks,
        label_signals,
    })
}

fn record_replaced_display(hit: &MyChemHit) {
    let events = crate::utils::sync::recover_poison(hit.conversion.lock()).clone();
    for event in events
        .iter()
        .filter(|event| event.stage == "get merge" && event.action == "select_display")
    {
        hit.record_value(
            event.claim_index,
            "get fallback",
            "replace_lookup_display",
            "accepted sparse initial name is replaced by accepted final candidate",
            event.target.clone(),
        );
    }
}
