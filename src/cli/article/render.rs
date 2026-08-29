//! Loaded article card rendering.

pub(crate) fn render_loaded_card(
    article: &crate::entities::article::Article,
    sections: &[String],
    json_output: bool,
) -> anyhow::Result<String> {
    let fulltext_summary = super::fulltext_view::article_summary(article)?;
    if json_output {
        let mut next_commands = crate::render::markdown::related_article(article);
        if let Some(not_included) = article.not_included.as_ref() {
            next_commands.extend(not_included.next_commands.clone());
        }
        let structured = crate::render::json::to_entity_json_with_workflow(
            article,
            crate::render::markdown::article_evidence_urls(article),
            next_commands,
            crate::render::provenance::article_section_sources(article),
            super::workflow::article_follow_up_workflow(article)?,
        )?;
        Ok(super::fulltext_view::decorate_json(
            structured,
            fulltext_summary,
        )?)
    } else {
        Ok(super::fulltext_view::decorate_human(
            article,
            crate::render::markdown::article_markdown(article, sections)?,
            fulltext_summary,
        ))
    }
}
