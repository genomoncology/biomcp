//! Bounded trial phase vocabulary advertised by the typed MCP search tool.

use serde_json::{Value, json};

pub(super) fn schema() -> Value {
    json!({"enum":[
        "NA","N/A","n/a",
        "EARLY_PHASE1","early_phase1","early1",
        "PHASE1","1","I",
        "PHASE2","2","II",
        "PHASE3","3","III",
        "PHASE4","4","IV",
        "PHASE1/PHASE2","1/2","I_II",
        "PHASE2/PHASE3","2/3","II_III"
    ]})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_advertises_each_documented_phase_spelling() {
        assert_eq!(
            schema()["enum"],
            json!([
                "NA",
                "N/A",
                "n/a",
                "EARLY_PHASE1",
                "early_phase1",
                "early1",
                "PHASE1",
                "1",
                "I",
                "PHASE2",
                "2",
                "II",
                "PHASE3",
                "3",
                "III",
                "PHASE4",
                "4",
                "IV",
                "PHASE1/PHASE2",
                "1/2",
                "I_II",
                "PHASE2/PHASE3",
                "2/3",
                "II_III"
            ])
        );
    }
}
