//! Recorded-capture pins for drug name resolution (ticket 2031).
//!
//! Every capture below was recorded 2026-10-08 from
//! `mychem.info/v1/query?q=<name>` with the `get` field list. Hits that carry
//! no name field are dropped, NDC rows are deduplicated to each row name's
//! first occurrence with unused rows between the ones the assertions read
//! removed in place (order preserved), and long synonym lists are trimmed
//! the same way.

use super::{merge_mychem_hits, select_hits_for_name};
use crate::sources::mychem::MyChemHit;

fn hits(value: serde_json::Value) -> Vec<MyChemHit> {
    value
        .get("hits")
        .and_then(|hits| hits.as_array())
        .expect("capture carries hits")
        .iter()
        .map(|hit| serde_json::from_value(hit.clone()).expect("recorded hit deserializes"))
        .collect()
}

fn selected_ids(hits: &[MyChemHit], query: &str) -> Vec<String> {
    select_hits_for_name(hits, query)
        .iter()
        .map(|hit| {
            hit.drugbank
                .as_ref()
                .and_then(|d| d.id.clone())
                .unwrap_or_default()
        })
        .collect()
}

/// `q=terfenadine`: MyChem returns terfenadine itself (DB00342) plus
/// fexofenadine (DB00950), whose DrugBank synonyms list "Terfenadine
/// carboxylate" and "Terfenadine acid metabolite". The richer fexofenadine
/// record used to win the merge and name the card.
pub(crate) const TERFENADINE_CAPTURE: &str = r#"
{
  "total": 4,
  "hits": [
    {
      "_id": "GUGOEEXESWIERI-UHFFFAOYSA-N",
      "_score": 30.143068,
      "drugbank": {
        "id": "DB00342",
        "name": "Terfenadine",
        "synonyms": [
          "(RS)-1-(4-tert-butylphenyl)-4-{4-[hydroxy(diphenyl)methyl]piperidin-1-yl}-butan-1-ol",
          "Terfenadin",
          "Terfenadina",
          "Terfénadine",
          "Terfenadine",
          "Terfenadinum"
        ]
      },
      "chembl": {"molecule_chembl_id": "CHEMBL17157", "pref_name": "TERFENADINE"},
      "unii": {"unii": "7BA5G9Y06Q", "display_name": "TERFENADINE"},
      "chebi": {"name": "Terfenadine"}
    },
    {
      "_id": "RWTNPBWLLIMQHL-UHFFFAOYSA-N",
      "_score": 26.00888,
      "drugbank": {
        "id": "DB00950",
        "name": "Fexofenadine",
        "synonyms": [
          "Carboxyterfenadine",
          "Fexofenadina",
          "Fexofenadine",
          "Terfenadine acid metabolite",
          "Terfenadine carboxylate",
          "Terfenadine-COOH"
        ]
      },
      "chembl": {"molecule_chembl_id": "CHEMBL914", "pref_name": "FEXOFENADINE"},
      "ndc": [
        {"nonproprietaryname": "Fexofenadine Hydrochloride"},
        {"nonproprietaryname": "Fexofenadine HCl"},
        {"nonproprietaryname": "FEXOFENADINE HYDROCHLORIDE"}
      ],
      "unii": {"unii": "E6582LOH6V", "display_name": "FEXOFENADINE"},
      "chebi": [{"name": "fexofenadine zwitterion"}, {"name": "fexofenadine"}]
    },
    {
      "_id": "GUGOEEXESWIERI-SSEXGKCCSA-N",
      "_score": 16.795,
      "chembl": {"molecule_chembl_id": "CHEMBL303454", "pref_name": "R-TERFENADINE"},
      "unii": {"unii": "6NH2EV93XU", "display_name": "TERFENADINE, (R)-"}
    }
  ]
}
"#;

#[test]
fn terfenadine_selects_only_terfenadine_and_never_fexofenadine() {
    let capture = hits(serde_json::from_str(TERFENADINE_CAPTURE).expect("valid capture"));
    assert_eq!(
        selected_ids(&capture, "terfenadine"),
        vec!["DB00342".to_string()],
        "the fexofenadine record is admitted only through synonyms that do not equal the query"
    );

    let selected = select_hits_for_name(&capture, "terfenadine");
    let drug = merge_mychem_hits(&selected, "terfenadine");
    assert_eq!(drug.name, "terfenadine");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB00342"));
    assert_eq!(drug.chembl_id.as_deref(), Some("CHEMBL17157"));
    assert!(
        !serde_json::to_string(&drug)
            .expect("card serializes")
            .to_ascii_lowercase()
            .contains("fexofenadine"),
        "the merged card carries no fexofenadine field"
    );
}

