//! Recorded-reply tests for the 2052 resolution rules: the label follows
//! the resolved ingredient, a brand whose labels mostly name one
//! ingredient resolves to it, an ingredient query takes the plain
//! ingredient's name and label, a card name never carries a product row's
//! strength, and the hyaluronidase pairing is derived from the record's
//! own ingredient list instead of a name string.

use super::label_choice::assert_label_data_with_set_id;

use super::*;

/// `q=5-fluorouracil` (live answer recorded 2026-10-11, trimmed to the
/// fields the selection reads): the fluorouracil identity record, whose
/// secondary database titles (GtoPdb, ChEBI) spell the request's synonym
/// while every establishment field spells fluorouracil.
const FLUOROURACIL_CAPTURE: &str = r#"
{
  "total": 1,
  "hits": [
    {
      "_id": "GHASVSINZRGABV-UHFFFAOYSA-N",
      "_score": 25.0,
      "drugbank": {"id": "DB00544", "name": "Fluorouracil"},
      "chembl": {"molecule_chembl_id": "CHEMBL185", "pref_name": "FLUOROURACIL"},
      "gtopdb": {"name": "5-fluorouracil"},
      "unii": {"unii": "U3P01618RT", "display_name": "FLUOROURACIL"},
      "chebi": {"name": "5-fluorouracil"},
      "drugcentral": {"synonyms": ["5-fluorouracil", "fluorouracil", "5-FU"]},
      "ndc": [
        {"nonproprietaryname": "Fluorouracil", "proprietaryname": "Fluorouracil"}
      ]
    }
  ]
}
"#;

