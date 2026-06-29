use super::*;

#[test]
fn search_entity_json_next_commands_matrix_covers_protein_and_phenotype() {
    let cases = [
        (
            "protein-search",
            crate::cli::search_json(
                vec![crate::entities::protein::ProteinSearchResult {
                    accession: "P00533".to_string(),
                    uniprot_id: "EGFR_HUMAN".to_string(),
                    name: "Epidermal growth factor receptor".to_string(),
                    gene_symbol: Some("EGFR".to_string()),
                    species: Some("Homo sapiens".to_string()),
                }],
                crate::cli::PaginationMeta::cursor(0, 1, 1, Some(1), None),
            )
            .expect("protein search json"),
        ),
        (
            "phenotype-search",
            crate::cli::search_json(
                vec![crate::entities::disease::PhenotypeSearchResult {
                    disease_id: "MONDO:0100135".to_string(),
                    disease_name: "Dravet syndrome".to_string(),
                    score: 15.036,
                }],
                crate::cli::PaginationMeta::offset(0, 1, 1, Some(1)),
            )
            .expect("phenotype search json"),
        ),
    ];

    let mut failures = Vec::new();
    for (label, json) in cases {
        let value: serde_json::Value = serde_json::from_str(&json)
            .unwrap_or_else(|err| panic!("{label}: invalid json: {err}"));
        match value
            .get("_meta")
            .and_then(|meta| meta.get("next_commands"))
        {
            Some(commands) if commands.as_array().is_some_and(|items| !items.is_empty()) => {
                assert_json_next_commands_parse(label, &json);
            }
            _ => failures.push(format!("{label}: missing _meta.next_commands")),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("; "));
}
