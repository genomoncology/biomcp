//! Bounded extraction of supplement link facts from JATS documents.

use roxmltree::Node;

use crate::xml::{ARTICLE_XML_NODE_LIMIT, parse_external_xml};

use super::{find_child, inline_text};
use crate::transform::article::{ArticleDocumentUnusable, ArticleSupplementLink};

pub(crate) fn extract_jats_supplement_links(
    xml: &str,
) -> Result<Vec<ArticleSupplementLink>, ArticleDocumentUnusable> {
    let doc = parse_external_xml(xml, ARTICLE_XML_NODE_LIMIT)
        .map_err(|_| ArticleDocumentUnusable::Malformed)?;
    if !doc.root_element().has_tag_name("article") {
        return Err(ArticleDocumentUnusable::Unsupported);
    }
    let mut links = Vec::new();
    for supplement in doc
        .descendants()
        .filter(|node| node.is_element() && node.has_tag_name("supplementary-material"))
    {
        let label = find_child(supplement, "label")
            .map(inline_text)
            .filter(|value| !value.is_empty());
        if let Some(href) = xlink_href(supplement) {
            push_supplement_link(&mut links, href, label.clone(), media_type(supplement));
        }
        for media in supplement
            .descendants()
            .filter(|node| node.is_element() && node.has_tag_name("media"))
        {
            if let Some(href) = xlink_href(media) {
                push_supplement_link(&mut links, href, label.clone(), media_type(media));
            }
        }
    }
    for media in doc.descendants().filter(|node| {
        node.is_element()
            && node.has_tag_name("media")
            && !node
                .ancestors()
                .any(|ancestor| ancestor.has_tag_name("supplementary-material"))
            && [
                node.attribute("specific-use"),
                node.attribute("content-type"),
            ]
            .into_iter()
            .flatten()
            .any(|value| value.to_ascii_lowercase().contains("supp"))
    }) {
        if let Some(href) = xlink_href(media) {
            push_supplement_link(&mut links, href, None, media_type(media));
        }
    }
    links.sort_by(|left, right| left.href.cmp(&right.href));
    links.dedup_by(|left, right| left.href == right.href);
    Ok(links)
}

fn xlink_href<'a>(node: Node<'a, '_>) -> Option<&'a str> {
    const XLINK_NS: &str = "http://www.w3.org/1999/xlink";
    node.attribute((XLINK_NS, "href"))
        .or_else(|| node.attribute("href"))
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn media_type(node: Node<'_, '_>) -> Option<String> {
    if let Some(value) = node
        .attribute("mime-type")
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Some(value.to_string());
    }
    let major = node.attribute("mimetype")?.trim();
    let minor = node.attribute("mime-subtype")?.trim();
    (!major.is_empty() && !minor.is_empty()).then(|| format!("{major}/{minor}"))
}

fn push_supplement_link(
    links: &mut Vec<ArticleSupplementLink>,
    href: &str,
    label: Option<String>,
    media_type: Option<String>,
) {
    let identity = href.split(['?', '#']).next().unwrap_or_default();
    let Some(filename) = identity
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    links.push(ArticleSupplementLink {
        href: href.to_string(),
        filename: filename.to_string(),
        label,
        media_type,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_nested_and_standalone_supplement_media_with_typed_facts() {
        let xml = r#"<article xmlns:xlink="http://www.w3.org/1999/xlink"><body>
          <supplementary-material><label>Data S1</label>
            <media xlink:href="folder/data-s1.csv?token=hidden" mimetype="text" mime-subtype="csv"/>
          </supplementary-material>
          <media content-type="supplement" xlink:href="standalone.xlsx" mime-type="application/xlsx"/>
          <media xlink:href="ordinary.pdf"/>
        </body></article>"#;

        let links = extract_jats_supplement_links(xml).expect("bounded JATS fixture");
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].filename, "data-s1.csv");
        assert_eq!(links[0].label.as_deref(), Some("Data S1"));
        assert_eq!(links[0].media_type.as_deref(), Some("text/csv"));
        assert_eq!(links[1].filename, "standalone.xlsx");
    }
}