/// The openFDA label answer for `fluorouracil` (live total 20, recorded
/// 2026-10-11, trimmed): the newest record is a qualified salt form, the
/// plain FLUOROURACIL record (set f0b60247) answers second. The
/// `5-fluorouracil` phrase itself matches no openFDA record.
const FLUOROURACIL_LABEL_BODY: &str = r#"{
 "meta": {"results": {"skip": 0, "limit": 5, "total": 20}},
 "results": [
  {
   "set_id": "1d2c3b4e-0000-4000-8000-000000000001",
   "effective_time": "20260801",
   "openfda": {
    "brand_name": ["Fluorouracil Injection"],
    "generic_name": ["FLUOROURACIL SODIUM"],
    "substance_name": ["FLUOROURACIL"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE Fluorouracil Injection is a nucleoside metabolic inhibitor indicated \u2026[recorded reply trimmed for the fixture]"
   ]
  },
  {
   "set_id": "f0b60247-26a8-4e6b-9a47-7b1c7de50d0a",
   "effective_time": "20260510",
   "openfda": {
    "brand_name": ["Fluorouracil"],
    "generic_name": ["FLUOROURACIL"],
    "substance_name": ["FLUOROURACIL"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE Fluorouracil is a nucleoside metabolic inhibitor indicated for the palliative treatment of patients with colorectal cancer \u2026[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

#[tokio::test]
#[serial_test::serial(source_env)]
async fn fluorouracil_synonym_takes_the_plain_ingredient_card_and_label() {
    let (base, server) = name_resolution_fixture_server(
        vec![(
            "5-fluorouracil".to_string(),
            FLUOROURACIL_CAPTURE.to_string(),
        )],
        vec![(
            "fluorouracil".to_string(),
            FLUOROURACIL_LABEL_BODY.to_string(),
        )],
        Vec::new(),
    )
    .await;

    // The request spells the synonym the record's secondary
    // database titles carry, and the card still takes the plain
    // ingredient's name with its own label (v0.9.1 named the card
    // "5-fluorouracil" and printed no label, because the lookup searched
    // the card name alone).
    let drug = name_resolution_fixture_drug(&base, "5-fluorouracil").await;
    assert_eq!(drug.name, "fluorouracil");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB00544"));
    assert_label_data_with_set_id(&drug, "f0b60247-26a8-4e6b-9a47-7b1c7de50d0a");
    server.abort();
}

/// `q=Motrin` (live answer recorded 2026-10-11, trimmed): the ibuprofen
/// identity record — whose first Motrin row pairs the PM combination —
/// beside the diphenhydramine record carrying the same PM row.
const MOTRIN_CAPTURE: &str = r#"
{
  "total": 2,
  "hits": [
    {
      "_id": "HEFNNWSXXWATRW-UHFFFAOYSA-N",
      "_score": 25.0,
      "drugbank": {"id": "DB01050", "name": "Ibuprofen"},
      "unii": {"unii": "WK2XYI10QM", "display_name": "IBUPROFEN"},
      "ndc": [
        {"nonproprietaryname": "ibuprofen and diphenhydramine citrate", "proprietaryname": "Motrin PM"},
        {"nonproprietaryname": "ibuprofen", "proprietaryname": "Motrin IB"}
      ]
    },
    {
      "_id": "ZZVUWRFHKOJYTH-UHFFFAOYSA-N",
      "_score": 20.0,
      "drugbank": {"id": "DB01075", "name": "Diphenhydramine"},
      "unii": {"unii": "8GTS82S83M", "display_name": "DIPHENHYDRAMINE"},
      "ndc": [
        {"nonproprietaryname": "ibuprofen and diphenhydramine citrate", "proprietaryname": "Motrin PM"}
      ]
    }
  ]
}
"#;

/// The openFDA label answer for `Motrin` (live total 21, recorded
/// 2026-10-11, trimmed): four plain IBUPROFEN records of the Motrin
/// family and one dual-action combination. The plain records' brands all
/// open with the requested brand; the newest is an infants' product, so
/// the family tier must prefer the record whose brand opens with the
/// request (Motrin IB) over the plain record that merely answers newest.
const MOTRIN_LABEL_BODY: &str = r#"{
 "meta": {"results": {"skip": 0, "limit": 5, "total": 21}},
 "results": [
  {
   "set_id": "c2302bfe-b367-4867-b803-29f066a42dc7",
   "effective_time": "20260630",
   "openfda": {
    "brand_name": ["Infants Motrin"],
    "generic_name": ["IBUPROFEN"],
    "substance_name": ["IBUPROFEN"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE Infants Motrin is a nonsteroidal anti-inflammatory drug \u2026[recorded reply trimmed for the fixture]"
   ]
  },
  {
   "set_id": "52c650cf-3085-7b2c-e063-6394a90a797f",
   "effective_time": "20260527",
   "openfda": {
    "brand_name": ["Motrin IB"],
    "generic_name": ["IBUPROFEN"],
    "substance_name": ["IBUPROFEN"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE Motrin IB is a nonsteroidal anti-inflammatory drug indicated for the palliative treatment of patients with metastatic cancer pain \u2026[recorded reply trimmed for the fixture]"
   ]
  },
  {
   "set_id": "f2ee9cf5-02b7-6612-e053-2995a90a90c4",
   "effective_time": "20260203",
   "openfda": {
    "brand_name": ["Motrin Dual Action With Tylenol"],
    "generic_name": ["ACETAMINOPHEN AND IBUPROFEN"],
    "substance_name": ["IBUPROFEN", "ACETAMINOPHEN"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE Motrin Dual Action With Tylenol is a combination of an NSAID and an analgesic \u2026[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

#[tokio::test]
#[serial_test::serial(source_env)]
async fn motrin_resolves_to_the_ingredient_its_labels_mostly_name() {
    let (base, server) = name_resolution_fixture_server(
        vec![("Motrin".to_string(), MOTRIN_CAPTURE.to_string())],
        vec![
            ("Motrin".to_string(), MOTRIN_LABEL_BODY.to_string()),
            ("ibuprofen".to_string(), MOTRIN_LABEL_BODY.to_string()),
        ],
        Vec::new(),
    )
    .await;

    // MyChem's Motrin rows pair the PM combination first, so
    // the merged card names the combination, and the brand's own labels —
    // mostly plain ibuprofen records — resolve the card to that one
    // ingredient (v0.9.2-dev named the card "ibuprofen and diphenhydramine
    // citrate" and served an Ibuprofen PM store-brand label). The label is
    // the Motrin family's own plain-ingredient record, never the PM
    // combination's.
    let drug = name_resolution_fixture_drug(&base, "Motrin").await;
    assert_eq!(drug.name, "ibuprofen");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB01050"));
    assert_label_data_with_set_id(&drug, "52c650cf-3085-7b2c-e063-6394a90a797f");
    server.abort();
}

/// `q=acetaminophen` (live answer recorded 2026-10-11, trimmed to the
/// voting records): the acetaminophen identity record beside the
/// phenylephrine and diphenhydramine records, whose merged product rows
/// name acetaminophen combinations. The combination rows used to outvote
/// the ingredient and refuse the query.
const ACETAMINOPHEN_CAPTURE: &str = r#"
{
  "total": 3,
  "hits": [
    {
      "_id": "RZVAJINKPMORJF-UHFFFAOYSA-N",
      "_score": 25.0,
      "drugbank": {"id": "DB00316", "name": "Acetaminophen"},
      "unii": {"unii": "362O9ITL9D", "display_name": "ACETAMINOPHEN"},
      "drugcentral": {"synonyms": ["acetaminophen", "paracetamol"]},
      "ndc": [
        {"nonproprietaryname": "acetaminophen", "proprietaryname": "Acetaminophen"}
      ]
    },
    {
      "_id": "SONNWYBIRXJNDC-VIFPVBQESA-N",
      "_score": 20.0,
      "drugbank": {"id": "DB00388", "name": "Phenylephrine"},
      "unii": {"unii": "1WS297W6KM", "display_name": "PHENYLEPHRINE"},
      "ndc": [
        {"nonproprietaryname": "acetaminophen phenylephrine hcl chlorpheniramine maleate", "proprietaryname": "Acetaminophen"}
      ]
    },
    {
      "_id": "ZZVUWRFHKOJYTH-UHFFFAOYSA-N",
      "_score": 18.0,
      "drugbank": {"id": "DB01075", "name": "Diphenhydramine"},
      "unii": {"unii": "8GTS82S83M", "display_name": "DIPHENHYDRAMINE"},
      "ndc": [
        {"nonproprietaryname": "acetaminophen and diphenhydramine hcl", "proprietaryname": "Acetaminophen"}
      ]
    }
  ]
}
"#;

/// The openFDA label answer for `acetaminophen` (live total 3287,
/// recorded 2026-10-11, trimmed): the newest page holds combination
/// products, and a plain single-ingredient record answers below them.
const ACETAMINOPHEN_LABEL_BODY: &str = r#"{
 "meta": {"results": {"skip": 0, "limit": 5, "total": 3287}},
 "results": [
  {
   "set_id": "1022767d-a456-4fbc-95cf-91337c74c330",
   "effective_time": "20260901",
   "openfda": {
    "brand_name": ["Daytime Sinus Severe"],
    "generic_name": ["ACETAMINOPHEN, GUAIFENESIN, PHENYLEPHRINE HCL"],
    "substance_name": ["ACETAMINOPHEN", "GUAIFENESIN", "PHENYLEPHRINE HYDROCHLORIDE"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE Daytime Sinus Severe is a combination product \u2026[recorded reply trimmed for the fixture]"
   ]
  },
  {
   "set_id": "78874258-16cc-40b5-9e15-5943eddfce94",
   "effective_time": "20260908",
   "openfda": {
    "brand_name": ["Acetaminophen Extra Strength"],
    "generic_name": ["ACETAMINOPHEN"],
    "substance_name": ["ACETAMINOPHEN"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE Acetaminophen Extra Strength is indicated for the palliative treatment of patients with cancer pain \u2026[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

#[tokio::test]
#[serial_test::serial(source_env)]
async fn acetaminophen_takes_the_plain_ingredient_card_and_label() {
    let (base, server) = name_resolution_fixture_server(
        vec![(
            "acetaminophen".to_string(),
            ACETAMINOPHEN_CAPTURE.to_string(),
        )],
        vec![(
            "acetaminophen".to_string(),
            ACETAMINOPHEN_LABEL_BODY.to_string(),
        )],
        Vec::new(),
    )
    .await;

    // An ingredient query takes the plain ingredient's name
    // and label. The other records' combination rows are product mentions,
    // not identity claims, so the record that is acetaminophen decides the
    // tier (v0.9.2-dev refused: "could not map 'acetaminophen' to a single
    // drug").
    let drug = name_resolution_fixture_drug(&base, "acetaminophen").await;
    assert_eq!(drug.name, "acetaminophen");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB00316"));
    assert_label_data_with_set_id(&drug, "78874258-16cc-40b5-9e15-5943eddfce94");
    server.abort();
}

/// `q=Tylenol` (live answer recorded 2026-10-11, trimmed): the
/// acetaminophen identity record carrying the family's product rows beside
/// the phenylephrine and guaifenesin records whose first rows name Tylenol
/// combination products.
const TYLENOL_CAPTURE: &str = r#"
{
  "total": 3,
  "hits": [
    {
      "_id": "RZVAJINKPMORJF-UHFFFAOYSA-N",
      "_score": 25.0,
      "drugbank": {"id": "DB00316", "name": "Acetaminophen"},
      "unii": {"unii": "362O9ITL9D", "display_name": "ACETAMINOPHEN"},
      "ndc": [
        {"nonproprietaryname": "acetaminophen", "proprietaryname": "TYLENOL"}
      ]
    },
    {
      "_id": "SONNWYBIRXJNDC-VIFPVBQESA-N",
      "_score": 20.0,
      "drugbank": {"id": "DB00388", "name": "Phenylephrine"},
      "unii": {"unii": "1WS297W6KM", "display_name": "PHENYLEPHRINE"},
      "ndc": [
        {"nonproprietaryname": "acetaminophen, guaifenesin, and phenylephrine hydrochloride", "proprietaryname": "Tylenol"}
      ]
    },
    {
      "_id": "HSRJKNPTNIJEKV-UHFFFAOYSA-N",
      "_score": 18.0,
      "drugbank": {"id": "DB01638", "name": "Guaifenesin"},
      "unii": {"unii": "495W7457VQ", "display_name": "GUAIFENESIN"},
      "ndc": [
        {"nonproprietaryname": "acetaminophen, guaifenesin, and phenylephrine hydrochloride", "proprietaryname": "Tylenol"}
      ]
    }
  ]
}
"#;

/// The openFDA label answer for `Tylenol` (live total 109, recorded
/// 2026-10-11, trimmed): the newest records are the family's combination
/// products, and a plain acetaminophen record of the family (Infants
/// TYLENOL) answers below them.
const TYLENOL_LABEL_BODY: &str = r#"{
 "meta": {"results": {"skip": 0, "limit": 5, "total": 109}},
 "results": [
  {
   "set_id": "5b95905e-b729-4caf-921a-fc4ef815289c",
   "effective_time": "20260912",
   "openfda": {
    "brand_name": ["Childrens TYLENOL COLD plus COUGH plus RUNNY NOSE"],
    "generic_name": ["ACETAMINOPHEN, CHLORPHENIRAMINE MALEATE, AND DEXTROMETHORPHAN HYDROBROMIDE"],
    "substance_name": ["ACETAMINOPHEN", "CHLORPHENIRAMINE MALEATE", "DEXTROMETHORPHAN HYDROBROMIDE"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE Childrens TYLENOL COLD is a combination product \u2026[recorded reply trimmed for the fixture]"
   ]
  },
  {
   "set_id": "997b9551-c222-4a9c-8ba1-09a5f97495fa",
   "effective_time": "20260820",
   "openfda": {
    "brand_name": ["Infants TYLENOL"],
    "generic_name": ["ACETAMINOPHEN"],
    "substance_name": ["ACETAMINOPHEN"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE Infants TYLENOL is indicated for the palliative treatment of patients with cancer pain \u2026[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

#[tokio::test]
#[serial_test::serial(source_env)]
async fn tylenol_resolves_to_the_plain_ingredient_its_labels_mostly_name() {
    let (base, server) = name_resolution_fixture_server(
        vec![("Tylenol".to_string(), TYLENOL_CAPTURE.to_string())],
        vec![
            ("Tylenol".to_string(), TYLENOL_LABEL_BODY.to_string()),
            ("acetaminophen".to_string(), TYLENOL_LABEL_BODY.to_string()),
        ],
        Vec::new(),
    )
    .await;

    // A brand whose labels mostly name one ingredient
    // resolves to it. The acetaminophen record's plain vote decides the
    // tier over the other records' combination mentions, and the label
    // choice takes the plain acetaminophen record of the family (v0.9.2-dev
    // refused the query outright).
    let drug = name_resolution_fixture_drug(&base, "Tylenol").await;
    assert_eq!(drug.name, "acetaminophen");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB00316"));
    assert_label_data_with_set_id(&drug, "997b9551-c222-4a9c-8ba1-09a5f97495fa");
    server.abort();
}

/// `q=aspirin` (live answer recorded 2026-10-11, trimmed): the aspirin
/// identity record, whose own names spell the ingredient (UNII display)
/// while its first product row carries a strength. On main the row's
/// strength named the card.
const ASPIRIN_CAPTURE: &str = r#"
{
  "total": 2,
  "hits": [
    {
      "_id": "BSYNRYMUTXBXSQ-UHFFFAOYSA-N",
      "_score": 25.0,
      "drugbank": {"id": "DB00945", "name": "Acetylsalicylic acid"},
      "unii": {"unii": "16SC6A2MAR", "display_name": "ASPIRIN"},
      "drugcentral": {"synonyms": ["acetylsalicylic acid", "aspirin"]},
      "ndc": [
        {"nonproprietaryname": "Aspirin 81 mg", "proprietaryname": "Aspirin"},
        {"nonproprietaryname": "Aspirin", "proprietaryname": "Aspirin Regular Strength"}
      ]
    },
    {
      "_id": "83139-288",
      "_score": 16.5,
      "ndc": [
        {"nonproprietaryname": "Aspirin 81mg", "proprietaryname": "Aspirin 81mg"}
      ]
    }
  ]
}
"#;

/// The openFDA label answer for `aspirin` (live total 713, recorded
/// 2026-10-11, trimmed): plain aspirin records answer first.
const ASPIRIN_LABEL_BODY: &str = r#"{
 "meta": {"results": {"skip": 0, "limit": 5, "total": 713}},
 "results": [
  {
   "set_id": "a0a19cd1-eb8a-4773-be00-78de90d2a3c3",
   "effective_time": "20260901",
   "openfda": {
    "brand_name": ["Aspirin"],
    "generic_name": ["ASPIRIN"],
    "substance_name": ["ASPIRIN"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE Aspirin is indicated for the treatment of patients with colorectal cancer \u2026[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

#[tokio::test]
#[serial_test::serial(source_env)]
async fn aspirin_card_names_the_ingredient_not_a_product_rows_strength() {
    let (base, server) = name_resolution_fixture_server(
        vec![
            ("aspirin".to_string(), ASPIRIN_CAPTURE.to_string()),
            ("aspirin 81 mg".to_string(), ASPIRIN_CAPTURE.to_string()),
        ],
        vec![
            ("aspirin".to_string(), ASPIRIN_LABEL_BODY.to_string()),
            ("aspirin 81 mg".to_string(), ASPIRIN_LABEL_BODY.to_string()),
        ],
        Vec::new(),
    )
    .await;

    // A card name never carries a product row's strength —
    // the row "Aspirin 81 mg/Aspirin" pairs the brand with the ingredient
    // aspirin, not with a strength-sized fragment of it (v0.9.2-dev named
    // the card "aspirin 81 mg").
    let drug = name_resolution_fixture_drug(&base, "aspirin").await;
    assert_eq!(drug.name, "aspirin");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB00945"));
    assert_label_data_with_set_id(&drug, "a0a19cd1-eb8a-4773-be00-78de90d2a3c3");

    // The strength-bearing request still resolves to the same card.
    let drug = name_resolution_fixture_drug(&base, "aspirin 81 mg").await;
    assert_eq!(drug.name, "aspirin");
    assert_label_data_with_set_id(&drug, "a0a19cd1-eb8a-4773-be00-78de90d2a3c3");
    server.abort();
}

/// `q=Qlex` (live answer recorded 2026-10-11): no record names the brand,
/// so the text query's answer is another drug's record and the brand
/// resolves through the openFDA identity candidate alone.
const QLEX_MYCHEM_MISS: &str = r#"
{
  "total": 1,
  "hits": [
    {
      "_id": "C9999999",
      "_score": 17.0,
      "ndc": {"nonproprietaryname": "otherdrug", "proprietaryname": "Otherdrug"}
    }
  ]
}
"#;

/// The openFDA label answer for `Qlex` (live total 1, recorded
/// 2026-10-11): the KEYTRUDA QLEX record, whose generic names the
/// under-the-skin pairing with a qualified form of a listed ingredient —
/// the record's own ingredient list is what makes the plain side the
/// ingredient, never a name string.
const QLEX_LABEL_BODY: &str = r#"{
 "meta": {"results": {"skip": 0, "limit": 5, "total": 1}},
 "results": [
  {
   "set_id": "097d166f-b73b-41d3-9b37-7653cd2a0c41",
   "effective_time": "20260610",
   "openfda": {
    "brand_name": ["KEYTRUDA QLEX"],
    "generic_name": ["PEMBROLIZUMAB AND BERAHYALURONIDASE ALFA-PMPH"],
    "substance_name": ["PEMBROLIZUMAB", "BERAHYALURONIDASE ALFA"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE KEYTRUDA QLEX is a combination of pembrolizumab, a programmed death receptor-1-blocking antibody, and berahyaluronidase alfa, indicated for patients with unresectable or metastatic cancer \u2026[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

/// `q=pembrolizumab` (live answer recorded 2026-10-11, trimmed to the
/// identity record).
const PEMBROLIZUMAB_FOR_QLEX_CAPTURE: &str = r#"
{
  "total": 1,
  "hits": [
    {
      "_id": "DPT0O3T46P",
      "_score": 25.6,
      "drugbank": {
        "id": "DB09037",
        "name": "Pembrolizumab",
        "synonyms": ["Lambrolizumab", "Pembrolizumab"]
      },
      "unii": {"unii": "DPT0O3T46P", "display_name": "PEMBROLIZUMAB"}
    }
  ]
}
"#;

#[tokio::test]
#[serial_test::serial(source_env)]
async fn qlex_pairing_derives_the_plain_ingredient_from_the_records_own_list() {
    let (base, server) = name_resolution_fixture_server(
        vec![
            ("Qlex".to_string(), QLEX_MYCHEM_MISS.to_string()),
            (
                "pembrolizumab".to_string(),
                PEMBROLIZUMAB_FOR_QLEX_CAPTURE.to_string(),
            ),
        ],
        vec![
            ("Qlex".to_string(), QLEX_LABEL_BODY.to_string()),
            ("pembrolizumab".to_string(), QLEX_LABEL_BODY.to_string()),
        ],
        Vec::new(),
    )
    .await;

    // "berahyaluronidase alfa-pmph" is a qualified form of
    // the listed ingredient "berahyaluronidase alfa", and pembrolizumab
    // is listed as the plain ingredient itself, so the pairing resolves to
    // pembrolizumab without any code naming the enzyme (the literal
    // hyaluronidase match refused this query).
    let drug = name_resolution_fixture_drug(&base, "Qlex").await;
    assert_eq!(drug.name, "pembrolizumab");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB09037"));
    assert_label_data_with_set_id(&drug, "097d166f-b73b-41d3-9b37-7653cd2a0c41");
    server.abort();
}

/// `q=Cold and Flu` (live answer recorded 2026-10-11, trimmed): two
/// records whose product rows pair the shelf name with different
/// combination product lines.
const COLD_AND_FLU_CAPTURE: &str = r#"
{
  "total": 2,
  "hits": [
    {
      "_id": "MKXZASYAUGDDCJ-NJAFHUGGSA-N",
      "_score": 25.0,
      "drugbank": {"id": "DB00852", "name": "Dextromethorphan"},
      "unii": {"unii": "7355X3ROTS", "display_name": "DEXTROMETHORPHAN"},
      "ndc": [
        {"nonproprietaryname": "acetaminophen, dextromethorphan hbr, guaifenesin, phenylephrine hcl", "proprietaryname": "Cold and Flu Relief"}
      ]
    },
    {
      "_id": "CBEQULMOCCWAQT-WOJGMQOQSA-N",
      "_score": 20.0,
      "drugbank": {"id": "DB00366", "name": "Triprolidine"},
      "unii": {"unii": "90341Y1V1E", "display_name": "TRIPROLIDINE"},
      "ndc": [
        {"nonproprietaryname": "acetaminophen, dextromethorphan hbr, triprolidine hcl", "proprietaryname": "Overnight Cold and Flu"}
      ]
    }
  ]
}
"#;

#[tokio::test]
#[serial_test::serial(source_env)]
async fn cold_and_flu_still_refuses_across_distinct_combination_products() {
    let (base, server) = name_resolution_fixture_server(
        vec![("Cold and Flu".to_string(), COLD_AND_FLU_CAPTURE.to_string())],
        Vec::new(),
        Vec::new(),
    )
    .await;
    let (_env, _ddinter) = super::drug_fixture_env(base.as_str(), "cold-and-flu-ddinter");

    // Kept honest by the shared-name rule: a shelf name several
    // distinct combination products share names no one drug, so the
    // records' combination votes disagree as ingredient sets and the
    // query refuses instead of merging one combination's card.
    let err = super::super::get("Cold and Flu", &["label".to_string()])
        .await
        .expect_err("a shared shelf name must refuse");
    let message = err.to_string();
    assert!(
        message.contains("No drug card matches \"Cold and Flu\""),
        "{message}"
    );
    server.abort();
}

/// A combination card shows product-level identifiers or none: the merged records are one ingredient's records, so the
/// combination card keeps none of that ingredient's stable identifier,
/// approval date or chemical synonyms. Recorded from the live "advil pm"
/// shape (2026-10-11).
#[test]
fn a_combination_card_carries_no_single_ingredient_identifier() {
    let ibuprofen: MyChemHit = serde_json::from_value(serde_json::json!({
        "_id": "HEFNNWSXXWATRW-UHFFFAOYSA-N",
        "drugbank": {
            "id": "DB01050",
            "name": "Ibuprofen",
            "synonyms": ["4'-isobutylacetophenone"]
        },
        "drugcentral": {"approval": [{"agency": "FDA", "date": "1998-09-19"}]},
        "unii": {"unii": "WK2XYI10QM", "display_name": "IBUPROFEN"},
        "ndc": [
            {"nonproprietaryname": "ibuprofen and diphenhydramine citrate", "proprietaryname": "ADVIL PM"}
        ]
    }))
    .expect("valid fixture hit");
    let selected = vec![&ibuprofen];

    let drug = crate::transform::drug::merge_mychem_hits(&selected, "advil pm");
    assert_eq!(drug.name, "ibuprofen and diphenhydramine citrate");
    assert_eq!(
        drug.drugbank_id, None,
        "no single ingredient's stable id stands for the product"
    );
    assert_eq!(drug.chembl_id, None);
    assert_eq!(drug.unii, None);
    assert_eq!(
        drug.approval_date, None,
        "no single ingredient's approval date stands for the product"
    );
    assert!(
        drug.brand_names.is_empty(),
        "no chemical synonym is listed as a brand: {:?}",
        drug.brand_names
    );
}
