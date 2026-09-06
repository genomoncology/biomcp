//! Round-trip contract for commands printed by detail-card markdown.
//!
//! Adding a detail-card family that prints commands requires adding its fixture to
//! `detail_card_commands_parse`.

use crate::cli::Cli;
use clap::Parser;

fn printed_commands(markdown: &str) -> Vec<String> {
    let mut commands = std::collections::BTreeSet::new();
    for line in markdown.lines() {
        for code in markdown_code_spans(line) {
            if let Some(start) = code.find("biomcp ") {
                commands.insert(code[start..].trim().to_string());
            }
        }
        let plain = line.trim();
        if let Some(command) = plain.strip_prefix("biomcp ") {
            let command = command
                .split_once("   - ")
                .map_or(command, |(command, _)| command);
            commands.insert(format!("biomcp {}", command.trim()));
        }
    }
    commands.into_iter().collect()
}

fn markdown_code_spans(line: &str) -> Vec<&str> {
    let bytes = line.as_bytes();
    let mut spans = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let Some(relative_start) = bytes[cursor..].iter().position(|byte| *byte == b'`') else {
            break;
        };
        let start = cursor + relative_start;
        let delimiter_len = bytes[start..]
            .iter()
            .take_while(|byte| **byte == b'`')
            .count();
        let content_start = start + delimiter_len;
        let mut search = content_start;
        let mut close = None;
        while search < bytes.len() {
            let Some(relative_tick) = bytes[search..].iter().position(|byte| *byte == b'`') else {
                break;
            };
            let tick = search + relative_tick;
            let run = bytes[tick..]
                .iter()
                .take_while(|byte| **byte == b'`')
                .count();
            if run == delimiter_len {
                close = Some(tick);
                break;
            }
            search = tick + run;
        }
        let Some(end) = close else {
            break;
        };
        spans.push(&line[content_start..end]);
        cursor = end + delimiter_len;
    }
    spans
}

fn parse_printed_commands(markdown: &str) -> Result<usize, String> {
    let commands = printed_commands(markdown);
    if commands.is_empty() {
        return Err("card fixture did not print a biomcp command".to_string());
    }
    for command in &commands {
        let argv = shlex::split(command)
            .ok_or_else(|| format!("rendered command is not valid shell text: `{command}`"))?;
        Cli::try_parse_from(argv)
            .map_err(|error| format!("rendered command does not parse: `{command}`: {error}"))?;
    }
    Ok(commands.len())
}

