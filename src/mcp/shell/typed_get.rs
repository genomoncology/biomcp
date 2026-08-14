use serde_json::{Map, json};

pub(super) fn typed_get_schema(schema: &mut rmcp::schemars::Schema) {
    let branches = ["author", "gene", "article", "disease", "diagnostic", "pgx", "trial", "variant", "drug", "pathway", "protein", "adverse-event"]
        .into_iter().map(|entity| {
            let mut properties = Map::from_iter([
                ("entity".into(), json!({"const":entity})),
                ("id".into(), json!({"type":"string","minLength":1,"maxLength":512})),
                ("json".into(), json!({"type":"boolean","default":false})),
            ]);
            if entity != "author" {
                let mut sections = json!({"type":"array","maxItems":16,"items":{"enum":crate::cli::list::catalog::sections(entity)}});
                if entity != "adverse-event" {
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
