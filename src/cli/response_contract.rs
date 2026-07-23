//! Command-owned JSON collection contracts and structured-error finalization.

use super::{CommandOutcome, Commands, GetEntity, OutputStream, SearchEntity};

type JsonPath = &'static [&'static str];

const RESULTS_PATH: JsonPath = &["results"];
const ITEMS_PATH: JsonPath = &["items"];
const BUCKETS_PATH: JsonPath = &["buckets"];
const EDGES_PATH: JsonPath = &["edges"];
const RECOMMENDATIONS_PATH: JsonPath = &["recommendations"];
const INTERACTIONS_PATH: JsonPath = &["interactions"];
const STRUCTURES_PATH: JsonPath = &["structures"];
const PATHWAYS_PATH: JsonPath = &["pathways"];
const ASSETS_PATH: JsonPath = &["assets"];
const DOCUMENTS_PATH: JsonPath = &["documents"];
const CONCEPTS_PATH: JsonPath = &["concepts"];
const DRUG_US_RESULTS_PATH: JsonPath = &["regions", "us", "results"];
const DRUG_EU_RESULTS_PATH: JsonPath = &["regions", "eu", "results"];
const DRUG_WHO_RESULTS_PATH: JsonPath = &["regions", "who", "results"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct JsonResponseContract {
    collection_paths: &'static [JsonPath],
}

fn sections_request_json(sections: &[String]) -> bool {
    sections.iter().any(|section| {
        matches!(
            section.trim().to_ascii_lowercase().as_str(),
            "--json" | "-j"
        )
    })
}

fn sections_contain(sections: &[String], expected: &str) -> bool {
    sections
        .iter()
        .any(|section| section.trim().eq_ignore_ascii_case(expected))
}

pub(super) fn command_requests_json(command: &Commands) -> bool {
    match command {
        Commands::Get { entity } => match entity {
            GetEntity::Author(_) => false,
            GetEntity::Gene(args) => sections_request_json(&args.sections),
            GetEntity::Article(args) => sections_request_json(&args.sections),
            GetEntity::Disease(args) => sections_request_json(&args.args),
            GetEntity::Diagnostic(args) => sections_request_json(&args.sections),
            GetEntity::Pgx(args) => sections_request_json(&args.sections),
            GetEntity::Trial(args) => sections_request_json(&args.sections),
            GetEntity::Variant(args) => sections_request_json(&args.sections),
            GetEntity::Drug(args) => sections_request_json(&args.args),
            GetEntity::Pathway(args) => sections_request_json(&args.sections),
            GetEntity::Protein(args) => sections_request_json(&args.sections),
            GetEntity::AdverseEvent(args) => sections_request_json(&args.sections),
        },
        _ => false,
    }
}

impl JsonResponseContract {
    const NONE: Self = Self {
        collection_paths: &[],
    };
    const RESULTS: Self = Self {
        collection_paths: &[RESULTS_PATH],
    };

    pub(super) fn for_command(command: &Commands) -> Self {
        match command {
            Commands::Search { entity } => Self::for_search(entity),
            Commands::Get {
                entity: GetEntity::Article(args),
            } if sections_contain(&args.sections, "assets") => Self {
                collection_paths: &[ASSETS_PATH],
            },
            Commands::Get {
                entity: GetEntity::Trial(args),
            } if sections_contain(&args.sections, "documents") => Self {
                collection_paths: &[DOCUMENTS_PATH],
            },
            Commands::Get { .. } => Self::NONE,
            Commands::Variant { cmd } => match cmd {
                super::VariantCommand::Articles { input: Some(_), .. }
                | super::VariantCommand::Erepo { .. } => Self {
                    collection_paths: &[ITEMS_PATH],
                },
                super::VariantCommand::Trials { .. }
                | super::VariantCommand::Articles { .. }
                | super::VariantCommand::Normalize { .. } => Self::RESULTS,
                super::VariantCommand::Structure { .. }
                | super::VariantCommand::Oncokb { .. }
                | super::VariantCommand::External(_) => Self::NONE,
            },
            Commands::Drug { cmd } => match cmd {
                super::DrugCommand::Trials { .. } => Self::RESULTS,
                super::DrugCommand::AdverseEvents { count, .. } if count.is_some() => Self {
                    collection_paths: &[BUCKETS_PATH],
                },
                super::DrugCommand::AdverseEvents { .. } => Self::RESULTS,
                super::DrugCommand::Interactions { .. } => Self {
                    collection_paths: &[INTERACTIONS_PATH],
                },
                super::DrugCommand::External(_) => Self::NONE,
            },
            Commands::Disease { .. } | Commands::Pathway { .. } => Self::RESULTS,
            Commands::Article { cmd } => match cmd {
                super::article::ArticleCommand::Citations { .. }
                | super::article::ArticleCommand::References { .. } => Self {
                    collection_paths: &[EDGES_PATH],
                },
                super::article::ArticleCommand::Recommendations { .. } => Self {
                    collection_paths: &[RECOMMENDATIONS_PATH],
                },
                super::article::ArticleCommand::Entities { .. }
                | super::article::ArticleCommand::Batch { .. } => Self::NONE,
            },
            Commands::Gene { cmd } => match cmd {
                super::GeneCommand::Trials { .. }
                | super::GeneCommand::Drugs { .. }
                | super::GeneCommand::Articles { .. } => Self::RESULTS,
                super::GeneCommand::Pathways { .. } => Self {
                    collection_paths: &[PATHWAYS_PATH],
                },
                super::GeneCommand::Definition { .. } | super::GeneCommand::External(_) => {
                    Self::NONE
                }
            },
            Commands::Protein {
                cmd: super::ProteinCommand::Structures { .. },
            } => Self {
                collection_paths: &[STRUCTURES_PATH],
            },
            Commands::Enrich(_) => Self::RESULTS,
            Commands::Study { .. }
            | Commands::Health(_)
            | Commands::Cache { .. }
            | Commands::Ema { .. }
            | Commands::Who { .. }
            | Commands::Cvx { .. }
            | Commands::Ddinter { .. }
            | Commands::Gtr { .. }
            | Commands::WhoIvd { .. }
            | Commands::Mcp
            | Commands::Serve
            | Commands::McpConfig(_)
            | Commands::ServeHttp(_)
            | Commands::ServeSse
            | Commands::Skill { .. }
            | Commands::Chart { .. }
            | Commands::Update(_)
            | Commands::Uninstall
            | Commands::List(_)
            | Commands::Batch(_)
            | Commands::Version(_) => Self::NONE,
            Commands::Discover(_) => Self {
                collection_paths: &[CONCEPTS_PATH],
            },
        }
    }

    fn for_search(entity: &SearchEntity) -> Self {
        match entity {
            SearchEntity::All(_) | SearchEntity::Author(_) => Self::NONE,
            SearchEntity::Trial(args) if args.count_only => Self::NONE,
            SearchEntity::Drug(args) => {
                let structured = args.target.is_some()
                    || args.indication.is_some()
                    || args.mechanism.is_some()
                    || args.drug_type.is_some()
                    || args.atc.is_some()
                    || args.pharm_class.is_some()
                    || args.interactions.is_some();
                match (args.region, structured) {
                    (Some(super::DrugRegionArg::Us), _) | (None, true) => Self {
                        collection_paths: &[DRUG_US_RESULTS_PATH],
                    },
                    (Some(super::DrugRegionArg::Eu), _) => Self {
                        collection_paths: &[DRUG_EU_RESULTS_PATH],
                    },
                    (Some(super::DrugRegionArg::Who), _) => Self {
                        collection_paths: &[DRUG_WHO_RESULTS_PATH],
                    },
                    (Some(super::DrugRegionArg::All), _) | (None, false) => Self {
                        collection_paths: &[
                            DRUG_US_RESULTS_PATH,
                            DRUG_EU_RESULTS_PATH,
                            DRUG_WHO_RESULTS_PATH,
                        ],
                    },
                }
            }
            SearchEntity::AdverseEvent(args) if args.count.is_some() => Self {
                collection_paths: &[BUCKETS_PATH],
            },
            SearchEntity::AdverseEvent(args) if args.source.eq_ignore_ascii_case("vaers") => {
                Self::NONE
            }
            SearchEntity::Gene(_)
            | SearchEntity::Disease(_)
            | SearchEntity::Diagnostic(_)
            | SearchEntity::Pgx(_)
            | SearchEntity::Phenotype(_)
            | SearchEntity::Gwas(_)
            | SearchEntity::Article(_)
            | SearchEntity::Trial(_)
            | SearchEntity::Variant(_)
            | SearchEntity::Pathway(_)
            | SearchEntity::Protein(_)
            | SearchEntity::AdverseEvent(_) => Self::RESULTS,
        }
    }
}

fn insert_empty_collection(value: &mut serde_json::Value, path: JsonPath) {
    let Some((segment, rest)) = path.split_first() else {
        return;
    };
    let Some(object) = value.as_object_mut() else {
        return;
    };
    if rest.is_empty() {
        object
            .entry((*segment).to_string())
            .or_insert_with(|| serde_json::Value::Array(Vec::new()));
        return;
    }
    let child = object
        .entry((*segment).to_string())
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
    insert_empty_collection(child, rest);
}

pub(super) fn finalize_structured_error(
    mut outcome: CommandOutcome,
    contract: JsonResponseContract,
) -> CommandOutcome {
    if outcome.exit_code == 0
        || outcome.stream != OutputStream::Stdout
        || outcome.bytes.is_some()
        || contract.collection_paths.is_empty()
    {
        return outcome;
    }
    let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&outcome.text) else {
        return outcome;
    };
    if !value.is_object() {
        return outcome;
    }
    let had_batch_items = value.get("items").is_some();
    for path in contract.collection_paths {
        insert_empty_collection(&mut value, path);
    }
    if contract.collection_paths == [ITEMS_PATH]
        && !had_batch_items
        && let Some(object) = value.as_object_mut()
    {
        object.insert("complete".into(), serde_json::Value::Bool(false));
        object.insert("truncated".into(), serde_json::Value::Bool(false));
        object.insert("_meta".into(), serde_json::json!({"next_commands": []}));
    }
    if let Ok(text) = crate::render::json::to_pretty(&value) {
        outcome.text = text;
    }
    outcome
}
#[cfg(test)]
mod tests {
    use super::{ITEMS_PATH, JsonResponseContract, finalize_structured_error};
    use crate::cli::{CommandOutcome, OutputStream};

