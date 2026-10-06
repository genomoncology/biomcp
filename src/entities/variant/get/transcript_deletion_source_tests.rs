// Source-selection caller cases; included under the single fixture owner.
source_case!(transcript_deletion_21_version_drift, {
    let change = "c.19_20insAC";
    let mut row = coding_hit(change);
    row["snpeff"]["ann"][0]["feature_id"] = json!("NM_012345.8");
    coding_exercise(&wrapper(change), change, vec![row], json!("absent")).await;
    let input = "NM_012345.07(TP53):c.19_20insAC";
    coding_exercise(input, change, vec![coding_hit(change)], json!("absent")).await;
    let mut row = coding_hit(change);
    row["snpeff"]["ann"][0]["feature_id"] = json!("NM_012345.07");
    let mut expected = coding_card(change);
    expected["transcript"] = json!("NM_012345.07");
    coding_exercise(input, change, vec![row], expected).await;
});
source_case!(transcript_deletion_22_no_cross_tuple_join, {
    let change = "c.19_21inv";
    let mut row = coding_hit("c.19_21dup");
    let mut other = coding_hit(change)["snpeff"]["ann"][0].clone();
    other["feature_id"] = json!("NM_099999.7");
    row["snpeff"]["ann"].as_array_mut().unwrap().push(other);
    coding_exercise(&wrapper(change), change, vec![row], json!("absent")).await;
});
source_case!(transcript_deletion_23_unpaired_vectors, {
    let row = json!({"_id":"chr17:g.101C>T","dbnsfp":{"genename":["TP53"],"hgvsc":["NM_012345.7:c.17_18del"]}});
    exercise(
        &wrapper(D),
        D,
        vec![json!({"total":1,"hits":[row]})],
        json!(EVIDENCE),
        &[0],
        false,
        false,
    )
    .await;
});
source_case!(transcript_deletion_24_decoder_incomplete, {
    let mut row = hit(D);
    row["snpeff"]["ann"] = json!(vec![row["snpeff"]["ann"][0].clone(); 33]);
    exercise(
        &wrapper(D),
        D,
        vec![json!({"total":1,"hits":[row]})],
        json!(EVIDENCE),
        &[0],
        false,
        false,
    )
    .await;
});
source_case!(transcript_deletion_25_tuple_protein_conflict, {
    let mut row = hit(D);
    let mut other = row["snpeff"]["ann"][0].clone();
    other["hgvs_p"] = json!("p.Gly7del");
    row["snpeff"]["ann"].as_array_mut().unwrap().push(other);
    exercise(
        &wrapper(D),
        D,
        vec![json!({"total":1,"hits":[row]})],
        json!(EVIDENCE),
        &[0],
        false,
        false,
    )
    .await;
});
source_case!(transcript_deletion_26_whole_matched_tuple, {
    let mut row = hit(D);
    let mut other = row["snpeff"]["ann"][0].clone();
    other["feature_id"] = json!("NM_099999.1");
    other["genename"] = json!("OTHER");
    other["hgvs_p"] = json!("p.Gly7del");
    row["snpeff"]["ann"]
        .as_array_mut()
        .unwrap()
        .insert(0, other);
    row["clinvar"] =
        json!({"rcv":[{"preferred_name":"NM_099999.1(OTHER):c.17_18del (p.Gly7del)"}]});
    exercise(
        &wrapper(D),
        D,
        vec![json!({"total":1,"hits":[row]})],
        card(D),
        &[0],
        false,
        false,
    )
    .await;
    let row = json!({"_id":"chr17:g.101C>T",
        "snpeff":{"ann":[{"feature_id":"NM_099999.1","genename":"OTHER","hgvs_c":"c.17dup","hgvs_p":"p.Gly7del"}]},
        "clinvar":{"rcv":[{"preferred_name":"NM_012345.7(TP53):c.17_18del (p.Gly6Val)"}]}});
    let mut expected = card(D);
    expected["hgvs_p"] = json!("p.Gly6Val");
    expected["legacy_name"] = json!("TP53 G6V");
    exercise(
        &wrapper(D),
        D,
        vec![json!({"total":1,"hits":[row]})],
        expected,
        &[0],
        false,
        false,
    )
    .await;
});
source_case!(transcript_deletion_27_absent_protein, {
    let change = "c.19C>T";
    let mut row = rcv_hit(change);
    row["snpeff"] = json!({"ann":[{"feature_id":"NM_099999.7","genename":"OTHER","hgvs_c":"c.20C>T","hgvs_p":"p.Gly7Val"}]});
    coding_exercise(&wrapper(change), change, vec![row], coding_card(change)).await;
});
source_case!(transcript_deletion_28_duplicate_raw_counts, {
    exercise(
        &wrapper(D),
        D,
        vec![
            json!({"total":51,"hits":vec![hit(D);50]}),
            json!({"total":51,"hits":[hit(D)]}),
        ],
        card(D),
        &[0, 50],
        false,
        false,
    )
    .await;
});
source_case!(transcript_deletion_29_distinct_sources_ambiguous, {
    let change = "c.19=";
    let mut other = coding_hit(change);
    other["_id"] = json!("chr17:g.102C>T");
    coding_exercise(
        &wrapper(change),
        change,
        vec![coding_hit(change), other],
        json!(AMBIGUOUS),
    )
    .await;
});
source_case!(transcript_deletion_30_early_empty_incomplete, {
    exercise(
        &wrapper(D),
        D,
        vec![
            json!({"total":2,"hits":[hit(D)]}),
            json!({"total":2,"hits":[]}),
        ],
        json!(INCOMPLETE),
        &[0, 1],
        false,
        false,
    )
    .await;
});
source_case!(transcript_deletion_31_unknown_total_terminal, {
    exercise(
        &wrapper(D),
        D,
        vec![json!({"hits":[hit(D)]}), json!({"hits":[]})],
        card(D),
        &[0, 1],
        false,
        false,
    )
    .await;
});
source_case!(transcript_deletion_32_continuation_source_failure, {
    exercise(
        &wrapper(D),
        D,
        vec![json!({"total":2,"hits":[hit(D)]}), json!("malformed")],
        json!("source"),
        &[0, 1],
        false,
        false,
    )
    .await;
});
source_case!(transcript_deletion_33_original_boundary, {
    let change = "c.19_20insAC";
    let input = format!(
        "{}{}",
        " ".repeat(512 - wrapper(change).len()),
        wrapper(change)
    );
    exercise(
        &input,
        change,
        vec![json!({"total":0,"hits":[]})],
        json!("absent"),
        &[0],
        false,
        false,
    )
    .await;
});
source_case!(
    transcript_deletion_41_rcv_exact_version_without_borrowing,
    {
        let change = "c.19_21delinsAC";
        let mut row = coding_hit(change);
        row["snpeff"]["ann"][0]["feature_id"] = json!("NM_012345.8");
        row["clinvar"] = json!({"rcv":[{"preferred_name":"NM_012345.7(TP53):c.19_21delinsAC"}]});
        coding_exercise(&wrapper(change), change, vec![row], coding_card(change)).await;
    }
);

