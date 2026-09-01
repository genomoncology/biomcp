use crate::entities::variant::VariantSearchFilters;
use crate::error::BioMcpError;
use crate::sources::mygene::MyGeneClient;
use crate::sources::myvariant::{
    MYVARIANT_FIELDS_SEARCH, MyVariantClient, MyVariantSearchResponse, VariantSearchParams,
};
use serde::{Serialize, Serializer};
use std::collections::{BTreeSet, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SearchDiagnostic {
    GeneAlias {
        requested: String,
        alias: String,
        total: usize,
    },
    GeneUnavailable {
        requested: String,
    },
    ProteinPositions {
        gene: String,
        protein: String,
        reference: char,
        alternate: char,
        positions: Vec<usize>,
    },
    EmptyIntersection,
}

impl std::fmt::Display for SearchDiagnostic {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GeneAlias {
                requested,
                alias,
                total,
            } => write!(
                formatter,
                "gene {requested} matched no dbNSFP records; retried as {alias} and matched {total}"
            ),
            Self::GeneUnavailable { requested } => write!(
                formatter,
                "gene {requested} matched no dbNSFP records under any known symbol or alias"
            ),
            Self::ProteinPositions {
                gene,
                protein,
                reference,
                alternate,
                positions,
            } => write!(
                formatter,
                "no dbNSFP record for {gene} {protein}; dbNSFP holds {reference} to {alternate} at positions {}",
                positions
                    .iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::EmptyIntersection => formatter.write_str("filters applied; no record matched"),
        }
    }
}

impl Serialize for SearchDiagnostic {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub(super) fn search_params(
    filters: &VariantSearchFilters,
    gene: Option<String>,
    limit: usize,
    offset: usize,
) -> VariantSearchParams {
    VariantSearchParams {
        gene,
        hgvsp: filters.hgvsp.clone(),
        hgvsc: filters.hgvsc.clone(),
        rsid: filters.rsid.clone(),
        protein_alias: filters.protein_alias.clone(),
        significance: filters.significance.clone(),
        max_frequency: filters.max_frequency,
        min_cadd: filters.min_cadd,
        consequence: filters.consequence.clone(),
        review_status: filters.review_status.clone(),
        population: filters.population.clone(),
        revel_min: filters.revel_min,
        gerp_min: filters.gerp_min,
        tumor_site: filters.tumor_site.clone(),
        condition: filters.condition.clone(),
        impact: filters.impact.clone(),
        lof: filters.lof,
        has: filters.has.clone(),
        missing: filters.missing.clone(),
        therapy: filters.therapy.clone(),
        limit,
        offset,
    }
}

fn gene_only_params(gene: &str) -> VariantSearchParams {
    search_params(
        &VariantSearchFilters::default(),
        Some(gene.to_string()),
        1,
        0,
    )
}

fn protein_parts(value: &str) -> Option<(String, char, usize, char)> {
    let normalized = crate::entities::variant::normalize_protein_change(value)?;
    let change = normalized.as_str();
    let mut chars = change.chars();
    let reference = chars.next()?;
    let alternate = change.chars().last()?;
    let position = change[reference.len_utf8()..change.len() - alternate.len_utf8()]
        .parse()
        .ok()?;
    Some((format!("p.{normalized}"), reference, position, alternate))
}

fn independent_filter_params(
    filters: &VariantSearchFilters,
    effective_gene: Option<&str>,
) -> Vec<VariantSearchParams> {
    let mut probes = Vec::new();
    macro_rules! probe {
        ($field:ident) => {
            if filters.$field.is_some() {
                let mut one = VariantSearchFilters::default();
                one.$field = filters.$field.clone();
                probes.push(search_params(&one, None, 1, 0));
            }
        };
    }
    if let Some(hgvsp) = filters.hgvsp.clone() {
        let one = VariantSearchFilters {
            hgvsp: Some(hgvsp),
            ..VariantSearchFilters::default()
        };
        probes.push(search_params(
            &one,
            effective_gene.map(str::to_string),
            1,
            0,
        ));
    }
    probe!(hgvsc);
    probe!(rsid);
    if let Some(protein_alias) = filters.protein_alias.clone() {
        let one = VariantSearchFilters {
            protein_alias: Some(protein_alias),
            ..VariantSearchFilters::default()
        };
        probes.push(search_params(
            &one,
            effective_gene.map(str::to_string),
            1,
            0,
        ));
    }
    probe!(significance);
    if filters.max_frequency.is_some() {
        let one = VariantSearchFilters {
            max_frequency: filters.max_frequency,
            population: filters.population.clone(),
            ..VariantSearchFilters::default()
        };
        probes.push(search_params(&one, None, 1, 0));
    }
    if filters.min_cadd.is_some() {
        let one = VariantSearchFilters {
            min_cadd: filters.min_cadd,
            ..VariantSearchFilters::default()
        };
        probes.push(search_params(&one, None, 1, 0));
    }
    probe!(consequence);
    probe!(review_status);
    if filters.population.is_some() && filters.max_frequency.is_none() {
        let one = VariantSearchFilters {
            population: filters.population.clone(),
            ..VariantSearchFilters::default()
        };
        probes.push(search_params(&one, None, 1, 0));
    }
    if filters.revel_min.is_some() {
        let one = VariantSearchFilters {
            revel_min: filters.revel_min,
            ..VariantSearchFilters::default()
        };
        probes.push(search_params(&one, None, 1, 0));
    }
    if filters.gerp_min.is_some() {
        let one = VariantSearchFilters {
            gerp_min: filters.gerp_min,
            ..VariantSearchFilters::default()
        };
        probes.push(search_params(&one, None, 1, 0));
    }
    probe!(tumor_site);
    probe!(condition);
    probe!(impact);
    if filters.lof {
        let one = VariantSearchFilters {
            lof: true,
            ..VariantSearchFilters::default()
        };
        probes.push(search_params(&one, None, 1, 0));
    }
    probe!(has);
    probe!(missing);
    probe!(therapy);
    probes
}

pub(super) async fn classify_provider_zero(
    client: &MyVariantClient,
    filters: &VariantSearchFilters,
    initial: MyVariantSearchResponse,
    limit: usize,
    offset: usize,
) -> Result<(MyVariantSearchResponse, Vec<SearchDiagnostic>), BioMcpError> {
    let mut response = initial;
    let mut diagnostics = Vec::new();
    let requested_gene = filters
        .gene
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let non_gene_filters = !independent_filter_params(filters, requested_gene).is_empty();
    let mut effective_gene = requested_gene.map(str::to_string);
    let mut gene_matched = requested_gene.is_none();

    if let Some(requested) = requested_gene {
        let gene_response = if non_gene_filters {
            client.search(&gene_only_params(requested)).await?
        } else {
            MyVariantSearchResponse {
                total: response.total,
                hits: Vec::new(),
            }
        };
        gene_matched = gene_response.total.is_some_and(|total| total > 0);
        if !gene_matched {
            let mygene = MyGeneClient::new()?;
            let matches = mygene
                .search(
                    &crate::entities::gene::mygene_query_term(requested),
                    10,
                    0,
                    None,
                )
                .await?;
            let requested_key = requested.to_ascii_uppercase();
            let mut seen = HashSet::new();
            'hits: for hit in matches.hits {
                let candidates = hit.symbol.into_iter().chain(hit.alias.into_vec());
                for candidate in candidates {
                    let candidate = candidate.trim();
                    let key = candidate.to_ascii_uppercase();
                    if candidate.is_empty() || key == requested_key || !seen.insert(key) {
                        continue;
                    }
                    let alias_response = client
                        .search_gene_alias(&gene_only_params(candidate))
                        .await?;
                    let alias_total = alias_response.total.unwrap_or(alias_response.hits.len());
                    if alias_total == 0 {
                        continue;
                    }
                    effective_gene = Some(candidate.to_string());
                    gene_matched = true;
                    diagnostics.push(SearchDiagnostic::GeneAlias {
                        requested: requested.to_string(),
                        alias: candidate.to_string(),
                        total: alias_total,
                    });
                    response = client
                        .search_gene_alias(&search_params(
                            filters,
                            Some(candidate.to_string()),
                            limit,
                            offset,
                        ))
                        .await?;
                    break 'hits;
                }
            }
            if !gene_matched {
                diagnostics.push(SearchDiagnostic::GeneUnavailable {
                    requested: requested.to_string(),
                });
            }
        }
    }

