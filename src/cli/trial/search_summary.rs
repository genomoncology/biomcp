pub(super) fn trial_search_query_summary(
    filters: &crate::entities::trial::TrialSearchFilters,
    query_intervention: Option<&str>,
    offset: usize,
    next_page: Option<&str>,
) -> String {
    let is_ctgov = matches!(
        filters.source,
        crate::entities::trial::TrialSource::ClinicalTrialsGov
    );
    let shows_alias_opt_out = filters.no_alias_expand
        && is_ctgov
        && filters
            .intervention
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty());

    vec![
        filters
            .condition
            .as_deref()
            .map(|v| format!("condition={v}")),
        query_intervention.map(|v| format!("intervention={v}")),
        shows_alias_opt_out.then(|| "alias_expand=off".to_string()),
        filters.facility.as_deref().map(|v| format!("facility={v}")),
        filters.age.map(|v| format!("age={v}")),
        filters.sex.as_deref().map(|v| format!("sex={v}")),
        filters.status.as_deref().map(|v| format!("status={v}")),
        filters.phase.as_deref().map(|v| format!("phase={v}")),
        filters
            .study_type
            .as_deref()
            .map(|v| format!("study_type={v}")),
        filters.sponsor.as_deref().map(|v| format!("sponsor={v}")),
        filters
            .sponsor_type
            .as_deref()
            .map(|v| format!("sponsor_type={v}")),
        filters
            .date_from
            .as_deref()
            .map(|v| format!("date_from={v}")),
        filters.date_to.as_deref().map(|v| format!("date_to={v}")),
        filters.mutation.as_deref().map(|v| format!("mutation={v}")),
        filters.criteria.as_deref().map(|v| format!("criteria={v}")),
        filters
            .biomarker
            .as_deref()
            .map(|v| format!("biomarker={v}")),
        filters
            .prior_therapies
            .as_deref()
            .map(|v| format!("prior_therapies={v}")),
        filters
            .progression_on
            .as_deref()
            .map(|v| format!("progression_on={v}")),
        filters
            .line_of_therapy
            .as_deref()
            .map(|v| format!("line_of_therapy={v}")),
        filters.lat.map(|v| format!("lat={v}")),
        filters.lon.map(|v| format!("lon={v}")),
        filters.distance.map(|v| format!("distance={v}")),
        matches!(filters.source, crate::entities::trial::TrialSource::NciCts)
            .then(|| "source=nci".to_string()),
        filters
            .results_available
            .then(|| "has_results=true".to_string()),
        (offset > 0).then(|| format!("offset={offset}")),
        next_page
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| format!("next_page={value}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(", ")
}

#[cfg(test)]
mod tests {
    use super::trial_search_query_summary;
    use crate::entities::trial::{TrialSearchFilters, TrialSource};

    #[test]
    fn empty_filters_and_blank_next_page_emit_no_terms() {
        assert_eq!(
            trial_search_query_summary(&TrialSearchFilters::default(), None, 0, Some(" ")),
            ""
        );
    }

    #[test]
    fn alias_expand_off_appears_only_for_a_nonblank_ctgov_intervention() {
        let base = TrialSearchFilters {
            intervention: Some("daraxonrasib".to_string()),
            no_alias_expand: true,
            ..Default::default()
        };
        assert_eq!(
            trial_search_query_summary(&base, Some("resolved"), 0, None),
            "intervention=resolved, alias_expand=off"
        );
        let mut flag_off = base.clone();
        flag_off.no_alias_expand = false;
        let mut blank = base.clone();
        blank.intervention = Some(" ".to_string());
        let mut nci = base;
        nci.source = TrialSource::NciCts;
        for filters in [&flag_off, &blank, &nci] {
            assert!(!trial_search_query_summary(filters, None, 0, None).contains("alias_expand"));
        }
    }

    #[test]
    fn every_active_filter_lists_in_the_reviewed_order() {
        let filters = TrialSearchFilters {
            condition: Some("melanoma".to_string()),
            intervention: Some("raw".to_string()),
            facility: Some("Boston".to_string()),
            status: Some("recruiting".to_string()),
            phase: Some("2".to_string()),
            study_type: Some("interventional".to_string()),
            age: Some(65.0),
            sex: Some("female".to_string()),
            sponsor: Some("nci".to_string()),
            sponsor_type: Some("nih".to_string()),
            date_from: Some("2024-01-01".to_string()),
            date_to: Some("2025-01-01".to_string()),
            mutation: Some("BRAF V600E".to_string()),
            criteria: Some("measurable".to_string()),
            biomarker: Some("BRAF".to_string()),
            prior_therapies: Some("pembro".to_string()),
            progression_on: Some("dab".to_string()),
            line_of_therapy: Some("2L".to_string()),
            results_available: true,
            lat: Some(42.36),
            lon: Some(-71.06),
            distance: Some(50),
            source: TrialSource::NciCts,
            ..Default::default()
        };
        assert_eq!(
            trial_search_query_summary(&filters, Some("resolved"), 20, Some(" tok ")),
            "condition=melanoma, intervention=resolved, facility=Boston, age=65, sex=female, status=recruiting, phase=2, study_type=interventional, sponsor=nci, sponsor_type=nih, date_from=2024-01-01, date_to=2025-01-01, mutation=BRAF V600E, criteria=measurable, biomarker=BRAF, prior_therapies=pembro, progression_on=dab, line_of_therapy=2L, lat=42.36, lon=-71.06, distance=50, source=nci, has_results=true, offset=20, next_page=tok"
        );
    }
}
