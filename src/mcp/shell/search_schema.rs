//! Existing typed search schemas, including the accepted disease consumer.
use super::{merge_branch_properties, short_string_schema, string_array_schema, trial_phase};
use rmcp::schemars;
use serde_json::{Value, json};

/// The typed search entities, in catalog order. The root schema enum,
/// the per-entity branch builder, and the argument checks share this one
/// list so they cannot drift.
pub(super) const ENTITIES: [&str; 10] = [
    "author", "gene", "disease", "pgx", "gwas", "article", "trial", "variant", "protein", "patient",
];

pub(super) fn typed_search_branch(entity: &str) -> Value {
    let (fields, required): (&[(&str, &str)], &[&str]) = match entity {
        "author" => (
            &[("query", "text"), ("source", "author_source")],
            &["query"],
        ),
        "gene" => (
            &[
                ("query", "text"),
                ("gene_type", "text"),
                ("chromosome", "text"),
                ("region", "text"),
            ],
            &["query", "gene_type", "chromosome", "region"],
        ),
        "disease" => (
            &[
                ("query", "text"),
                ("source", "disease_source"),
                ("inheritance", "text"),
                ("phenotype", "text"),
                ("onset", "text"),
                ("no_fallback", "bool"),
            ],
            &["query"],
        ),
        "pgx" => (
            &[("gene", "text"), ("drug", "text"), ("cpic_level", "cpic")],
            &["gene", "drug"],
        ),
        "gwas" => (
            &[
                ("gene", "text"),
                ("trait", "text"),
                ("p_value", "probability"),
            ],
            &["gene", "trait"],
        ),
        "article" => (
            &[
                ("keyword", "array"),
                ("gene", "text"),
                ("disease", "array"),
                ("drug", "array"),
                ("author", "array"),
                ("journal", "array"),
                ("date_from", "date"),
                ("date_to", "date"),
                ("article_type", "article_type"),
                ("source", "article_source"),
                ("open_access", "bool"),
                ("no_preprints", "bool"),
                ("sort", "sort"),
            ],
            &["keyword", "gene", "disease", "drug", "author"],
        ),
        "trial" => (
            &[
                ("condition", "array"),
                ("intervention", "array"),
                ("mutation", "array"),
                ("criteria", "array"),
                ("biomarker", "array"),
                ("phase", "phase"),
                ("status", "status"),
                ("source", "trial_source"),
                ("age", "age"),
                ("count_only", "bool"),
            ],
            &[
                "condition",
                "intervention",
                "mutation",
                "criteria",
                "biomarker",
            ],
        ),
        "variant" => (
            &[
                ("query", "text"),
                ("gene", "text"),
                ("hgvsp", "text"),
                ("significance", "text"),
                ("max_frequency", "unit"),
                ("consequence", "text"),
                ("review_status", "review"),
                ("revel_min", "unit"),
            ],
            &["query", "gene", "hgvsp"],
        ),
        "protein" => (
            &[
                ("query", "text"),
                ("all_species", "bool"),
                ("reviewed", "bool"),
                ("disease", "text"),
                ("existence", "existence"),
            ],
            &["query"],
        ),
        // Patient search takes no filters until it ships; the CLI refuses it.
        "patient" => (&[], &[]),
        _ => unreachable!(),
    };
    let mut properties = serde_json::Map::from_iter([
        ("entity".into(), json!({"const":entity})),
        (
            "limit".into(),
            json!({"type":"integer","minimum":1,"maximum":25,"default":10}),
        ),
        (
            "offset".into(),
            json!({"type":"integer","minimum":0,"maximum":1000,"default":0}),
        ),
        ("json".into(), json!({"type":"boolean","default":false})),
    ]);
    if entity == "patient" {
        properties.remove("limit");
        properties.remove("offset");
    }
    for &(name, kind) in fields {
        let value = match kind {
            "array" => string_array_schema(),
            "bool" => json!({"type":"boolean"}),
            "probability" => json!({"type":"number","exclusiveMinimum":0,"maximum":1}),
            "unit" => json!({"type":"number","minimum":0,"maximum":1}),
            "age" => json!({"type":"number","minimum":0,"maximum":150}),
            "existence" => json!({"type":"integer","minimum":1,"maximum":5}),
            "date" => json!({"type":"string","pattern":"^[0-9]{4}(-[0-9]{2}(-[0-9]{2})?)?$"}),
            "author_source" => json!({"const":"semanticscholar"}),
            "cpic" => json!({"enum":["A","B","C","D"]}),
            "article_type" => {
                json!({"enum":["research-article","review","case-reports","meta-analysis"]})
            }
            "article_source" => {
                json!({"enum":["all","pubtator","europepmc","pubmed","semanticscholar","litsense2"]})
            }
            "sort" => json!({"enum":["date","citations","relevance"]}),
            "phase" => trial_phase::schema(),
            "status" => {
                json!({"enum":["recruiting","not_yet_recruiting","enrolling_by_invitation","active_not_recruiting","completed","suspended","terminated","withdrawn"]})
            }
            "trial_source" => json!({"enum":["ctgov","nci"]}),
            "disease_source" => json!({"enum":["mondo","doid","mesh"]}),
            "review" => {
                json!({"enum":["0","1","2","3","4","none","criteria_provided","expert_panel"]})
            }
            _ => short_string_schema(),
        };
        properties.insert(name.into(), value);
    }
    let mut branch = json!({"type":"object","additionalProperties":false,"properties":properties,"required":["entity"]});
    if !required.is_empty() {
        let any_of = required
            .iter()
            .map(|name| json!({"required":[name]}))
            .collect::<Vec<_>>();
        branch["anyOf"] = Value::Array(any_of);
    }
    branch
}

pub(super) fn typed_search_schema(schema: &mut schemars::Schema) {
    // Flat root for OpenAI/Gemini function calling, which reject top-level
    // oneOf: the entity enum plus the union of every branch's properties.
    // The body stays prescriptive (ADR 0002); the root is descriptive.
    let branches = ENTITIES
        .iter()
        .map(|entity| typed_search_branch(entity))
        .collect::<Vec<_>>();
    let mut properties = merge_branch_properties(&branches);
    properties.insert("entity".into(), json!({"type":"string","enum":ENTITIES}));
    *schema = serde_json::from_value(json!({
        "type":"object",
        "additionalProperties":false,
        "properties":properties,
        "required":["entity"]
    }))
    .expect("valid typed search schema");
}
