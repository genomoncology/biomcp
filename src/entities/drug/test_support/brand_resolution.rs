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
/// 2026-10-09 (ticket 2037) and repinned 2026-10-10 for ticket 2043:
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

    let (_env, _ddinter) = super::drug_fixture_env(base.as_str(), "ambifton-ddinter");

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

/// The live openFDA label answer for the "Pain Relief" identity search,
/// recorded 2026-10-10 (ticket 2039) and trimmed to the openFDA blocks: no
/// label carries the shelf name as its own brand, and 1,252 labels merely
/// mention it, so no row's record owns the requested name.
const PAIN_RELIEF_LABEL_BODY: &str = r#"{
 "meta": {
  "results": {
   "skip": 0,
   "limit": 5,
   "total": 1252
  }
 },
 "results": [
  {
   "openfda": {
    "brand_name": ["4% Lidocaine Pain Relief Patch"],
    "generic_name": ["LIDOCAINE 4%"]
   }
  },
  {
   "openfda": {
    "brand_name": ["Pain Relief Extra Strength"],
    "generic_name": ["ACETAMINOPHEN"]
   }
  },
  {
   "openfda": {
    "brand_name": ["MUCINEX INSTASOOTHE SORE THROAT PLUS PAIN RELIEF"],
    "generic_name": ["HEXYLRESORCINOL"]
   }
  }
 ]
}"#;

/// The live openFDA label answer for the "Sleep Aid" identity search,
/// recorded 2026-10-10 (ticket 2039) and trimmed to the openFDA blocks: two
/// labels carry the shelf name as their own brand and pair it with
/// different ingredients (doxylamine succinate and diphenhydramine), so the
/// name owns no one drug through openFDA either.
const SLEEP_AID_LABEL_BODY: &str = r#"{
 "meta": {
  "results": {
   "skip": 0,
   "limit": 5,
   "total": 249
  }
 },
 "results": [
  {
   "openfda": {
    "brand_name": ["Nighttime Sleep Aid Berry Flavor"],
    "generic_name": ["DIPHENHYDRAMINE HYDROCHLORIDE"]
   }
  },
  {
   "openfda": {
    "brand_name": ["SLEEP AID"],
    "generic_name": ["DOXYLAMINE SUCCINATE"]
   }
  },
  {
   "openfda": {
    "brand_name": ["Sleep Aid"],
    "generic_name": ["DIPHENHYDRAMINE HCL"]
   }
  }
 ]
}"#;

/// `q=doxylamine succinate` (recorded 2026-10-11, ticket 2050): the
/// doxylamine identity record with its own pairing row, trimmed to the
/// fields the selection reads. The openFDA identity fallback orders its
/// candidates by the label page, and doxylamine succinate is the first
/// candidate for the "Sleep Aid" search above; the refusal stands only
/// because two candidates answer, so this reply exists to fail the test
/// the moment the fallback starts adopting the first candidate instead of
/// refusing.
const DOXYLAMINE_SUCCINATE_CAPTURE: &str = r#"
{
  "total": 1,
  "hits": [
    {
      "_id": "HCFDWZZGGLSKEP-UHFFFAOYSA-N",
      "_score": 56.14622,
      "drugbank": {"id": "DB00366", "name": "Doxylamine"},
      "chembl": {"molecule_chembl_id": "CHEMBL1004", "pref_name": "DOXYLAMINE"},
      "ndc": [
        {"nonproprietaryname": "Doxylamine succinate", "proprietaryname": "Sleep Aid"}
      ],
      "unii": {"unii": "XZV7JAA763", "display_name": "DOXYLAMINE"}
    }
  ]
}
"#;

