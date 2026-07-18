//! Tier 3 — local GTR parsing and write validation. Pure: uses in-memory fixtures
//! and temporary directories. No network, no process env mutation.

use super::super::*;
use crate::test_support::TempDirGuard;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;
use std::path::Path;

pub(super) fn gzip_bytes(payload: &str) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(payload.as_bytes())
        .expect("write gzip fixture");
    encoder.finish().expect("finish gzip fixture")
}

pub(super) fn test_version_gz_bytes() -> Vec<u8> {
    let payload = "test_accession_ver\tnow_current\tlab_test_name\tmanufacturer_test_name\ttest_type\tname_of_laboratory\tname_of_institution\tCLIA_number\tstate_licenses\tfacility_country\ttest_currStat\ttest_pubStat\tmethod_categories\tmethods\tgenes\tcondition_identifiers\n\
GTR000000001.1\t1\tBRCA1 Hereditary Cancer Panel\tOncoPanel BRCA1\tmolecular\tGenomOncology Lab\tGenomOncology Institute\t12D3456789\tNY|CA\tUSA\tCurrent\tPublic\tMolecular genetics\tSequence analysis|Deletion/duplication analysis\tBRCA1|BARD1\tOMIM:604370\n\
GTR000000002.1\t1\tEGFR Melanoma Molecular Assay\tPrecision EGFR\tmolecular\tPrecision Diagnostics\tPrecision Health\t34D5678901\tCA\tUSA\tCurrent\tPublic\tMolecular genetics\tTargeted variant analysis\tEGFR\tDOID:1909\n\
GTR000000099.1\t0\tLegacy Retired Test\tLegacyCo\tmolecular\tLegacy Lab\tLegacy Institute\t00D0000000\tMA\tUSA\tRetired\tPrivate\tMolecular genetics\tSanger sequencing\tRET\tOMIM:123456\n";
    gzip_bytes(payload)
}

fn refreshed_test_version_gz_bytes() -> Vec<u8> {
    let payload = "test_accession_ver\tnow_current\tlab_test_name\tmanufacturer_test_name\ttest_type\tname_of_laboratory\tname_of_institution\tCLIA_number\tstate_licenses\tfacility_country\ttest_currStat\ttest_pubStat\tmethod_categories\tmethods\tgenes\tcondition_identifiers\n\
GTR000000001.1\t1\tBRCA1 Refreshed Cancer Panel\tOncoPanel BRCA1\tmolecular\tGenomOncology Lab\tGenomOncology Institute\t12D3456789\tNY|CA\tUSA\tCurrent\tPublic\tMolecular genetics\tSequence analysis|Deletion/duplication analysis\tBRCA1|BARD1\tOMIM:604370\n\
GTR000000002.1\t1\tEGFR Melanoma Molecular Assay\tPrecision EGFR\tmolecular\tPrecision Diagnostics\tPrecision Health\t34D5678901\tCA\tUSA\tCurrent\tPublic\tMolecular genetics\tTargeted variant analysis\tEGFR\tDOID:1909\n\
GTR000000099.1\t0\tLegacy Retired Test\tLegacyCo\tmolecular\tLegacy Lab\tLegacy Institute\t00D0000000\tMA\tUSA\tRetired\tPrivate\tMolecular genetics\tSanger sequencing\tRET\tOMIM:123456\n";
    gzip_bytes(payload)
}

pub(super) fn condition_gene_bytes() -> Vec<u8> {
    b"#accession_version\tobject\tobject_name\n\
GTR000000001.1\tgene\tBRCA1\n\
GTR000000001.1\tgene\tBARD1\n\
GTR000000001.1\tcondition\tHereditary breast ovarian cancer syndrome\n\
GTR000000001.1\tcondition\tBreast cancer\n\
GTR000000002.1\tgene\tEGFR\n\
GTR000000002.1\tcondition\tCutaneous melanoma\n\
GTR000000099.1\tgene\tRET\n\
GTR000000099.1\tcondition\tLegacy syndrome\n"
        .to_vec()
}

fn condition_gene_bytes_with_test_type() -> Vec<u8> {
    b"#accession_version\ttest_type\tobject\tobject_name\n\
GTR000000001.1\tClinical\tgene\tBRCA1\n\
GTR000000001.1\tClinical\tcondition\tBreast cancer\n"
        .to_vec()
}

