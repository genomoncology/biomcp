use clap::{CommandFactory, Parser};

use super::dispatch::{
    LocationPaginationMeta, paginate_trial_locations, parse_trial_location_paging,
    should_show_trial_zero_result_nickname_hint, trial_locations_json, trial_search_query_summary,
};
use crate::cli::test_support::{
    Mock, MockServer, ResponseTemplate, lock_env, method, path, query_param, set_env_var,
};
use crate::cli::{Cli, Commands, GetEntity, SearchEntity};

async fn mount_trial_alias_lookup(
    server: &MockServer,
    requested: &str,
    canonical: &str,
    aliases: &[&str],
) {
    Mock::given(method("GET"))
        .and(path("/v1/query"))
        .and(query_param("q", requested))
        .and(query_param("size", "25"))
        .and(query_param("from", "0"))
        .and(query_param(
            "fields",
            crate::sources::mychem::MYCHEM_FIELDS_GET,
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "total": 1,
            "hits": [{
                "_id": "drug-test-id",
                "_score": 42.0,
                "drugbank": {
                    "id": "DBTEST",
                    "name": canonical,
                    "synonyms": aliases,
                }
            }]
        })))
        .expect(1)
        .mount(server)
        .await;
}

fn render_trial_search_long_help() -> String {
    let mut command = Cli::command();
    let search = command
        .find_subcommand_mut("search")
        .expect("search subcommand should exist");
    let trial = search
        .find_subcommand_mut("trial")
        .expect("trial subcommand should exist");
    let mut help = Vec::new();
    trial
        .write_long_help(&mut help)
        .expect("trial help should render");
    String::from_utf8(help).expect("help should be utf-8")
}

#[test]
fn trial_facility_help_names_text_search_and_geo_verify_modes() {
    let help = render_trial_search_long_help();

    assert!(help.contains("text-search mode"));
    assert!(help.contains("geo-verify mode"));
    assert!(help.contains("materially more expensive"));
}

#[test]
fn trial_phase_help_explains_combined_phase_label() {
    let help = render_trial_search_long_help();

    assert!(help.contains("1/2"));
    assert!(help.contains("combined Phase 1/Phase 2 label"));
    assert!(help.contains("not Phase 1 OR Phase 2"));
}

#[test]
fn trial_sex_help_explains_all_means_no_restriction() {
    let help = render_trial_search_long_help();

    assert!(help.contains("all"));
    assert!(help.contains("no sex restriction"));
}

#[test]
fn trial_phase_help_explains_canonical_numeric_forms_and_aliases() {
    let help = render_trial_search_long_help();

    assert!(help.contains("Canonical CLI forms: NA, 1, 1/2, 2, 3, 4."));
    assert!(help.contains("Accepted aliases: EARLY_PHASE1, PHASE1, PHASE2, PHASE3, PHASE4."));
}

#[test]
fn trial_help_documents_nci_source_specific_notes() {
    let help = render_trial_search_long_help();

    assert!(help.contains("Source-specific notes"));
    assert!(help.contains("grounds to an NCI disease ID when available"));
    assert!(help.contains("one mapped status at a time"));
    assert!(help.contains("I_II"));
    assert!(help.contains("early_phase1"));
    assert!(help.contains("sites.org_coordinates"));
    assert!(help.contains("no separate NCI keyword flag"));
}

#[test]
fn trial_help_documents_alias_expansion_controls() {
    let help = render_trial_search_long_help();

    assert!(help.contains("auto-expands known aliases"));
    assert!(help.contains("--no-alias-expand"));
    assert!(help.contains("matched_intervention_label"));
    assert!(help.contains("Matched Intervention"));
    assert!(help.contains("use `--offset` or `--no-alias-expand`"));
}

#[test]
fn trial_age_help_explains_age_only_count_is_approximate() {
    let help = render_trial_search_long_help();

    assert!(help.contains("age-only CTGov searches report an approximate upstream total"));
}