/// `q=mannitol`: the mannitol record (DB00742) is the top hit, but MyChem
/// merged NDC product rows into it whose established names are other
/// products — the first is "Analgesic" — and sorbitol, bortezomib D-mannitol
/// and mannitol busulfan also match the text query.
pub(crate) const MANNITOL_CAPTURE: &str = r#"
{
  "total": 68,
  "hits": [
    {
      "_id": "FBPFZTCFMRRESA-KVTDHHQDSA-N",
      "_score": 30.644459,
      "drugbank": {"id": "DB00742", "name": "Mannitol"},
      "chembl": {"molecule_chembl_id": "CHEMBL689", "pref_name": "MANNITOL"},
      "ndc": [
        {"nonproprietaryname": "Analgesic"},
        {"nonproprietaryname": "mannitol"},
        {"nonproprietaryname": "MANNITOL"},
        {"nonproprietaryname": "Anticoagulant Citrate Phosphate Dextrose (CPD) AND AS-5 Red Cell Preservative"},
        {"nonproprietaryname": "SORBITOL and MANNITOL"},
        {"nonproprietaryname": "Mannitol"}
      ],
      "unii": {"unii": "3OWL53L36A", "display_name": "MANNITOL"},
      "chebi": {"name": "D-mannitol"}
    },
    {
      "_id": "84549-024",
      "_score": 23.043425,
      "ndc": {"nonproprietaryname": "MANNITOL"}
    },
    {
      "_id": "FBPFZTCFMRRESA-JGWLITMVSA-N",
      "_score": 22.902176,
      "drugbank": {"id": "DB01638", "name": "Sorbitol"},
      "chembl": {"molecule_chembl_id": "CHEMBL1682", "pref_name": "SORBITOL"},
      "ndc": [
        {"nonproprietaryname": "Sorbitol"},
        {"nonproprietaryname": "SORBITOL and MANNITOL"}
      ],
      "unii": {"unii": "506T60A25R", "display_name": "SORBITOL"},
      "chebi": {"name": "D-glucitol"}
    },
    {
      "_id": "QDMRNLRJDHCHLB-DNNBANOASA-N",
      "_score": 20.26575,
      "drugbank": {"id": "DB16741", "name": "Bortezomib D-mannitol"},
      "chembl": {"molecule_chembl_id": "CHEMBL5315122", "pref_name": "BORTEZOMIB D-MANNITOL"},
      "unii": {"unii": "P2AWN9VSQ6", "display_name": "BORTEZOMIB D-MANNITOL"}
    },
    {
      "_id": "ODOISJJCWUVNDJ-WCTZXXKLSA-N",
      "_score": 14.859291,
      "drugbank": {"id": "DB12097", "name": "Mannitol busulfan"}
    }
  ]
}
"#;

#[test]
fn mannitol_names_the_card_mannitol_not_the_first_ndc_product() {
    let capture = hits(serde_json::from_str(MANNITOL_CAPTURE).expect("valid capture"));
    let selected_ids = selected_ids(&capture, "mannitol");
    assert_eq!(
        selected_ids,
        vec!["DB00742".to_string(), String::new()],
        "the exact-tier keeps the mannitol record and its own NDC row; the sorbitol and \
         combination records only mention mannitol in wider names"
    );

    let selected = select_hits_for_name(&capture, "mannitol");
    let drug = merge_mychem_hits(&selected, "mannitol");
    assert_eq!(drug.name, "mannitol");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB00742"));
    assert!(
        !serde_json::to_string(&drug)
            .expect("card serializes")
            .to_ascii_lowercase()
            .contains("analgesic"),
        "the analgesic NDC product row never names the card"
    );
}

