use std::collections::BTreeMap;

use crate::xml::{ARTICLE_XML_NODE_LIMIT, parse_external_xml};

use super::node_text;

mod construction;
mod parsing;

const REACTIONS_REQUEST_FIXTURE: &str =
    include_str!("../../../../spec/fixtures/vaers/reactions-request.xml");
const SERIOUS_REQUEST_FIXTURE: &str =
    include_str!("../../../../spec/fixtures/vaers/serious-request.xml");
const AGE_REQUEST_FIXTURE: &str = include_str!("../../../../spec/fixtures/vaers/age-request.xml");
const REACTIONS_RESPONSE_FIXTURE: &str =
    include_str!("../../../../spec/fixtures/vaers/reactions-response.xml");
const SERIOUS_RESPONSE_FIXTURE: &str =
    include_str!("../../../../spec/fixtures/vaers/serious-response.xml");
const AGE_RESPONSE_FIXTURE: &str = include_str!("../../../../spec/fixtures/vaers/age-response.xml");

fn parameter_map(xml: &str) -> BTreeMap<String, String> {
    let doc =
        parse_external_xml(xml, ARTICLE_XML_NODE_LIMIT).expect("request fixture should parse");
    doc.descendants()
        .filter(|node| node.has_tag_name("parameter"))
        .filter_map(|parameter| {
            let name = parameter
                .children()
                .find(|node| node.has_tag_name("name"))
                .and_then(node_text)?;
            let value = parameter
                .children()
                .find(|node| node.has_tag_name("value"))
                .map(|node| node.text().unwrap_or_default().to_string())
                .unwrap_or_default();
            Some((name, value))
        })
        .collect()
}