#[test]
fn search_trial_parses_positional_query() {
    let cli = Cli::try_parse_from(["biomcp", "search", "trial", "melanoma", "--limit", "2"])
        .expect("search trial should parse");

    let Cli {
        command:
            Commands::Search {
                entity:
                    SearchEntity::Trial(crate::cli::trial::TrialSearchArgs {
                        condition,
                        positional_query,
                        intervention,
                        no_alias_expand,
                        facility,
                        phase,
                        study_type,
                        age,
                        sex,
                        status,
                        mutation,
                        criteria,
                        biomarker,
                        prior_therapies,
                        progression_on,
                        line_of_therapy,
                        sponsor,
                        sponsor_type,
                        date_from,
                        date_to,
                        lat,
                        lon,
                        distance,
                        results_available,
                        count_only,
                        source,
                        offset,
                        next_page,
                        limit,
                    }),
            },
        ..
    } = cli
    else {
        panic!("expected search trial command");
    };

    assert!(condition.is_empty());
    assert_eq!(positional_query.as_deref(), Some("melanoma"));
    assert!(intervention.is_empty());
    assert!(!no_alias_expand);
    assert!(facility.is_empty());
    assert_eq!(phase, None);
    assert_eq!(study_type, None);
    assert_eq!(age, None);
    assert_eq!(sex, None);
    assert_eq!(status, None);
    assert!(mutation.is_empty());
    assert!(criteria.is_empty());
    assert!(biomarker.is_empty());
    assert!(prior_therapies.is_empty());
    assert!(progression_on.is_empty());
    assert_eq!(line_of_therapy, None);
    assert!(sponsor.is_empty());
    assert_eq!(sponsor_type, None);
    assert_eq!(date_from, None);
    assert_eq!(date_to, None);
    assert_eq!(lat, None);
    assert_eq!(lon, None);
    assert_eq!(distance, None);
    assert!(!results_available);
    assert!(!count_only);
    assert_eq!(source, "ctgov");
    assert_eq!(offset, 0);
    assert_eq!(next_page, None);
    assert_eq!(limit, 2);
}

#[test]
fn search_trial_parses_no_alias_expand() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "search",
        "trial",
        "--intervention",
        "daraxonrasib",
        "--no-alias-expand",
    ])
    .expect("search trial should parse");

    let Cli {
        command:
            Commands::Search {
                entity:
                    SearchEntity::Trial(crate::cli::trial::TrialSearchArgs {
                        intervention,
                        no_alias_expand,
                        ..
                    }),
            },
        ..
    } = cli
    else {
        panic!("expected search trial command");
    };

    assert_eq!(intervention, vec!["daraxonrasib".to_string()]);
    assert!(no_alias_expand);
}

#[test]
fn search_trial_parses_multi_word_positional_query() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "search",
        "trial",
        "endometrial cancer",
        "--status",
        "recruiting",
    ])
    .expect("search trial should parse");

    let Cli {
        command:
            Commands::Search {
                entity:
                    SearchEntity::Trial(crate::cli::trial::TrialSearchArgs {
                        positional_query,
                        status,
                        ..
                    }),
            },
        ..
    } = cli
    else {
        panic!("expected search trial command");
    };

    assert_eq!(positional_query.as_deref(), Some("endometrial cancer"));
    assert_eq!(status.as_deref(), Some("recruiting"));
}

#[test]
fn search_trial_parses_positional_query_with_status_flag() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "search",
        "trial",
        "melanoma",
        "--status",
        "recruiting",
        "--limit",
        "2",
    ])
    .expect("search trial should parse");

    let Cli {
        command:
            Commands::Search {
                entity:
                    SearchEntity::Trial(crate::cli::trial::TrialSearchArgs {
                        positional_query,
                        status,
                        limit,
                        ..
                    }),
            },
        ..
    } = cli
    else {
        panic!("expected search trial command");
    };

    assert_eq!(positional_query.as_deref(), Some("melanoma"));
    assert_eq!(status.as_deref(), Some("recruiting"));
    assert_eq!(limit, 2);
}

#[test]
fn search_trial_parses_new_filter_flags() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "search",
        "trial",
        "--age",
        "0.5",
        "--study-type",
        "interventional",
        "--has-results",
    ])
    .expect("search trial should parse");

    let Cli {
        command:
            Commands::Search {
                entity:
                    SearchEntity::Trial(crate::cli::trial::TrialSearchArgs {
                        age,
                        study_type,
                        results_available,
                        ..
                    }),
            },
        ..
    } = cli
    else {
        panic!("expected search trial command");
    };

    assert_eq!(age, Some(0.5));
    assert_eq!(study_type.as_deref(), Some("interventional"));
    assert!(results_available);
}

