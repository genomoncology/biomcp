use serde_json::{Map, json};

#[derive(Debug)]
pub(super) struct TypedGetCapability {
    pub(super) entity: &'static str,
    pub(super) sections: Option<Vec<&'static str>>,
    pub(super) reject_duplicate_sections: bool,
}

pub(super) fn typed_get_capabilities() -> Vec<TypedGetCapability> {
    crate::cli::list::catalog::entities()
        .into_iter()
        .filter(|entity| entity.gettable)
        .map(|entity| TypedGetCapability {
            entity: entity.name,
            sections: (entity.name != "author").then(|| {
                entity
                    .sections
                    .iter()
                    .copied()
                    .filter(|section| !(entity.name == "article" && *section == "asset"))
                    .collect()
            }),
            reject_duplicate_sections: entity.name != "adverse-event",
        })
        .collect()
}

pub(super) fn typed_get_schema(schema: &mut rmcp::schemars::Schema) {
    let branches = typed_get_capabilities()
        .into_iter().map(|capability| {
            let entity = capability.entity;
            let mut properties = Map::from_iter([
                ("entity".into(), json!({"const":entity})),
                ("id".into(), json!({"type":"string","minLength":1,"maxLength":512})),
                ("json".into(), json!({"type":"boolean","default":false})),
            ]);
            if let Some(section_names) = capability.sections {
                let mut sections = json!({"type":"array","maxItems":16,"items":{"enum":section_names}});
                if capability.reject_duplicate_sections {
                    sections["uniqueItems"] = json!(true);
                }
                properties.insert("sections".into(), sections);
            }
            if entity == "variant" {
                properties.insert("assembly".into(), json!({"enum":["grch37","hg19","grch38","hg38"]}));
            }
            json!({"type":"object","additionalProperties":false,"properties":properties,"required":["entity","id"]})
        }).collect::<Vec<_>>();
    *schema = serde_json::from_value(json!({"oneOf":branches})).expect("valid typed get schema");
}