    fn contract_paths(args: &[&str]) -> Vec<String> {
        let cli = crate::cli::try_parse_cli(args).expect("command should parse");
        JsonResponseContract::for_command(&cli.command)
            .collection_paths
            .iter()
            .map(|path| path.join("."))
            .collect()
    }

    #[test]
    fn command_collection_contract_inventory() {
        let rows: &[(&[&str], &[&str])] = &[
            (&["biomcp", "search", "author", "-q", "A. Butte"], &[]),
            (&["biomcp", "search", "gene", "BRAF"], &["results"]),
            (&["biomcp", "search", "disease", "melanoma"], &["results"]),
            (
                &["biomcp", "search", "diagnostic", "--gene", "BRAF"],
                &["results"],
            ),
            (&["biomcp", "search", "pgx", "-g", "CYP2D6"], &["results"]),
            (&["biomcp", "search", "phenotype", "seizure"], &["results"]),
            (&["biomcp", "search", "gwas", "-g", "BRAF"], &["results"]),
            (&["biomcp", "search", "article", "BRAF"], &["results"]),
            (
                &["biomcp", "search", "trial", "-c", "melanoma"],
                &["results"],
            ),
            (&["biomcp", "search", "variant", "BRAF"], &["results"]),
            (&["biomcp", "search", "pathway", "cancer"], &["results"]),
            (&["biomcp", "search", "protein", "BRAF"], &["results"]),
            (
                &["biomcp", "search", "adverse-event", "-d", "aspirin"],
                &["results"],
            ),
            (
                &[
                    "biomcp",
                    "search",
                    "adverse-event",
                    "-d",
                    "aspirin",
                    "--count",
                    "reaction",
                ],
                &["buckets"],
            ),
            (&["biomcp", "variant", "trials", "BRAF"], &["results"]),
            (&["biomcp", "variant", "articles", "BRAF"], &["results"]),
            (
                &["biomcp", "variant", "normalize", "all", "NM_1:c.1A>T"],
                &["results"],
            ),
            (&["biomcp", "gene", "trials", "BRAF"], &["results"]),
            (&["biomcp", "gene", "drugs", "BRAF"], &["results"]),
            (&["biomcp", "gene", "articles", "BRAF"], &["results"]),
            (&["biomcp", "gene", "pathways", "BRAF"], &["pathways"]),
            (&["biomcp", "disease", "trials", "melanoma"], &["results"]),
            (&["biomcp", "disease", "articles", "melanoma"], &["results"]),
            (&["biomcp", "disease", "drugs", "melanoma"], &["results"]),
            (&["biomcp", "pathway", "trials", "R-HSA-1"], &["results"]),
            (&["biomcp", "pathway", "articles", "R-HSA-1"], &["results"]),
            (&["biomcp", "pathway", "drugs", "R-HSA-1"], &["results"]),
            (&["biomcp", "drug", "trials", "aspirin"], &["results"]),
            (
                &["biomcp", "drug", "adverse-events", "aspirin"],
                &["results"],
            ),
            (
                &[
                    "biomcp",
                    "drug",
                    "adverse-events",
                    "aspirin",
                    "--count",
                    "reaction",
                ],
                &["buckets"],
            ),
            (
                &["biomcp", "drug", "interactions", "aspirin"],
                &["interactions"],
            ),
            (&["biomcp", "article", "citations", "1"], &["edges"]),
            (&["biomcp", "article", "references", "1"], &["edges"]),
            (
                &["biomcp", "article", "recommendations", "1"],
                &["recommendations"],
            ),
            (
                &["biomcp", "protein", "structures", "P15056"],
                &["structures"],
            ),
            (&["biomcp", "enrich", "BRAF,TP53"], &["results"]),
            (&["biomcp", "discover", "melanoma"], &["concepts"]),
            (
                &["biomcp", "get", "article", "1", "assets", "--json"],
                &["assets"],
            ),
            (
                &["biomcp", "get", "trial", "NCT1", "documents", "-j"],
                &["documents"],
            ),
        ];

        for (args, expected) in rows {
            assert_eq!(contract_paths(args), *expected, "args={args:?}");
        }
    }