#[test]
fn search_trial_rejects_non_numeric_age() {
    let err = Cli::try_parse_from(["biomcp", "search", "trial", "--age", "abc", "--count-only"])
        .expect_err("invalid age should fail");
    let rendered = err.to_string();

    assert!(rendered.contains("invalid value 'abc' for '--age <AGE>'"));
    assert!(rendered.contains("invalid float literal"));
}

#[test]
fn search_trial_parses_unquoted_multi_token_mutation() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "search",
        "trial",
        "--mutation",
        "BRAF",
        "V600E",
        "--limit",
        "5",
    ])
    .expect("search trial should parse");

    let Cli {
        command:
            Commands::Search {
                entity:
                    SearchEntity::Trial(crate::cli::trial::TrialSearchArgs {
                        mutation, limit, ..
                    }),
            },
        ..
    } = cli
    else {
        panic!("expected search trial command");
    };

    assert_eq!(mutation, vec!["BRAF".to_string(), "V600E".to_string()]);
    assert_eq!(limit, 5);
}

#[test]
fn get_trial_parses_source_before_sections() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "get",
        "trial",
        "NCT02576665",
        "--source",
        "ctgov",
        "eligibility",
    ])
    .expect("get trial should parse");

    let Cli {
        command:
            Commands::Get {
                entity:
                    GetEntity::Trial(crate::cli::trial::TrialGetArgs {
                        nct_id,
                        sections,
                        source,
                    }),
            },
        ..
    } = cli
    else {
        panic!("expected get trial command");
    };

    assert_eq!(nct_id, "NCT02576665");
    assert_eq!(sections, vec!["eligibility".to_string()]);
    assert_eq!(source, "ctgov");
}

#[tokio::test]
async fn handle_search_rejects_next_page_with_offset() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "search",
        "trial",
        "melanoma",
        "--next-page",
        "page-2",
        "--offset",
        "1",
    ])
    .expect("search trial should parse");

    let Cli {
        command: Commands::Search {
            entity: SearchEntity::Trial(args),
        },
        json,
        ..
    } = cli
    else {
        panic!("expected trial search command");
    };

    let err = super::handle_search(args, json)
        .await
        .expect_err("next-page plus offset should fail fast");
    assert!(
        err.to_string()
            .contains("--next-page cannot be used together with --offset")
    );
}

#[tokio::test]
async fn handle_search_rejects_next_page_when_alias_expansion_uses_multiple_queries() {
    let _env_lock = lock_env().await;
    let requested = "review-cli-next-page";

    let mychem = MockServer::start().await;
    mount_trial_alias_lookup(&mychem, requested, requested, &["review-cli-alt"]).await;
    let mychem_base = format!("{}/v1", mychem.uri());
    let _mychem_env = set_env_var("BIOMCP_MYCHEM_BASE", Some(&mychem_base));

    let cli = Cli::try_parse_from([
        "biomcp",
        "search",
        "trial",
        "--intervention",
        requested,
        "--next-page",
        "page-2",
    ])
    .expect("search trial should parse");

    let Cli {
        command: Commands::Search {
            entity: SearchEntity::Trial(args),
        },
        json,
        ..
    } = cli
    else {
        panic!("expected trial search command");
    };

    let err = super::handle_search(args, json)
        .await
        .expect_err("multi-alias search should reject next-page");
    assert!(err.to_string().contains("--next-page is not supported"));
    assert!(err.to_string().contains("--no-alias-expand"));
}

#[tokio::test]
async fn handle_search_rejects_no_alias_expand_without_intervention() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "search",
        "trial",
        "--condition",
        "melanoma",
        "--no-alias-expand",
    ])
    .expect("search trial should parse");

    let Cli {
        command: Commands::Search {
            entity: SearchEntity::Trial(args),
        },
        json,
        ..
    } = cli
    else {
        panic!("expected trial search command");
    };

    let err = super::handle_search(args, json)
        .await
        .expect_err("no-alias-expand without intervention should fail");
    assert!(
        err.to_string()
            .contains("--no-alias-expand is only supported for CTGov intervention searches")
    );
}

