//! Complete provider ranking and retained source custody.
use super::*;

#[derive(Debug)]
pub(super) struct RankedSearchCustody {
    pub(super) pages: Vec<crate::sources::mychem::MyChemQueryResponse>,
    pub(super) candidates: Vec<(DrugSearchResult, DrugSearchMatchKind, MyChemHit)>,
}

pub(super) async fn search_ranked_name_us_page(
    filters: &DrugSearchFilters,
    query: &str,
    limit: usize,
    offset: usize,
) -> Result<RankedDrugSearchPage<DrugSearchResult>, BioMcpError> {
    Ok(
        search_ranked_name_us_page_with_custody(filters, query, limit, offset)
            .await?
            .0,
    )
}

pub(super) async fn search_ranked_name_us_page_with_custody(
    filters: &DrugSearchFilters,
    query: &str,
    limit: usize,
    offset: usize,
) -> Result<(RankedDrugSearchPage<DrugSearchResult>, RankedSearchCustody), BioMcpError> {
    const PAGE_SIZE: usize = 50;
    crate::sources::validate_biothings_result_window("MyChem search", limit, offset)?;
    let q = build_mychem_query(filters)?;
    let client = crate::sources::mychem::MyChemClient::new()?;
    let mut provider_offset = 0;
    let mut candidates: Vec<(DrugSearchResult, DrugSearchMatchKind)> = Vec::new();
    let mut positions: HashMap<String, usize> = HashMap::new();
    let mut pages = Vec::new();
    let mut origins: HashMap<String, MyChemHit> = HashMap::new();

    loop {
        let response = client
            .query_with_fields(
                &q,
                PAGE_SIZE,
                provider_offset,
                crate::sources::mychem::MYCHEM_FIELDS_SEARCH,
            )
            .await?;
        if response.total > crate::sources::BIOTHINGS_MAX_RESULT_WINDOW {
            return Err(BioMcpError::InvalidArgument(format!(
                "Drug search matched {} MyChem rows; narrow the query so complete ranking fits the 10,000-row provider window.",
                response.total
            )));
        }
        let returned = response.hits.len();
        for hit in &response.hits {
            let Some(mut row) = transform::drug::from_mychem_search_hit(hit) else {
                continue;
            };
            row.name = normalized_name(&row.name);
            if row.name.is_empty() {
                continue;
            }
            let kind = mychem_match_kind(hit, query);
            if let Some(index) = positions.get(&row.name).copied() {
                if kind.rank() < candidates[index].1.rank() {
                    let prior = candidates[index].1;
                    candidates[index].1 = kind;
                    if let Some(retained) = origins.get(&row.name) {
                        for claim_index in 0..hit.row.identity().claims().len() {
                            hit.record_value(Some(claim_index), "search ranking", "tier_upgrade_keep_first_row", "later stronger match; original result row retained", Some(serde_json::json!({"name":row.name,"match_kind":kind.as_str(),"prior_match_kind":prior.as_str(),"deduplication_key":row.name,"retained_digest":retained.page.digest(),"retained_ordinal":retained.row.source().ordinal(),"retained_result":{"name":candidates[index].0.name,"drugbank_id":candidates[index].0.drugbank_id,"drug_type":candidates[index].0.drug_type,"mechanism":candidates[index].0.mechanism,"target":candidates[index].0.target}})));
                        }
                    }
                } else {
                    hit.record_claims(
                        "search ranking",
                        "omit_duplicate",
                        "later match does not improve retained tier",
                    );
                }
            } else {
                origins.insert(row.name.clone(), hit.clone());
                hit.record_row(
                    "search ranking",
                    "select_candidate",
                    "first normalized display row",
                );
                positions.insert(row.name.clone(), candidates.len());
                candidates.push((row, kind));
            }
        }
        provider_offset = provider_offset.saturating_add(returned);
        let total = response.total;
        pages.push(response);
        if returned == 0 || provider_offset >= total {
            break;
        }
    }

    if candidates.is_empty() {
        let (page, custody) = search_page_with_custody(filters, limit, offset).await?;
        pages.push(custody);
        let query = normalized_name(query);
        let kinds = page
            .results
            .iter()
            .map(|row| {
                if normalized_name(&row.name) == query {
                    DrugSearchMatchKind::ProductName
                } else {
                    DrugSearchMatchKind::BroadText
                }
            })
            .collect();
        return Ok((
            RankedDrugSearchPage::offset(page.results, page.total, kinds),
            RankedSearchCustody {
                pages,
                candidates: Vec::new(),
            },
        ));
    }

    rank_drug_candidates(&mut candidates);
    let total = candidates.len();
    for (index, (row, _)) in candidates.iter().enumerate() {
        if let Some(hit) = origins.get(&row.name) {
            if index < offset {
                hit.record_claims(
                    "search pagination",
                    "omit_page",
                    "offset after complete ranking",
                );
            } else if index - offset >= limit {
                hit.record_claims(
                    "search pagination",
                    "omit_page",
                    "limit after complete ranking",
                );
            } else {
                hit.record_row(
                    "search pagination",
                    "select",
                    "within page after complete ranking",
                );
            }
        }
    }
    let candidate_custody = candidates
        .iter()
        .filter_map(|(row, kind)| {
            origins
                .get(&row.name)
                .map(|hit| (row.clone(), *kind, hit.clone()))
        })
        .collect();
    let selected = candidates
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();
    let kinds = selected.iter().map(|(_, kind)| *kind).collect();
    let results = selected.into_iter().map(|(row, _)| row).collect();
    Ok((
        RankedDrugSearchPage::offset(results, Some(total), kinds),
        RankedSearchCustody {
            pages,
            candidates: candidate_custody,
        },
    ))
}