    #[test]
    fn argument_dependent_and_keyless_contract_inventory() {
        let rows: &[(&[&str], &[&str])] = &[
            (
                &["biomcp", "search", "drug", "aspirin"],
                &[
                    "regions.us.results",
                    "regions.eu.results",
                    "regions.who.results",
                ],
            ),
            (
                &["biomcp", "search", "drug", "--target", "BRAF"],
                &["regions.us.results"],
            ),
            (
                &["biomcp", "search", "drug", "aspirin", "--region", "us"],
                &["regions.us.results"],
            ),
            (
                &["biomcp", "search", "drug", "aspirin", "--region", "eu"],
                &["regions.eu.results"],
            ),
            (
                &["biomcp", "search", "drug", "aspirin", "--region", "who"],
                &["regions.who.results"],
            ),
            (&["biomcp", "search", "all", "--gene", "BRAF"], &[]),
            (
                &[
                    "biomcp",
                    "search",
                    "trial",
                    "-c",
                    "melanoma",
                    "--count-only",
                ],
                &[],
            ),
            (
                &[
                    "biomcp",
                    "search",
                    "adverse-event",
                    "MMR",
                    "--source",
                    "vaers",
                ],
                &[],
            ),
            (
                &[
                    "biomcp",
                    "search",
                    "adverse-event",
                    "MMR",
                    "--source",
                    "vaers",
                    "--type",
                    "recall",
                ],
                &[],
            ),
            (&["biomcp", "article", "batch", "1"], &[]),
            (
                &["biomcp", "variant", "articles", "--input", "variants.json"],
                &["items"],
            ),
            (&["biomcp", "batch", "gene", "BRAF"], &[]),
            (&["biomcp", "article", "entities", "1"], &[]),
            (&["biomcp", "variant", "structure", "BRAF"], &[]),
            (&["biomcp", "gene", "definition", "BRAF"], &[]),
            (&["biomcp", "get", "author", "semanticscholar:1"], &[]),
            (&["biomcp", "get", "gene", "BRAF"], &[]),
        ];

        for (args, expected) in rows {
            assert_eq!(contract_paths(args), *expected, "args={args:?}");
        }
    }

