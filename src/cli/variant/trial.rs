pub(super) async fn variant_trial_mutation_query(id: &str) -> String {
    let id = id.trim();
    if id.is_empty() {
        return String::new();
    }

    if let Ok(crate::entities::variant::VariantIdFormat::GeneProteinChange { gene, change }) =
        crate::entities::variant::parse_variant_id(id)
    {
        let normalized = crate::entities::variant::normalize_protein_change(&change)
            .unwrap_or_else(|| super::dispatch::trim_protein_change_prefix(&change).to_string());
        if !normalized.is_empty() {
            return format!("{gene} {normalized}");
        }
    }

    if let Ok(variant) = crate::entities::variant::get(id, super::super::empty_sections()).await {
        let gene = variant.gene.trim();
        let protein = variant
            .hgvs_p
            .as_deref()
            .map(|value| {
                crate::entities::variant::normalize_protein_change(value).unwrap_or_else(|| {
                    super::dispatch::trim_protein_change_prefix(value).to_string()
                })
            })
            .unwrap_or_default();
        if !gene.is_empty() && !protein.is_empty() {
            return format!("{gene} {protein}");
        }
    }

    id.to_string()
}