/// `q=edetate disodium`: the top hit is edetate disodium anhydrous (DB14600)
/// whose only NDC names are "urea, glycerin, aloe, disodium edta" — a foot
/// cream — while the exact ChEMBL/UNII record for edetate disodium sits
/// further down, and edetate calcium disodium is a different drug.
pub(crate) const EDETATE_DISODIUM_CAPTURE: &str = r#"
{
  "total": 12,
  "hits": [
    {
      "_id": "ZGTMUACCHSMWAC-UHFFFAOYSA-L",
      "_score": 50.06777,
      "drugbank": {
        "id": "DB14600",
        "name": "Edetate disodium anhydrous",
        "synonyms": ["Anhydrous disodium edetate", "Edetate disodium anhydrous"]
      },
      "chembl": {"molecule_chembl_id": "CHEMBL1749", "pref_name": "EDETATE DISODIUM ANHYDROUS"},
      "ndc": [
        {"nonproprietaryname": "urea, glycerin, aloe, disodium edta"},
        {"nonproprietaryname": "urea, glycerin, aloe, disodium edta"}
      ],
      "unii": {"unii": "8NLQ36F6MM", "display_name": "EDETATE DISODIUM ANHYDROUS"}
    },
    {
      "_id": "SHWNNYZBHZIQQV-UHFFFAOYSA-J",
      "_score": 47.67743,
      "drugbank": {"id": "DB14598", "name": "Edetate calcium disodium anhydrous"},
      "chembl": {"molecule_chembl_id": "CHEMBL1200375", "pref_name": "EDETATE CALCIUM DISODIUM ANHYDROUS"},
      "ndc": {"nonproprietaryname": "Edetate Calcium Disodium"},
      "unii": {"unii": "8U5D034955", "display_name": "EDETATE CALCIUM DISODIUM ANHYDROUS"}
    },
    {
      "_id": "OVBJJZOQPCKUOR-UHFFFAOYSA-L",
      "_score": 39.596664,
      "chembl": {"molecule_chembl_id": "CHEMBL3989507", "pref_name": "EDETATE DISODIUM"},
      "unii": {"unii": "7FLD91C86K", "display_name": "EDETATE DISODIUM"},
      "chebi": [{"name": "edetate disodium"}, {"name": "EDTA disodium salt dihydrate"}]
    },
    {
      "_id": "PEDCQBHIVMGVHV-UHFFFAOYSA-N",
      "_score": 40.981567,
      "drugbank": {"id": "DB09462", "name": "Glycerin"},
      "chembl": {"molecule_chembl_id": "CHEMBL692", "pref_name": "GLYCERIN"},
      "ndc": [
        {"nonproprietaryname": "Glycerin, Povidone"},
        {"nonproprietaryname": "urea, glycerin, aloe, disodium edta"}
      ],
      "unii": {"unii": "PDC6A3C0OX", "display_name": "GLYCERIN"}
    }
  ]
}
"#;

#[test]
fn edetate_disodium_prefers_the_exact_record_and_never_the_urea_cream() {
    let capture = hits(serde_json::from_str(EDETATE_DISODIUM_CAPTURE).expect("valid capture"));
    assert_eq!(
        selected_ids(&capture, "edetate disodium"),
        vec![String::new()],
        "only the ChEMBL/UNII record names edetate disodium exactly; the anhydrous and \
         calcium-disodium records are qualified or different drugs"
    );

    let selected = select_hits_for_name(&capture, "edetate disodium");
    let drug = merge_mychem_hits(&selected, "edetate disodium");
    assert_eq!(drug.name, "edetate disodium");
    assert_eq!(drug.chembl_id.as_deref(), Some("CHEMBL3989507"));
    assert!(
        !serde_json::to_string(&drug)
            .expect("card serializes")
            .to_ascii_lowercase()
            .contains("urea"),
        "the urea foot-cream NDC row never reaches the card"
    );
}

