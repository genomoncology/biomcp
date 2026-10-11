//! Recorded-capture pins for product names several unrelated drugs share
//! (ticket 2039).
//!
//! Every capture below was recorded 2026-10-10 from
//! `mychem.info/v1/query?q=<name>` with the `get` field list, trimmed the
//! same way as the ticket-2031 captures: only the hits the assertions read
//! stay, NDC rows deduplicate to the rows the assertions read, and unused
//! fields are removed.

use super::name_resolution_tests::hits;
use super::{merge_mychem_hits, select_hits_for_name};

/// `q=Pain Relief`: the store-shelf product name is carried exactly by
/// records of unrelated drugs, each pairing it with its own ingredient on
/// its own rows (acetaminophen; lidocaine and menthol; acetaminophen and
/// diphenhydramine; benzocaine; ibuprofen). The phenylephrine record's
/// hemorrhoidal row carries "Pain Relief" as its established name, and the
/// menthol record answers through its ChEMBL identifier alone. On main the
/// exact tier merges them all, so one card carried acetaminophen's DB00316
/// with other records' targets (ticket 2035 finding 20).
pub(crate) const PAIN_RELIEF_CAPTURE: &str = r#"
{
  "total": 673,
  "hits": [
    {
      "_id": "RZVAJINKPMORJF-UHFFFAOYSA-N",
      "_score": 49.638573,
      "drugbank": {"id": "DB00316", "name": "Acetaminophen"},
      "chembl": {"molecule_chembl_id": "CHEMBL112", "pref_name": "ACETAMINOPHEN"},
      "gtopdb": {
        "name": "paracetamol",
        "interaction_targets": [
          {"symbol": "TRPV4"},
          {"symbol": "PTGS1"},
          {"symbol": "PTGS2"}
        ]
      },
      "ndc": [
        {"nonproprietaryname": "acetaminophen", "proprietaryname": "pain relief"},
        {"nonproprietaryname": "Pain Relief", "proprietaryname": "Pain Relief"},
        {"nonproprietaryname": "Acetaminophen", "proprietaryname": "PAIN RELIEF"}
      ],
      "unii": {"unii": "362O9ITL9D", "display_name": "ACETAMINOPHEN"}
    },
    {
      "_id": "NOOLISFMXDJSKH-UHFFFAOYSA-N",
      "_score": 49.59012,
      "chembl": {"molecule_chembl_id": "CHEMBL256087", "pref_name": "MENTHOL"},
      "ndc": [
        {"nonproprietaryname": "Lidocaine, Menthol", "proprietaryname": "Pain Relief"},
        {"nonproprietaryname": "Analgesic Menthol", "proprietaryname": "Pain Relief"}
      ]
    },
    {
      "_id": "NNJVILVZKWQKPM-UHFFFAOYSA-N",
      "_score": 49.38565,
      "drugbank": {"id": "DB00281", "name": "Lidocaine"},
      "chembl": {"molecule_chembl_id": "CHEMBL79", "pref_name": "LIDOCAINE"},
      "gtopdb": {
        "name": "lidocaine",
        "interaction_targets": [{"symbol": "SCN5A"}, {"symbol": "Scn9a"}]
      },
      "ndc": [
        {"nonproprietaryname": "Lidocaine, Menthol", "proprietaryname": "Pain Relief"}
      ],
      "unii": {"unii": "98PI200987", "display_name": "LIDOCAINE"}
    },
    {
      "_id": "ZZVUWRFHKOJYTH-UHFFFAOYSA-N",
      "_score": 49.1605,
      "drugbank": {"id": "DB01075", "name": "Diphenhydramine"},
      "chembl": {"molecule_chembl_id": "CHEMBL657", "pref_name": "DIPHENHYDRAMINE"},
      "gtopdb": {
        "name": "diphenhydramine",
        "interaction_targets": [{"symbol": "HRH1"}]
      },
      "ndc": [
        {"nonproprietaryname": "Diphenhydramine HCl", "proprietaryname": "Sleep Aid"},
        {"nonproprietaryname": "Acetaminophen, Diphenhydramine HCl", "proprietaryname": "Pain Relief"}
      ],
      "unii": {"unii": "8GTS82S83M", "display_name": "DIPHENHYDRAMINE"}
    },
    {
      "_id": "SONNWYBIRXJNDC-VIFPVBQESA-N",
      "_score": 49.132862,
      "drugbank": {"id": "DB00388", "name": "Phenylephrine"},
      "chembl": {"molecule_chembl_id": "CHEMBL1215", "pref_name": "PHENYLEPHRINE"},
      "gtopdb": {
        "name": "phenylephrine",
        "interaction_targets": [{"symbol": "ADRA1A"}, {"symbol": "ADRA1B"}]
      },
      "ndc": [
        {"nonproprietaryname": "Pain Relief", "proprietaryname": "Hemorrhoidal"}
      ],
      "unii": {"unii": "1WS297W6MV", "display_name": "PHENYLEPHRINE"}
    },
    {
      "_id": "BLFLLBZGZJTVJG-UHFFFAOYSA-N",
      "_score": 47.824516,
      "drugbank": {"id": "DB01086", "name": "Benzocaine"},
      "chembl": {"molecule_chembl_id": "CHEMBL278172", "pref_name": "BENZOCAINE"},
      "ndc": [
        {"nonproprietaryname": "benzocaine", "proprietaryname": "Pain Relief"}
      ],
      "unii": {"unii": "U3RSY48JW5", "display_name": "BENZOCAINE"}
    },
    {
      "_id": "HEFNNWSXXWATRW-UHFFFAOYSA-N",
      "_score": 47.50985,
      "drugbank": {"id": "DB01050", "name": "Ibuprofen"},
      "chembl": {"molecule_chembl_id": "CHEMBL521", "pref_name": "IBUPROFEN"},
      "gtopdb": {
        "name": "ibuprofen",
        "interaction_targets": [{"symbol": "ASIC1"}, {"symbol": "PTGS2"}]
      },
      "ndc": [
        {"nonproprietaryname": "Ibuprofen", "proprietaryname": "pain relief"}
      ],
      "unii": {"unii": "WK2XYI10QM", "display_name": "IBUPROFEN"}
    }
  ]
}
"#;

