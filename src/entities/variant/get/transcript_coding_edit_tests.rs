// Independently authored synthetic MIT coding edit cases; one callable owner.
fn coding_hit(change: &str) -> Value {
    json!({"_id":"chr17:g.101C>T","snpeff":{"ann":[{"feature_id":"NM_012345.7","genename":"TP53","hgvs_c":change}]}})
}
fn rcv_hit(change: &str) -> Value {
    json!({"_id":"chr17:g.101C>T","clinvar":{"rcv":[{"preferred_name":format!("NM_012345.7(TP53):{change}")}]}})
}
pub(super) fn coding_card(change: &str) -> Value {
    let mut expected = card(change);
    expected.as_object_mut().unwrap().remove("hgvs_p");
    expected
}
async fn coding_exercise(input: &str, change: &str, rows: Vec<Value>, expected: Value) {
    let total = rows.len();
    exercise(
        input,
        change,
        vec![json!({"total":total,"hits":rows})],
        expected,
        &[0],
        false,
        false,
    )
    .await;
}
source_case!(transcript_deletion_coding_insertion, {
    let change = "c.19_20insAC";
    coding_exercise(
        &wrapper(change),
        change,
        vec![coding_hit(change)],
        coding_card(change),
    )
    .await;
});
source_case!(transcript_deletion_coding_family, {
    for (change, rcv, prefix) in [
        ("c.19_21dupACG", true, false),
        ("c.19_21delinsAC", false, true),
        ("c.19_21inv", true, false),
        ("c.19=", false, false),
        ("c.(19C>T)", true, false),
        ("c.(19_21)C>T", false, false),
        ("c.19+1G>A", false, false),
        ("c.-7dupAC", false, false),
        ("c.*19_20inv", false, false),
        ("c.2147483648_2147483649insAC", false, false),
    ] {
        let mut row = if rcv {
            rcv_hit(change)
        } else {
            coding_hit(change)
        };
        if prefix {
            row["snpeff"]["ann"][0]["hgvs_c"] = json!(format!("NM_012345.7:{change}"));
        }
        coding_exercise(&wrapper(change), change, vec![row], coding_card(change)).await;
    }
});
source_case!(transcript_deletion_coding_rcv_substitution_protein, {
    let change = "c.19C>T";
    let row = json!({"_id":"chr17:g.101C>T","clinvar":{"rcv":[{"preferred_name":"NM_012345.7(TP53):c.19C>T (p.Gly7Val)"}]}});
    let mut expected = coding_card(change);
    expected["hgvs_p"] = json!("p.Gly7Val");
    expected["legacy_name"] = json!("TP53 G7V");
    coding_exercise(&wrapper(change), change, vec![row], expected).await;
});
source_case!(transcript_deletion_coding_padding, {
    let change = "c.0019_0020insAC";
    coding_exercise(
        &wrapper(change),
        change,
        vec![coding_hit(change)],
        coding_card(change),
    )
    .await;
    let mut other = coding_hit("c.19_20insAC");
    other["_id"] = json!("chr17:g.100C>T");
    coding_exercise(
        &wrapper(change),
        change,
        vec![other.clone(), coding_hit(change)],
        coding_card(change),
    )
    .await;
    coding_exercise(&wrapper(change), change, vec![other], json!("absent")).await;
});
source_case!(transcript_deletion_coding_prediction_spelling, {
    let change = "c.(19C>T)";
    coding_exercise(
        &wrapper(change),
        change,
        vec![coding_hit("c.19C>T")],
        json!("absent"),
    )
    .await;
});
source_case!(transcript_deletion_coding_prefixed_transcript_mismatch, {
    let change = "c.19_21delinsAC";
    let mut row = coding_hit(change);
    row["snpeff"]["ann"][0]["hgvs_c"] = json!("NM_012345.8:c.19_21delinsAC");
    coding_exercise(&wrapper(change), change, vec![row], json!(EVIDENCE)).await;
});
source_case!(transcript_deletion_coding_invalid_neighbors, {
    for change in [
        "c.19insAC",
        "c.19_21insAC",
        "c.19_20ins",
        "c.19_21delins",
        "c.19_21dupac",
        "c.19_21invAC",
        "c.(19dup)",
        "c.(19_20insAC)",
        "c.(19_21delinsAC)",
        "c.(19_21inv)",
        "c.(19=)",
        "c.(19+1C>T)",
        "c.(-19C>T)",
        "c.(*19C>T)",
        "c.[19C>T;21dup]",
        "c.19AC[2]",
        "n.19dup",
        "r.19dup",
        "g.19dup",
        "NM_012345.7:c.19dup",
    ] {
        exercise(
            &wrapper(change),
            change,
            vec![],
            json!(INVALID),
            &[],
            false,
            false,
        )
        .await;
    }
    exercise(
        "NM_012345.7(tp53):c.19_20insAC",
        "c.19_20insAC",
        vec![],
        json!(INVALID),
        &[],
        false,
        false,
    )
    .await;
});
source_case!(transcript_deletion_coding_prepared_debug_is_private, {
    let prepared = crate::entities::variant::resolution::transcript_deletion_get::prepare(
        "NM_012345.7(TP53):c.19_20insAC",
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        format!("{prepared:?}"),
        "TranscriptDeletion { transcript_bytes: 11, gene_bytes: 4, change_bytes: 12 }"
    );
});