/// The env a refusal test needs: the name-resolution fixture base with the
/// shared HTTP client's cache walk pointed at its own tree (ticket 2039).
async fn shared_name_refusal_drug(name: &str, base: &str) -> crate::error::BioMcpError {
    let (_env, _ddinter) = super::drug_fixture_env(&base, "shared-name-ddinter");
    super::super::get(name, &["label".to_string()])
        .await
        .expect_err("a shared product name must refuse, never fuse")
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn pain_relief_refuses_naming_the_drugs_that_share_the_name() {
    let (base, server) = name_resolution_fixture_server(
        vec![(
            "Pain Relief".to_string(),
            crate::transform::drug::shared_name_tests::PAIN_RELIEF_CAPTURE.to_string(),
        )],
        vec![(
            "Pain Relief".to_string(),
            PAIN_RELIEF_LABEL_BODY.to_string(),
        )],
        Vec::new(),
    )
    .await;

    // Ticket 2039: the shelf name brands unrelated drugs' products, MyChem
    // carries it on each of their records, and openFDA holds no label whose
    // own brand is "Pain Relief". No record owns the name, so the card
    // refuses and names drugs the text search matched instead of fusing
    // acetaminophen's identifiers with other drugs' targets.
    let err = shared_name_refusal_drug("Pain Relief", &base).await;
    let message = err.to_string();
    assert!(
        message.contains("No drug card matches \"Pain Relief\""),
        "{message}"
    );
    assert!(
        message.contains("acetaminophen"),
        "the refusal names a drug the text search matched: {message}"
    );
    server.abort();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn sleep_aid_refuses_when_openfdas_own_brands_disagree() {
    let (base, server) = name_resolution_fixture_server(
        vec![
            (
                "Sleep Aid".to_string(),
                crate::transform::drug::shared_name_tests::SLEEP_AID_CAPTURE.to_string(),
            ),
            (
                "doxylamine succinate".to_string(),
                DOXYLAMINE_SUCCINATE_CAPTURE.to_string(),
            ),
        ],
        vec![("Sleep Aid".to_string(), SLEEP_AID_LABEL_BODY.to_string())],
        Vec::new(),
    )
    .await;

    // Ticket 2039: MyChem refuses the shelf name because diphenhydramine's
    // and doxylamine's records pair it with different ingredients, and the
    // openFDA identity fallback refuses too, because its two labels that
    // carry "Sleep Aid" as their own brand pair it with different
    // ingredients. The refusal names the matched drugs. The doxylamine
    // succinate reply pins the fallback's refusal (ticket 2050): adopting
    // the first of the two candidates would resolve a card and fail this
    // test.
    let err = shared_name_refusal_drug("Sleep Aid", &base).await;
    let message = err.to_string();
    assert!(
        message.contains("No drug card matches \"Sleep Aid\""),
        "{message}"
    );
    assert!(
        message.contains("diphenhydramine"),
        "the refusal names a drug the text search matched: {message}"
    );
    server.abort();
}

/// Ticket 2050: the live sweep showed `get drug 5-FU` waiting four to six
/// seconds on one EBI OLS4 search while refusing. The refusal rescue runs
/// inside a two-second budget, so a slow discover round gives up and the
/// refusal arrives without it.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_slow_discover_round_never_holds_the_refusal_hostage() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind slow OLS4 fixture");
    let ols_base = format!(
        "http://{}",
        listener.local_addr().expect("fixture address")
    );
    let ols_server = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut request = vec![0_u8; 16 * 1024];
                let Ok(len) = stream.read(&mut request).await else {
                    return;
                };
                let _ = &request[..len];
                // Far past the rescue's budget and past the test's
                // deadline: only a bounded rescue can answer inside it.
                tokio::time::sleep(std::time::Duration::from_secs(8)).await;
                let body = r#"{"response":{"docs":[]}}"#;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes()).await;
            });
        }
    });

    let (base, server) = name_resolution_fixture_server(
        vec![(
            "Tarceva".to_string(),
            crate::transform::drug::name_resolution_tests::TARCEVA_CAPTURE.to_string(),
        )],
        Vec::new(),
        Vec::new(),
    )
    .await;
    let (mut env, _ddinter) = super::drug_fixture_env(base.as_str(), "slow-rescue-ddinter");
    env.set("BIOMCP_OLS4_BASE", &ols_base);

    let started = std::time::Instant::now();
    let err = super::super::get("Tarceva", &["label".to_string()])
        .await
        .expect_err("the text-only match still refuses");
    let elapsed = started.elapsed();
    drop(env);
    ols_server.abort();
    server.abort();

    let message = err.to_string();
    assert!(
        message.contains("No drug card matches \"Tarceva\""),
        "{message}"
    );
    assert!(
        elapsed < std::time::Duration::from_secs(5),
        "the refusal waited {elapsed:?} on a slow outside service"
    );
}

