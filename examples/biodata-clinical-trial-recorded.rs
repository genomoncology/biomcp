//! Reproduce the admitted ClinicalTrials.gov projection without provider access.

use biodata::{ClinicalTrialsGovApiV2, ClinicalTrialsGovApiV2Limits, Document};
use std::error::Error;
use std::io::{self, Write};

const RECORDED_INPUT: &[u8] =
    include_bytes!("../website/public/downloads/biodata/nct02576665-provider-types.json");
const EXPECTED: &[u8] =
    include_bytes!("../website/public/downloads/biodata/ctgov-clinical-trial-projection.json");

fn main() -> Result<(), Box<dyn Error>> {
    let projection =
        ClinicalTrialsGovApiV2::parse(RECORDED_INPUT, &ClinicalTrialsGovApiV2Limits::default())?
            .project_clinical_trial()?
            .into_digest_assertion();
    let encoded = Document::ClinicalTrialProjection(projection).to_json()?;
    Document::from_json(&encoded)?;
    if encoded.as_bytes() != EXPECTED {
        return Err("recorded projection differs from the adopted catalog artifact".into());
    }

    match std::env::args().nth(1).as_deref() {
        Some("--check") => Ok(()),
        None => {
            io::stdout().write_all(encoded.as_bytes())?;
            Ok(())
        }
        Some(_) => Err("usage: biodata-clinical-trial-recorded [--check]".into()),
    }
}
