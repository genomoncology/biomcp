//! Recorded-reply tests for brand drug lookups (ticket 2037): the guarded
//! discover rescue before a name-miss refusal, and the brand card named for
//! its own product row.

use super::*;

/// The live OLS4 answer for `q=Tarceva`, recorded 2026-10-09 (ticket 2037)
/// and trimmed to the fields the discover pipeline reads, with descriptions
/// shortened and each doc's synonym list cut to its Tarceva-bearing entries.
/// NCIT:C2693 "Erlotinib Hydrochloride" is the only exact synonym match and
/// names the drug the guarded rescue adopts.
const OLS_TARCEVA_BODY: &str = r#"{
 "response": {
  "docs": [
   {
    "iri": "http://purl.obolibrary.org/obo/DRON_00085777",
    "ontology_name": "dron",
    "ontology_prefix": "DRON",
    "short_form": "DRON_00085777",
    "obo_id": "DRON:00085777",
    "label": "erlotinib 100 MG Oral Tablet [Tarceva]",
    "description": [],
    "exact_synonyms": [],
    "type": "class"
   },
   {
    "iri": "http://purl.obolibrary.org/obo/DRON_00085778",
    "ontology_name": "dron",
    "ontology_prefix": "DRON",
    "short_form": "DRON_00085778",
    "obo_id": "DRON:00085778",
    "label": "erlotinib 150 MG Oral Tablet [Tarceva]",
    "description": [],
    "exact_synonyms": [],
    "type": "class"
   },
   {
    "iri": "http://purl.obolibrary.org/obo/DRON_00085779",
    "ontology_name": "dron",
    "ontology_prefix": "DRON",
    "short_form": "DRON_00085779",
    "obo_id": "DRON:00085779",
    "label": "erlotinib 25 MG Oral Tablet [Tarceva]",
    "description": [],
    "exact_synonyms": [],
    "type": "class"
   },
   {
    "iri": "http://purl.obolibrary.org/obo/NCIT_C37557",
    "ontology_name": "ncit",
    "ontology_prefix": "NCIT",
    "short_form": "NCIT_C37557",
    "obo_id": "NCIT:C37557",
    "label": "Bevacizumab/Erlotinib Regimen",
    "description": [
     "A regimen consisting of bevacizumab and erlotinib that may be used in the treatment of epidermal growth factor receptor  \u2026"
    ],
    "exact_synonyms": [
     "Avastin-Tarceva",
     "Avastin/Tarceva",
     "Tarceva/Avastin"
    ],
    "type": "class"
   },
   {
    "iri": "http://purl.obolibrary.org/obo/NCIT_C63503",
    "ontology_name": "ncit",
    "ontology_prefix": "NCIT",
    "short_form": "NCIT_C63503",
    "obo_id": "NCIT:C63503",
    "label": "Erlotinib/Gemcitabine Regimen",
    "description": [
     "A regimen consisting of gemcitabine and erlotinib used for the treatment of pancreatic cancer."
    ],
    "exact_synonyms": [
     "Gemcitabine-Tarceva Regimen",
     "gemcitabine-Tarceva regimen"
    ],
    "type": "class"
   },
   {
    "iri": "http://purl.obolibrary.org/obo/CHEBI_53509",
    "ontology_name": "chebi",
    "ontology_prefix": "CHEBI",
    "short_form": "CHEBI_53509",
    "obo_id": "CHEBI:53509",
    "label": "erlotinib hydrochloride",
    "description": [
     "The hydrochloride salt of erlotinib."
    ],
    "exact_synonyms": [],
    "type": "class"
   },
   {
    "iri": "http://id.nlm.nih.gov/mesh/D000069347",
    "ontology_name": "mesh",
    "ontology_prefix": "mesh",
    "short_form": "mesh_D000069347",
    "obo_id": "mesh:D000069347",
    "label": "Erlotinib Hydrochloride",
    "description": [
     "A quinazoline derivative and ANTINEOPLASTIC AGENT that functions as a PROTEIN KINASE INHIBITOR for EGFR associated tyros \u2026"
    ],
    "exact_synonyms": [],
    "type": "class"
   },
   {
    "iri": "http://purl.obolibrary.org/obo/NCIT_C160031",
    "ontology_name": "ncit",
    "ontology_prefix": "NCIT",
    "short_form": "NCIT_C160031",
    "obo_id": "NCIT:C160031",
    "label": "Erlotinib Regimen",
    "description": [
     "A regimen consisting of erlotinib that may be used in the treatment of soft tissue sarcoma, kidney, vulvar and bone canc \u2026"
    ],
    "exact_synonyms": [
     "Tarceva Regimen"
    ],
    "type": "class"
   },
   {
    "iri": "http://purl.obolibrary.org/obo/NCIT_C2693",
    "ontology_name": "ncit",
    "ontology_prefix": "NCIT",
    "short_form": "NCIT_C2693",
    "obo_id": "NCIT:C2693",
    "label": "Erlotinib Hydrochloride",
    "description": [
     "The hydrochloride salt of a quinazoline derivative with antineoplastic properties.  Competing with adenosine triphosphat \u2026"
    ],
    "exact_synonyms": [
     "Tarceva"
    ],
    "type": "class"
   }
  ],
  "numFound": 9,
  "start": 0,
  "maxScore": 1.0
 }
}"#;