fn render_fixtures() -> Vec<(&'static str, String)> {
    use crate::render::markdown::*;

    let variant = serde_json::from_value(serde_json::json!({
        "gene": "BRAF",
        "id": "chr7:g.140453136A>T",
        "hgvs_p": "p.Val600Glu",
        "rsid": "rs113488022"
    }))
    .expect("variant fixture");
    let gene = serde_json::from_value(serde_json::json!({
        "symbol": "BRAF",
        "name": "B-Raf proto-oncogene",
        "entrez_id": "673",
        "aliases": []
    }))
    .expect("gene fixture");
    let disease = serde_json::from_value(serde_json::json!({
        "id": "MONDO:0005105",
        "name": "melanoma"
    }))
    .expect("disease fixture");
    let drug = serde_json::from_value(serde_json::json!({
        "name": "drug with space"
    }))
    .expect("drug fixture");
    let trial = serde_json::from_value(serde_json::json!({
        "nct_id": "NCT01234567",
        "title": "Quoted intervention trial",
        "status": "Recruiting",
        "conditions": ["rare disease subtype"],
        "interventions": [{"id": 1, "name": "drug with space", "type": null, "description": null, "other_names": []}]
    }))
    .expect("trial fixture");
    let article = serde_json::from_value(serde_json::json!({
        "pmid": "22663011",
        "title": "BRAF article fixture",
        "author_count": 1,
        "author_completeness": "complete",
        "author_source": "pubmed"
    }))
    .expect("article fixture");
    let adverse_event = serde_json::from_value(serde_json::json!({
        "report_id": "10329882",
        "drug": "drug with space",
        "reactions": ["Cough"],
        "serious": false
    }))
    .expect("adverse-event fixture");
    let diagnostic = serde_json::from_value(serde_json::json!({
        "source": "gtr",
        "source_id": "GTR000000001.1",
        "accession": "GTR000000001.1",
        "name": "Diagnostic fixture"
    }))
    .expect("diagnostic fixture");
    let protein = serde_json::from_value(serde_json::json!({
        "accession": "P15056",
        "name": "BRAF protein",
        "gene_symbol": "BRAF"
    }))
    .expect("protein fixture");
    let pgx = serde_json::from_value(serde_json::json!({
        "query": "drug with space",
        "gene": "CYP2D6",
        "drug": "drug with space"
    }))
    .expect("PGx fixture");
    let pathway = serde_json::from_value(serde_json::json!({
        "source": "Reactome",
        "id": "R-HSA-5673001",
        "name": "RAF/MAP kinase cascade"
    }))
    .expect("pathway fixture");

    let author_id: crate::entities::author::ProviderAuthorId =
        "semanticscholar:1716151".parse().expect("author ID");
    let author = crate::entities::author::AuthorDetail {
        identity: crate::entities::author::AuthorIdentity::ExactProvider {
            id: author_id.clone(),
        },
        display_name: "A. Butte".into(),
        provider_records: vec![crate::entities::author::ProviderAuthorRecord {
            id: author_id,
            source: "semantic_scholar",
            status: crate::entities::author::ProviderStatus::Available,
        }],
        affiliations: vec![],
        paper_count: None,
        citation_count: None,
        h_index: None,
        conflicts: vec![],
        warnings: vec![],
        _meta: crate::entities::author::AuthorMeta {
            source_status: vec![],
            evidence_urls: vec![],
            next_commands: vec!["biomcp author papers semanticscholar:1716151".to_string()],
        },
    };

    vec![
        (
            "variant",
            variant_markdown(&variant, &[]).expect("variant card"),
        ),
        ("gene", gene_markdown(&gene, &[]).expect("gene card")),
        (
            "disease",
            disease_markdown(&disease, &[]).expect("disease card"),
        ),
        ("drug", drug_markdown(&drug, &[]).expect("drug card")),
        ("trial", trial_markdown(&trial, &[]).expect("trial card")),
        (
            "article",
            article_markdown(&article, &[]).expect("article card"),
        ),
        ("author", author_detail_markdown(&author)),
        (
            "adverse-event",
            adverse_event_markdown(&adverse_event, &[]).expect("adverse-event card"),
        ),
        (
            "diagnostic",
            diagnostic_markdown(&diagnostic, &[]).expect("diagnostic card"),
        ),
        (
            "protein",
            protein_markdown(&protein, &[]).expect("protein card"),
        ),
        ("PGx", pgx_markdown(&pgx, &[]).expect("PGx card")),
        (
            "pathway",
            pathway_markdown(&pathway, &[]).expect("pathway card"),
        ),
    ]
}

#[test]
fn detail_card_commands_parse() {
    for (family, markdown) in render_fixtures() {
        parse_printed_commands(&markdown)
            .unwrap_or_else(|error| panic!("{family} detail card: {error}\n{markdown}"));
    }
}

#[test]
fn detail_card_contract_rejects_malformed_command() {
    let malformed = "See also: `biomcp get --definitely-not-a-real-option value`";
    assert_eq!(
        printed_commands(malformed).len(),
        1,
        "the malformed command must first be extracted"
    );
    assert!(
        parse_printed_commands(malformed).is_err(),
        "the extracted malformed command must be rejected"
    );
}

#[test]
fn recovery_commands_round_trip_to_their_registered_routes() {
    use crate::entities::section_outcome::{SectionOutcome, SectionOutcomes};

    for row in crate::entities::source_state_registry::SOURCE_STATE_ROWS {
        let identity = "BR` AF;&";
        let mut outcomes = SectionOutcomes::with_keys(&[row.key]);
        let key = row.key;
        outcomes.complete(key, SectionOutcome::unavailable("provider unavailable"));
        let commands =
            crate::render::markdown::section_recovery_commands(row.entity, identity, &outcomes);
        assert_eq!(commands.len(), 1, "{}/{}", row.entity, row.key);
        let command = &commands[0];
        let markdown = format!(
            "Retry: {}",
            crate::render::markdown::markdown_command_code_span(command)
        );
        let extracted = printed_commands(&markdown);
        assert_eq!(extracted, commands);
        let argv = shlex::split(command).expect("valid shell command");
        Cli::try_parse_from(argv).unwrap_or_else(|error| {
            panic!(
                "{}/{} recovery does not parse: {command}: {error}",
                row.entity, row.key
            )
        });
    }
}
