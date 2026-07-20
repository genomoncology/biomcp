//! Article HTML-to-markdown extraction and structural classification helpers.

use readability_rust::Readability;
use scraper::{ElementRef, Html, Selector};

use crate::error::BioMcpError;

use super::{
    ArticleDocumentCoverage, ArticleDocumentUnusable, ArticleSupplementLink,
    ClassifiedArticleDocument, collapse_whitespace,
};

pub(crate) fn classify_html_document(
    html: &str,
    base_url: &str,
) -> Result<ClassifiedArticleDocument, ArticleDocumentUnusable> {
    let document = Html::parse_document(html);
    let root = select_content_root(&document).ok_or(ArticleDocumentUnusable::Unsupported)?;
    let blocks = Selector::parse("p, li, td, th, caption, figcaption, blockquote, pre")
        .expect("static body selector");
    let paragraphs = Selector::parse("p").expect("static paragraph selector");

    let mut has_body = false;
    let mut abstract_parts = Vec::new();
    for block in root.select(&blocks) {
        let text = collapse_whitespace(&block.text().collect::<String>());
        if text.is_empty() || is_excluded_block(block, root) {
            continue;
        }
        if is_abstract_block(block, root) {
            if block.value().name() == "p"
                || (block.value().name() == "li" && block.select(&paragraphs).next().is_none())
            {
                abstract_parts.push(text);
            }
        } else {
            has_body = true;
        }
    }
    let abstract_text = (!abstract_parts.is_empty()).then(|| abstract_parts.join(" "));

    let coverage = if has_body {
        ArticleDocumentCoverage::FullText
    } else if abstract_text.is_some() {
        ArticleDocumentCoverage::AbstractOnly
    } else {
        ArticleDocumentCoverage::MetadataOnly
    };
    let markdown = if coverage == ArticleDocumentCoverage::FullText {
        let rendered = extract_text_from_html(html, base_url)
            .map_err(|_| ArticleDocumentUnusable::Conversion)?;
        if rendered.trim().is_empty() {
            return Err(ArticleDocumentUnusable::Conversion);
        }
        Some(rendered)
    } else {
        None
    };

    Ok(ClassifiedArticleDocument {
        coverage,
        markdown,
        abstract_text,
        quality: crate::entities::article::ArticleFulltextQuality {
            has_fulltext_signal: coverage == ArticleDocumentCoverage::FullText,
            ..crate::entities::article::ArticleFulltextQuality::default()
        },
    })
}

pub(crate) fn extract_pmc_supplement_links(
    html: &str,
) -> Result<Vec<ArticleSupplementLink>, ArticleDocumentUnusable> {
    let document = Html::parse_document(html);
    let root = select_content_root(&document).ok_or(ArticleDocumentUnusable::Unsupported)?;
    let anchors = Selector::parse("a[href]").expect("static supplement-link selector");
    let mut links = Vec::new();
    for anchor in root.select(&anchors) {
        if is_excluded_block(anchor, root) {
            continue;
        }
        let provider_marker = anchor
            .value()
            .attr("data-ga-action")
            .is_some_and(|value| value.to_ascii_lowercase().contains("suppl"));
        let in_supplement = anchor
            .ancestors()
            .filter_map(ElementRef::wrap)
            .any(|ancestor| {
                ancestor.id() != root.id()
                    && semantic_tokens(ancestor).any(|token| {
                        matches!(
                            token.to_ascii_lowercase().as_str(),
                            "sm" | "supp" | "supplement" | "supplementary"
                        )
                    })
            });
        if !provider_marker && !in_supplement {
            continue;
        }
        let href = anchor.value().attr("href").unwrap_or_default().trim();
        let identity = href.split(['?', '#']).next().unwrap_or_default();
        let Some(filename) = identity
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let label = collapse_whitespace(&anchor.text().collect::<String>());
        links.push(ArticleSupplementLink {
            href: href.to_string(),
            filename: filename.to_string(),
            label: (!label.is_empty()).then_some(label),
            media_type: None,
        });
    }
    links.sort_by(|left, right| left.href.cmp(&right.href));
    links.dedup_by(|left, right| left.href == right.href);
    Ok(links)
}

