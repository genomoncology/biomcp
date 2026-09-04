use super::*;
use crate::sources::clinicaltrials::ClinicalTrialsClient;
use reqwest::StatusCode;

const TOCA_511_DESCRIPTION: &str = "Toca 511 consists of a purified retroviral replicating vector encoding a modified yeast cytosine deaminase (CD) gene. The CD gene converts the antifungal 5-fluorocytosine (5FC) to the anticancer drug 5-FU in cells that have been infected by the Toca 511 vector";
const TOCA_FC_DESCRIPTION: &str = "Toca FC is an extended-release formulation of flucytosine. Toca FC is supplied as 500 mg white, oblong tablets with \"TOCA FC\" embossed on one side and \"500\" embossed on the other side";

#[test]
fn receipted_ctgov_intervention_descriptions_keep_their_associations_in_json() {
    let study = ClinicalTrialsClient::decode_get_response(
        "NCT02576665",
        StatusCode::OK,
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/ctgov/get_nct02576665_full_20260903.json"
        )),
    )
    .expect("receipted unrestricted NCT02576665 capture");

    let trial = from_ctgov_study(&study);
    assert_eq!(trial.nct_id, "NCT02576665");
    assert_eq!(trial.intervention_details.len(), 2);

    let toca_511 = &trial.intervention_details[0];
    assert_eq!(toca_511.name, "Toca 511");
    assert_eq!(toca_511.intervention_type.as_deref(), Some("BIOLOGICAL"));
    assert_eq!(toca_511.description.as_deref(), Some(TOCA_511_DESCRIPTION));
    assert_eq!(
        toca_511.other_names,
        [
            "vocimagene amiretrorepvec",
            "RRV",
            "retroviral replicating viral"
        ]
    );

    let toca_fc = &trial.intervention_details[1];
    assert_eq!(toca_fc.name, "Toca FC");
    assert_eq!(toca_fc.intervention_type.as_deref(), Some("DRUG"));
    assert_eq!(toca_fc.description.as_deref(), Some(TOCA_FC_DESCRIPTION));
    assert_eq!(
        toca_fc.other_names,
        ["Flucytosine", "5-FC", "5-Fluorocytosine"]
    );

    let json = serde_json::to_value(&trial).expect("structured trial JSON");
    assert_eq!(
        json["intervention_details"],
        serde_json::json!([
            {
                "name": "Toca 511",
                "intervention_type": "BIOLOGICAL",
                "description": TOCA_511_DESCRIPTION,
                "other_names": [
                    "vocimagene amiretrorepvec",
                    "RRV",
                    "retroviral replicating viral"
                ]
            },
            {
                "name": "Toca FC",
                "intervention_type": "DRUG",
                "description": TOCA_FC_DESCRIPTION,
                "other_names": ["Flucytosine", "5-FC", "5-Fluorocytosine"]
            }
        ])
    );
}
