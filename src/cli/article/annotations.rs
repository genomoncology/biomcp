pub(crate) fn truncate_article_annotations(
    mut annotations: crate::entities::article::ArticleAnnotations,
    limit: usize,
) -> crate::entities::article::ArticleAnnotations {
    annotations.genes.truncate(limit);
    annotations.diseases.truncate(limit);
    annotations.chemicals.truncate(limit);
    annotations.mutations.truncate(limit);
    annotations
}

/// The verified `get disease` command per article disease row identifier
/// (ticket 2047): a MeSH or OMIM identifier opens a card only when the
/// MyDisease crosswalk holds exactly one hit naming the identifier's
/// PubTator3 concept name. Rows that verify use the hit's ontology ID; every
/// other row stays off the map, so its entity row keeps the v0.9.1 search
/// command. One identifier resolves once, no matter how many mention texts
/// share it, and a crosswalk that cannot be read changes no row's command.
pub(super) async fn verified_disease_get_commands(
    annotations: Option<&crate::entities::article::ArticleAnnotations>,
) -> std::collections::HashMap<String, String> {
    use futures::future::join_all;

    let Some(annotations) = annotations else {
        return std::collections::HashMap::new();
    };
    let mut seen = std::collections::HashSet::new();
    let rows = annotations
        .diseases
        .iter()
        .filter(|row| {
            matches!(
                row.namespace.as_deref(),
                Some("MESH") | Some("OMIM")
            ) && row.identifier.as_deref().is_some_and(|identifier| {
                seen.insert(identifier.trim().to_string())
            })
        })
        .collect::<Vec<_>>();
    let commands = join_all(rows.iter().map(|row| async move {
        let identifier = row.identifier.as_deref().unwrap_or_default().trim();
        let command = crate::entities::disease::article_disease_row_get_command(
            row.namespace.as_deref().unwrap_or_default(),
            identifier,
            row.name.as_deref(),
        )
        .await;
        (identifier.to_string(), command)
    }))
    .await;
    commands
        .into_iter()
        .filter_map(|(identifier, command)| command.map(|command| (identifier, command)))
        .collect()
}
