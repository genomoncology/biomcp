use super::*;
use crate::entities::disease::PhenotypeSearchResult;
use crate::entities::protein::ProteinSearchResult;

#[test]
fn protein_search_json_next_commands_parse() {
    let results = vec![ProteinSearchResult {
        accession: "P00533".to_string(),
        uniprot_id: "EGFR_HUMAN".to_string(),
        name: "Epidermal growth factor receptor".to_string(),
        gene_symbol: Some("EGFR".to_string()),
        species: Some("Homo sapiens".to_string()),
    }];
    let pagination = crate::cli::PaginationMeta::cursor(0, 1, results.len(), Some(1), None);
    let json = crate::cli::search_json_with_meta(
        results.clone(),
        pagination,
        crate::render::markdown::search_next_commands_protein(&results),
    )
    .expect("protein search json");
    let commands = collect_next_commands(&json);

    assert_eq!(
        commands,
        vec![
            "biomcp get protein P00533".to_string(),
            "biomcp list protein".to_string(),
        ]
    );
    assert_json_next_commands_parse("protein-search", &json);
}

#[test]
fn phenotype_search_json_next_commands_parse() {
    let results = vec![PhenotypeSearchResult {
        disease_id: "MONDO:0100135".to_string(),
        disease_name: "Dravet syndrome".to_string(),
        score: 0.98,
    }];
    let pagination = crate::cli::PaginationMeta::offset(0, 1, results.len(), Some(1));
    let json = crate::cli::search_json_with_meta(
        results.clone(),
        pagination,
        crate::render::markdown::search_next_commands_phenotype(&results),
    )
    .expect("phenotype search json");
    let commands = collect_next_commands(&json);

    assert_eq!(
        commands,
        vec![
            "biomcp get disease \"Dravet syndrome\" genes phenotypes".to_string(),
            "biomcp list phenotype".to_string(),
        ]
    );
    assert!(
        !commands
            .iter()
            .any(|command| command.contains("get phenotype"))
    );
    assert_json_next_commands_parse("phenotype-search", &json);
}