#[tokio::test]
async fn handle_search_rejects_no_alias_expand_for_nci_source() {
    let cli = Cli::try_parse_from([
        "biomcp",
        "search",
        "trial",
        "--intervention",
        "daraxonrasib",
        "--source",
        "nci",
        "--no-alias-expand",
    ])
    .expect("search trial should parse");

    let Cli {
        command: Commands::Search {
            entity: SearchEntity::Trial(args),
        },
        json,
        ..
    } = cli
    else {
        panic!("expected trial search command");
    };

    let err = super::handle_search(args, json)
        .await
        .expect_err("nci no-alias-expand should fail");
    assert!(
        err.to_string()
            .contains("--no-alias-expand is only supported for CTGov intervention searches")
    );
}

#[test]
fn parse_trial_location_paging_extracts_offset_limit_flags() {
    let sections = vec![
        "locations".to_string(),
        "--offset".to_string(),
        "20".to_string(),
        "--limit=10".to_string(),
    ];
    let (cleaned, offset, limit) =
        parse_trial_location_paging(&sections).expect("valid pagination flags");
    assert_eq!(cleaned, vec!["locations".to_string()]);
    assert_eq!(offset, Some(20));
    assert_eq!(limit, Some(10));
}

#[test]
fn trial_locations_json_preserves_location_pagination_and_section_sources() {
    let trial = crate::entities::trial::Trial {
        nct_id: "NCT00000001".to_string(),
        source: Some("ctgov".to_string()),
        title: "Example trial".to_string(),
        status: "Recruiting".to_string(),
        phase: Some("Phase 2".to_string()),
        study_type: Some("Interventional".to_string()),
        age_range: Some("18 Years and older".to_string()),
        conditions: vec!["melanoma".to_string()],
        interventions: vec!["osimertinib".to_string()],
        sponsor: Some("Example Sponsor".to_string()),
        enrollment: Some(100),
        summary: Some("Example summary".to_string()),
        start_date: Some("2024-01-01".to_string()),
        completion_date: None,
        eligibility_text: None,
        locations: Some(vec![crate::entities::trial::TrialLocation {
            facility: "Example Hospital".to_string(),
            city: "Boston".to_string(),
            state: Some("MA".to_string()),
            country: "United States".to_string(),
            status: Some("Recruiting".to_string()),
            contact_name: None,
            contact_phone: None,
        }]),
        outcomes: None,
        arms: None,
        references: None,
    };

    let json = trial_locations_json(
        &trial,
        LocationPaginationMeta {
            total: 42,
            offset: 20,
            limit: 10,
            has_more: true,
        },
    )
    .expect("trial locations json");

    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert_eq!(value["nct_id"], "NCT00000001");
    assert_eq!(value["location_pagination"]["total"], 42);
    assert_eq!(value["location_pagination"]["offset"], 20);
    assert_eq!(value["location_pagination"]["limit"], 10);
    assert_eq!(value["location_pagination"]["has_more"], true);
    assert!(value.get("_meta").is_some());
    assert_eq!(value["_meta"]["section_sources"][0]["key"], "overview");
    assert_eq!(
        value["_meta"]["section_sources"][0]["sources"][0],
        "ClinicalTrials.gov"
    );
    assert!(
        value["_meta"]["section_sources"]
            .as_array()
            .expect("section sources array")
            .iter()
            .any(|entry| entry["key"] == "locations")
    );
}

#[test]
fn paginate_trial_locations_handles_missing_locations() {
    let mut trial = crate::entities::trial::Trial {
        nct_id: "NCT00000001".to_string(),
        source: Some("ctgov".to_string()),
        title: "Example trial".to_string(),
        status: "Recruiting".to_string(),
        phase: Some("Phase 2".to_string()),
        study_type: Some("Interventional".to_string()),
        age_range: Some("18 Years and older".to_string()),
        conditions: vec!["melanoma".to_string()],
        interventions: vec!["osimertinib".to_string()],
        sponsor: Some("Example Sponsor".to_string()),
        enrollment: Some(100),
        summary: Some("Example summary".to_string()),
        start_date: Some("2024-01-01".to_string()),
        completion_date: None,
        eligibility_text: None,
        locations: None,
        outcomes: None,
        arms: None,
        references: None,
    };

    let meta = paginate_trial_locations(&mut trial, 20, 10);
    assert_eq!(meta.total, 0);
    assert_eq!(meta.offset, 20);
    assert_eq!(meta.limit, 10);
    assert!(!meta.has_more);
    assert!(trial.locations.is_some());
    assert_eq!(trial.locations.as_ref().map_or(usize::MAX, Vec::len), 0);
}