#[test]
fn pain_relief_refuses_instead_of_fusing_unrelated_drugs() {
    // Ticket 2039: "Pain Relief" brands acetaminophen, lidocaine, menthol,
    // diphenhydramine, phenylephrine, benzocaine and ibuprofen products
    // alike, and each record's own rows pair the name with a different
    // ingredient. No one drug owns the name, so the selection refuses and
    // the caller answers with the honest no-match instead of one card
    // carrying acetaminophen's identifiers beside other drugs' targets.
    let capture = hits(serde_json::from_str(PAIN_RELIEF_CAPTURE).expect("valid capture"));
    let selected = select_hits_for_name(&capture, "Pain Relief");
    assert!(
        selected.is_empty(),
        "a product name several unrelated drugs share names no one drug"
    );
}

/// `q=Sleep Aid`: diphenhydramine's and doxylamine's records each pair the
/// shelf name with their own ingredient, and three bare NDC product rows
/// carry it too (rows trimmed to one).
pub(crate) const SLEEP_AID_CAPTURE: &str = r#"
{
  "total": 87,
  "hits": [
    {
      "_id": "ZZVUWRFHKOJYTH-UHFFFAOYSA-N",
      "_score": 57.620647,
      "drugbank": {"id": "DB01075", "name": "Diphenhydramine"},
      "chembl": {"molecule_chembl_id": "CHEMBL657", "pref_name": "DIPHENHYDRAMINE"},
      "gtopdb": {
        "name": "diphenhydramine",
        "interaction_targets": [{"symbol": "HRH1"}]
      },
      "ndc": [
        {"nonproprietaryname": "Diphenhydramine HCl", "proprietaryname": "Sleep Aid"}
      ],
      "unii": {"unii": "8GTS82S83M", "display_name": "DIPHENHYDRAMINE"}
    },
    {
      "_id": "HCFDWZZGGLSKEP-UHFFFAOYSA-N",
      "_score": 56.14622,
      "drugbank": {"id": "DB00366", "name": "Doxylamine"},
      "chembl": {"molecule_chembl_id": "CHEMBL1004", "pref_name": "DOXYLAMINE"},
      "ndc": [
        {"nonproprietaryname": "Doxylamine succinate", "proprietaryname": "Sleep Aid"}
      ],
      "unii": {"unii": "XZV7JAA763", "display_name": "DOXYLAMINE"}
    },
    {
      "_id": "21130-052",
      "_score": 37.281734,
      "ndc": [
        {"nonproprietaryname": "Sleep Aid", "proprietaryname": "Sleep Aid"}
      ]
    }
  ]
}
"#;

