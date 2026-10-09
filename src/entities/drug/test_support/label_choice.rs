//! Recorded-reply tests for the two-sided label choice (ticket 2043): a
//! brand query takes its own brand's label, and an ingredient query takes
//! the plain single-ingredient product's label, never a combination,
//! biosimilar or under-the-skin form.

use super::*;

/// The live openFDA label answers below were recorded 2026-10-13 (ticket
/// 2043) from `api.fda.gov/drug/label.json` with the drug card's search
/// plan, with each section's text cut after its opening words. Rows the
/// choice rule never reads were removed and the removal is noted per body;
/// the kept rows keep their live order and identity fields.

/// Search `Keytruda` and `pembrolizumab` (identical live answers): the
/// under-the-skin KEYTRUDA QLEX combination answers first and the plain
/// KEYTRUDA record second.
const KEYTRUDA_LABEL_BODY: &str = r#"{
 "meta": {"results": {"skip": 0, "limit": 5, "total": 2}},
 "results": [
  {
   "set_id": "097d166f-b73b-41d3-9b37-7653cd2a0c41",
   "effective_time": "20260731",
   "openfda": {
    "brand_name": ["KEYTRUDA QLEX"],
    "generic_name": ["PEMBROLIZUMAB AND BERAHYALURONIDASE ALFA-PMPH"],
    "route": ["SUBCUTANEOUS"],
    "substance_name": ["PEMBROLIZUMAB", "BERAHYALURONIDASE ALFA"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE KEYTRUDA QLEX is a combination of pembrolizumab, a programmed death receptor-1 (PD-1)-blocking antibody, and berahyaluronidase alfa, an endoglycosidase, ind \u2026[recorded reply trimmed for the fixture]"
   ],
   "warnings_and_cautions": [
    "5 WARNINGS AND PRECAUTIONS Immune-Mediated Adverse Reactions ( 5.1 ) Immune-mediated adverse reactions, which may be severe or fatal, can oc \u2026[recorded reply trimmed for the fixture]"
   ]
  },
  {
   "set_id": "9333c79b-d487-4538-a9f0-71b91a02b287",
   "effective_time": "20260710",
   "openfda": {
    "brand_name": ["KEYTRUDA"],
    "generic_name": ["PEMBROLIZUMAB"],
    "route": ["INTRAVENOUS"],
    "substance_name": ["PEMBROLIZUMAB"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE KEYTRUDA is a programmed death receptor-1 (PD-1)-blocking antibody indicated: Melanoma for the treatment of patients with unresectable or metastatic melanom \u2026[recorded reply trimmed for the fixture]"
   ],
   "warnings_and_cautions": [
    "5 WARNINGS AND PRECAUTIONS Immune-Mediated Adverse Reactions ( 5.1 ) Immune-mediated adverse reactions, which may be severe or fatal, can oc \u2026[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

/// Search `Herceptin`: Herceptin's own record is the only answer, because
/// openFDA's phrase search on the brand token does not reach the
/// under-the-skin HYLECTO pairing.
const HERCEPTIN_LABEL_BODY: &str = r#"{
 "meta": {"results": {"skip": 0, "limit": 5, "total": 1}},
 "results": [
  {
   "set_id": "492dbdb2-077e-4064-bff3-372d6af0a7a2",
   "effective_time": "20260529",
   "openfda": {
    "brand_name": ["Herceptin"],
    "generic_name": ["TRASTUZUMAB"],
    "route": ["INTRAVENOUS"],
    "substance_name": ["TRASTUZUMAB"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE Herceptin is a HER2/neu receptor antagonist indicated in adults for: The treatment of HER2-overexpressing breast cancer. ( 1.1 , 1.2 ) The treatment of HER2 \u2026[recorded reply trimmed for the fixture]"
   ],
   "warnings_and_cautions": [
    "5 WARNINGS AND PRECAUTIONS Exacerbation of Chemotherapy-Induced Neutropenia. ( 5.5 , 6.1 ) 5.1 Cardiomyopathy Herceptin can cause left ventr \u2026[recorded reply trimmed for the fixture]"
   ],
   "drug_interactions": [
    "7 DRUG INTERACTIONS Anthracyclines Patients who receive anthracycline after stopping Herceptin may b \u2026[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

/// Search `trastuzumab` (live total 8): the OGIVRI biosimilar answers
/// first and Herceptin's own plain record third; the Kanjinti, Enhertu and
/// Ontruzant rows between them were removed for the fixture. On main this
/// order names the card's label OGIVRI's.
const TRASTUZUMAB_LABEL_BODY: &str = r#"{
 "meta": {"results": {"skip": 0, "limit": 5, "total": 8}},
 "results": [
  {
   "set_id": "b6465b44-a6dd-f99c-d009-6851cf05169c",
   "effective_time": "20260626",
   "openfda": {
    "brand_name": ["OGIVRI"],
    "generic_name": ["TRASTUZUMAB-DKST"],
    "route": ["INTRAVENOUS"],
    "substance_name": ["TRASTUZUMAB"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE Ogivri is a HER2/neu receptor antagonist indicated in adults for: The treatment of HER2-overexpressing breast cancer. ( 1.1 , 1.2 ) The treatment of HER2-ov \u2026[recorded reply trimmed for the fixture]"
   ],
   "warnings_and_cautions": [
    "5 WARNINGS AND PRECAUTIONS \u2022 Exacerbation of Chemotherapy-Induced Neutropenia. ( 5.5 , 6.1 ) 5.1 Cardiomyopathy Trastuzumab products can cau \u2026[recorded reply trimmed for the fixture]"
   ]
  },
  {
   "set_id": "492dbdb2-077e-4064-bff3-372d6af0a7a2",
   "effective_time": "20260529",
   "openfda": {
    "brand_name": ["Herceptin"],
    "generic_name": ["TRASTUZUMAB"],
    "route": ["INTRAVENOUS"],
    "substance_name": ["TRASTUZUMAB"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE Herceptin is a HER2/neu receptor antagonist indicated in adults for: The treatment of HER2-overexpressing breast cancer. ( 1.1 , 1.2 ) The treatment of HER2 \u2026[recorded reply trimmed for the fixture]"
   ],
   "warnings_and_cautions": [
    "5 WARNINGS AND PRECAUTIONS Exacerbation of Chemotherapy-Induced Neutropenia. ( 5.5 , 6.1 ) 5.1 Cardiomyopathy Herceptin can cause left ventr \u2026[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

/// Search `Rybrevant` and `amivantamab` (identical live answers): the
/// under-the-skin Rybrevant Faspro combination answers first and the plain
/// intravenous Rybrevant record second.
const RYBREVANT_LABEL_BODY: &str = r#"{
 "meta": {"results": {"skip": 0, "limit": 5, "total": 2}},
 "results": [
  {
   "set_id": "9e58b045-e352-4f62-99bf-77fae1bebc69",
   "effective_time": "20260617",
   "openfda": {
    "brand_name": ["Rybrevant Faspro"],
    "generic_name": ["AMIVANTAMAB AND HYALURONIDASE-LPUJ (HUMAN RECOMBINANT)"],
    "route": ["SUBCUTANEOUS"],
    "substance_name": ["AMIVANTAMAB", "HYALURONIDASE (HUMAN RECOMBINANT)"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE RYBREVANT FASPRO is a combination of amivantamab, a bispecific EGF receptor-directed and MET receptor-directed antibody, and hyaluronidase, an endoglycosida \u2026[recorded reply trimmed for the fixture]"
   ],
   "warnings_and_cautions": [
    "5 WARNINGS AND PRECAUTIONS Hypersensitivity and Administration-Related Reactions (ARR) : Premedicate with antihistamines, antipyretics, and  \u2026[recorded reply trimmed for the fixture]"
   ]
  },
  {
   "set_id": "1466c070-9f97-4fa4-a955-6a6b59981fb8",
   "effective_time": "20251111",
   "openfda": {
    "brand_name": ["Rybrevant"],
    "generic_name": ["AMIVANTAMAB-VMJW"],
    "route": ["INTRAVENOUS"],
    "substance_name": ["AMIVANTAMAB"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE RYBREVANT is a bispecific EGF receptor-directed and MET receptor-directed antibody indicated: in combination with lazertinib for the first-line treatment of \u2026[recorded reply trimmed for the fixture]"
   ],
   "warnings_and_cautions": [
    "5 WARNINGS AND PRECAUTIONS Infusion-Related Reactions (IRR) : Interrupt infusion at the first sign of IRRs. Reduce the infusion rate or perm \u2026[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

/// Search `Avastin` (live total 1): Avastin's own plain record.
const AVASTIN_LABEL_BODY: &str = r#"{
 "meta": {"results": {"skip": 0, "limit": 5, "total": 1}},
 "results": [
  {
   "set_id": "939b5d1f-9fb2-4499-80ef-0607aa6b114e",
   "effective_time": "20250106",
   "openfda": {
    "brand_name": ["Avastin"],
    "generic_name": ["BEVACIZUMAB"],
    "route": ["INTRAVENOUS"],
    "substance_name": ["BEVACIZUMAB"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE Avastin is a vascular endothelial growth factor inhibitor indicated for the treatment of: Metastatic colorectal cancer, in combination with intravenous fluo \u2026[recorded reply trimmed for the fixture]"
   ],
   "warnings_and_cautions": [
    "5 WARNINGS AND PRECAUTIONS Gastrointestinal Perforations and Fistula : Discontinue for gastrointestinal perforations, tracheoesophageal fist \u2026[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

/// Search `bevacizumab` (live total 7): all five newest-first records are
/// biosimilars — Vegzelma, Zirabev, Mvasi, Alymsys, Jobevne — and Avastin's
/// own plain record sits past the page, which is what the exact-field
/// escalation exists to reach.
const BEVACIZUMAB_BIOSIMILAR_PAGE_BODY: &str = r#"{
 "meta": {"results": {"skip": 0, "limit": 5, "total": 7}},
 "results": [
  {
   "set_id": "4cdcccda-7b9a-49d0-ac50-f4605a3c0ebe",
   "effective_time": "20260821",
   "openfda": {
    "brand_name": ["Vegzelma"],
    "generic_name": ["BEVACIZUMAB-ADCD"],
    "route": ["INTRAVENOUS"],
    "substance_name": ["BEVACIZUMAB"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE VEGZELMA is a vascular endothelial growth factor inhibitor indicated for the treatment of: Metastatic colorectal cancer, in combination with intravenous flu \u2026[recorded reply trimmed for the fixture]"
   ]
  },
  {
   "set_id": "aa27acbd-d117-4350-aeee-17bc2e2c0ca4",
   "effective_time": "20260721",
   "openfda": {
    "brand_name": ["Zirabev"],
    "generic_name": ["BEVACIZUMAB-BVZR"],
    "route": ["INTRAVENOUS"],
    "substance_name": ["BEVACIZUMAB"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE ZIRABEV is a vascular endothelial growth factor inhibitor indicated for the treatment of: \u2022 Metastatic colorectal cancer, in combination with intravenous fl \u2026[recorded reply trimmed for the fixture]"
   ]
  },
  {
   "set_id": "0a562b4e-67ce-4bab-852c-d9ee71e55fb8",
   "effective_time": "20260623",
   "openfda": {
    "brand_name": ["MVASI"],
    "generic_name": ["BEVACIZUMAB-AWWB"],
    "route": ["INTRAVENOUS"],
    "substance_name": ["BEVACIZUMAB"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE MVASI is a vascular endothelial growth factor inhibitor indicated for the treatment of: Metastatic colorectal cancer, in combination with intravenous fluoro \u2026[recorded reply trimmed for the fixture]"
   ]
  },
  {
   "set_id": "6b4040a2-a5c5-4ff0-ab45-935d7e49cf78",
   "effective_time": "20260323",
   "openfda": {
    "brand_name": ["ALYMSYS"],
    "generic_name": ["BEVACIZUMAB-MALY"],
    "route": ["INTRAVENOUS"],
    "substance_name": ["BEVACIZUMAB"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE Alymsys is a vascular endothelial growth factor inhibitor indicated for the treatment of: Metastatic colorectal cancer, in combination with intravenous fluo \u2026[recorded reply trimmed for the fixture]"
   ]
  },
  {
   "set_id": "70ab1de6-fb68-aee4-a6cb-f9a0f146687f",
   "effective_time": "20260311",
   "openfda": {
    "brand_name": ["JOBEVNE"],
    "generic_name": ["BEVACIZUMAB-NWGD"],
    "route": ["INTRAVASCULAR"],
    "substance_name": ["BEVACIZUMAB"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE Jobevne is a vascular endothelial growth factor inhibitor indicated for the treatment of: \u2022 Metastatic colorectal cancer, in combination with intravenous fl \u2026[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

/// Search `Darzalex` (live total 1): the only current daratumumab label is
/// the under-the-skin DARZALEX FASPRO pairing.
const DARZALEX_LABEL_BODY: &str = r#"{
 "meta": {"results": {"skip": 0, "limit": 5, "total": 1}},
 "results": [
  {
   "set_id": "4bb241af-4299-4373-8762-2d6709515db0",
   "effective_time": "20260903",
   "openfda": {
    "brand_name": ["Darzalex Faspro"],
    "generic_name": ["DARATUMUMAB AND HYALURONIDASE-FIHJ (HUMAN RECOMBINANT)"],
    "route": ["SUBCUTANEOUS"],
    "substance_name": ["DARATUMUMAB", "HYALURONIDASE (HUMAN RECOMBINANT)"]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE DARZALEX FASPRO is a combination of daratumumab, a CD38-directed cytolytic antibody, and hyaluronidase, an endoglycosidase, indicated for the treatment of a \u2026[recorded reply trimmed for the fixture]"
   ],
   "warnings_and_cautions": [
    "5 WARNINGS AND PRECAUTIONS Hypersensitivity and Other Administration Reactions : Permanently discontinue DARZALEX FASPRO for life-threatenin \u2026[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

/// The live MyChem answers below were recorded 2026-10-13 (ticket 2043)
/// from `mychem.info/v1/query?q=<name>` with the `get` field list. Hits
/// that carry no name field were dropped, product rows deduplicated to
/// first occurrences and long synonym lists trimmed, as each header notes.

/// `q=herceptin`: the brand lives on a naked NDC row pairing HERCEPTIN
/// with Trastuzumab, beside the Hylecto combination row and two records
/// with no name in the `get` field list.
const HERCEPTIN_CAPTURE: &str = r#"
{
  "total": 4,
  "hits": [
    {
      "_id": "50242-077",
      "_score": 17.670006,
      "ndc": {
        "nonproprietaryname": "Trastuzumab and hyaluronidase-oysk",
        "proprietaryname": "Herceptin Hylecta"
      }
    },
    {
      "_id": "50242-132",
      "_score": 16.795,
      "ndc": {"nonproprietaryname": "Trastuzumab", "proprietaryname": "Herceptin"}
    }
  ]
}
"#;

/// `q=trastuzumab`: the DrugBank/UNII identity record beside naked product
/// rows — Kadcyla's conjugate, two OGIVRI rows, and Herceptin's own row.
/// Rows deduplicated to first occurrences; the fieldless text-only hits
/// and the remaining biosimilar rows removed.
const TRASTUZUMAB_CAPTURE: &str = r#"
{
  "total": 100,
  "hits": [
    {
      "_id": "50242-088",
      "_score": 19.983921,
      "ndc": {"nonproprietaryname": "ADO-TRASTUZUMAB EMTANSINE", "proprietaryname": "KADCYLA"}
    },
    {
      "_id": "67457-991",
      "_score": 19.983921,
      "ndc": {"nonproprietaryname": "trastuzumab", "proprietaryname": "OGIVRI"}
    },
    {
      "_id": "83257-003",
      "_score": 19.983921,
      "ndc": {"nonproprietaryname": "trastuzumab-dkst", "proprietaryname": "OGIVRI"}
    },
    {
      "_id": "P188ANX8CK",
      "_score": 19.983921,
      "drugbank": {
        "id": "DB00072",
        "name": "Trastuzumab",
        "synonyms": ["Trastuzumab", "trastuzumab-dkst"]
      },
      "unii": {"unii": "P188ANX8CK", "display_name": "TRASTUZUMAB"}
    },
    {
      "_id": "50242-132",
      "_score": 19.700676,
      "ndc": {"nonproprietaryname": "Trastuzumab", "proprietaryname": "Herceptin"}
    }
  ]
}
"#;

/// `q=pembrolizumab`: the DrugBank/UNII identity record with DrugCentral
/// synonyms naming the brand, beside the QLEX combination row and the
/// plain KEYTRUDA row. Synonym lists trimmed; the fieldless text-only hits
/// and the duplicate QLEX row removed.
const PEMBROLIZUMAB_CAPTURE: &str = r#"
{
  "total": 14,
  "hits": [
    {
      "_id": "DPT0O3T46P",
      "_score": 25.60765,
      "drugbank": {
        "id": "DB09037",
        "name": "Pembrolizumab",
        "synonyms": ["Lambrolizumab", "Pembrolizumab"]
      },
      "unii": {"unii": "DPT0O3T46P", "display_name": "PEMBROLIZUMAB"},
      "drugcentral": {"synonyms": ["pembrolizumab", "keytruda", "lambrolizumab"]}
    },
    {
      "_id": "0006-3083",
      "_score": 22.244265,
      "ndc": {
        "nonproprietaryname": "pembrolizumab and berahyaluronidase alfa-pmph",
        "proprietaryname": "KEYTRUDA QLEX"
      }
    },
    {
      "_id": "0006-3026",
      "_score": 22.238565,
      "ndc": {"nonproprietaryname": "pembrolizumab", "proprietaryname": "KEYTRUDA"}
    }
  ]
}
"#;

/// `q=avastin`: MyChem holds the brand only on naked NDC rows pairing
/// Avastin with bevacizumab. The duplicate row and the fieldless hits were
/// removed.
const AVASTIN_CAPTURE: &str = r#"
{
  "total": 4,
  "hits": [
    {
      "_id": "50242-060",
      "_score": 16.80278,
      "ndc": {"nonproprietaryname": "bevacizumab", "proprietaryname": "Avastin"}
    }
  ]
}
"#;

/// `q=bevacizumab`: the DrugBank/UNII identity record beside biosimilar
/// rows and Avastin's own row. Rows deduplicated to one per product;
/// synonym lists trimmed; the fieldless text-only hits removed.
const BEVACIZUMAB_CAPTURE: &str = r#"
{
  "total": 37,
  "hits": [
    {
      "_id": "72606-012",
      "_score": 21.62735,
      "ndc": {"nonproprietaryname": "bevacizumab-adcd", "proprietaryname": "Vegzelma"}
    },
    {
      "_id": "2S9ZZM9Q9V",
      "_score": 20.876358,
      "drugbank": {
        "id": "DB00112",
        "name": "Bevacizumab",
        "synonyms": ["BAT1706", "Bevacizumab", "bevacizumab-awwb"]
      },
      "unii": {"unii": "2S9ZZM9Q9V", "display_name": "BEVACIZUMAB"}
    },
    {
      "_id": "0069-0315",
      "_score": 20.876358,
      "ndc": {"nonproprietaryname": "bevacizumab-bvzr", "proprietaryname": "Zirabev"}
    },
    {
      "_id": "83257-009",
      "_score": 20.876358,
      "ndc": {"nonproprietaryname": "bevacizumab-nwgd", "proprietaryname": "JOBEVNE"}
    },
    {
      "_id": "70121-1755",
      "_score": 20.876358,
      "ndc": {"nonproprietaryname": "bevacizumab-maly", "proprietaryname": "ALYMSYS"}
    },
    {
      "_id": "50242-060",
      "_score": 20.789146,
      "ndc": {"nonproprietaryname": "bevacizumab", "proprietaryname": "Avastin"}
    }
  ]
}
"#;

/// `q=darzalex`: the brand lives on naked NDC rows — the plain DARZALEX
/// row, the Faspro combination row and the DARZALEX IV row — beside
/// records with no name in the `get` field list.
const DARZALEX_CAPTURE: &str = r#"
{
  "total": 5,
  "hits": [
    {"_id": "PA166334883", "_score": 17.676718},
    {
      "_id": "57894-502",
      "_score": 16.53591,
      "ndc": {"nonproprietaryname": "Daratumumab", "proprietaryname": "DARZALEX"}
    },
    {
      "_id": "57894-503",
      "_score": 16.53591,
      "ndc": {
        "nonproprietaryname": "daratumumab and hyaluronidase-fihj (human recombinant)",
        "proprietaryname": "Darzalex Faspro"
      }
    },
    {
      "_id": "57894-505",
      "_score": 16.53591,
      "ndc": {"nonproprietaryname": "Daratumumab", "proprietaryname": "Darzalex IV"}
    },
    {"_id": "C4058940", "_score": 16.53591}
  ]
}
"#;

/// `q=daratumumab`: the DrugBank/UNII identity record beside the plain
/// DARZALEX row. The duplicate rows and the fieldless hits were removed.
const DARATUMUMAB_CAPTURE: &str = r#"
{
  "total": 15,
  "hits": [
    {
      "_id": "4Z63YK6E0E",
      "_score": 25.0,
      "drugbank": {"id": "DB09331", "name": "Daratumumab"},
      "unii": {"unii": "4Z63YK6E0E", "display_name": "DARATUMUMAB"}
    },
    {
      "_id": "57894-502",
      "_score": 16.53591,
      "ndc": {"nonproprietaryname": "Daratumumab", "proprietaryname": "DARZALEX"}
    }
  ]
}
"#;

fn assert_label_data_with_set_id(drug: &super::super::Drug, set_id: &str) {
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
        drug.label
            .as_ref()
            .is_some_and(|label| !label.indication_summary.is_empty()),
        "the chosen label's text reached the card"
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn keytruda_takes_keytrudas_own_label_not_the_qlex_pairing() {
    let (base, server) = name_resolution_fixture_server(
        vec![
            (
                "Keytruda".to_string(),
                crate::transform::drug::name_resolution_tests::KEYTRUDA_CAPTURE.to_string(),
            ),
            (
                "pembrolizumab".to_string(),
                PEMBROLIZUMAB_CAPTURE.to_string(),
            ),
        ],
        vec![
            ("Keytruda".to_string(), KEYTRUDA_LABEL_BODY.to_string()),
            ("pembrolizumab".to_string(), KEYTRUDA_LABEL_BODY.to_string()),
        ],
        Vec::new(),
    )
    .await;

    // Ticket 2043: the brand query takes its own brand's label. openFDA
    // answers the under-the-skin QLEX combination first, and only the plain
    // KEYTRUDA record's brand name equals the query. On main the card
    // searched its own name, took QLEX's record and stayed a sparse card.
    let drug = name_resolution_fixture_drug(&base, "Keytruda").await;
    assert_eq!(drug.name, "pembrolizumab");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB09037"));
    assert_label_data_with_set_id(&drug, "9333c79b-d487-4538-a9f0-71b91a02b287");
    server.abort();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn pembrolizumab_takes_the_plain_keytruda_label() {
    let (base, server) = name_resolution_fixture_server(
        vec![(
            "pembrolizumab".to_string(),
            PEMBROLIZUMAB_CAPTURE.to_string(),
        )],
        vec![("pembrolizumab".to_string(), KEYTRUDA_LABEL_BODY.to_string())],
        Vec::new(),
    )
    .await;

    // Ticket 2043: the ingredient query takes the plain product's label,
    // never the under-the-skin combination, even though openFDA answers
    // QLEX first.
    let drug = name_resolution_fixture_drug(&base, "pembrolizumab").await;
    assert_eq!(drug.name, "pembrolizumab");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB09037"));
    assert_label_data_with_set_id(&drug, "9333c79b-d487-4538-a9f0-71b91a02b287");
    server.abort();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn herceptin_takes_herceptins_own_label_not_the_biosimilar() {
    let (base, server) = name_resolution_fixture_server(
        vec![
            ("Herceptin".to_string(), HERCEPTIN_CAPTURE.to_string()),
            ("trastuzumab".to_string(), TRASTUZUMAB_CAPTURE.to_string()),
        ],
        vec![
            ("Herceptin".to_string(), HERCEPTIN_LABEL_BODY.to_string()),
            (
                "trastuzumab".to_string(),
                TRASTUZUMAB_LABEL_BODY.to_string(),
            ),
        ],
        Vec::new(),
    )
    .await;

    // Ticket 2043: Herceptin's own label first, the biosimilar only if no
    // other. The card resolves through the openFDA identity fallback and
    // the request's own name travels to the label lookup.
    let drug = name_resolution_fixture_drug(&base, "Herceptin").await;
    assert_eq!(drug.name, "trastuzumab");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB00072"));
    assert_label_data_with_set_id(&drug, "492dbdb2-077e-4064-bff3-372d6af0a7a2");
    server.abort();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn trastuzumab_takes_the_plain_herceptin_label_not_the_biosimilar() {
    let (base, server) = name_resolution_fixture_server(
        vec![("trastuzumab".to_string(), TRASTUZUMAB_CAPTURE.to_string())],
        vec![(
            "trastuzumab".to_string(),
            TRASTUZUMAB_LABEL_BODY.to_string(),
        )],
        Vec::new(),
    )
    .await;

    // Ticket 2043: the ingredient query takes the label whose generic name
    // is exactly the ingredient — Herceptin's record — and never the
    // biosimilar records that answer first.
    let drug = name_resolution_fixture_drug(&base, "trastuzumab").await;
    assert_eq!(drug.name, "trastuzumab");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB00072"));
    assert_label_data_with_set_id(&drug, "492dbdb2-077e-4064-bff3-372d6af0a7a2");
    server.abort();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn rybrevant_takes_the_plain_rybrevant_label_not_faspro() {
    let (base, server) = name_resolution_fixture_server(
        vec![(
            "Rybrevant".to_string(),
            crate::transform::drug::name_resolution_tests::RYBREVANT_CAPTURE.to_string(),
        )],
        vec![
            ("Rybrevant".to_string(), RYBREVANT_LABEL_BODY.to_string()),
            ("amivantamab".to_string(), RYBREVANT_LABEL_BODY.to_string()),
        ],
        Vec::new(),
    )
    .await;

    // Ticket 2043: the brand query takes the record whose brand name
    // equals it — the intravenous RYBREVANT record — while openFDA answers
    // the under-the-skin Faspro combination first.
    let drug = name_resolution_fixture_drug(&base, "Rybrevant").await;
    assert_eq!(drug.name, "amivantamab");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB16695"));
    assert_label_data_with_set_id(&drug, "1466c070-9f97-4fa4-a955-6a6b59981fb8");
    server.abort();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn amivantamab_takes_the_plain_rybrevant_label_not_faspro() {
    let (base, server) = name_resolution_fixture_server(
        vec![(
            "amivantamab".to_string(),
            crate::transform::drug::name_resolution_tests::AMIVANTAMAB_CAPTURE.to_string(),
        )],
        vec![("amivantamab".to_string(), RYBREVANT_LABEL_BODY.to_string())],
        Vec::new(),
    )
    .await;

    // Ticket 2043: no label's generic name is exactly "amivantamab" — the
    // plain product carries the proper-name suffix "amivantamab-vmjw" — so
    // the qualified form serves after the combination is skipped and the
    // exact-field escalation (which openFDA answers with no match) settles
    // without a record.
    let drug = name_resolution_fixture_drug(&base, "amivantamab").await;
    assert_eq!(drug.name, "amivantamab");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB16695"));
    assert_label_data_with_set_id(&drug, "1466c070-9f97-4fa4-a955-6a6b59981fb8");
    server.abort();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn avastin_takes_avastins_own_label() {
    let (base, server) = name_resolution_fixture_server(
        vec![
            ("Avastin".to_string(), AVASTIN_CAPTURE.to_string()),
            ("bevacizumab".to_string(), BEVACIZUMAB_CAPTURE.to_string()),
        ],
        vec![
            ("Avastin".to_string(), AVASTIN_LABEL_BODY.to_string()),
            (
                "bevacizumab".to_string(),
                BEVACIZUMAB_BIOSIMILAR_PAGE_BODY.to_string(),
            ),
            // The exact-field escalation: openFDA stores generic names in
            // SPL casing, so the plan sends the uppercased card name.
            ("BEVACIZUMAB".to_string(), AVASTIN_LABEL_BODY.to_string()),
        ],
        Vec::new(),
    )
    .await;

    // Ticket 2043: the brand query takes Avastin's own record even though
    // MyChem holds the brand only on a naked product row and the card
    // resolves through the identity fallback.
    let drug = name_resolution_fixture_drug(&base, "Avastin").await;
    assert_eq!(drug.name, "bevacizumab");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB00112"));
    assert_label_data_with_set_id(&drug, "939b5d1f-9fb2-4499-80ef-0607aa6b114e");
    server.abort();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn bevacizumab_escalates_past_the_biosimilar_page_to_avastin() {
    let (base, server) = name_resolution_fixture_server(
        vec![("bevacizumab".to_string(), BEVACIZUMAB_CAPTURE.to_string())],
        vec![
            (
                "bevacizumab".to_string(),
                BEVACIZUMAB_BIOSIMILAR_PAGE_BODY.to_string(),
            ),
            ("BEVACIZUMAB".to_string(), AVASTIN_LABEL_BODY.to_string()),
        ],
        Vec::new(),
    )
    .await;

    // Ticket 2043: openFDA's newest-first page holds only biosimilars, so
    // the exact-field escalation must ask for the plain product's own
    // record before any suffixed biosimilar can serve as the qualified
    // form.
    let drug = name_resolution_fixture_drug(&base, "bevacizumab").await;
    assert_eq!(drug.name, "bevacizumab");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB00112"));
    assert_label_data_with_set_id(&drug, "939b5d1f-9fb2-4499-80ef-0607aa6b114e");
    server.abort();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn darzalex_resolves_daratumumab_through_the_pairings_plain_ingredient() {
    let (base, server) = name_resolution_fixture_server(
        vec![
            ("Darzalex".to_string(), DARZALEX_CAPTURE.to_string()),
            ("daratumumab".to_string(), DARATUMUMAB_CAPTURE.to_string()),
        ],
        vec![("Darzalex".to_string(), DARZALEX_LABEL_BODY.to_string())],
        Vec::new(),
    )
    .await;

    // Ticket 2043: the only current daratumumab label is the under-the-skin
    // Faspro pairing, whose generic name is "daratumumab and
    // hyaluronidase-fihj"; the identity fallback resolves the brand to the
    // plain ingredient that pairing names. On main the combination string
    // itself was the candidate, no MyChem record named it, and the card
    // stayed sparse.
    let drug = name_resolution_fixture_drug(&base, "Darzalex").await;
    assert_eq!(drug.name, "daratumumab");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB09331"));
    assert_label_data_with_set_id(&drug, "4bb241af-4299-4373-8762-2d6709515db0");
    server.abort();
}

/// The OLS4 answer for a brand whose only concept match is a non-exact
/// regimen-shaped label, recorded 2026-10-13 from the live `q=Ambifton`
/// shape: no label or synonym equals the query, so the discover rescue
/// must decline and `get` keeps the honest refusal. This pin bites when
/// the rescue's exact-match requirement on its top result is removed
/// (ticket 2043's first mutation).
#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_contains_tier_ols_match_never_adopts_a_card() {
    let ols_contains_body = r#"{"response":{"docs":[
        {"iri":"http://purl.obolibrary.org/obo/CHEBI_888881","ontology_prefix":"CHEBI","obo_id":"CHEBI:888881","label":"Ambifton Compounded Product","exact_synonyms":["ambifton-based product"]}
    ]}}"#;
    let (base, server) = name_resolution_fixture_server(
        vec![
            (
                "Ambifton".to_string(),
                r#"{"total":1,"hits":[{"_id":"C9999999","_score":17.668518}]}"#.to_string(),
            ),
            (
                "Ambifton Compounded Product".to_string(),
                r#"{"total":1,"hits":[{"_id":"REG1","_score":10.0,"drugbank":{"id":"DB999991","name":"Ambifton Compounded Product"}}]}"#
                    .to_string(),
            ),
        ],
        Vec::new(),
        vec![("Ambifton".to_string(), ols_contains_body.to_string())],
    )
    .await;

    let root = crate::test_support::TempDirGuard::new("ambifton-contains-ddinter");
    let missing_ddinter = root.path().join("missing-ddinter");
    let cache_root = crate::test_support::TempDirGuard::new("ambifton-contains-cache");
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

    // The regimen-shaped concept is a drug with a canonical id but only a
    // non-exact (prefix-tier) match, so the guard declines and the refusal
    // names the miss. With the exact-match requirement removed the rescue
    // would adopt it and return its card.
    let err = super::super::get("Ambifton", &["label".to_string()])
        .await
        .expect_err("a contains-tier match must not adopt a card");
    let message = err.to_string();
    assert!(
        message.contains("No drug card matches \"Ambifton\""),
        "{message}"
    );
    server.abort();
}

/// The discover rescue's canonical candidate is adopted only through
/// `named_drug_response`: a candidate whose own MyChem answer holds no
/// record naming it leaves the sparse card alone. This pin bites when the
/// adoption skips that check (ticket 2043's second mutation).
#[tokio::test]
#[serial_test::serial(source_env)]
async fn the_rescue_adopts_a_candidate_only_through_its_own_record() {
    let ols_exact_body = r#"{"response":{"docs":[
        {"iri":"http://purl.obolibrary.org/obo/CHEBI_999990","ontology_prefix":"CHEBI","obo_id":"CHEBI:999990","label":"Fixture Plain Drug","exact_synonyms":["Ambifton"]}
    ]}}"#;
    let (base, server) = name_resolution_fixture_server(
        vec![
            (
                "Ambifton".to_string(),
                r#"{"total":1,"hits":[{"_id":"1","_score":22.0,"ndc":{"nonproprietaryname":"ambifton"}}]}"#
                    .to_string(),
            ),
            (
                "Fixture Plain Drug".to_string(),
                r#"{"total":1,"hits":[{"_id":"2","_score":10.0,"drugbank":{"id":"DB777777","name":"Fixture Related"}}]}"#
                    .to_string(),
            ),
        ],
        Vec::new(),
        vec![("Ambifton".to_string(), ols_exact_body.to_string())],
    )
    .await;

    // The exact canonical concept exists, but its MyChem answer names only
    // "Fixture Related", so the adoption declines and the sparse ambifton
    // card stands. Adopting the response anyway renames the card after the
    // candidate while carrying no record of its own.
    let drug = name_resolution_fixture_drug(&base, "Ambifton").await;
    assert_eq!(drug.name, "ambifton");
    assert!(
        drug.drugbank_id.is_none(),
        "no record that names the candidate was adopted"
    );
    server.abort();
}