#[test]
fn trial_search_query_summary_includes_geo_filters() {
    let summary = trial_search_query_summary(
        &crate::entities::trial::TrialSearchFilters {
            condition: Some("melanoma".into()),
            facility: Some("MD Anderson".into()),
            age: Some(67.0),
            sex: Some("female".into()),
            criteria: Some("mismatch repair deficient".into()),
            sponsor_type: Some("nih".into()),
            lat: Some(40.7128),
            lon: Some(-74.006),
            distance: Some(50),
            ..Default::default()
        },
        None,
        0,
        None,
    );
    assert!(summary.contains("condition=melanoma"));
    assert!(summary.contains("facility=MD Anderson"));
    assert!(summary.contains("age=67"));
    assert!(summary.contains("sex=female"));
    assert!(summary.contains("criteria=mismatch repair deficient"));
    assert!(summary.contains("sponsor_type=nih"));
    assert!(summary.contains("lat=40.7128"));
    assert!(summary.contains("lon=-74.006"));
    assert!(summary.contains("distance=50"));
}

#[test]
fn trial_search_query_summary_includes_nci_source_marker() {
    let summary = trial_search_query_summary(
        &crate::entities::trial::TrialSearchFilters {
            condition: Some("melanoma".into()),
            source: crate::entities::trial::TrialSource::NciCts,
            ..Default::default()
        },
        None,
        0,
        None,
    );

    assert!(summary.contains("condition=melanoma"));
    assert!(summary.contains("source=nci"));
}

#[test]
fn trial_search_query_summary_includes_alias_opt_out_marker() {
    let summary = trial_search_query_summary(
        &crate::entities::trial::TrialSearchFilters {
            intervention: Some("daraxonrasib".into()),
            no_alias_expand: true,
            ..Default::default()
        },
        Some("daraxonrasib"),
        0,
        None,
    );

    assert!(summary.contains("intervention=daraxonrasib"));
    assert!(summary.contains("alias_expand=off"));
}

#[test]
fn trial_search_query_summary_omits_alias_opt_out_marker_when_not_applicable() {
    let no_intervention = trial_search_query_summary(
        &crate::entities::trial::TrialSearchFilters {
            condition: Some("melanoma".into()),
            no_alias_expand: true,
            ..Default::default()
        },
        None,
        0,
        None,
    );
    let nci = trial_search_query_summary(
        &crate::entities::trial::TrialSearchFilters {
            intervention: Some("daraxonrasib".into()),
            no_alias_expand: true,
            source: crate::entities::trial::TrialSource::NciCts,
            ..Default::default()
        },
        Some("daraxonrasib"),
        0,
        None,
    );

    assert!(!no_intervention.contains("alias_expand=off"));
    assert!(!nci.contains("alias_expand=off"));
}

#[test]
fn trial_search_query_summary_can_show_canonical_intervention() {
    let summary = trial_search_query_summary(
        &crate::entities::trial::TrialSearchFilters {
            intervention: Some("Keytruda".into()),
            ..Default::default()
        },
        Some("pembrolizumab"),
        0,
        None,
    );

    assert!(summary.contains("intervention=pembrolizumab"));
    assert!(!summary.contains("intervention=Keytruda"));
}

#[test]
fn trial_zero_result_nickname_hint_requires_positional_ctgov_query_with_zero_results() {
    use crate::entities::trial::TrialSource;

    assert!(should_show_trial_zero_result_nickname_hint(
        Some("CodeBreaK 300"),
        TrialSource::ClinicalTrialsGov,
        0
    ));
    assert!(!should_show_trial_zero_result_nickname_hint(
        None,
        TrialSource::ClinicalTrialsGov,
        0
    ));
    assert!(!should_show_trial_zero_result_nickname_hint(
        Some("CodeBreaK 300"),
        TrialSource::NciCts,
        0
    ));
    assert!(!should_show_trial_zero_result_nickname_hint(
        Some("CodeBreaK 300"),
        TrialSource::ClinicalTrialsGov,
        1
    ));
}
