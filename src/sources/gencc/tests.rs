use std::sync::atomic::AtomicBool;

use super::model::{GenCcDataset, HEADER};

fn fixture() -> &'static [u8] {
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/sources/gencc/submissions-new-odc1.csv"
    ))
}

#[test]
fn receipt_backed_odc1_rows_remain_separate_and_ordered() {
    let dataset = GenCcDataset::parse(fixture(), &AtomicBool::new(false)).unwrap();
    let matches = dataset.matching("odc1", "HGNC:8109");
    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0].classification.code, "strong");
    assert_eq!(matches[0].mode_of_inheritance.id, "HP:0000006");
    assert_eq!(
        matches
            .iter()
            .map(|row| row.submitter.label.as_str())
            .collect::<Vec<_>>(),
        [
            "G2P",
            "Labcorp Genetics (formerly Invitae)",
            "PanelApp Australia"
        ]
    );
    assert_eq!(matches[1].id, "SGC-113621.1");
    assert_eq!(
        matches[1]
            .publications
            .iter()
            .map(|publication| publication.pmid.as_str())
            .collect::<Vec<_>>(),
        ["30239107", "30475435"]
    );
}

#[test]
fn exact_header_and_all_classification_pairs_are_closed() {
    let base = String::from_utf8(fixture().to_vec()).unwrap();
    assert_eq!(base.lines().next().unwrap(), HEADER.join(","));
    for (id, label, code) in [
        ("GENCC:100001", "Definitive", "definitive"),
        ("GENCC:100002", "Strong", "strong"),
        ("GENCC:100003", "Moderate", "moderate"),
        ("GENCC:100004", "Limited", "limited"),
        ("GENCC:100005", "Disputed Evidence", "disputed_evidence"),
        ("GENCC:100006", "Refuted Evidence", "refuted_evidence"),
        ("GENCC:100007", "Animal Model Only", "animal_model_only"),
        (
            "GENCC:100008",
            "No Known Disease Relationship",
            "no_known_disease_relationship",
        ),
        ("GENCC:100009", "Supportive", "supportive"),
    ] {
        let mut row = base
            .lines()
            .nth(1)
            .unwrap()
            .split(',')
            .map(str::to_string)
            .collect::<Vec<_>>();
        row[8] = id.into();
        row[9] = label.into();
        let csv = format!("{}\n{}\n", HEADER.join(","), row.join(","));
        let parsed = GenCcDataset::parse(csv.as_bytes(), &AtomicBool::new(false)).unwrap();
        assert_eq!(parsed.assertions()[0].classification.code, code);
    }
}

#[test]
fn malformed_classification_version_and_pmid_fail_closed() {
    let base = String::from_utf8(fixture().to_vec()).unwrap();
    for (column, value) in [
        (1, "0"),
        (1, "4294967296"),
        (8, "GENCC:999999"),
        (9, "Moderate"),
        (27, "0"),
        (27, "18446744073709551616"),
        (27, "42,,43"),
    ] {
        let mut row = base
            .lines()
            .nth(1)
            .unwrap()
            .split(',')
            .map(str::to_string)
            .collect::<Vec<_>>();
        row[column] = value.into();
        let csv = format!("{}\n{}\n", HEADER.join(","), row.join(","));
        assert!(GenCcDataset::parse(csv.as_bytes(), &AtomicBool::new(false)).is_err());
    }
}

#[test]
fn duplicate_comparison_uses_normalized_retained_tuple() {
    let base = String::from_utf8(fixture().to_vec()).unwrap();
    let row = base.lines().nth(1).unwrap();
    let identical = format!("{}\n{row}\n{row}\n", HEADER.join(","));
    assert_eq!(
        GenCcDataset::parse(identical.as_bytes(), &AtomicBool::new(false))
            .unwrap()
            .assertions()
            .len(),
        1
    );
    let mut changed = row.split(',').map(str::to_string).collect::<Vec<_>>();
    changed[13] = "Different submitter".into();
    let conflict = format!("{}\n{row}\n{}\n", HEADER.join(","), changed.join(","));
    assert!(GenCcDataset::parse(conflict.as_bytes(), &AtomicBool::new(false)).is_err());
}