/// The openFDA label answer for the "Headache Relief" search (live total
/// 68, recorded 2026-10-11, ticket 2050), trimmed to the openFDA blocks and
/// the leading section text: the newest record is a store brand whose
/// generic is the three-ingredient line, and the exact "Headache Relief"
/// brand record (a876daa7) carries the same three ingredients.
const HEADACHE_RELIEF_LABEL_BODY: &str = r#"{
 "meta": {"results": {"skip": 0, "limit": 5, "total": 68}},
 "results": [
  {
   "set_id": "495c75bf-c09a-accd-e063-6294a90a75ec",
   "effective_time": "20260824",
   "openfda": {
    "brand_name": ["HyVee Extra Strength Headache Relief CAPLET"],
    "generic_name": ["ACETAMINOPHEN, ASPIRIN, CAFFEINE"]
   },
   "indications_and_usage": [
    "Uses • temporarily relieves minor aches and pains due to: headache, a cold, arthritis, muscular aches, toothache, premenstrual and menstrual cramps …[recorded reply trimmed for the fixture]"
   ]
  },
  {
   "set_id": "a876daa7-ade3-497d-87bb-46d6e4dc185a",
   "effective_time": "20260821",
   "openfda": {
    "brand_name": ["Headache Relief"],
    "generic_name": ["ACETAMINOPHEN, ASPIRIN, CAFFEINE"]
   },
   "indications_and_usage": [
    "Uses temporarily relieves minor aches and pains due to: headache arthritis a cold muscular aches toothache premenstrual and menstrual cramps …[recorded reply trimmed for the fixture]"
   ]
  },
  {
   "set_id": "5e77dca4-2ff8-4a8c-94bd-f224b680ca23",
   "effective_time": "20260819",
   "openfda": {
    "brand_name": ["Headache Relief Extra Strength"],
    "generic_name": ["ACETAMINOPHEN, ASPIRIN, CAFFEINE"]
   },
   "indications_and_usage": [
    "Uses temporarily relieves minor aches and pains due to: headache a cold arthritis muscular aches toothache premenstrual and menstrual cramps …[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

/// `q=headache relief` (recorded 2026-10-11, ticket 2050): the shelf name
/// brands a three-ingredient combination whose rows sit merged on the
/// acetaminophen, caffeine and aspirin records alike, and those records
/// also carry the generic-labeled "Tension Headache Relief" product row —
/// the merged product row whose name ends with the request and used to
/// name the card on main. Rows trimmed to the ones the selection reads.
const HEADACHE_RELIEF_CAPTURE: &str = r#"
{
  "total": 34,
  "hits": [
    {
      "_id": "RZVAJINKPMORJF-UHFFFAOYSA-N",
      "_score": 49.638573,
      "drugbank": {"id": "DB00316", "name": "Acetaminophen"},
      "chembl": {"molecule_chembl_id": "CHEMBL112", "pref_name": "ACETAMINOPHEN"},
      "ndc": [
        {"nonproprietaryname": "Tension Headache Relief", "proprietaryname": "Tension Headache Relief"},
        {"nonproprietaryname": "Acetaminophen, Aspirin and Caffeine", "proprietaryname": "Headache Relief"},
        {"nonproprietaryname": "Acetaminophen", "proprietaryname": "NOTTS - Muscle Aches and Pain"}
      ],
      "unii": {"unii": "362O9ITL9D", "display_name": "ACETAMINOPHEN"}
    },
    {
      "_id": "RYYVLZVUVIJVGH-UHFFFAOYSA-N",
      "_score": 49.59012,
      "drugbank": {"id": "DB00201", "name": "Caffeine"},
      "chembl": {"molecule_chembl_id": "CHEMBL113", "pref_name": "CAFFEINE"},
      "ndc": [
        {"nonproprietaryname": "Acetaminophen, Aspirin and Caffeine", "proprietaryname": "Headache Relief"}
      ],
      "unii": {"unii": "3G6A5W338E", "display_name": "CAFFEINE"}
    },
    {
      "_id": "BSYNRYMUTXBXSQ-UHFFFAOYSA-N",
      "_score": 49.4,
      "drugbank": {"id": "DB00945", "name": "Aspirin"},
      "chembl": {"molecule_chembl_id": "CHEMBL25", "pref_name": "ASPIRIN"},
      "ndc": [
        {"nonproprietaryname": "Acetaminophen, Aspirin and Caffeine", "proprietaryname": "Headache Relief"}
      ],
      "unii": {"unii": "R16CO5Y76E", "display_name": "ASPIRIN"}
    },
    {
      "_id": "57243-334",
      "_score": 47.9,
      "ndc": {"nonproprietaryname": "Acetaminophen, Aspirin and Caffeine", "proprietaryname": "Headache Relief"}
    }
  ]
}
"#;

/// The OTC label bodies carry no numbered indication subsections, so their
/// summary rows stay empty while the raw section text reaches the card.
fn assert_label_set_id_and_text(drug: &super::super::Drug, set_id: &str, needle: &str) {
    assert_eq!(drug.label_set_id.as_deref(), Some(set_id));
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
                    .is_some_and(|text| text.contains(needle))
        }),
        "the chosen label's text reached the card"
    );
}