fn live_like_test_version_gz_bytes() -> Vec<u8> {
    let payload = "test_accession_ver\tname_of_laboratory\tname_of_institution\tfacility_country\tCLIA_number\tstate_licenses\tlab_test_name\tmanufacturer_test_name\tmethod_categories\tmethods\tgenes\tnow_current\ttest_currStat\ttest_pubStat\n\
GTR000000001.1\tGenomOncology Lab\tGenomOncology Institute\tUSA\t12D3456789\tNY|CA\tBRCA1 Hereditary Cancer Panel\t\tSequence analysis\tBi-directional Sanger Sequence Analysis\tBRCA1\t1\tCurrent\tPublic\n";
    gzip_bytes(payload)
}

fn write_valid_fixture_pair(root: &Path) {
    std::fs::write(root.join(GTR_TEST_VERSION_FILE), test_version_gz_bytes())
        .expect("write test_version.gz");
    std::fs::write(root.join(GTR_CONDITION_GENE_FILE), condition_gene_bytes())
        .expect("write test_condition_gene.txt");
}

#[test]
fn parse_test_version_filters_to_current_only() {
    let records =
        parse_test_version_records_from_gzip_bytes(&test_version_gz_bytes()).expect("parse");

    assert_eq!(records.len(), 2);
    assert!(records.contains_key("GTR000000001.1"));
    assert!(records.contains_key("GTR000000002.1"));
    assert!(!records.contains_key("GTR000000099.1"));
    assert_eq!(
        records["GTR000000001.1"].methods,
        vec!["Sequence analysis", "Deletion/duplication analysis"]
    );
}

#[test]
fn parse_test_version_accepts_live_header_without_test_type() {
    let records = parse_test_version_records_from_gzip_bytes(&live_like_test_version_gz_bytes())
        .expect("live-like parse");

    assert_eq!(records.len(), 1);
    assert_eq!(records["GTR000000001.1"].test_type, "");
}

#[test]
fn parse_condition_gene_joins_correctly() {
    let (genes_by_id, conditions_by_id, test_types_by_id) =
        parse_condition_gene_links_bytes(&condition_gene_bytes()).expect("parse");

    assert_eq!(
        genes_by_id["GTR000000001.1"],
        vec!["BRCA1".to_string(), "BARD1".to_string()]
    );
    assert_eq!(
        conditions_by_id["GTR000000001.1"],
        vec![
            "Hereditary breast ovarian cancer syndrome".to_string(),
            "Breast cancer".to_string()
        ]
    );
    assert!(test_types_by_id.is_empty());
}

#[test]
fn load_index_unions_linked_and_inline_genes() {
    let root = TempDirGuard::new("gtr-load-index");
    write_valid_fixture_pair(root.path());

    let client = GtrClient::from_root(root.path());
    let index = client.load_index().expect("load index");

    assert_eq!(
        index.merged_genes("GTR000000001.1"),
        vec!["BRCA1".to_string(), "BARD1".to_string()]
    );
    assert_eq!(
        index.conditions("GTR000000002.1"),
        vec!["Cutaneous melanoma".to_string()]
    );
    assert!(index.record("GTR000000001.1").is_some());
}

#[test]
fn merged_genes_deduplicates_symbol_colon_description_form() {
    let mut index = GtrIndex::default();
    index
        .genes_by_id
        .insert("GTR000000003.1".to_string(), vec!["BRAF".to_string()]);
    index.records_by_id.insert(
        "GTR000000003.1".to_string(),
        GtrRecord {
            accession: "GTR000000003.1".to_string(),
            lab_test_name: "Broad Hereditary Cancer Panel".to_string(),
            manufacturer_test_name: String::new(),
            test_type: "molecular".to_string(),
            name_of_laboratory: String::new(),
            name_of_institution: String::new(),
            clia_number: String::new(),
            state_licenses: String::new(),
            facility_country: String::new(),
            test_curr_stat: "Current".to_string(),
            test_pub_stat: "Public".to_string(),
            method_categories: Vec::new(),
            methods: Vec::new(),
            genes: vec![
                "BRAF:B-Raf proto-oncogene, serine/threonine kinase".to_string(),
                "BRAF".to_string(),
                "ATM".to_string(),
                ":orphan-gene".to_string(),
            ],
        },
    );

    assert_eq!(
        index.merged_genes("GTR000000003.1"),
        vec!["BRAF".to_string(), "ATM".to_string()]
    );
}

