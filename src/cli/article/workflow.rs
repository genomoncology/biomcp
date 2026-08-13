use super::dispatch::ArticleSuggestion;

pub(super) fn resolved_article_date_bounds(
    args: &super::ArticleSearchArgs,
) -> (Option<String>, Option<String>) {
    let date_from = args
        .date_from
        .clone()
        .or_else(|| args.year_min.map(|year| format!("{year:04}-01-01")));
    let date_to = args
        .date_to
        .clone()
        .or_else(|| args.year_max.map(|year| format!("{year:04}-12-31")));
    (date_from, date_to)
}

pub(super) fn article_keyword_token_count(keyword: &str) -> usize {
    keyword
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'))
        .filter(|token| !token.is_empty())
        .count()
}

pub(super) fn article_follow_up_workflow(
    article: &crate::entities::article::Article,
) -> Result<Option<crate::workflow_ladders::WorkflowMeta>, crate::error::BioMcpError> {
    let has_pmid = article
        .pmid
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    let has_annotations = article.annotations.as_ref().is_some_and(|annotations| {
        !annotations.genes.is_empty()
            || !annotations.diseases.is_empty()
            || !annotations.chemicals.is_empty()
            || !annotations.mutations.is_empty()
    });

    (has_pmid && has_annotations)
        .then(|| {
            crate::workflow_ladders::meta_for(crate::workflow_ladders::Workflow::ArticleFollowUp)
        })
        .transpose()
}

pub(super) fn article_entity_suggestion(
    entity: &crate::entities::discover::ExactArticleKeywordEntity,
) -> ArticleSuggestion {
    let entity_name = entity.entity_type.cli_name();
    let label = entity.label.trim();
    let quoted_label = crate::render::markdown::shell_quote_arg(label);
    let command = format!("biomcp get {entity_name} {quoted_label}");
    let reason = if entity.matched_alias {
        format!(
            "Exact {entity_name} alias match for article keyword \"{}\"; suggested canonical {entity_name} \"{}\".",
            entity.matched_query, entity.label
        )
    } else {
        format!(
            "Exact {entity_name} vocabulary match for article keyword \"{}\".",
            entity.matched_query
        )
    };

    ArticleSuggestion {
        command,
        reason,
        sections: article_entity_sections(entity.entity_type),
    }
}

fn article_entity_sections(entity_type: crate::entities::discover::DiscoverType) -> Vec<String> {
    let (valid_sections, sections): (&[&str], &[&str]) = match entity_type {
        crate::entities::discover::DiscoverType::Gene => (
            crate::entities::gene::GENE_SECTION_NAMES,
            &["protein", "diseases", "expression"],
        ),
        crate::entities::discover::DiscoverType::Drug => (
            crate::entities::drug::DRUG_SECTION_NAMES,
            &["label", "targets", "indications"],
        ),
        crate::entities::discover::DiscoverType::Disease => (
            crate::entities::disease::DISEASE_SECTION_NAMES,
            &["genes", "phenotypes", "diagnostics"],
        ),
        _ => (&[], &[]),
    };
    debug_assert!(
        sections
            .iter()
            .all(|section| valid_sections.contains(section))
    );
    sections
        .iter()
        .map(|section| (*section).to_string())
        .collect()
}