async fn page_conflict(reverse: bool) {
    let change = "c.(19C>T)";
    for rcv in [false, true] {
        for protein in [Some("p.Gly7Val"), None] {
            let mut first = hit(change);
            first["snpeff"]["ann"][0]["hgvs_p"] = json!(if protein.is_some() {
                "p.Gly6Val"
            } else {
                "p.Gly7Val"
            });
            first["dbnsfp"] = json!({"genename":["TP53"],"hgvsc":["NM_012345.7:c.(19C>T)"],"hgvsp":["p.Gly6Val"]});
            let mut second = first.clone();
            if rcv {
                first.as_object_mut().unwrap().remove("snpeff");
                second.as_object_mut().unwrap().remove("snpeff");
                let first_name = if protein.is_some() {
                    "NM_012345.7(TP53):c.(19C>T) (p.Gly6Val)"
                } else {
                    "NM_012345.7(TP53):c.(19C>T) (p.Gly7Val)"
                };
                first["clinvar"] = json!({"rcv":[{"preferred_name":first_name}]});
                let name = if protein.is_some() {
                    "NM_012345.7(TP53):c.(19C>T) (p.Gly7Val)"
                } else {
                    "NM_012345.7(TP53):c.(19C>T)"
                };
                second["clinvar"] = json!({"rcv":[{"preferred_name":name}]});
            } else if let Some(protein) = protein {
                second["snpeff"]["ann"][0]["hgvs_p"] = json!(protein);
            } else {
                second["snpeff"]["ann"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("hgvs_p");
            }
            let rows = if reverse {
                [second, first]
            } else {
                [first, second]
            };
            exercise(
                &wrapper(change),
                change,
                rows.into_iter()
                    .map(|row| json!({"total":2,"hits":[row]}))
                    .collect(),
                json!(EVIDENCE),
                &[0, 1],
                false,
                false,
            )
            .await;
        }
    }
}
source_case!(transcript_deletion_42_cross_page_conflict, {
    page_conflict(false).await;
});
source_case!(transcript_deletion_43_reverse_page_conflict, {
    page_conflict(true).await;
});
source_case!(transcript_deletion_46_rcv_trailing_prose, {
    let change = "c.19_21inv";
    let row = json!({"_id":"chr17:g.101C>T","clinvar":{"rcv":[{"preferred_name":"NM_012345.7(TP53):c.19_21inv trailing-prose"}]}});
    coding_exercise(&wrapper(change), change, vec![row], json!(EVIDENCE)).await;
});

// Affected population owner: the new detail format must retain rsID recovery.
source_case!(transcript_deletion_population_recovers_source_rsid, {
    for change in [D, "c.19_20insAC"] {
        let mut row = if change == D {
            hit(change)
        } else {
            coding_hit(change)
        };
        row["dbsnp"] = json!({"rsid":"rs101"});
        let (fixture, requests) = fixture(vec![
            json!({"total":1,"hits":[row]}),
            json!({"primary_snapshot_data":{"placements_with_allele":[]}}),
        ])
        .await;
        let mut env = TestEnv::new();
        env.set("BIOMCP_MYVARIANT_BASE", &fixture.base);
        env.set("BIOMCP_DBSNP_BASE", &fixture.base);
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
        env.set("BIOMCP_CACHE_MODE", "off");
        let actual = serde_json::to_value(
            variant::get(&wrapper(change), &["population".into()])
                .await
                .unwrap(),
        )
        .unwrap();
        let message = "Direct gnomAD v4 population data requires a trustworthy GRCh38 coordinate; dbSNP could not provide a compatible coordinate.";
        let mut expected = if change == D {
            card(change)
        } else {
            coding_card(change)
        };
        expected["rsid"] = json!("rs101");
        expected["population"] = json!({"status":"inapplicable","dataset":"gnomad_r4","release":"gnomAD v4","message":message,"exome":null,"genome":null,"faf_caveat":"gnomAD excludes bottlenecked genetic ancestry groups when selecting grpmax FAF."});
        expected["section_outcomes"]["population"] =
            json!({"outcome":"inapplicable","sources":[],"message":message});
        assert_eq!(actual, expected);
        let log = requests.lock().unwrap();
        assert_eq!(log.len(), 2);
        assert!(log[1].starts_with("GET /refsnp/101 HTTP/1.1\r\n"));
        assert_eq!(log[1].split_once("\r\n\r\n").unwrap().1, "");
        let first = Arc::new(Mutex::new(vec![log[0].clone()]));
        drop(log);
        ledger(&first, change, "NM_012345.7", &[0]);
    }
});
