pub(super) fn drug_search_page_has_results(
    page_with_region: &crate::entities::drug::DrugSearchPageWithRegion,
) -> bool {
    match page_with_region {
        crate::entities::drug::DrugSearchPageWithRegion::Us(page) => !page.results.is_empty(),
        crate::entities::drug::DrugSearchPageWithRegion::Eu(page) => !page.results.is_empty(),
        crate::entities::drug::DrugSearchPageWithRegion::Who(page) => !page.results.is_empty(),
        crate::entities::drug::DrugSearchPageWithRegion::All {
            us,
            eu,
            who,
            who_note: _,
        } => !us.results.is_empty() || !eu.results.is_empty() || !who.results.is_empty(),
    }
}
