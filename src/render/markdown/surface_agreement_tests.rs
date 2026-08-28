use super::*;
use std::collections::BTreeSet;

fn markdown_commands(markdown: &str) -> BTreeSet<String> {
    markdown
        .lines()
        .filter_map(|line| line.strip_prefix("  biomcp "))
        .map(|line| {
            let command = line.split("   - ").next().unwrap_or(line);
            format!("biomcp {command}")
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
    documented_markdown_only: &[&str],
) {
    let mut markdown = markdown_commands(&markdown);
    let expected_extra = documented_markdown_only
        .iter()
        .map(|command| (*command).to_string())
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

fn entity_json<T: serde::Serialize>(entity: &T, commands: Vec<String>) -> String {
    crate::render::json::to_entity_json(entity, Vec::<(&str, String)>::new(), commands, Vec::new())
        .expect("entity JSON")
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
        variant_markdown(&variant, &[]).unwrap(),
        entity_json(&variant, related_variant(&variant)),
        &["biomcp get variant \"chr7:g.140453136A>T\" all"],
    );

    let gene: Gene = serde_json::from_value(serde_json::json!({
        "symbol": "BRAF", "name": "B-Raf", "entrez_id": "673",
        "aliases": []
    }))
    .unwrap();
    assert_command_surfaces(
        "gene",
        gene_markdown(&gene, &[]).unwrap(),
        entity_json(&gene, gene_next_commands(&gene, &[])),
        &["biomcp get gene BRAF all"],
    );

    let disease: Disease = serde_json::from_value(serde_json::json!({
        "id": "MONDO:0005105", "name": "melanoma"
    }))
    .unwrap();
    assert_command_surfaces(
        "disease",
        disease_markdown(&disease, &[]).unwrap(),
        entity_json(&disease, disease_next_commands(&disease, &[])),
        &["biomcp get disease MONDO:0005105 all"],
    );

    let drug: Drug = serde_json::from_value(serde_json::json!({
        "name": "osimertinib", "targets": ["EGFR"]
    }))
    .unwrap();
    assert_command_surfaces(
        "drug",
        drug_markdown(&drug, &[]).unwrap(),
        entity_json(&drug, related_drug(&drug)),
        &["biomcp get drug osimertinib all"],
    );

    let trial: Trial = serde_json::from_value(serde_json::json!({
        "nct_id": "NCT01234567", "title": "Example trial", "status": "Completed",
        "conditions": ["melanoma"], "interventions": ["dabrafenib"]
    }))
    .unwrap();
    assert_command_surfaces(
        "trial",
        trial_markdown(&trial, &[]).unwrap(),
        entity_json(&trial, related_trial(&trial)),
        &["biomcp get trial NCT01234567 all"],
    );

    let article: Article = serde_json::from_value(serde_json::json!({
        "pmid": "22663011", "title": "Example about melanoma",
        "author_count": 0, "author_completeness": "unavailable",
        "author_source": "pub_tator"
    }))
    .unwrap();
    assert_command_surfaces(
        "article",
        article_markdown(&article, &[]).unwrap(),
        entity_json(&article, related_article(&article)),
        &["biomcp get article 22663011 all"],
    );

    let event: AdverseEvent = serde_json::from_value(serde_json::json!({
        "report_id": "1001", "drug": "osimertinib", "serious": true
    }))
    .unwrap();
    assert_command_surfaces(
        "adverse-event",
        adverse_event_markdown(&event, &[]).unwrap(),
        entity_json(&event, related_adverse_event(&event)),
        &["biomcp get adverse-event 1001 all"],
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
        author_detail_markdown(&author),
        serde_json::to_string(&author).unwrap(),
        &[],
    );

    let diagnostic: Diagnostic = serde_json::from_value(serde_json::json!({
        "source": "gtr", "source_id": "GTR000000001.1",
        "accession": "GTR000000001.1", "name": "Diagnostic fixture"
    }))
    .unwrap();
    assert_command_surfaces(
        "diagnostic",
        diagnostic_markdown(&diagnostic, &[]).unwrap(),
        entity_json(&diagnostic, diagnostic_next_commands(&diagnostic, &[])),
        &["biomcp get diagnostic GTR000000001.1 all"],
    );

    let protein: Protein = serde_json::from_value(serde_json::json!({
        "accession": "P15056", "name": "BRAF protein", "gene_symbol": "BRAF"
    }))
    .unwrap();
    assert_command_surfaces(
        "protein",
        protein_markdown(&protein, &[]).unwrap(),
        entity_json(&protein, related_protein(&protein, &[])),
        &["biomcp get protein P15056 all"],
    );

    let pgx: Pgx = serde_json::from_value(serde_json::json!({
        "query": "drug with space", "gene": "CYP2D6", "drug": "drug with space"
    }))
    .unwrap();
    assert_command_surfaces(
        "PGx",
        pgx_markdown(&pgx, &[]).unwrap(),
        entity_json(&pgx, related_pgx(&pgx)),
        &["biomcp get pgx \"drug with space\" all"],
    );

    let pathway: Pathway = serde_json::from_value(serde_json::json!({
        "source": "Reactome", "id": "R-HSA-5673001",
        "name": "RAF/MAP kinase cascade"
    }))
    .unwrap();
    assert_command_surfaces(
        "pathway",
        pathway_markdown(&pathway, &[]).unwrap(),
        entity_json(&pathway, related_pathway(&pathway)),
        &["biomcp get pathway R-HSA-5673001 all"],
    );
}
