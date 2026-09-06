use crate::entities::adverse_event::{AdverseEvent, AdverseEventReport};
use crate::entities::article::Article;
use crate::entities::diagnostic::Diagnostic;
use crate::entities::disease::Disease;
use crate::entities::drug::{Drug, DrugRegion};
use crate::entities::gene::Gene;
use crate::entities::pathway::Pathway;
use crate::entities::pgx::Pgx;
use crate::entities::protein::Protein;
use crate::entities::trial::Trial;
use crate::entities::variant::Variant;
use std::collections::BTreeSet;

fn markdown_commands(markdown: &str) -> BTreeSet<String> {
    markdown
        .lines()
        .filter_map(|line| {
            line.strip_prefix("  biomcp ")
                .map(|command| (command, true))
                .or_else(|| {
                    line.strip_prefix("Retry: ")
                        .and_then(|line| line.find("biomcp ").map(|start| (&line[start..], false)))
                })
        })
        .map(|(line, needs_prefix)| {
            let command = line.split("   - ").next().unwrap_or(line);
            if needs_prefix {
                format!("biomcp {command}")
            } else {
                command.trim_end_matches('`').to_string()
            }
        })
        .collect()
}

fn json_commands(json: &str) -> BTreeSet<String> {
    let value: serde_json::Value = serde_json::from_str(json).expect("valid card JSON");
    value["_meta"]["next_commands"]
        .as_array()
        .expect("card JSON has _meta.next_commands")
        .iter()
        .map(|command| command.as_str().expect("command string").to_string())
        .collect()
}

fn assert_command_surfaces(
    family: &str,
    markdown: String,
    json: String,
    documented_markdown_only: &[(&str, &str)],
) {
    let mut markdown = markdown_commands(&markdown);
    let expected_extra = documented_markdown_only
        .iter()
        .map(|(command, reason)| {
            assert!(
                !reason.trim().is_empty(),
                "{family}: asymmetry needs a reason"
            );
            (*command).to_string()
        })
        .collect::<BTreeSet<_>>();
    let json = json_commands(&json);
    let actual_extra = markdown.difference(&json).cloned().collect::<BTreeSet<_>>();
    assert_eq!(
        actual_extra, expected_extra,
        "{family}: undocumented Markdown-only commands"
    );
    markdown.retain(|command| !expected_extra.contains(command));
    assert_eq!(
        markdown, json,
        "{family}: Markdown and JSON commands drifted"
    );
}