/// Ticket 2050: the merged "Tension Headache Relief" product row used to
/// name the acetaminophen card because its name ends with the request. A
/// product name now counts only when it equals the request, so the card
/// keeps the ingredient line the "Headache Relief" rows themselves pair,
/// and the exact-brand label still serves.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn headache_relief_names_the_product_it_pairs_not_a_merged_row() {
    let (base, server) = name_resolution_fixture_server(
        vec![(
            "Headache Relief".to_string(),
            HEADACHE_RELIEF_CAPTURE.to_string(),
        )],
        vec![(
            "Headache Relief".to_string(),
            HEADACHE_RELIEF_LABEL_BODY.to_string(),
        )],
        Vec::new(),
    )
    .await;

    let (_env, _ddinter) = super::drug_fixture_env(base.as_str(), "headache-relief-ddinter");
    let drug = super::super::get("Headache Relief", &["label".to_string()])
        .await
        .expect("the shared combination product settles a card");
    assert_eq!(drug.name, "acetaminophen, aspirin and caffeine");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB00316"));
    assert!(
        !drug.name.contains("tension"),
        "the merged Tension Headache Relief row never names the card: {}",
        drug.name
    );
    assert_label_set_id_and_text(&drug, "a876daa7-ade3-497d-87bb-46d6e4dc185a", "minor aches and pains");
    server.abort();
}

