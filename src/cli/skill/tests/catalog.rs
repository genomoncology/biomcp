//! Tests for read-only BioMCP skill catalog behavior.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::error::BioMcpError;

use super::super::assets::{canonical_prompt_body, canonical_prompt_file_bytes, embedded_text};
use super::super::catalog::{list_use_case_refs, use_case_index};
use super::super::{list_use_cases, render_system_prompt, show_overview, show_use_case};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_json_fixture(path: &Path) -> Value {
    let contents = fs::read_to_string(path).expect("read JSON fixture");
    serde_json::from_str(&contents).expect("parse JSON fixture")
}

#[test]
fn embedded_skill_overview_is_routing_first_and_points_to_worked_examples()
-> Result<(), BioMcpError> {
    let overview = show_overview()?;

    assert!(overview.contains("biomcp suggest \"<question>\""));
    assert!(overview.contains("## Routing rules"));
    assert!(overview.contains("## Section reference"));
    assert!(overview.contains("## Cross-entity pivot rules"));
    assert!(overview.contains("## How-to reference"));
    assert!(overview.contains("## Anti-patterns"));
    assert!(overview.contains("## Output and evidence rules"));
    assert!(overview.contains("## Answer commitment"));
    assert!(overview.contains("biomcp search drug --indication \"<disease>\""));
    assert!(overview.contains("biomcp ema sync"));
    assert!(overview.contains("biomcp who sync"));
    assert!(overview.contains("biomcp cvx sync"));
    assert!(overview.contains("biomcp discover \"<free text>\""));
    assert!(overview.contains("biomcp search article -k \"<query>\" --type review --limit 5"));
    assert!(!overview.contains("../docs/"));
    assert!(!overview.contains(".md)"));
    assert!(overview.contains("Never do more than 3 article searches for one question."));
    assert!(overview.contains("pass `--json --session <token>`"));
    assert!(overview.contains("session loop-breaker suggestions with `command` and `reason` only"));
    assert!(overview.contains("ClinicalTrials.gov usually does not index nicknames"));
    assert!(overview.contains("add `--drug <name>` to `search article`"));
    assert!(
        overview.contains("`biomcp article batch <pmid1> <pmid2> ...` uses spaces between PMIDs.")
    );
    assert!(
        overview
            .contains("If one command already answers the question, stop searching and answer.")
    );
    assert!(overview.find("## Cross-entity pivot rules") < overview.find("## How-to reference"));
    assert!(overview.find("biomcp suggest \"<question>\"") < overview.find("## Routing rules"));
    assert!(overview.find("biomcp ema sync") < overview.find("## Section reference"));
    assert!(overview.find("biomcp who sync") < overview.find("## Section reference"));
    assert!(overview.find("biomcp cvx sync") < overview.find("## Section reference"));
    assert!(overview.find("## How-to reference") < overview.find("## Anti-patterns"));
    assert!(overview.find("## Anti-patterns") < overview.find("## Output and evidence rules"));
    assert!(overview.find("## Output and evidence rules") < overview.find("## Answer commitment"));
    assert!(
        overview.find("## Answer commitment")
            < overview.find("Run `biomcp skill list` for worked examples")
    );
    assert!(overview.contains("Run `biomcp skill list` for worked examples"));

    Ok(())
}

#[test]
fn canonical_prompt_body_matches_overview_and_normalizes_newlines() -> Result<(), BioMcpError> {
    let body = canonical_prompt_body()?;
    let rendered = render_system_prompt()?;
    let file_bytes = canonical_prompt_file_bytes()?;

    assert_eq!(show_overview()?, body);
    assert_eq!(rendered, body);
    assert!(!body.ends_with('\n'));
    assert_eq!(file_bytes, format!("{body}\n").into_bytes());
    assert!(file_bytes.ends_with(b"\n"));
    assert!(!file_bytes.ends_with(b"\n\n"));

    Ok(())
}

#[test]
fn validate_skills_target_uses_project_free_uv_dev_environment() {
    let makefile = fs::read_to_string(repo_root().join("Makefile")).expect("read Makefile");
    let pyproject = fs::read_to_string(repo_root().join("pyproject.toml")).expect("read pyproject");

    assert!(makefile.contains("validate-skills:"));
    assert!(makefile.contains("$(MAKE) sync-python-dev"));
    assert!(makefile.contains("uv sync --extra dev --no-install-project"));
    assert!(makefile.contains("uv run --no-sync sh -c"));
    assert!(makefile.contains("./scripts/validate-skills.sh"));
    assert!(makefile.contains("PATH=\"$(CURDIR)/target/release:$$PATH\""));
    assert!(pyproject.contains("\"jsonschema>="));
}

