//! Temporary identity display conversion; remove before direct disease-document presentation.
pub(crate) fn identity_display(
    row: &biodata::MyDiseaseRow,
) -> (String, Vec<(&'static str, &'static str)>) {
    let mut losses = Vec::new();
    let selected = [
        biodata::DiseaseNameSection::DiseaseOntology,
        biodata::DiseaseNameSection::Mondo,
    ]
    .into_iter()
    .find_map(|section| {
        row.identity()
            .names()
            .iter()
            .find(|claim| claim.section() == section && !claim.text().trim().is_empty())
    });
    let name = if let Some(claim) = selected {
        if claim.text() != claim.text().trim() {
            losses.push(("name", "trimmed only for display"));
        }
        if row.identity().names().len() > 1 {
            losses.push(("name", "selected one of retained source assertions"));
        }
        claim.text().trim().to_owned()
    } else {
        losses.push(("name", "provider ID display fallback; not a source name"));
        row.identity().provider_id().to_owned()
    };
    (name, losses)
}