fn select_content_root(document: &Html) -> Option<ElementRef<'_>> {
    let main_article = Selector::parse("main article").expect("static content-root selector");
    if let Some(root) = document.select(&main_article).next() {
        return Some(root);
    }

    let article = Selector::parse("article").expect("static content-root selector");
    if let Some(root) = document.select(&article).find(|candidate| {
        candidate
            .ancestors()
            .filter_map(ElementRef::wrap)
            .all(|ancestor| {
                !matches!(
                    ancestor.value().name(),
                    "article" | "main" | "header" | "nav" | "footer" | "aside"
                )
            })
    }) {
        return Some(root);
    }

    let main = Selector::parse("main").expect("static content-root selector");
    document.select(&main).next()
}

fn semantic_tokens(element: ElementRef<'_>) -> impl Iterator<Item = &str> {
    [
        element.value().attr("id"),
        element.value().attr("class"),
        element.value().attr("role"),
    ]
    .into_iter()
    .flatten()
    .flat_map(|value| value.split(|ch: char| !ch.is_ascii_alphanumeric()))
    .filter(|token| !token.is_empty())
}

fn has_semantic_token(element: ElementRef<'_>, expected: &str) -> bool {
    semantic_tokens(element).any(|token| token.eq_ignore_ascii_case(expected))
}

fn is_abstract_block(block: ElementRef<'_>, root: ElementRef<'_>) -> bool {
    if has_semantic_token(block, "abstract") {
        return true;
    }
    for ancestor in block.ancestors().filter_map(ElementRef::wrap) {
        if has_semantic_token(ancestor, "abstract") {
            return true;
        }
        if ancestor.id() == root.id() {
            break;
        }
    }
    false
}

fn is_excluded_block(block: ElementRef<'_>, root: ElementRef<'_>) -> bool {
    const EXCLUDED_TAGS: &[&str] = &["header", "nav", "footer", "aside", "script", "style"];
    const EXCLUDED_TOKENS: &[&str] = &[
        "metadata",
        "byline",
        "author",
        "authors",
        "affiliation",
        "affiliations",
        "keyword",
        "keywords",
        "permission",
        "permissions",
        "ref",
        "reference",
        "references",
        "bibliography",
    ];

    if EXCLUDED_TAGS.contains(&block.value().name())
        || EXCLUDED_TOKENS
            .iter()
            .any(|token| has_semantic_token(block, token))
    {
        return true;
    }
    for ancestor in block.ancestors().filter_map(ElementRef::wrap) {
        if EXCLUDED_TAGS.contains(&ancestor.value().name())
            || EXCLUDED_TOKENS
                .iter()
                .any(|token| has_semantic_token(ancestor, token))
        {
            return true;
        }
        if ancestor.id() == root.id() {
            break;
        }
    }
    false
}

pub fn extract_text_from_html(html: &str, base_url: &str) -> Result<String, BioMcpError> {
    let extracted_html = extract_readable_html(html, base_url)?;
    let source_html = if extracted_html.trim().is_empty() {
        html
    } else {
        extracted_html.as_str()
    };

    let markdown = htmd::convert(source_html).map_err(|err| BioMcpError::Api {
        api: "article".to_string(),
        message: format!("HTML to markdown conversion failed: {err}"),
    })?;

    Ok(markdown.trim().to_string())
}