/// The live openFDA label answer for the erlotinib hydrochloride search,
/// recorded 2026-10-09 (ticket 2037): the newest erlotinib record, with the
/// section text trimmed for the fixture.
const ERLOTINIB_LABEL_BODY: &str = r#"{
 "meta": {
  "results": {
   "skip": 0,
   "limit": 5,
   "total": 1
  }
 },
 "results": [
  {
   "set_id": "ab6f3cb3-34a8-4492-a5d7-fb6b055a2d6b",
   "effective_time": "20240610",
   "openfda": {
    "brand_name": [
     "ERLOTINIB HYDROCHLORIDE"
    ],
    "generic_name": [
     "ERLOTINIB HYDROCHLORIDE"
    ],
    "route": [
     "ORAL"
    ],
    "substance_name": [
     "ERLOTINIB HYDROCHLORIDE"
    ]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE Erlotinib tablets are a kinase inhibitor indicated for: The treatment of patients with metastatic non-small cell lung cancer (NSCLC) whose tumors have epidermal growth factor receptor (EGFR) exon 19 deletions or exon 21 (L858R) substitution mutations as detected by an FDA-approved test receiving first-line, maintenance, or second or greater line treatment after progression following at least one prior chemotherapy regimen. (1.1) First-line treatment of patients with locally advanced, unresectable or metastatic pancreatic cancer, in combination with gemcitabine. (1.2) Limitations of Use: Safety and efficacy of erlotinib tablets has not been established in patients with \u2026[recorded reply trimmed for the fixture]"
   ],
   "warnings_and_cautions": [
    "5 WARNINGS AND PRECAUTIONS Interstitial lung disease (ILD): Occurs in 1.1% of patients. Withhold erlotinib for acute onset of new or progressive unexplained pulmonary symptoms, such as dyspnea, cough and fever. Discontinue erlotinib if ILD is diagnosed. (5.1) Renal failure: Monitor renal function and electrolytes, particularly in patients at risk of dehydration. Withhold erlotinib for severe renal toxicity. (5.2) Hepatotoxicity: Occurs with or without hepatic impairment, including hepatic failure and hepatorenal syndrome: Monitor periodic liver testing. Withhold or discontinue erlotinib for severe or worsening liver tests. (5.3) Gastrointestinal perforations: Discontinue erlotinib. (5.4) Bul \u2026[recorded reply trimmed for the fixture]"
   ],
   "dosage_and_administration": [
    "2 DOSAGE AND ADMINISTRATION NSCLC: 150 mg orally, on an empty stomach, once daily. (2.2) Pancreatic cancer: 100 mg orally, on an empty stomach, once daily. (2.3) 2.1 Selection of Patients with Metastatic NSCLC Select patients for the treatment of metastatic NSCLC with erlotinib tablets based on the presence of EGFR exon 19 deletions or exon 21 (L858R) substitution mutations in tumor or plasma specimens [See Clinical Studies (14.1, 14.2)]. If these mutations are not detected in a plasma specimen, test tumor tissue if available. Information on FDA-approved tests for the detection of EGFR mutations in NSCLC is available at: http://www.fda.gov/CompanionDiagnostics . 2.2 Recommended Dose \u2013 NSCLC  \u2026[recorded reply trimmed for the fixture]"
   ],
   "drug_interactions": [
    "7 DRUG INTERACTIONS CYP3A4 Inhibitors Co-administration of erlotinib with a strong CYP3A4 inhibitor or a combined CYP3A4 and CYP1A2 inhibitor increased erlotinib exposure. Erlotinib is metabolized primarily by CYP3A4 and to a lesser extent by CYP1A2. Increased erlotinib exposure may increase the risk of exposure-related toxicity [see Clinical Pharmacology (12.3)] . Avoid co-administering erlotinib with strong CYP3A4 inhibitors (e.g., boceprevir, clarithromycin, conivaptan, indinavir, itraconazole, ketoconazole, lopinavir/ritonavir, nefazodone, nelfinavir, posaconazole, ritonavir, saquinavir, telithromycin, voriconazole, grapefruit or grapefruit juice) or a combined CYP3A4 and CYP1A2 inhibito \u2026[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

/// The live openFDA label answer for the niraparib search, recorded
/// The live openFDA label answer for the niraparib search, recorded
/// 2026-10-09 (ticket 2037) and repinned 2026-10-13 for ticket 2043:
/// the Akeega combination record promoted ahead of Zejula's own record,
/// the order openFDA serves whenever Akeega's label is the newer one,
/// so the fixture pins the choice rule instead of that day's
/// effective_time order. Section text trimmed for the fixture.
const NIRAPARIB_LABEL_BODY: &str = r#"{
 "meta": {
  "results": {
   "skip": 0,
   "limit": 5,
   "total": 2
  }
 },
 "results": [
  {
   "set_id": "8245a990-3268-4613-b5c5-9537858a1eb9",
   "effective_time": "20251218",
   "openfda": {
    "brand_name": [
     "AKEEGA"
    ],
    "generic_name": [
     "NIRAPARIB TOSYLATE MONOHYDRATE AND ABIRATERONE ACETATE"
    ],
    "route": [
     "ORAL"
    ],
    "substance_name": [
     "NIRAPARIB TOSYLATE MONOHYDRATE",
     "ABIRATERONE ACETATE"
    ]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE AKEEGA with prednisone is indicated for the treatment of adult patients with deleterious or suspected deleterious BRCA2 -mutated ( BRCA2 m) metastatic castration-sensitive prostate cancer (mCSPC). AKEEGA with prednisone is indicated for the treatment of adult patients with deleterious or suspected deleterious BRCA -mutated ( BRCA m) metastatic castration-resistant prostate cancer (mCRPC). Select patients for therapy based on an FDA-approved test for AKEEGA [see Dosage and Administration (2.1) ] . AKEEGA is a combination of niraparib, a poly (ADP-ribose) polymerase (PARP) inhibitor, and abiraterone acetate, a CYP17 inhibitor indicated with prednisone for the treatment  …[recorded reply trimmed for the fixture]"
   ],
   "warnings_and_cautions": [
    "5 WARNINGS AND PRECAUTIONS Myelodysplastic Syndrome/Acute Myeloid Leukemia (MDS/AML) : MDS/AML, including a case with fatal outcome, has been observed in patients treated with AKEEGA. Monitor patients for hematological toxicity and discontinue if MDS/AML is confirmed. ( 5.1 ) Myelosuppression: Test complete blood counts weekly for the first month, every two weeks for the next two months, monthly for the remainder of the first year, then every other month, and as clinically indicated. ( 2.3 , 5.2 ) Hypokalemia, Fluid Retention, and Cardiovascular Adverse Reactions: Monitor patients for hypertension, hypokalemia, and fluid retention at least weekly for the first two months, then once a month.  …[recorded reply trimmed for the fixture]"
   ],
   "dosage_and_administration": [
    "2 DOSAGE AND ADMINISTRATION BRCA2 m mCSPC: The recommended dosage of AKEEGA is 200 mg niraparib/1,000 mg abiraterone acetate orally once daily in combination with 5 mg prednisone daily until disease progression or unacceptable toxicity. ( 2.2 ) BRCA m mCRPC : The recommended dosage of AKEEGA is 200 mg niraparib/1,000 mg abiraterone acetate orally once daily in combination with 10 mg prednisone daily until disease progression or unacceptable toxicity. ( 2.2 ) Patients receiving AKEEGA should also receive a gonadotropin-releasing hormone (GnRH) analog concurrently or should have had bilateral orchiectomy. ( 2.2 ) Take AKEEGA on an empty stomach at least one hour before or two hours after food. …[recorded reply trimmed for the fixture]"
   ],
   "drug_interactions": [
    "7 DRUG INTERACTIONS Strong CYP3A4 Inducers: Avoid coadministration. ( 7.1 ) CYP2D6 Substrates: Avoid coadministration of AKEEGA with CYP2D6 substrates for which minimal changes in concentration may lead to serious toxicities. If alternative treatments cannot be used, consider a dose reduction of the concomitant CYP2D6 substrate. ( 7.2 ) 7.1 Effect of Other Drugs on AKEEGA Effect of CYP3A4 Inducers Avoid coadministration with strong CYP3A4 inducers [see Clinical Pharmacology (12.3) ] . Abiraterone is a substrate of CYP3A4. Strong CYP3A4 inducers may decrease abiraterone concentrations [see Clinical Pharmacology (12.3) ], which may reduce the effectiveness of abiraterone. 7.2 Effects of AKEEGA …[recorded reply trimmed for the fixture]"
   ]
  },
  {
   "set_id": "b7f675e2-159c-490c-b6f4-3f16d9492b7d",
   "effective_time": "20260728",
   "openfda": {
    "brand_name": [
     "ZEJULA"
    ],
    "generic_name": [
     "NIRAPARIB"
    ],
    "route": [
     "ORAL"
    ],
    "substance_name": [
     "NIRAPARIB TOSYLATE"
    ]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE ZEJULA is a poly (ADP-ribose) polymerase (PARP) inhibitor indicated: • for the maintenance treatment of adult patients with advanced epithelial ovarian, fallopian tube, or primary peritoneal cancer who are in a complete or partial response to first-line platinum-based chemotherapy and whose cancer is associated with homologous recombination deficiency (HRD)-positive status defined by either: o a deleterious or suspected deleterious BRCA mutation, and/or o genomic instability. Select patients for therapy based on an FDA‑authorized companion diagnostic for ZEJULA. ( 1.1 , 2.1 ) • for the maintenance treatment of adult patients with deleterious or suspected deleterious g …[recorded reply trimmed for the fixture]"
   ],
   "warnings_and_cautions": [
    "5 WARNINGS AND PRECAUTIONS • Myelodysplastic Syndrome/Acute Myeloid Leukemia (MDS/AML): MDS/AML occurred in patients exposed to ZEJULA, and some cases were fatal. Monitor patients for hematological toxicity and discontinue if MDS/AML is confirmed. ( 5.1 ) • Bone Marrow Suppression: Test complete blood counts weekly for the first month, monthly for the next 11 months, and periodically thereafter for clinically significant changes. ( 5.2 ) • Hypertension and Cardiovascular Effects: Monitor blood pressure and heart rate at least weekly for the first 2 months, then monthly for the first year and periodically thereafter during treatment with ZEJULA. Manage with antihypertensive medications and ad …[recorded reply trimmed for the fixture]"
   ],
   "dosage_and_administration": [
    "2 DOSAGE AND ADMINISTRATION • First ‑ Line Maintenance Treatment of HRD ‑ Positive Advanced Ovarian Cancer: o For patients weighing <77 kg (<170 lbs) OR with a platelet count <150,000/mcL, the recommended dosage is 200 mg taken orally once daily. ( 2.2 ) o For patients weighing ≥77 kg (≥170 lbs) AND a platelet count ≥150,000/mcL, the recommended dosage is 300 mg taken orally once daily. ( 2.2 ) • Maintenance Treatment of Recurrent Germline BRCA ‑ Mutated Ovarian Cancer: The recommended dosage is 300 mg taken orally once daily. ( 2.2 ) • Continue treatment until disease progression or unacceptable toxicity. ( 2.2 ) • ZEJULA may be taken with or without food. ( 2.2 ) • For adverse reactions, c …[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

#[tokio::test]
#[serial_test::serial(source_env)]
async fn tarceva_resolves_erlotinib_through_the_guarded_discover_rescue() {
    let (base, server) = name_resolution_fixture_server(
        vec![
            (
                "Tarceva".to_string(),
                crate::transform::drug::name_resolution_tests::TARCEVA_CAPTURE.to_string(),
            ),
            (
                "erlotinib hydrochloride".to_string(),
                crate::transform::drug::name_resolution_tests::ERLOTINIB_HYDROCHLORIDE_CAPTURE
                    .to_string(),
            ),
        ],
        vec![(
            "erlotinib hydrochloride".to_string(),
            ERLOTINIB_LABEL_BODY.to_string(),
        )],
        vec![("Tarceva".to_string(), OLS_TARCEVA_BODY.to_string())],
    )
    .await;

    // Ticket 2037: MyChem holds Tarceva only on a record with no name and
    // openFDA holds no Tarceva label, so the guarded discover rescue must
    // run before the name-miss refusal. The rescue's top concept is the
    // merged erlotinib hydrochloride entry (CHEBI:53509 label spelling),
    // and the card carries the erlotinib label.
    let drug = name_resolution_fixture_drug(&base, "Tarceva").await;
    assert_eq!(drug.name, "erlotinib hydrochloride");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB00530"));
    assert_eq!(
        drug.label_set_id.as_deref(),
        Some("ab6f3cb3-34a8-4492-a5d7-fb6b055a2d6b")
    );
    let outcome = drug
        .section_outcomes
        .get("label")
        .expect("label outcome completed");
    assert_eq!(
        outcome.outcome(),
        crate::entities::section_outcome::SectionOutcomeState::Data
    );
    assert!(
        drug.label.as_ref().is_some_and(|label| {
            !label.indication_summary.is_empty()
                || label
                    .indications
                    .as_deref()
                    .is_some_and(|text| text.contains("erlotinib"))
        }),
        "the erlotinib label text reached the card"
    );
    server.abort();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn zejula_returns_niraparib_with_the_zejula_label() {
    let (base, server) = name_resolution_fixture_server(
        vec![(
            "Zejula".to_string(),
            crate::transform::drug::name_resolution_tests::ZEJULA_CAPTURE.to_string(),
        )],
        vec![("niraparib".to_string(), NIRAPARIB_LABEL_BODY.to_string())],
        Vec::new(),
    )
    .await;

    // Ticket 2037: the Zejula card takes the name its own product row pairs
    // (niraparib), never the Akeega combination row that sits first, and
    // ticket 2043: the label choice picks Zejula's own record out of the
    // Akeega-first page, because only ZEJULA's brand name equals the query
    // and the Akeega generic name is a combination.
    let drug = name_resolution_fixture_drug(&base, "Zejula").await;
    assert_eq!(drug.name, "niraparib");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB11793"));
    assert_eq!(
        drug.label_set_id.as_deref(),
        Some("b7f675e2-159c-490c-b6f4-3f16d9492b7d")
    );
    assert!(
        !drug.name.contains("abiraterone"),
        "the card name never takes the Akeega combination row"
    );
    let outcome = drug
        .section_outcomes
        .get("label")
        .expect("label outcome completed");
    assert_eq!(
        outcome.outcome(),
        crate::entities::section_outcome::SectionOutcomeState::Data
    );
    assert!(
        drug.label.as_ref().is_some_and(|label| label
            .indication_summary
            .iter()
            .any(|row| row.name.contains("ovarian")))
            || drug.label.as_ref().is_some_and(|label| label
                .indications
                .as_deref()
                .is_some_and(|text| text.contains("ZEJULA"))),
        "the Zejula label text reached the card"
    );
    server.abort();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn niraparib_names_the_plain_card_and_takes_its_own_label() {
    let (base, server) = name_resolution_fixture_server(
        vec![(
            "niraparib".to_string(),
            crate::transform::drug::name_resolution_tests::NIRAPARIB_CAPTURE.to_string(),
        )],
        vec![("niraparib".to_string(), NIRAPARIB_LABEL_BODY.to_string())],
        Vec::new(),
    )
    .await;

    // Ticket 2043: the ingredient query names the card for the plain
    // ingredient — never the Akeega combination rows MyChem lists first on
    // the same record — and the label choice picks the record whose generic
    // name is exactly niraparib out of the Akeega-first page.
    let drug = name_resolution_fixture_drug(&base, "niraparib").await;
    assert_eq!(drug.name, "niraparib");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB11793"));
    assert_eq!(
        drug.label_set_id.as_deref(),
        Some("b7f675e2-159c-490c-b6f4-3f16d9492b7d")
    );
    assert!(
        !drug.name.contains("abiraterone"),
        "the card never takes the Akeega combination row's name"
    );
    let outcome = drug
        .section_outcomes
        .get("label")
        .expect("label outcome completed");
    assert_eq!(
        outcome.outcome(),
        crate::entities::section_outcome::SectionOutcomeState::Data
    );
    assert!(
        drug.label.as_ref().is_some_and(|label| label
            .indication_summary
            .iter()
            .any(|row| row.name.contains("ovarian"))
            || drug.label.as_ref().is_some_and(|label| label
                .indications
                .as_deref()
                .is_some_and(|text| text.contains("ZEJULA")))),
        "the plain niraparib label text reached the card"
    );
    server.abort();
}

/// Two exact canonical drugs answering one brand: the discover rescue must
/// refuse to pick between them, so `get` keeps the honest no-match refusal.
/// This pin bites when the rescue guard's competing-exact check is removed
/// (ticket 2037).
#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_brand_two_exact_drugs_answer_refuses_instead_of_picking_one() {
    let ambiguous_ols = r#"{"response":{"docs":[
        {"iri":"http://purl.obolibrary.org/obo/CHEBI_999991","ontology_prefix":"CHEBI","obo_id":"CHEBI:999991","label":"Fixture Drug Aaa","exact_synonyms":["Ambifton"]},
        {"iri":"http://purl.obolibrary.org/obo/CHEBI_999992","ontology_prefix":"CHEBI","obo_id":"CHEBI:999992","label":"Fixture Drug Bbb","exact_synonyms":["Ambifton"]}
    ]}}"#;
    let (base, server) = name_resolution_fixture_server(
        vec![
            (
                "Ambifton".to_string(),
                r#"{"total":1,"hits":[{"_id":"C9999999","_score":17.668518}]}"#.to_string(),
            ),
            (
                "Fixture Drug Aaa".to_string(),
                r#"{"total":1,"hits":[{"_id":"aaa","_score":10.0,"drugbank":{"id":"DBAAA","name":"Fixture Drug Aaa"}}]}"#
                    .to_string(),
            ),
        ],
        Vec::new(),
        vec![("Ambifton".to_string(), ambiguous_ols.to_string())],
    )
    .await;

    let root = crate::test_support::TempDirGuard::new("ambifton-ddinter");
    let missing_ddinter = root.path().join("missing-ddinter");
    let cache_root = crate::test_support::TempDirGuard::new("ambifton-cache");
    let _cache_mode = crate::sources::test_cache_mode::off();
    let mut env = RequiredLabelFixtureEnv(Vec::new());
    env.set(
        "BIOMCP_CACHE_DIR",
        cache_root.path().to_string_lossy().as_ref(),
    );
    env.set("BIOMCP_MYCHEM_BASE", &format!("{base}/v1"));
    env.set("BIOMCP_OPENFDA_BASE", &base);
    env.set("BIOMCP_OLS4_BASE", &base);
    env.set("BIOMCP_HPO_BASE", &format!("{base}/hp"));
    env.set("BIOMCP_UMLS_BASE", &format!("{base}/umls"));
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &base);
    env.set(
        "BIOMCP_DDINTER_DIR",
        missing_ddinter.to_str().expect("UTF-8 fixture path"),
    );

    // With the guard intact the two exact canonical drugs compete, the
    // rescue declines, and the refusal names the miss. Unguarded, the
    // rescue would adopt Fixture Drug Aaa and return its card instead.
    let err = super::super::get("Ambifton", &["label".to_string()])
        .await
        .expect_err("two competing exact drugs must refuse");
    let message = err.to_string();
    assert!(
        message.contains("No drug card matches \"Ambifton\""),
        "{message}"
    );
    server.abort();
}