    #[test]
    fn structured_error_finalizer_adds_paths_without_overwriting_values() {
        let contract = JsonResponseContract {
            collection_paths: &[
                &["results"],
                &["regions", "us", "results"],
                &["regions", "eu", "results"],
            ],
        };
        let outcome = CommandOutcome::stdout_with_exit(
            r#"{"error":{"code":"api"},"_meta":{},"results":["keep"],"regions":{"us":{"results":null}}}"#.to_string(),
            1,
        );
        let finalized = finalize_structured_error(outcome, contract);
        let value: serde_json::Value = serde_json::from_str(&finalized.text).expect("valid JSON");

        assert_eq!(value["results"], serde_json::json!(["keep"]));
        assert!(value["regions"]["us"]["results"].is_null());
        assert_eq!(value["regions"]["eu"]["results"], serde_json::json!([]));
        assert_eq!(value["error"]["code"], "api");
        assert!(value["_meta"].is_object());
        assert_eq!(finalized.exit_code, 1);
        assert_eq!(finalized.stream, OutputStream::Stdout);
    }

    #[test]
    fn variant_article_batch_errors_keep_the_stable_envelope() {
        let outcome = CommandOutcome::stdout_with_exit(
            r#"{"error":{"code":"invalid_argument","message":"bad input"}}"#.into(),
            2,
        );
        let finalized = finalize_structured_error(
            outcome,
            JsonResponseContract {
                collection_paths: &[ITEMS_PATH],
            },
        );
        let value: serde_json::Value = serde_json::from_str(&finalized.text).expect("valid JSON");

        assert_eq!(value["items"], serde_json::json!([]));
        assert_eq!(value["complete"], false);
        assert_eq!(value["truncated"], false);
        assert_eq!(value["_meta"]["next_commands"], serde_json::json!([]));
    }