#[test]
fn refreshed_search_examples_are_non_empty() {
    let article_path = repo_root().join("skills/examples/search-article.json");
    let article_payload = read_json_fixture(&article_path);
    let article_count = article_payload
        .get("count")
        .and_then(Value::as_u64)
        .expect("article count should be present");
    let article_returned = article_payload
        .pointer("/pagination/returned")
        .and_then(Value::as_u64)
        .expect("article pagination.returned should be present");
    let article_results = article_payload
        .get("results")
        .and_then(Value::as_array)
        .expect("article results should be an array");

    assert!(
        article_count > 0,
        "article example should keep at least one row"
    );
    assert!(
        article_returned > 0,
        "article example should report returned rows"
    );
    assert!(
        !article_results.is_empty(),
        "article example should keep non-empty results"
    );

    let drug_path = repo_root().join("skills/examples/search-drug.json");
    let drug_payload = read_json_fixture(&drug_path);
    let drug_count = drug_payload
        .pointer("/regions/us/count")
        .and_then(Value::as_u64)
        .expect("drug regions.us.count should be present");
    let drug_returned = drug_payload
        .pointer("/regions/us/pagination/returned")
        .and_then(Value::as_u64)
        .expect("drug regions.us.pagination.returned should be present");
    let drug_results = drug_payload
        .pointer("/regions/us/results")
        .and_then(Value::as_array)
        .expect("drug regions.us.results should be an array");

    assert_eq!(
        drug_payload.get("region"),
        Some(&Value::String("us".to_string()))
    );
    assert!(drug_count > 0, "drug example should keep at least one row");
    assert!(
        drug_returned > 0,
        "drug example should report returned rows"
    );
    assert!(
        !drug_results.is_empty(),
        "drug example should keep non-empty results"
    );
}

#[test]
fn embedded_use_case_catalog_lists_expected_worked_examples() -> Result<(), BioMcpError> {
    let refs = list_use_case_refs()?;
    let slugs = refs
        .iter()
        .map(|case| case.slug.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        slugs,
        vec![
            "treatment-lookup",
            "symptom-phenotype",
            "gene-disease-orientation",
            "article-follow-up",
            "variant-pathogenicity",
            "drug-regulatory",
            "gene-function-localization",
            "mechanism-pathway",
            "trial-recruitment",
            "pharmacogene-cumulative",
            "disease-locus-mapping",
            "cellular-process-regulation",
            "mutation-catalog",
            "syndrome-disambiguation",
            "negative-evidence",
        ]
    );

    let listing = list_use_cases()?;
    assert!(listing.contains("# BioMCP Worked Examples"));
    assert!(listing.contains("05 variant-pathogenicity"));
    assert!(
        listing.contains(
            "15 negative-evidence - Pattern: Negative evidence and no-association checks"
        )
    );

    let numbered = show_use_case("05")?;
    assert!(numbered.contains("# Pattern: Variant pathogenicity evidence"));

    let mutation = show_use_case("13")?;
    assert!(mutation.contains("# Pattern: Mutation catalog for one gene and disease"));

    Ok(())
}

#[test]
fn embedded_use_case_anchor_commands_parse() -> Result<(), BioMcpError> {
    let cases = use_case_index()?
        .into_iter()
        .filter(|case| {
            case.number
                .parse::<u32>()
                .is_ok_and(|number| (5..=15).contains(&number))
        })
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 11);

    for case in cases {
        let content = embedded_text(&case.embedded_path)?;
        let mut blocks = Vec::new();
        let mut current = String::new();
        let mut in_bash_block = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "```bash" {
                assert!(
                    !in_bash_block,
                    "{} should not nest fenced bash blocks",
                    case.slug
                );
                in_bash_block = true;
                current.clear();
                continue;
            }
            if trimmed == "```" && in_bash_block {
                blocks.push(current.trim_end().to_string());
                in_bash_block = false;
                continue;
            }
            if in_bash_block {
                current.push_str(line);
                current.push('\n');
            }
        }

        assert!(
            !in_bash_block,
            "{} has an unterminated fenced bash block",
            case.slug
        );
        assert_eq!(blocks.len(), 1, "{} should have one bash block", case.slug);

        let commands = blocks[0]
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>();
        assert!(
            (3..=4).contains(&commands.len()),
            "{} should have 3-4 anchor commands",
            case.slug
        );

        for command in commands {
            assert!(
                command.starts_with("biomcp "),
                "{} command should start with biomcp: {command}",
                case.slug
            );
            for forbidden in [
                "|",
                ">",
                "<",
                "2>&1",
                "grep",
                "cat ",
                "jq ",
                "/home/ian/workspace/research/",
            ] {
                assert!(
                    !command.contains(forbidden),
                    "{} command contains forbidden token {forbidden}: {command}",
                    case.slug
                );
            }

            let argv = shlex::split(command)
                .unwrap_or_else(|| panic!("shlex failed for {}: {command}", case.slug));
            crate::cli::try_parse_cli(argv).unwrap_or_else(|err| {
                panic!("{} command did not parse: {command}: {err}", case.slug)
            });
        }
    }

    Ok(())
}

#[test]
fn missing_skill_suggests_skill_catalog() {
    let err = show_use_case("99").expect_err("missing skill should error");
    let msg = err.to_string();

    assert!(msg.contains("skill '99' not found"));
    assert!(msg.contains("Try: biomcp skill list"));
}