#[test]
fn every_detail_card_markdown_and_json_commands_agree() {
    // Keep this list exhaustive for detail-card families that ship
    // `_meta.next_commands`. Full expansion has its own `All:` heading in
    // Markdown. It is navigation, not a follow-up in the More/See-also
    // contract represented by JSON metadata, so each such asymmetry is named.
    let variant: Variant = serde_json::from_value(serde_json::json!({
        "gene": "BRAF", "id": "chr7:g.140453136A>T",
        "hgvs_p": "p.Val600Glu", "rsid": "rs113488022"
    }))
    .unwrap();
    assert_command_surfaces(
        "variant",
        crate::cli::variant::render_loaded_card(&variant, false, &[], false).unwrap(),
        crate::cli::variant::render_loaded_card(&variant, false, &[], true).unwrap(),
        &[
            (
                "biomcp get variant \"chr7:g.140453136A>T\" all",
                "Markdown offers full-card navigation; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get variant \"chr7:g.140453136A>T\" clinvar",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get variant \"chr7:g.140453136A>T\" predict",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get variant \"chr7:g.140453136A>T\" predictions",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
        ],
    );

    let gene: Gene = serde_json::from_value(serde_json::json!({
        "symbol": "BRAF", "name": "B-Raf", "entrez_id": "673",
        "aliases": [],
        "section_outcomes": {
            "civic": {"outcome": "unavailable", "sources": [], "message": "provider unavailable"}
        }
    }))
    .unwrap();
    let gene_sections = ["civic".to_string()];
    assert_command_surfaces(
        "gene",
        crate::cli::gene::render_loaded_card(&gene, &gene_sections, false).unwrap(),
        crate::cli::gene::render_loaded_card(&gene, &gene_sections, true).unwrap(),
        &[(
            "biomcp get gene BRAF all",
            "Markdown offers full-card navigation; JSON metadata carries follow-up pivots.",
        )],
    );

    let disease: Disease = serde_json::from_value(serde_json::json!({
        "id": "MONDO:0005105", "name": "melanoma"
    }))
    .unwrap();
    assert_command_surfaces(
        "disease",
        crate::cli::disease::render_loaded_card(&disease, &[], false).unwrap(),
        crate::cli::disease::render_loaded_card(&disease, &[], true).unwrap(),
        &[(
            "biomcp get disease MONDO:0005105 all",
            "Markdown offers full-card navigation; JSON metadata carries follow-up pivots.",
        )],
    );

    let drug: Drug = serde_json::from_value(serde_json::json!({
        "name": "osimertinib", "targets": ["EGFR"]
    }))
    .unwrap();
    assert_command_surfaces(
        "drug",
        crate::cli::drug::render_loaded_card(&drug, &[], DrugRegion::Us, false, false).unwrap(),
        crate::cli::drug::render_loaded_card(&drug, &[], DrugRegion::Us, false, true).unwrap(),
        &[
            (
                "biomcp get drug osimertinib all",
                "Markdown offers full-card navigation; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get drug osimertinib label",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get drug osimertinib regulatory",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get drug osimertinib safety",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
        ],
    );

    let trial: Trial = serde_json::from_value(serde_json::json!({
        "nct_id": "NCT01234567", "title": "Example trial", "status": "Completed",
        "conditions": ["melanoma"],
        "interventions": [{"id": 1, "name": "dabrafenib", "type": null, "description": null, "other_names": []}]
    }))
    .unwrap();
    assert_command_surfaces(
        "trial",
        crate::cli::trial::render_loaded_card(&trial, &[], false).unwrap(),
        crate::cli::trial::render_loaded_card(&trial, &[], true).unwrap(),
        &[
            (
                "biomcp get trial NCT01234567 all",
                "Markdown offers full-card navigation; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get trial NCT01234567 arms",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get trial NCT01234567 outcomes",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get trial NCT01234567 references",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
        ],
    );

    let article: Article = serde_json::from_value(serde_json::json!({
        "pmid": "22663011", "title": "Example about melanoma",
        "author_count": 0, "author_completeness": "unavailable",
        "author_source": "pubtator"
    }))
    .unwrap();
    assert_command_surfaces(
        "article",
        crate::cli::article::render_loaded_card(&article, &[], false).unwrap(),
        crate::cli::article::render_loaded_card(&article, &[], true).unwrap(),
        &[
            (
                "biomcp get article 22663011 all",
                "Markdown offers full-card navigation; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get article 22663011 annotations",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get article 22663011 fulltext",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get article 22663011 tldr",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
        ],
    );

    let event: AdverseEvent = serde_json::from_value(serde_json::json!({
        "report_id": "1001", "drug": "osimertinib", "serious": true
    }))
    .unwrap();
    assert_command_surfaces(
        "adverse-event",
        crate::cli::adverse_event::render_loaded_card(
            &AdverseEventReport::Faers(event.clone()),
            &[],
            false,
        )
        .unwrap(),
        crate::cli::adverse_event::render_loaded_card(&AdverseEventReport::Faers(event), &[], true)
            .unwrap(),
        &[
            (
                "biomcp get adverse-event 1001 all",
                "Markdown offers full-card navigation; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get adverse-event 1001 concomitant",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get adverse-event 1001 outcomes",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get adverse-event 1001 reactions",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
        ],
    );

    use crate::entities::author::*;
    let id: ProviderAuthorId = "semanticscholar:1716151".parse().unwrap();
    let author = AuthorDetail {
        identity: AuthorIdentity::ExactProvider { id: id.clone() },
        display_name: "A. Butte".into(),
        provider_records: vec![ProviderAuthorRecord {
            id,
            source: "semantic_scholar",
            status: ProviderStatus::Available,
        }],
        affiliations: vec![],
        paper_count: None,
        citation_count: None,
        h_index: None,
        conflicts: vec![],
        warnings: vec![AuthorWarning::unresolved_orcid()],
        _meta: AuthorMeta {
            source_status: vec![],
            evidence_urls: vec![],
            next_commands: vec!["biomcp author papers semanticscholar:1716151".into()],
        },
    };
    assert_command_surfaces(
        "author",
        crate::cli::author::render_loaded_card(&author, false).unwrap(),
        crate::cli::author::render_loaded_card(&author, true).unwrap(),
        &[],
    );

    let diagnostic: Diagnostic = serde_json::from_value(serde_json::json!({
        "source": "gtr", "source_id": "GTR000000001.1",
        "accession": "GTR000000001.1", "name": "Diagnostic fixture"
    }))
    .unwrap();
    assert_command_surfaces(
        "diagnostic",
        crate::cli::diagnostic::render_loaded_card(&diagnostic, &[], false).unwrap(),
        crate::cli::diagnostic::render_loaded_card(&diagnostic, &[], true).unwrap(),
        &[(
            "biomcp get diagnostic GTR000000001.1 all",
            "Markdown offers full-card navigation; JSON metadata carries follow-up pivots.",
        )],
    );

    let protein: Protein = serde_json::from_value(serde_json::json!({
        "accession": "P15056", "name": "BRAF protein", "gene_symbol": "BRAF"
    }))
    .unwrap();
    assert_command_surfaces(
        "protein",
        crate::cli::protein::render_loaded_card(&protein, &[], false).unwrap(),
        crate::cli::protein::render_loaded_card(&protein, &[], true).unwrap(),
        &[
            (
                "biomcp get protein P15056 all",
                "Markdown offers full-card navigation; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get protein P15056 domains",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get protein P15056 interactions",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
        ],
    );

    let pgx: Pgx = serde_json::from_value(serde_json::json!({
        "query": "drug with space", "gene": "CYP2D6", "drug": "drug with space"
    }))
    .unwrap();
    assert_command_surfaces(
        "PGx",
        crate::cli::pgx::render_loaded_card(&pgx, &[], false).unwrap(),
        crate::cli::pgx::render_loaded_card(&pgx, &[], true).unwrap(),
        &[
            (
                "biomcp get pgx \"drug with space\" all",
                "Markdown offers full-card navigation; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get pgx \"drug with space\" frequencies",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get pgx \"drug with space\" interactions",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get pgx \"drug with space\" recommendations",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
        ],
    );

    let pathway: Pathway = serde_json::from_value(serde_json::json!({
        "source": "Reactome", "id": "R-HSA-5673001",
        "name": "RAF/MAP kinase cascade"
    }))
    .unwrap();
    assert_command_surfaces(
        "pathway",
        crate::cli::pathway::render_loaded_card(&pathway, &[], false).unwrap(),
        crate::cli::pathway::render_loaded_card(&pathway, &[], true).unwrap(),
        &[
            (
                "biomcp get pathway R-HSA-5673001 all",
                "Markdown offers full-card navigation; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get pathway R-HSA-5673001 enrichment",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get pathway R-HSA-5673001 events",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
            (
                "biomcp get pathway R-HSA-5673001 genes",
                "Markdown offers a requested-section navigation shortcut; JSON metadata carries follow-up pivots.",
            ),
        ],
    );
}