    #[test]
    fn variant_article_item_failures_preserve_aggregate_state_and_followups() {
        let text = r#"{"items":[{"error":{"code":"source_unavailable"}}],"complete":false,"truncated":true,"_meta":{"next_commands":["biomcp get article 1"]}}"#;
        let finalized = finalize_structured_error(
            CommandOutcome::stdout_with_exit(text.into(), 1),
            JsonResponseContract {
                collection_paths: &[ITEMS_PATH],
            },
        );
        let value: serde_json::Value = serde_json::from_str(&finalized.text).expect("valid JSON");

        assert_eq!(value["truncated"], true);
        assert_eq!(
            value["_meta"]["next_commands"],
            serde_json::json!(["biomcp get article 1"])
        );
    }

    #[test]
    fn structured_error_finalizer_leaves_unowned_shapes_unchanged() {
        let contract = JsonResponseContract {
            collection_paths: &[&["results"]],
        };
        for text in ["not json", "[]"] {
            let outcome = CommandOutcome::stdout_with_exit(text.to_string(), 1);
            let finalized = finalize_structured_error(outcome, contract);
            assert_eq!(finalized.text, text);
        }

        let successful = CommandOutcome::stdout("{}".to_string());
        assert_eq!(finalize_structured_error(successful, contract).text, "{}");
        let stderr = CommandOutcome::stderr_with_exit("{}".to_string(), 1);
        assert_eq!(finalize_structured_error(stderr, contract).text, "{}");
    }
}
