//! Recorded-reply tests for the product-name and sparse-brand gaps: a
//! merged NDC product name counts as the
//! record's own only when it equals the request, a record's own identity
//! name outranks merged product rows, and one sparse-record reader keeps
//! multi-word brands whole.

use super::*;

const HERCEPTIN_HYLECTA_ELEMENTS_BODY: &str = r#"{
 "meta": {
  "results": {
   "skip": 0,
   "limit": 10,
   "total": 1
  }
 },
 "results": [
  {
   "set_id": "ebf30894-41cf-480c-8bc3-56f592a13813",
   "effective_time": "20260529",
   "openfda": {},
   "spl_product_data_elements": [
    "Herceptin Hylecta Trastuzumab and hyaluronidase-oysk TRASTUZUMAB TRASTUZUMAB HYALURONIDASE (HUMAN RECOMBINANT) HYALURONIDASE (HUMAN RECOMBINANT) HISTIDINE HYDROCHLORIDE MONOHYDRATE HISTIDINE TREHALOSE DIHYDRATE POLYSORBATE 20 METHIONINE WATER"
   ],
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE HERCEPTIN HYLECTA is a combination of trastuzumab, a HER2/neu receptor antagonist, and hyaluronidase, an endoglycosidase, indicated in adults for: The treatment of HER2-overexpressing breast cancer. ( 1.1 , 1.2 ) Select patients for therapy based on an FDA \u2026[recorded reply trimmed for the fixture]"
   ],
   "boxed_warning": [
    "WARNING: CARDIOMYOPATHY, EMBRYO-FETAL TOXICITY, and PULMONARY TOXICITY WARNING: CARDIOMYOPATHY, EMBRYO-FETAL TOXICITY, and PULMONARY TOXICITY See full prescribing information for complete boxed warning. Cardiomyopathy: HERCEPTIN HYLECTA can result in subclinical and clinical card \u2026[recorded reply trimmed for the fixture]"
   ],
   "warnings_and_cautions": [
    "5 WARNINGS AND PRECAUTIONS Exacerbation of Chemotherapy-Induced Neutropenia. ( 5.4 , 6.1 ) Hypersensitivity and Administration-Related Reactions (ARRs): Severe ARRs, including anaphylaxis, have been reported with HERCEPTIN HYLECTA. Monitor patients for systemic hypersensitivity r \u2026[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

const RITUXAN_HYCELA_ELEMENTS_BODY: &str = r#"{
 "meta": {
  "results": {
   "skip": 0,
   "limit": 10,
   "total": 1
  }
 },
 "results": [
  {
   "set_id": "3e5b7e82-f018-4eaf-ae78-d6145a906b20",
   "effective_time": "20251215",
   "openfda": {},
   "spl_product_data_elements": [
    "Rituxan Hycela rituximab and hyaluronidase RITUXIMAB RITUXIMAB HYALURONIDASE (HUMAN RECOMBINANT) HYALURONIDASE (HUMAN RECOMBINANT) HISTIDINE HISTIDINE HYDROCHLORIDE MONOHYDRATE TREHALOSE DIHYDRATE METHIONINE POLYSORBATE 80 WATER Rituxan Hycela rituximab and hyaluronidase RITUXIMAB RITUXIMAB HYALURONIDASE (HUMAN RECOMBINANT) HYALURONIDASE (HUMAN RECOMBINANT) HISTIDINE HISTIDINE HYDROCHLORIDE MONOHYDRATE TREHALOSE DIHYDRATE METHIONINE POLYSORBATE 80 WATER"
   ],
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE RITUXAN HYCELA is a combination of rituximab, a CD20-directed cytolytic antibody, and hyaluronidase human, an endoglycosidase, indicated for the treatment of adult patients with: Follicular Lymphoma (FL) ( 1.1 ) Relapsed or refractory, follicular lymphoma  \u2026[recorded reply trimmed for the fixture]"
   ],
   "boxed_warning": [
    "WARNING: SEVERE MUCOCUTANEOUS REACTIONS, HEPATITIS B VIRUS REACTIVATION and PROGRESSIVE MULTIFOCAL LEUKOENCEPHALOPATHY WARNING: SEVERE MUCOCUTANEOUS REACTIONS, HEPATITIS B VIRUS REACTIVATION and PROGRESSIVE MULTIFOCAL LEUKOENCEPHALOPATHY See full prescribing information for compl \u2026[recorded reply trimmed for the fixture]"
   ],
   "warnings_and_cautions": [
    "5 WARNINGS AND PRECAUTIONS Hypersensitivity and other administration reactions: Local cutaneous reactions may occur more than 24 hours after administration. Interrupt injection if severe reaction develops. Premedicate before injection. ( 5.4 ) Tumor lysis syndrome: Administer agg \u2026[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

const TECENTRIQ_HYBREZA_ELEMENTS_BODY: &str = r#"{
 "meta": {
  "results": {
   "skip": 0,
   "limit": 10,
   "total": 1
  }
 },
 "results": [
  {
   "set_id": "a617b089-ce66-464b-987b-b45d029b4d6f",
   "effective_time": "20260520",
   "openfda": {},
   "spl_product_data_elements": [
    "Tecentriq Hybreza atezolizumab and hyaluronidase-tqjs ATEZOLIZUMAB ATEZOLIZUMAB HYALURONIDASE (HUMAN RECOMBINANT) HYALURONIDASE (HUMAN RECOMBINANT) HISTIDINE SUCROSE POLYSORBATE 20 METHIONINE ACETIC ACID WATER"
   ],
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE TECENTRIQ HYBREZA is a combination of atezolizumab, a programmed death-ligand 1 (PD-L1) blocking antibody, and hyaluronidase, an endoglycosidase indicated: Non-Small Cell Lung Cancer (NSCLC) as adjuvant treatment following resection and platinum-based chem \u2026[recorded reply trimmed for the fixture]"
   ],
   "warnings_and_cautions": [
    "5 WARNINGS AND PRECAUTIONS Immune-Mediated Adverse Reactions Immune-mediated adverse reactions, which may be severe or fatal, can occur in any organ system or tissue, including the following: immune-mediated pneumonitis, immune-mediated colitis, immune-mediated hepatitis, immune- \u2026[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

/// The MyChem answers for the multi-word sparse brands, recorded 2026-10-11
/// from `mychem.info/v1/query?q=<name>` with the `get` field
/// list: each brand lives on a naked NDC row whose nonproprietary line is
/// the hyaluronidase pairing.
const HERCEPTIN_HYLECTA_CAPTURE: &str = r#"
{
  "total": 1,
  "hits": [
    {
      "_id": "50242-077",
      "_score": 17.670006,
      "ndc": {"nonproprietaryname": "Trastuzumab and hyaluronidase-oysk", "proprietaryname": "Herceptin Hylecta"}
    }
  ]
}
"#;

const RITUXAN_HYCELA_CAPTURE: &str = r#"
{
  "total": 2,
  "hits": [
    {
      "_id": "50242-108",
      "_score": 17.670006,
      "ndc": {"nonproprietaryname": "rituximab and hyaluronidase", "proprietaryname": "Rituxan Hycela"}
    }
  ]
}
"#;

const TECENTRIQ_HYBREZA_CAPTURE: &str = r#"
{
  "total": 1,
  "hits": [
    {
      "_id": "50242-933",
      "_score": 17.670006,
      "ndc": {"nonproprietaryname": "atezolizumab and hyaluronidase-tqjs", "proprietaryname": "Tecentriq Hybreza"}
    }
  ]
}
"#;

/// A sparse record whose only identity is the element line can
/// carry a multi-word brand. The element opens "BRAND BRAND ESTABLISHED …",
/// and one reader keeps the whole leading brand run, so the identity guard
/// confirms the record for the queried name and the brand tier serves the
/// label. On main the reader took the first word alone, so these three
/// brands printed "No openFDA SPL label record matched this drug."
async fn sparse_multi_word_brand_label(
    brand: &str,
    capture: &str,
    elements: &str,
    set_id: &str,
) -> (super::super::Drug, crate::test_support::TestHttpServer) {
    let (base, server) = name_resolution_fixture_server_with_elements(
        vec![(brand.to_string(), capture.to_string())],
        Vec::new(),
        Vec::new(),
        vec![(brand.to_string(), elements.to_string())],
    )
    .await;
    let drug = name_resolution_fixture_drug(&base, brand).await;
    super::label_choice::assert_label_data_with_set_id(&drug, set_id);
    assert!(
        drug.brand_names
            .iter()
            .any(|name| name.eq_ignore_ascii_case(brand)),
        "the card's brands include the whole multi-word brand: {:?}",
        drug.brand_names
    );
    (drug, server)
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn herceptin_hylecta_keeps_its_multi_word_brand_and_own_label() {
    let (_drug, mut server) = sparse_multi_word_brand_label(
        "Herceptin Hylecta",
        HERCEPTIN_HYLECTA_CAPTURE,
        HERCEPTIN_HYLECTA_ELEMENTS_BODY,
        "ebf30894-41cf-480c-8bc3-56f592a13813",
    )
    .await;
    server.finish().unwrap();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn rituxan_hycela_keeps_its_multi_word_brand_and_own_label() {
    let (_drug, mut server) = sparse_multi_word_brand_label(
        "Rituxan Hycela",
        RITUXAN_HYCELA_CAPTURE,
        RITUXAN_HYCELA_ELEMENTS_BODY,
        "3e5b7e82-f018-4eaf-ae78-d6145a906b20",
    )
    .await;
    server.finish().unwrap();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn tecentriq_hybreza_keeps_its_multi_word_brand_and_own_label() {
    let (_drug, mut server) = sparse_multi_word_brand_label(
        "Tecentriq Hybreza",
        TECENTRIQ_HYBREZA_CAPTURE,
        TECENTRIQ_HYBREZA_ELEMENTS_BODY,
        "a617b089-ce66-464b-987b-b45d029b4d6f",
    )
    .await;
    server.finish().unwrap();
}

/// The live sweep showed `get drug 5-FU` waiting four to six
/// seconds on one EBI OLS4 search while refusing. The refusal rescue runs
/// inside a two-second budget, so a slow discover round gives up and the
/// refusal arrives without it.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_slow_discover_round_never_holds_the_refusal_hostage() {
    let mut ols_server = crate::test_support::TestHttpServer::spawn(move |mut stream| {
        async move {
            let Some(_request) = crate::test_support::read_request(&mut stream).await? else {
                return Ok(());
            };

            // Far past the rescue's budget and past the test's
            // deadline: only a bounded rescue can answer inside it.
            tokio::time::sleep(std::time::Duration::from_secs(8)).await;
            let body = r#"{"response":{"docs":[]}}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await?;
            Ok(())
        }
    })
    .expect("start HTTP fixture");
    let ols_base = ols_server.base.clone();

    let (base, mut server) = name_resolution_fixture_server(
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
    ols_server.finish().unwrap();
    server.finish().unwrap();

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
/// 68, recorded 2026-10-11), trimmed to the openFDA blocks and
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

/// `q=headache relief` (recorded 2026-10-11): the shelf name
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

/// The merged "Tension Headache Relief" product row used to
/// name the acetaminophen card because its name ends with the request. A
/// product name now counts only when it equals the request, so the card
/// keeps the ingredient line the "Headache Relief" rows themselves pair,
/// and the exact-brand label still serves. A combination card
/// shows product-level identifiers or none, so the merged acetaminophen
/// record's DrugBank id never stands in for the whole product.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn headache_relief_names_the_product_it_pairs_not_a_merged_row() {
    let (base, mut server) = name_resolution_fixture_server(
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
    assert_eq!(drug.drugbank_id.as_deref(), None);
    assert_eq!(drug.chembl_id.as_deref(), None);
    assert_eq!(drug.unii.as_deref(), None);
    assert_eq!(drug.approval_date.as_deref(), None);
    assert!(
        !drug.name.contains("tension"),
        "the merged Tension Headache Relief row never names the card: {}",
        drug.name
    );
    assert_label_set_id_and_text(
        &drug,
        "a876daa7-ade3-497d-87bb-46d6e4dc185a",
        "minor aches and pains",
    );
    server.finish().unwrap();
}

/// The openFDA label answer for the "Advil" search (live total 39,
/// recorded 2026-10-11), trimmed to the openFDA blocks and the
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

/// `q=advil` (recorded 2026-10-11): the ibuprofen identity
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

/// `get drug Advil` used to name the card "advil pm" — the
/// merged Advil PM product row's established name starts with the request —
/// while showing the plain Advil label. A product name now counts only
/// when it equals the request, so the card names the ingredient the Advil
/// rows pair and keeps the plain label.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn advil_names_the_ingredient_it_pairs_not_the_merged_pm_row() {
    let (base, mut server) = name_resolution_fixture_server(
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
    server.finish().unwrap();
}

/// The openFDA label answer for the "Nighttime Sleep Aid" search (live
/// total 126, recorded 2026-10-11), trimmed to the openFDA
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

/// `q=nighttime sleep aid` (recorded 2026-10-11): the shelf
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

/// `get drug "Nighttime Sleep Aid"` used to name the card with
/// the first merged row's two-ingredient line ("diphenhydramine
/// hydrochloride zinc acetate" — another product entirely). The record's
/// own identity name now outranks merged product rows, so the card names
/// the drug whose record answered, and the brand-tier label check keeps the
/// exact-brand label.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn nighttime_sleep_aid_names_the_record_not_another_products_row() {
    let (base, mut server) = name_resolution_fixture_server(
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
    server.finish().unwrap();
}
