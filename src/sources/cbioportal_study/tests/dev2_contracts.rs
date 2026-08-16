use super::*;

#[test]
fn list_studies_treats_a_missing_root_as_an_empty_catalog() {
    let fixture = TestStudyDir::new("missing-study-root");
    let missing = fixture.root.join("not-created");

    let studies = list_studies(&missing).expect("a missing root is an empty catalog");

    assert!(studies.is_empty());
    assert!(!missing.exists());
}

#[test]
fn patient_survival_data_requires_canonical_columns_and_filters_invalid_rows() {
    let fixture = TestStudyDir::new("patient-survival");
    let study_dir = fixture.study_path("survival_study");
    write_clinical_patients(
        &study_dir,
        &[
            "P1\t1:DECEASED\t10\t1:Recurred\t8\t1:Progressed\t7\t1:Died of disease\t10",
            "P2\t0:LIVING\t24\t0:DiseaseFree\t22\t0:No progression\t20\t0:Alive\t24",
            "P3\t0:LIVING\tNA\t0:DiseaseFree\t14\t0:No progression\t12\t0:Alive\t18",
            "P4\tUNKNOWN\t12\t0:DiseaseFree\t16\t0:No progression\t15\t0:Alive\t12",
            "P5\t1:DECEASED\t-1\t0:DiseaseFree\t16\t0:No progression\t15\t0:Alive\t12",
            "P6\t1:DECEASED\tinf\t0:DiseaseFree\t16\t0:No progression\t15\t0:Alive\t12",
            "P7\t1:DECEASED\t-Inf\t0:DiseaseFree\t16\t0:No progression\t15\t0:Alive\t12",
            "P8\t1:DECEASED\t1e9999\t0:DiseaseFree\t16\t0:No progression\t15\t0:Alive\t12",
            "P9\t1:DECEASED\t0\t0:DiseaseFree\t16\t0:No progression\t15\t0:Alive\t12",
        ],
    );

    let result = patient_survival_data(&study_dir, "os").expect("survival records");
    assert_eq!(result.len(), 3);
    assert!(matches!(
        result.get("P1").map(|row| row.status),
        Some(SurvivalStatus::Event)
    ));
    assert_eq!(result.get("P1").map(|row| row.months), Some(10.0));
    assert!(matches!(
        result.get("P2").map(|row| row.status),
        Some(SurvivalStatus::Censored)
    ));
    for patient in ["P3", "P4", "P5", "P6", "P7", "P8"] {
        assert!(!result.contains_key(patient));
    }
    assert_eq!(result.get("P9").map(|row| row.months), Some(0.0));
}