fn extract_readable_html(html: &str, base_url: &str) -> Result<String, BioMcpError> {
    let mut parser =
        Readability::new_with_base_uri(html, base_url, None).map_err(|err| BioMcpError::Api {
            api: "article".to_string(),
            message: format!("HTML readability initialization failed: {err}"),
        })?;

    Ok(parser
        .parse()
        .and_then(|article| article.content)
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PMC_ARTICLE_PAGE: &str =
        include_str!("../../../tests/fixtures/article/fulltext/html/pmc_article_page.html");
    const BIORXIV_PREPRINT_PAGE: &str =
        include_str!("../../../tests/fixtures/article/fulltext/html/biorxiv_preprint_page.html");
    const NIH_NEWS_RELEASE_PAGE: &str =
        include_str!("../../../tests/fixtures/article/fulltext/html/nih_news_release.html");

    #[test]
    fn extract_text_from_html_keeps_article_signals_across_fixture_family() {
        let cases: [(&str, &str, &[&str]); 3] = [
            (
                PMC_ARTICLE_PAGE,
                "https://pmc.ncbi.nlm.nih.gov/articles/PMC123457/",
                &["PMC HTML fallback winner", "PMC HTML fallback body text."],
            ),
            (
                BIORXIV_PREPRINT_PAGE,
                "https://www.biorxiv.org/content/10.1101/2025.01.01.123456v1",
                &["Preprint markdown quality guard body."],
            ),
            (
                NIH_NEWS_RELEASE_PAGE,
                "https://www.nih.gov/news-events/news-releases/nih-quality-guard",
                &["News release markdown quality guard body."],
            ),
        ];

        for (html, base_url, expected) in cases {
            let markdown =
                extract_text_from_html(html, base_url).expect("fixture HTML should convert");

            for needle in expected {
                assert!(
                    markdown.contains(needle),
                    "missing HTML fixture signal: {needle}"
                );
            }
        }
    }

    #[test]
    fn supplement_links_are_limited_to_the_selected_article_supplement_region() {
        let html = r#"<main><article>
          <nav><a href='/articles/instance/1/bin/nav.csv'>nav</a></nav>
          <section class='sm'><div class='media'><a href='/articles/instance/1/bin/data.xlsx'>Workbook</a></div></section>
          <section class='references'><a data-ga-action='click_feat_suppl' href='/articles/instance/1/bin/ref.pdf'>reference</a></section>
          <p><a href='https://doi.org/10.1/example'>ordinary</a></p>
        </article></main>"#;

        let links = extract_pmc_supplement_links(html).expect("selected article root");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].filename, "data.xlsx");
        assert_eq!(links[0].label.as_deref(), Some("Workbook"));
    }

    #[test]
    fn html_classification_uses_article_structure_without_size_thresholds() {
        let fulltext_cases = [
            "<main><article><p>x</p></article></main>",
            "<article><ul><li>one item</li></ul></article>",
            "<main><table><tr><td>one cell</td></tr></table></main>",
            "<main><figure><figcaption>one caption</figcaption></figure></main>",
        ];
        for html in fulltext_cases {
            let classified = classify_html_document(html, "https://example.test/")
                .expect("structural body fixture");
            assert_eq!(classified.coverage, ArticleDocumentCoverage::FullText);
            assert!(classified.quality.has_fulltext_signal);
        }

        let partial_cases = [
            (
                "<main><h1>Title</h1><section class='abstract'><p>Abstract evidence</p></section></main>",
                ArticleDocumentCoverage::AbstractOnly,
            ),
            (
                "<article><h1>Title only</h1></article>",
                ArticleDocumentCoverage::MetadataOnly,
            ),
            (
                "<main><section><h2>Heading only</h2><p> </p></section></main>",
                ArticleDocumentCoverage::MetadataOnly,
            ),
            (
                "<main><nav><p>Chrome only</p></nav><section class='ref-list'><p>Reference only</p></section></main>",
                ArticleDocumentCoverage::MetadataOnly,
            ),
        ];
        for (html, expected) in partial_cases {
            let classified = classify_html_document(html, "https://example.test/")
                .expect("structural partial fixture");
            assert_eq!(classified.coverage, expected);
            assert!(!classified.quality.has_fulltext_signal);
            if expected == ArticleDocumentCoverage::AbstractOnly {
                assert_eq!(
                    classified.abstract_text.as_deref(),
                    Some("Abstract evidence")
                );
            }
        }

        for html in [
            "<html><body><p>no provider root</p></body></html>",
            "<html><body><aside><article><p>aside article</p></article></aside></body></html>",
        ] {
            assert!(
                classify_html_document(html, "https://example.test/").is_err(),
                "fixture: {html}"
            );
        }
    }
}
