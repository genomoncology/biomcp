use std::path::Path;

mod construction;
mod parsing;
mod sync;

fn fixture_csv() -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("spec")
            .join("fixtures")
            .join("who-pq")
            .join(super::WHO_PQ_CSV_FILE),
    )
    .expect("WHO fixture should be readable")
}

fn fixture_api_csv() -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("spec")
            .join("fixtures")
            .join("who-pq")
            .join(super::WHO_PQ_API_CSV_FILE),
    )
    .expect("WHO API fixture should be readable")
}

fn fixture_vaccine_csv() -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("spec")
            .join("fixtures")
            .join("who-pq")
            .join(super::WHO_VACCINES_CSV_FILE),
    )
    .expect("WHO vaccine fixture should be readable")
}
