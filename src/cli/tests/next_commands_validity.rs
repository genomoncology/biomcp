use crate::cli::Cli;
use clap::Parser;

fn parse_cmd(cmd: &str) -> Vec<String> {
    shlex::split(cmd).unwrap_or_else(|| panic!("shlex failed on: {cmd}"))
}

fn assert_parses(cmd: &str) {
    Cli::try_parse_from(parse_cmd(cmd)).unwrap_or_else(|e| panic!("failed to parse '{cmd}': {e}"));
}

#[test]
fn suggest_command_parse() {
    assert_parses(r#"biomcp suggest "What drugs treat melanoma?""#);
    assert_parses(r#"biomcp --json suggest "When was imatinib approved?""#);

    for example in crate::cli::suggest::route_examples() {
        let response = crate::cli::suggest::suggest_question(example.question);
        assert_eq!(
            response.matched_skill.as_deref(),
            Some(example.expected_skill),
            "{}",
            example.question
        );
        for command in response.first_commands {
            assert_parses(&command);
        }
    }
}

#[test]
fn workflow_ladder_sidecar_commands_parse() {
    for workflow in crate::workflow_ladders::Workflow::ALL {
        let ladder = crate::workflow_ladders::load(workflow).expect("sidecar should load");
        for step in ladder.ladder {
            assert_parses(&step.command);
        }
    }
}

#[test]
fn gene_next_commands_parse() {
    assert_parses("biomcp get gene BRAF diagnostics");
    assert_parses("biomcp get gene BRAF funding");
    assert_parses(r#"biomcp search trial -c "Dravet syndrome" -s recruiting"#);
    assert_parses("biomcp search pgx -g BRAF");
    assert_parses("biomcp search variant -g BRAF");
    assert_parses("biomcp search article -g BRAF");
    assert_parses("biomcp search drug --target BRAF");
    assert_parses("biomcp search diagnostic --gene BRAF");
    assert_parses("biomcp gene trials BRAF");
}

#[test]
fn gene_search_json_next_commands_parse() {
    assert_parses("biomcp get gene BRAF");
    assert_parses("biomcp list gene");
}

#[test]
fn variant_next_commands_parse() {
    assert_parses("biomcp get gene BRAF");
    assert_parses(r#"biomcp search article -g SCN1A -d "Dravet syndrome" -k "T1174S" --limit 5"#);
    assert_parses(r#"biomcp search article -g SCN1A -k "T1174S" --limit 5"#);
    assert_parses(r#"biomcp search article -d "Dravet syndrome" -k "T1174S" --limit 5"#);
    assert_parses(r#"biomcp search article -k "T1174S" --limit 5"#);
    assert_parses("biomcp search drug --target BRAF");
    assert_parses(r#"biomcp variant trials "rs113488022""#);
    assert_parses(r#"biomcp variant articles "rs113488022""#);
    assert_parses(r#"biomcp variant oncokb "rs113488022""#);
}

#[test]
fn variant_search_json_next_commands_parse() {
    assert_parses("biomcp get variant rs113488022");
    assert_parses("biomcp list variant");
}

#[test]
fn article_next_commands_parse() {
    assert_parses("biomcp search gene -q EGFR");
    assert_parses(r#"biomcp search gene -q "serine-threonine protein kinase""#);
    assert_parses("biomcp search disease --query melanoma");
    assert_parses("biomcp get drug osimertinib");
    assert_parses("biomcp article entities 12345");
    assert_parses("biomcp article citations 12345 --limit 3");
    assert_parses("biomcp article references 12345 --limit 3");
    assert_parses("biomcp article recommendations 12345 67890 --negative 11111 --limit 3");
}

#[test]
fn article_search_json_next_commands_parse() {
    assert_parses("biomcp get article 12345");
    assert_parses("biomcp get gene BRAF");
    assert_parses("biomcp get drug imatinib");
    assert_parses("biomcp get disease melanoma");
    assert_parses("biomcp list article");
}

#[test]
fn article_loop_suggestion_commands_parse() {
    assert_parses("biomcp article batch 22663011 24200969");
    assert_parses(r#"biomcp discover "Oncotype DX DCIS study""#);
    assert_parses(
        r#"biomcp search article -k "Oncotype DX DCIS study" --year-min 2025 --year-max 2025 --limit 5"#,
    );
}

#[test]
fn article_and_discover_next_commands_parse() {
    assert_parses("biomcp get gene BRAF");
    assert_parses("biomcp get drug imatinib");
    assert_parses("biomcp get disease melanoma");
    assert_parses(r#"biomcp search article -g CTCF -k cohesin --limit 5"#);
}

#[test]
fn trial_next_commands_parse() {
    assert_parses(
        r#"biomcp search article --drug dabrafenib -q "NCT01234567 Example trial" --limit 5"#,
    );
    assert_parses(r#"biomcp search article -q "NCT01234567 Example trial" --limit 5"#);
    assert_parses("biomcp search disease --query melanoma");
    assert_parses("biomcp search article -d melanoma");
    assert_parses("biomcp search trial -c melanoma");
    assert_parses("biomcp get drug dabrafenib");
    assert_parses("biomcp drug trials dabrafenib");
}

#[test]
fn trial_search_json_next_commands_parse() {
    assert_parses("biomcp get trial NCT01234567");
    assert_parses("biomcp list trial");
}

#[test]
fn disease_next_commands_parse() {
    assert_parses("biomcp get disease MONDO:0005105 diagnostics");
    assert_parses("biomcp get disease MONDO:0005105 survival");
    assert_parses("biomcp get disease MONDO:0005105 funding");
    assert_parses("biomcp get gene SCN1A clingen constraint");
    assert_parses(r#"biomcp get disease "Dravet syndrome" genes phenotypes"#);
    assert_parses("biomcp search trial -c melanoma");
    assert_parses("biomcp search article -d melanoma");
    assert_parses(r#"biomcp search drug --indication "melanoma""#);
    assert_parses(r#"biomcp search diagnostic --disease "melanoma""#);
}

#[test]
fn disease_search_json_next_commands_parse() {
    assert_parses("biomcp get disease MONDO:0005105");
    assert_parses("biomcp list disease");
}

#[test]
fn diagnostic_next_commands_parse() {
    assert_parses("biomcp get diagnostic GTR000000001.1 genes");
    assert_parses("biomcp get diagnostic GTR000000001.1 conditions");
    assert_parses("biomcp get diagnostic GTR000000001.1 methods");
    assert_parses("biomcp get diagnostic GTR000000001.1 regulatory");
    assert_parses("biomcp get diagnostic \"ITPW02232- TC40\" conditions");
    assert_parses("biomcp get diagnostic \"ITPW02232- TC40\" regulatory");
    assert_parses("biomcp list diagnostic");
}

#[test]
fn diagnostic_search_json_next_commands_parse() {
    assert_parses("biomcp get diagnostic GTR000000001.1");
    assert_parses("biomcp get diagnostic \"ITPW02232- TC40\"");
    assert_parses("biomcp list diagnostic");
}

#[test]
fn pgx_next_commands_parse() {
    assert_parses("biomcp search pgx -g CYP2D6");
    assert_parses("biomcp search pgx -d warfarin");
}

#[test]
fn pgx_search_json_next_commands_parse() {
    assert_parses("biomcp get pgx CYP2D6");
    assert_parses("biomcp list pgx");
}

#[test]
fn drug_next_commands_parse() {
    assert_parses("biomcp drug trials osimertinib");
    assert_parses("biomcp drug adverse-events osimertinib");
    assert_parses("biomcp drug interactions osimertinib");
    assert_parses("biomcp get drug osimertinib safety");
    assert_parses("biomcp search article --drug osimertinib --limit 5");
    assert_parses("biomcp get gene EGFR");
}

#[test]
fn drug_search_json_next_commands_parse() {
    assert_parses("biomcp get drug pembrolizumab");
    assert_parses("biomcp list drug");
}

#[test]
fn pathway_next_commands_parse() {
    assert_parses("biomcp pathway drugs R-HSA-5673001");
}

#[test]
fn pathway_search_json_next_commands_parse() {
    assert_parses("biomcp get pathway R-HSA-5673001");
    assert_parses("biomcp list pathway");
}

#[test]
fn protein_next_commands_parse() {
    assert_parses("biomcp get protein P00533 structures");
    assert_parses("biomcp get protein P00533 complexes");
    assert_parses("biomcp get gene EGFR");
}

#[test]
fn adverse_event_search_json_next_commands_parse() {
    assert_parses("biomcp get adverse-event 12345");
    assert_parses("biomcp list adverse-event");
}

#[test]
fn adverse_event_next_commands_parse() {
    assert_parses("biomcp get drug osimertinib");
    assert_parses("biomcp drug adverse-events osimertinib");
    assert_parses("biomcp drug trials osimertinib");
}

#[test]
fn device_event_next_commands_parse() {
    assert_parses("biomcp search adverse-event --type device --device HeartValve");
    assert_parses(r#"biomcp search adverse-event --type recall --classification "Class I""#);
}

#[test]
fn gwas_search_json_next_commands_parse() {
    assert_parses("biomcp get variant rs7903146");
    assert_parses("biomcp list gwas");
}

#[test]
fn discover_next_commands_parse() {
    // gene — unambiguous and ambiguous
    assert_parses("biomcp get gene EGFR");
    assert_parses(r#"biomcp search gene -q "ERBB1" --limit 10"#);
    // drug
    assert_parses(r#"biomcp get drug "pembrolizumab""#);
    assert_parses(r#"biomcp drug adverse-events pembrolizumab"#);
    assert_parses(r#"biomcp get drug pembrolizumab safety"#);
    assert_parses(r#"biomcp search drug --indication "Myasthenia gravis" --limit 5"#);
    // disease — unambiguous helpers and ambiguous fallback
    assert_parses(r#"biomcp get disease "cystic fibrosis""#);
    assert_parses(r#"biomcp disease trials "cystic fibrosis""#);
    assert_parses(r#"biomcp search article -k "cystic fibrosis" --limit 5"#);
    assert_parses(r#"biomcp search disease -q "diabetes" --limit 10"#);
    assert_parses(r#"biomcp get disease MONDO:0007947 phenotypes"#);
    // symptom
    assert_parses(r#"biomcp search disease -q "chest pain" --limit 10"#);
    assert_parses(r#"biomcp search trial -c "chest pain" --limit 5"#);
    assert_parses(r#"biomcp search article -k "chest pain" --limit 5"#);
    // pathway
    assert_parses(r#"biomcp search pathway -q "MAPK signaling" --limit 5"#);
    // gene+disease orientation
    assert_parses(r#"biomcp search all --gene BRAF --disease "melanoma""#);
    // variant with and without gene inference
    assert_parses(r#"biomcp get variant "BRAF V600E""#);
    assert_parses(r#"biomcp search article -k "V600E" --limit 5"#);
    // empty and low-confidence fallbacks
    assert_parses(r#"biomcp search article -k qzvxxptl --type review --limit 5"#);
    assert_parses(r#"biomcp search article -k FAKE1"#);
    // trial intent
    assert_parses(r#"biomcp search trial -c "Breast Cancer" --limit 5"#);
    assert_parses(r#"biomcp search article -k "Breast Cancer" --limit 5"#);
}