/// `q=ferric oxide`: the ferric oxide record (DB11576) carries NDC rows for
/// calamine and pramoxine products, and zinc oxide and pramocaine records
/// match the text query the same way.
pub(crate) const FERRIC_OXIDE_CAPTURE: &str = r#"
{
  "total": 56,
  "hits": [
    {
      "_id": "XLOMVQKBTHCTTD-UHFFFAOYSA-N",
      "_score": 49.535606,
      "drugbank": {"id": "DB09321", "name": "Zinc oxide"},
      "chebi": {"name": "zinc oxide"},
      "ndc": [
        {"nonproprietaryname": "Zinc Oxide, Titanium Dioxide"},
        {"nonproprietaryname": "ZINC OXIDE"},
        {"nonproprietaryname": "Zinc Oxide, Ferric Oxide Red, and Pramoxine Hydrochloride"}
      ]
    },
    {
      "_id": "JEIPFZHSYJVQDO-UHFFFAOYSA-N",
      "_score": 49.343464,
      "drugbank": {
        "id": "DB11576",
        "name": "Ferric oxide",
        "synonyms": ["Anhydrous Ferric Oxide", "Ferric oxide", "Ferric oxide red"]
      },
      "chebi": {"name": "ferric oxide"},
      "ndc": [
        {"nonproprietaryname": "Calamine and Zinc Oxide"},
        {"nonproprietaryname": "Calamine Plus Pramoxine HCL"},
        {"nonproprietaryname": "Calamine, Pramoxine HCl"},
        {"nonproprietaryname": "FERRIC OXIDE RED"},
        {"nonproprietaryname": "Zinc Oxide, Ferric Oxide Red, and Pramoxine Hydrochloride"}
      ]
    },
    {
      "_id": "DQKXQSGTHWVTAD-UHFFFAOYSA-N",
      "_score": 48.144127,
      "drugbank": {"id": "DB09345", "name": "Pramocaine"},
      "ndc": [
        {"nonproprietaryname": "Calamine and Pramoxine Hydrochloride"},
        {"nonproprietaryname": "PRAMOXINE HYDROCHLORIDE"}
      ]
    },
    {
      "_id": "83324-248",
      "_score": 34.17872,
      "ndc": {"nonproprietaryname": "FERRIC OXIDE RED"}
    }
  ]
}
"#;

#[test]
fn ferric_oxide_selects_the_ferric_oxide_record_not_calamine() {
    let capture = hits(serde_json::from_str(FERRIC_OXIDE_CAPTURE).expect("valid capture"));
    assert_eq!(
        selected_ids(&capture, "ferric oxide"),
        vec!["DB11576".to_string()],
        "only the ferric oxide record carries the exact name; the calamine product rows \
         on other records are wider-name matches"
    );

    let selected = select_hits_for_name(&capture, "ferric oxide");
    let drug = merge_mychem_hits(&selected, "ferric oxide");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB11576"));
    assert_eq!(
        drug.name, "ferric oxide red",
        "the card keeps the NDC established name that matched the query"
    );
    assert!(
        !serde_json::to_string(&drug)
            .expect("card serializes")
            .to_ascii_lowercase()
            .contains("calamine"),
        "no calamine product name reaches the card"
    );
}

/// `q=amivantamab`: the DrugBank record (DB16695) names the drug exactly
/// while the NDC rows around it carry "amivantamab-vmjw" and combination
/// names (ticket 2031 keep-list).
pub(crate) const AMIVANTAMAB_CAPTURE: &str = r#"
{
  "total": 11,
  "hits": [
    {
      "_id": "57894-501",
      "_score": 23.345608,
      "ndc": {"nonproprietaryname": "amivantamab-vmjw"}
    },
    {
      "_id": "57894-514",
      "_score": 22.519108,
      "ndc": {"nonproprietaryname": "AMIVANTAMAB and HYALURONIDASE-lpuj (HUMAN RECOMBINANT)"}
    },
    {
      "_id": "0JSR7Z0NB6",
      "_score": 16.336586,
      "drugbank": {
        "id": "DB16695",
        "name": "Amivantamab",
        "synonyms": ["Amivantamab", "amivantamab-vmjw"]
      },
      "unii": {"unii": "0JSR7Z0NB6", "display_name": "AMIVANTAMAB"}
    },
    {
      "_id": "CHEMBL4297774",
      "_score": 16.336586,
      "chembl": {"molecule_chembl_id": "CHEMBL4297774", "pref_name": "AMIVANTAMAB"}
    }
  ]
}
"#;