#[test]
fn sleep_aid_refuses_instead_of_merging_diphenhydramine_with_doxylamine() {
    // Ticket 2039: diphenhydramine's rows pair "Sleep Aid" with
    // diphenhydramine and doxylamine's rows pair it with doxylamine
    // succinate — two unrelated drugs, one shelf name, no single owner.
    let capture = hits(serde_json::from_str(SLEEP_AID_CAPTURE).expect("valid capture"));
    let selected = select_hits_for_name(&capture, "Sleep Aid");
    assert!(
        selected.is_empty(),
        "the shelf name names neither diphenhydramine nor doxylamine alone"
    );
}

/// `q=terfenadine` with terfenadine's own record absent from the reply:
/// what the leading-name tier sees when the text query ranks only
/// fexofenadine, whose DrugBank synonyms list "Terfenadine carboxylate" and
/// "Terfenadine acid metabolite". A foreign drug's synonym is not this
/// drug's salt form (ticket 2039).
pub(crate) const FEXOFENADINE_TEXT_ONLY_CAPTURE: &str = r#"
{
  "total": 4,
  "hits": [
    {
      "_id": "RWTNPBWLLIMQHL-UHFFFAOYSA-N",
      "_score": 26.00888,
      "drugbank": {
        "id": "DB00950",
        "name": "Fexofenadine",
        "synonyms": [
          "Carboxyterfenadine",
          "Fexofenadine",
          "Terfenadine acid metabolite",
          "Terfenadine carboxylate",
          "Terfenadine-COOH"
        ]
      },
      "chembl": {"molecule_chembl_id": "CHEMBL914", "pref_name": "FEXOFENADINE"},
      "ndc": [
        {"nonproprietaryname": "Fexofenadine Hydrochloride"},
        {"nonproprietaryname": "Fexofenadine HCl"}
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
fn a_foreign_synonym_never_satisfies_the_leading_name_tier() {
    // Ticket 2039: with terfenadine's own record absent, the only name that
    // leads with the query is fexofenadine's synonym "Terfenadine
    // carboxylate" — another drug's synonym, not a salt form of the
    // requested drug — so nothing resolves and the caller refuses.
    let capture =
        hits(serde_json::from_str(FEXOFENADINE_TEXT_ONLY_CAPTURE).expect("valid capture"));
    let selected = select_hits_for_name(&capture, "terfenadine");
    assert!(
        selected.is_empty(),
        "fexofenadine's \"Terfenadine carboxylate\" synonym names fexofenadine, never terfenadine"
    );
}

/// A brand two identity-bearing records share because both carry the same
/// product's rows: the pairings agree, one product resolves, and a bare
/// product row rides along. The refusal is for disagreeing owners, never
/// for every shared brand — Lonsurf's trifluridine and tipiracil records
/// look exactly like this fixture (tickets 2037 and 2039).
pub(crate) const AGREEING_PAIRINGS_CAPTURE: &str = r#"
{
  "total": 3,
  "hits": [
    {
      "_id": "FIXTUREONE-KEY",
      "_score": 26.5,
      "drugbank": {"id": "DB90001", "name": "Fixturedrug One"},
      "chembl": {"molecule_chembl_id": "CHEMBL9000001", "pref_name": "FIXTUREDRUG ONE"},
      "ndc": [
        {"nonproprietaryname": "FIXTUREDRUG ONE AND FIXTUREDRUG TWO", "proprietaryname": "FIXTUREGO"}
      ],
      "unii": {"unii": "FIXTURE1", "display_name": "FIXTUREDRUG ONE"}
    },
    {
      "_id": "FIXTURETWO-KEY",
      "_score": 23.3,
      "drugbank": {"id": "DB90002", "name": "Fixturedrug Two"},
      "ndc": [
        {"nonproprietaryname": "FIXTUREDRUG ONE AND FIXTUREDRUG TWO", "proprietaryname": "FIXTUREGO"}
      ]
    },
    {
      "_id": "55555-070",
      "_score": 17.1,
      "ndc": [
        {"nonproprietaryname": "fixturedrug one and fixturedrug two", "proprietaryname": "Fixturego"}
      ]
    }
  ]
}
"#;

#[test]
fn agreeing_pairings_still_merge_one_product() {
    // Ticket 2039: both identity-bearing records pair the brand with the
    // same combined ingredient, so the shared name owns exactly one
    // product and the merge keeps its rows (including the bare product row
    // that carries no identity of its own).
    let capture = hits(serde_json::from_str(AGREEING_PAIRINGS_CAPTURE).expect("valid capture"));
    let selected = select_hits_for_name(&capture, "Fixturego");
    assert_eq!(selected.len(), 3);
    let drug = merge_mychem_hits(&selected, "Fixturego");
    assert_eq!(drug.name, "fixturedrug one and fixturedrug two");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB90001"));
}