    if response.total.is_some_and(|total| total > 0) {
        return Ok((response, diagnostics));
    }

    let mut protein_matched = filters.hgvsp.is_none();
    if gene_matched
        && let (Some(gene), Some(hgvsp)) = (effective_gene.as_deref(), filters.hgvsp.as_deref())
        && let Some((protein, reference, _requested_position, alternate)) = protein_parts(hgvsp)
    {
        let exact_filter = VariantSearchFilters {
            hgvsp: Some(protein.clone()),
            ..VariantSearchFilters::default()
        };
        let exact = client
            .search(&search_params(&exact_filter, Some(gene.to_string()), 1, 0))
            .await?;
        protein_matched = exact.total.is_some_and(|total| total > 0);
        if !protein_matched {
            let wildcard = format!(
                "dbnsfp.genename:{} AND dbnsfp.hgvsp:p.{reference}*{alternate}",
                MyVariantClient::escape_query_value(gene)
            );
            let alternatives = client
                .query_with_fields(&wildcard, 50, 0, MYVARIANT_FIELDS_SEARCH)
                .await?;
            let mut positions = BTreeSet::new();
            for value in alternatives
                .hits
                .into_iter()
                .filter_map(|hit| hit.dbnsfp)
                .flat_map(|dbnsfp| dbnsfp.hgvsp.into_vec())
            {
                if let Some((_value, found_reference, position, found_alternate)) =
                    protein_parts(&value)
                    && found_reference == reference
                    && found_alternate == alternate
                {
                    positions.insert(position);
                }
            }
            if !positions.is_empty() {
                diagnostics.push(SearchDiagnostic::ProteinPositions {
                    gene: gene.to_string(),
                    protein,
                    reference,
                    alternate,
                    positions: positions.into_iter().collect(),
                });
            }
        }
    }

    if gene_matched && protein_matched {
        let probes = independent_filter_params(filters, effective_gene.as_deref());
        if requested_gene.is_some() || probes.len() > 1 {
            let mut all_matched = true;
            for probe in probes {
                if client
                    .search(&probe)
                    .await?
                    .total
                    .is_none_or(|total| total == 0)
                {
                    all_matched = false;
                    break;
                }
            }
            if all_matched {
                diagnostics.push(SearchDiagnostic::EmptyIntersection);
            }
        }
    }

    Ok((response, diagnostics))
}