#[test]
fn amivantamab_keeps_resolving_through_its_exact_drugbank_record() {
    let capture = hits(serde_json::from_str(AMIVANTAMAB_CAPTURE).expect("valid capture"));
    let selected = select_hits_for_name(&capture, "amivantamab");
    let drug = merge_mychem_hits(&selected, "amivantamab");
    assert_eq!(drug.name, "amivantamab");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB16695"));
}

/// `q=TAGRISSO`: the MyChem fields fetched carry no brand array, so no hit
/// names the query and `get` must fall through to the openFDA identity
/// fallback instead of merging the osimertinib record unasked (ticket 2031
/// keep-list).
pub(crate) const TAGRISSO_CAPTURE: &str = r#"
{
  "total": 4,
  "hits": [
    {
      "_id": "DUYJMQONPNNFPI-UHFFFAOYSA-N",
      "_score": 24.06902,
      "drugbank": {"id": "DB09330", "name": "Osimertinib"},
      "chembl": {"molecule_chembl_id": "CHEMBL3353410", "pref_name": "OSIMERTINIB"},
      "ndc": {"nonproprietaryname": "osimertinib"},
      "unii": {"unii": "3C06JJ0Z2O", "display_name": "OSIMERTINIB"},
      "chebi": {"name": "osimertinib"}
    },
    {
      "_id": "0310-1248",
      "_score": 17.670006,
      "ndc": {"nonproprietaryname": "osimertinib"}
    },
    {
      "_id": "0310-1251",
      "_score": 17.141884,
      "ndc": {"nonproprietaryname": "osimertinib"}
    }
  ]
}
"#;

#[test]
fn tagrisso_hits_select_nothing_by_name_so_get_uses_the_openfda_fallback() {
    let capture = hits(serde_json::from_str(TAGRISSO_CAPTURE).expect("valid capture"));
    assert!(
        select_hits_for_name(&capture, "TAGRISSO").is_empty(),
        "no fetched name field carries the brand, so nothing merges by name"
    );
}

/// `q=ferric oxide` with the ferric oxide records absent: what the text
/// search returns when only other drugs' records mention the term in wider
/// names. `get` must refuse and name these matches (ticket 2031).
pub(crate) const FERRIC_OXIDE_TEXT_ONLY_CAPTURE: &str = r#"
{
  "total": 56,
  "hits": [
    {
      "_id": "XLOMVQKBTHCTTD-UHFFFAOYSA-N",
      "_score": 49.535606,
      "drugbank": {"id": "DB09321", "name": "Zinc oxide"},
      "chebi": {"name": "zinc oxide"},
      "ndc": [
        {"nonproprietaryname": "Zinc Oxide, Titanium Dioxide"},
        {"nonproprietaryname": "ZINC OXIDE"},
        {"nonproprietaryname": "Zinc Oxide, Ferric Oxide Red, and Pramoxine Hydrochloride"}
      ]
    },
    {
      "_id": "DQKXQSGTHWVTAD-UHFFFAOYSA-N",
      "_score": 48.144127,
      "drugbank": {"id": "DB09345", "name": "Pramocaine"},
      "ndc": [
        {"nonproprietaryname": "Calamine and Pramoxine Hydrochloride"},
        {"nonproprietaryname": "PRAMOXINE HYDROCHLORIDE"}
      ]
    }
  ]
}
"#;

/// The imatinib record carries NDC names for the mesylate product and exact
/// DrugBank/ChEMBL names for the base molecule. The canonical field order
/// keeps the established product name the brand bridge pins.
#[test]
fn merge_mychem_hits_keeps_the_imatinib_mesylate_established_name() {
    let imatinib: MyChemHit = serde_json::from_value(serde_json::json!({
        "_id": "KTUFNOKKBVMGRW-UHFFFAOYSA-N",
        "_score": 26.0,
        "drugbank": {"id": "DB00619", "name": "Imatinib"},
        "chembl": {"molecule_chembl_id": "CHEMBL941", "pref_name": "IMATINIB"},
        "ndc": [
            {"nonproprietaryname": "Imatinib Mesylate"},
            {"nonproprietaryname": "imatinib mesylate"}
        ],
        "unii": {"unii": "BKJ8M8G5HI", "display_name": "IMATINIB"}
    }))
    .expect("valid imatinib hit");

    let drug = merge_mychem_hits(&[&imatinib], "imatinib");
    assert_eq!(drug.name, "imatinib mesylate");
}