#[test]
fn load_index_backfills_test_type_from_condition_gene_when_test_version_omits_it() {
    let root = TempDirGuard::new("gtr-load-index-live-like");
    std::fs::write(
        root.path().join(GTR_TEST_VERSION_FILE),
        live_like_test_version_gz_bytes(),
    )
    .expect("write live-like gzip");
    std::fs::write(
        root.path().join(GTR_CONDITION_GENE_FILE),
        condition_gene_bytes_with_test_type(),
    )
    .expect("write live-like tsv");

    let client = GtrClient::from_root(root.path());
    let index = client.load_index().expect("load index");

    assert_eq!(
        index
            .record("GTR000000001.1")
            .map(|record| record.test_type.as_str()),
        Some("Clinical")
    );
    assert_eq!(
        index.test_types_by_id.get("GTR000000001.1"),
        Some(&vec!["Clinical".to_string()])
    );
}

#[test]
fn validate_test_version_rejects_missing_header() {
    let payload = "test_accession_ver\tlab_test_name\nGTR000000001.1\tPanel\n";
    let invalid = gzip_bytes(payload);

    let err = validate_test_version_payload(&invalid).expect_err("missing header should fail");
    assert!(format!("{err:?}").contains("now_current"));
}

#[test]
fn validate_condition_gene_rejects_missing_header() {
    let invalid = b"accession_version\tobject\tobject_name\nGTR1\tgene\tBRCA1\n";

    let err = validate_condition_gene_payload(invalid).expect_err("missing header should fail");
    assert!(format!("{err:?}").contains("#accession_version"));
}

#[tokio::test]
async fn write_validated_pair_preserves_existing_files_when_validation_fails() {
    let root = TempDirGuard::new("gtr-validated-pair");
    write_valid_fixture_pair(root.path());
    let original_test_version =
        std::fs::read(root.path().join(GTR_TEST_VERSION_FILE)).expect("read original gzip");
    let original_condition_gene =
        std::fs::read(root.path().join(GTR_CONDITION_GENE_FILE)).expect("read original tsv");

    let invalid = b"not-a-gzip-payload".to_vec();
    let err = write_validated_pair(root.path(), &invalid, &condition_gene_bytes())
        .await
        .expect_err("invalid pair should fail");
    assert!(format!("{err:?}").contains(GTR_TEST_VERSION_FILE));

    assert_eq!(
        std::fs::read(root.path().join(GTR_TEST_VERSION_FILE)).expect("gzip unchanged"),
        original_test_version
    );
    assert_eq!(
        std::fs::read(root.path().join(GTR_CONDITION_GENE_FILE)).expect("tsv unchanged"),
        original_condition_gene
    );
}

#[tokio::test]
async fn write_validated_pair_rolls_back_first_file_when_second_write_fails() {
    let root = TempDirGuard::new("gtr-validated-pair-rollback");
    write_valid_fixture_pair(root.path());
    let original_test_version =
        std::fs::read(root.path().join(GTR_TEST_VERSION_FILE)).expect("read original gzip");

    std::fs::remove_file(root.path().join(GTR_CONDITION_GENE_FILE))
        .expect("remove condition gene fixture");
    std::fs::create_dir(root.path().join(GTR_CONDITION_GENE_FILE))
        .expect("create directory collision");

    let err = write_validated_pair(
        root.path(),
        &refreshed_test_version_gz_bytes(),
        &condition_gene_bytes(),
    )
    .await
    .expect_err("second write failure should error");
    assert!(
        err.to_string().contains("directory")
            || err.to_string().contains("Is a directory")
            || err.to_string().contains("Access is denied"),
        "unexpected error: {err}"
    );
    assert_eq!(
        std::fs::read(root.path().join(GTR_TEST_VERSION_FILE)).expect("gzip rolled back"),
        original_test_version
    );
}
