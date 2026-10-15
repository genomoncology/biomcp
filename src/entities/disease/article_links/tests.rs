//! The article disease-row link rule's boundaries: the recorded MESH
//! crosswalk replies drive the narrower-title refusal and its negative
//! control directly, one assertion per recorded fact.

use super::*;
use crate::sources::mydisease::MyDiseaseQueryResponse;

macro_rules! recorded {
    ($name:expr) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/mydisease/",
            $name
        ))
    };
}

fn recorded_hits(fixture: &str) -> Vec<crate::sources::mydisease::MyDiseaseHit> {
    let response: MyDiseaseQueryResponse =
        serde_json::from_str(recorded!(fixture)).expect("the recorded crosswalk reply parses");
    response.hits
}

fn hit_by_id(
    hits: &[crate::sources::mydisease::MyDiseaseHit],
    id: &str,
) -> crate::sources::mydisease::MyDiseaseHit {
    hits.iter()
        .find(|hit| hit.id == id)
        .cloned()
        .unwrap_or_else(|| panic!("missing {id} in the recorded reply"))
}

/// The install test's row: "skin neoplasms" (MESH:D012878) crosswalks to
/// the benign-neoplasm record and the skin-cancer record, and only the
/// benign record holds the concept name — through its MONDO label
/// "skin neoplasm" and its synonyms. Its card title "skin benign
/// neoplasm" narrows the concept (every concept word plus "benign"), so
/// the hit holds nothing and the row keeps its search command.
#[test]
fn skin_neoplasms_refuses_the_benign_card_because_its_title_narrows() {
    let hits = recorded_hits("query_xref_mesh_d012878.json");
    let benign = hit_by_id(&hits, "MONDO:0002531");
    let cancer = hit_by_id(&hits, "MONDO:0002898");

    assert!(
        card_title_narrows_the_concept("skin neoplasms", "skin benign neoplasm"),
        "the benign title carries every concept word plus 'benign'"
    );
    assert!(
        !crosswalk_hit_holds_name("skin neoplasms", &benign),
        "the narrower card never holds the concept"
    );
    assert!(
        !crosswalk_hit_holds_name("skin neoplasms", &cancer),
        "the skin-cancer record holds no matching name either"
    );
    assert!(
        exact_named_crosswalk_hit("skin neoplasms", &hits).is_none(),
        "no holder means no verified link"
    );
}

/// The rule's negative control, on the same recorded reply: a title that
/// shares words with the concept without carrying ALL of them is not
/// narrower. "zone of skin neoplasm" is an exact synonym of the benign
/// record, and the title "skin benign neoplasm" lacks "zone" and "of",
/// so the record still holds the concept and the link stands.
#[test]
fn a_title_that_shares_words_without_all_concept_words_still_links() {
    let hits = recorded_hits("query_xref_mesh_d012878.json");
    let benign = hit_by_id(&hits, "MONDO:0002531");

    assert!(
        !card_title_narrows_the_concept("zone of skin neoplasm", "skin benign neoplasm"),
        "the title lacks two concept words, so it is not narrower"
    );
    assert!(
        crosswalk_hit_holds_name("zone of skin neoplasm", &benign),
        "the synonym holder keeps the concept"
    );
    assert_eq!(
        exact_named_crosswalk_hit("zone of skin neoplasm", &hits)
            .expect("exactly one holder")
            .id,
        "MONDO:0002531"
    );
}

/// The narrower rule keeps an exact title: a concept equal to the card
/// title has no extra word, so the 2047 links hold their shape.
#[test]
fn an_exact_card_title_is_not_narrower() {
    assert!(!card_title_narrows_the_concept(
        "lung neoplasms",
        "lung neoplasm"
    ));
    assert!(!card_title_narrows_the_concept("Neoplasms", "neoplasm"));
    assert!(!card_title_narrows_the_concept("Adenoma", "adenoma"));
    // A title that names a broader concept ("disease of cellular
    // proliferation" for "Neoplasms") shares no word subset trick: it
    // simply is not narrower, and the 2047 Neoplasms link keeps holding
    // through its MONDO label.
    assert!(!card_title_narrows_the_concept(
        "Neoplasms",
        "disease of cellular proliferation"
    ));
}