/// The openFDA label answer for the "Advil" search (live total 39,
/// recorded 2026-10-11, ticket 2050), trimmed to the openFDA blocks and the
/// leading section text: the plain Advil record (1f01c10a) answers third
/// behind a menstrual-pain salt form and a combination.
const ADVIL_LABEL_BODY: &str = r#"{
 "meta": {"results": {"skip": 0, "limit": 5, "total": 39}},
 "results": [
  {
   "set_id": "ae1625f4-ef32-4d4f-bf54-c44085843e41",
   "effective_time": "20260820",
   "openfda": {
    "brand_name": ["Advil Menstrual Pain"],
    "generic_name": ["IBUPROFEN SODIUM"]
   },
   "indications_and_usage": [
    "Uses temporarily relieves minor aches and pains due to: menstrual cramps backache headache muscular aches the common cold toothache minor pain of arthritis temporarily reduces fever …[recorded reply trimmed for the fixture]"
   ]
  },
  {
   "set_id": "0da9ed22-bfb4-d61b-110b-0c7760332a98",
   "effective_time": "20260729",
   "openfda": {
    "brand_name": ["ADVIL COLD AND SINUS"],
    "generic_name": ["IBUPROFEN, PSEUDOEPHEDRINE HYDROCHLORIDE"]
   },
   "indications_and_usage": [
    "Uses temporarily relieves these symptoms associated with the common cold or flu: headache fever sinus pressure nasal congestion minor body aches and pains …[recorded reply trimmed for the fixture]"
   ]
  },
  {
   "set_id": "1f01c10a-9434-91a4-2ee4-352315a6b610",
   "effective_time": "20260729",
   "openfda": {
    "brand_name": ["Advil"],
    "generic_name": ["IBUPROFEN"]
   },
   "indications_and_usage": [
    "Uses temporarily relieves minor aches and pains due to: headache toothache backache menstrual cramps the common cold muscular aches minor pain of arthritis temporarily reduces fever …[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

/// `q=advil` (recorded 2026-10-11, ticket 2050): the ibuprofen identity
/// record carries the merged Advil rows, and one labeler row lists the
/// Advil PM combination's brand as its established name — the merged
/// product row whose name starts with the request and used to name the
/// card on main. Rows trimmed to the ones the selection reads.
const ADVIL_CAPTURE: &str = r#"
{
  "total": 17,
  "hits": [
    {
      "_id": "HEFNNWSXXWATRW-UHFFFAOYSA-N",
      "_score": 47.824516,
      "drugbank": {"id": "DB01050", "name": "Ibuprofen"},
      "chembl": {"molecule_chembl_id": "CHEMBL521", "pref_name": "IBUPROFEN"},
      "ndc": [
        {"nonproprietaryname": "Advil PM", "proprietaryname": "ADVIL PM- diphenhydramine citrate and ibuprofen tablet, coated"},
        {"nonproprietaryname": "Ibuprofen", "proprietaryname": "Advil"},
        {"nonproprietaryname": "ibuprofen sodium", "proprietaryname": "Advil"}
      ],
      "unii": {"unii": "WK2XYI10QM", "display_name": "IBUPROFEN"}
    },
    {
      "_id": "85237-1786",
      "_score": 20.876358,
      "ndc": {"nonproprietaryname": "ibuprofen", "proprietaryname": "ADVIL"}
    }
  ]
}
"#;

/// Ticket 2050: `get drug Advil` used to name the card "advil pm" — the
/// merged Advil PM product row's established name starts with the request —
/// while showing the plain Advil label. A product name now counts only
/// when it equals the request, so the card names the ingredient the Advil
/// rows pair and keeps the plain label.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn advil_names_the_ingredient_it_pairs_not_the_merged_pm_row() {
    let (base, server) = name_resolution_fixture_server(
        vec![("Advil".to_string(), ADVIL_CAPTURE.to_string())],
        vec![("Advil".to_string(), ADVIL_LABEL_BODY.to_string())],
        Vec::new(),
    )
    .await;

    let (_env, _ddinter) = super::drug_fixture_env(base.as_str(), "advil-ddinter");
    let drug = super::super::get("Advil", &["label".to_string()])
        .await
        .expect("Advil settles a card");
    assert_eq!(drug.name, "ibuprofen");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB01050"));
    assert!(
        !drug.name.contains("pm"),
        "the merged Advil PM row never names the card: {}",
        drug.name
    );
    assert_label_set_id_and_text(
        &drug,
        "1f01c10a-9434-91a4-2ee4-352315a6b610",
        "minor aches and pains",
    );
    server.abort();
}

/// The openFDA label answer for the "Nighttime Sleep Aid" search (live
/// total 126, recorded 2026-10-11, ticket 2050), trimmed to the openFDA
/// blocks and the leading section text: the flavor variant answers first
/// and the exact "Nighttime Sleep Aid" brand record (514aa6b5) second,
/// both plain diphenhydramine hydrochloride.
const NIGHTTIME_SLEEP_AID_LABEL_BODY: &str = r#"{
 "meta": {"results": {"skip": 0, "limit": 5, "total": 126}},
 "results": [
  {
   "set_id": "303d7651-9c8f-4a14-9f2c-7181fa0f003e",
   "effective_time": "20260909",
   "openfda": {
    "brand_name": ["Nighttime Sleep Aid Berry Flavor"],
    "generic_name": ["DIPHENHYDRAMINE HYDROCHLORIDE"]
   },
   "indications_and_usage": [
    "Uses ▪ for the relief of occasional sleeplessness ▪ reduces time to fall asleep if you have difficulty falling asleep …[recorded reply trimmed for the fixture]"
   ]
  },
  {
   "set_id": "514aa6b5-c33b-494c-e063-6294a90a7ace",
   "effective_time": "20260827",
   "openfda": {
    "brand_name": ["Nighttime Sleep Aid"],
    "generic_name": ["DIPHENHYDRAMINE HYDROCHLORIDE"]
   },
   "indications_and_usage": [
    "Uses for the relief of occasional sleeplessness reduces time to fall asleep if you have difficulty falling asleep …[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

/// `q=nighttime sleep aid` (recorded 2026-10-11, ticket 2050): the shelf
/// name reaches the diphenhydramine record only through its merged Berry
/// Flavor product row — a proprietary name that extends the request — and
/// the record's first row is the Savvy two-ingredient product, whose line
/// used to name the card on main. Rows trimmed to the ones the selection
/// reads.
const NIGHTTIME_SLEEP_AID_CAPTURE: &str = r#"
{
  "total": 50,
  "hits": [
    {
      "_id": "ZZVUWRFHKOJYTH-UHFFFAOYSA-N",
      "_score": 57.620647,
      "drugbank": {"id": "DB01075", "name": "Diphenhydramine"},
      "chembl": {"molecule_chembl_id": "CHEMBL657", "pref_name": "DIPHENHYDRAMINE"},
      "ndc": [
        {"nonproprietaryname": "Diphenhydramine hydrochloride Zinc acetate", "proprietaryname": "Savvy"},
        {"nonproprietaryname": "Diphenhydramine Hydrochloride", "proprietaryname": "Nighttime Sleep Aid Berry Flavor"}
      ],
      "unii": {"unii": "8GTS82S83M", "display_name": "DIPHENHYDRAMINE"}
    }
  ]
}
"#;

/// Ticket 2050: `get drug "Nighttime Sleep Aid"` used to name the card with
/// the first merged row's two-ingredient line ("diphenhydramine
/// hydrochloride zinc acetate" — another product entirely). The record's
/// own identity name now outranks merged product rows, so the card names
/// the drug whose record answered, and the brand-tier label check keeps the
/// exact-brand label.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn nighttime_sleep_aid_names_the_record_not_another_products_row() {
    let (base, server) = name_resolution_fixture_server(
        vec![(
            "Nighttime Sleep Aid".to_string(),
            NIGHTTIME_SLEEP_AID_CAPTURE.to_string(),
        )],
        vec![(
            "Nighttime Sleep Aid".to_string(),
            NIGHTTIME_SLEEP_AID_LABEL_BODY.to_string(),
        )],
        Vec::new(),
    )
    .await;

    let (_env, _ddinter) = super::drug_fixture_env(base.as_str(), "nighttime-sleep-aid-ddinter");
    let drug = super::super::get("Nighttime Sleep Aid", &["label".to_string()])
        .await
        .expect("the shelf name settles a card");
    assert_eq!(drug.name, "diphenhydramine");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB01075"));
    assert!(
        !drug.name.contains("zinc"),
        "another product's ingredient line never names the card: {}",
        drug.name
    );
    assert_label_set_id_and_text(
        &drug,
        "514aa6b5-c33b-494c-e063-6294a90a7ace",
        "occasional sleeplessness",
    );
    server.abort();
}
