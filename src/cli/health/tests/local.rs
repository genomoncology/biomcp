//! Local data and cache readiness tests for `biomcp health`.

use std::io;
use std::path::PathBuf;

use super::super::HealthStatus;
use super::super::catalog::{
    CVX_LOCAL_DATA_AFFECTS, EMA_LOCAL_DATA_AFFECTS, GTR_LOCAL_DATA_AFFECTS,
    WHO_IVD_LOCAL_DATA_AFFECTS, WHO_LOCAL_DATA_AFFECTS,
};
use super::super::local::{
    cvx_local_data_outcome, ddinter_local_data_outcome, ema_local_data_outcome,
    fhir_config_outcome, gtr_local_data_outcome, who_ivd_local_data_outcome,
    who_local_data_outcome,
};
use super::super::runner::{ProbeClass, report_from_outcomes};
use super::{
    assert_millisecond_latency, block_on, fixture_ema_root, set_fresh_ema_mtimes,
    set_stale_ema_mtimes, set_stale_mtime, set_stale_mtime_with_age, write_cvx_files,
    write_ema_files, write_gtr_files, write_who_files, write_who_ivd_files,
};
use crate::error::BioMcpError;
use crate::test_support::TempDirGuard;

#[test]
fn ddinter_local_data_reports_available_when_default_root_is_complete() {
    let root = TempDirGuard::new("health");
    for file_name in crate::sources::ddinter::DDINTER_REQUIRED_FILES {
        std::fs::write(root.path().join(file_name), b"fixture").expect("write DDInter file");
    }

    let outcome = ddinter_local_data_outcome(root.path(), false, false);

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(outcome.row.status, HealthStatus::Available);
    assert_eq!(outcome.row.affects, None);
    assert!(outcome.row.api.starts_with("DDInter local data ("));
}

#[test]
fn ema_local_data_not_configured_when_default_root_is_empty() {
    let root = TempDirGuard::new("health");

    let outcome = ema_local_data_outcome(root.path(), false, false);

    assert_eq!(outcome.class, ProbeClass::Excluded);
    assert_eq!(
        outcome.row.api,
        format!("EMA local data ({})", root.path().display())
    );
    assert_eq!(outcome.row.status, HealthStatus::NotConfigured);
    assert_eq!(outcome.row.latency, "n/a");
    assert_eq!(outcome.row.affects.as_deref(), Some(EMA_LOCAL_DATA_AFFECTS));
}

#[test]
fn ema_local_data_errors_when_default_root_is_partial() {
    let root = TempDirGuard::new("health");
    write_ema_files(root.path(), &[crate::sources::ema::EMA_REQUIRED_FILES[0]]);

    let outcome = ema_local_data_outcome(root.path(), false, false);

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(outcome.row.status, HealthStatus::Error);
    assert_eq!(outcome.row.affects.as_deref(), Some(EMA_LOCAL_DATA_AFFECTS));
}

#[test]
fn ema_local_data_errors_when_env_root_is_missing_files() {
    let root = TempDirGuard::new("health");

    let outcome = ema_local_data_outcome(root.path(), true, false);

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(outcome.row.status, HealthStatus::Error);
    assert_eq!(outcome.row.affects.as_deref(), Some(EMA_LOCAL_DATA_AFFECTS));
}

#[test]
fn ema_local_data_reports_available_when_default_root_is_complete() {
    let fixture_root = fixture_ema_root();
    set_stale_ema_mtimes(fixture_root.path());
    set_fresh_ema_mtimes(fixture_root.path());

    let outcome = ema_local_data_outcome(fixture_root.path(), false, false);

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(
        outcome.row.api,
        format!("EMA local data ({})", fixture_root.path().display())
    );
    assert_eq!(outcome.row.status, HealthStatus::Available);
    assert_eq!(outcome.row.latency, "n/a");
    assert_eq!(outcome.row.affects, None);
}

#[test]
fn ema_local_data_reports_configured_when_env_root_is_complete() {
    let fixture_root = fixture_ema_root();
    set_stale_ema_mtimes(fixture_root.path());
    set_fresh_ema_mtimes(fixture_root.path());

    let outcome = ema_local_data_outcome(fixture_root.path(), true, false);

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(outcome.row.status, HealthStatus::Configured);
    assert_eq!(outcome.row.affects, None);
}

#[test]
fn ema_local_data_json_reports_healthy_row_without_affects() {
    let fixture_root = fixture_ema_root();
    set_stale_ema_mtimes(fixture_root.path());
    set_fresh_ema_mtimes(fixture_root.path());
    let report = report_from_outcomes(vec![ema_local_data_outcome(
        fixture_root.path(),
        false,
        false,
    )]);

    let value = serde_json::to_value(&report).expect("serialize health report");
    let rows = value["rows"].as_array().expect("rows array");
    let row = rows.first().expect("EMA row");

    assert_eq!(
        row["api"],
        format!("EMA local data ({})", fixture_root.path().display())
    );
    assert_eq!(row["status"], "available");
    assert_eq!(row["local_path"], fixture_root.path().display().to_string());
    assert_eq!(row["latency"], "n/a");
    assert!(row.get("affects").is_none());
    assert!(row.get("key_configured").is_none());
}

#[test]
fn ema_local_data_json_reports_error_row_with_affects() {
    let root = TempDirGuard::new("health");
    write_ema_files(root.path(), &[crate::sources::ema::EMA_REQUIRED_FILES[0]]);
    let report = report_from_outcomes(vec![ema_local_data_outcome(root.path(), false, false)]);

    let value = serde_json::to_value(&report).expect("serialize health report");
    let rows = value["rows"].as_array().expect("rows array");
    let row = rows.first().expect("EMA row");

    assert_eq!(row["status"], "error");
    assert_eq!(
        row["missing_files"],
        serde_json::json!(crate::sources::ema::EMA_REQUIRED_FILES[1..])
    );
    assert_eq!(row["affects"], EMA_LOCAL_DATA_AFFECTS);
    assert!(row.get("key_configured").is_none());
}

#[test]
fn mcp_local_rows_name_datasets_without_paths() {
    let fixture_root = fixture_ema_root();
    set_stale_ema_mtimes(fixture_root.path());

    let outcome = ema_local_data_outcome(fixture_root.path(), false, true);

    assert_eq!(outcome.class, ProbeClass::Warning);
    assert_eq!(outcome.row.api, "EMA local data");
    assert_eq!(outcome.row.local_path, None);
    assert_eq!(outcome.row.stale, Some(true));
}

#[test]
fn mcp_local_rows_serialize_and_render_without_local_paths() {
    let fixture_root = fixture_ema_root();
    set_stale_ema_mtimes(fixture_root.path());
    let outcome = ema_local_data_outcome(fixture_root.path(), false, true);
    let report = report_from_outcomes(vec![outcome]);

    let value = serde_json::to_value(&report).expect("serialize health report");
    let rows = value["rows"].as_array().expect("rows array");
    let row = rows.first().expect("EMA row");
    assert_eq!(row["api"], "EMA local data");
    assert!(row.get("local_path").is_none(), "row={row}");
    assert_eq!(row["stale"], true);

    let markdown = report.to_markdown();
    assert!(
        markdown.contains("| EMA local data | available (default path, stale) |"),
        "{markdown}"
    );
    let root = fixture_root.path().display().to_string();
    assert!(!markdown.contains(root.as_str()));
}

#[test]
fn cvx_local_data_not_configured_when_default_root_is_empty() {
    let root = TempDirGuard::new("health");

    let outcome = cvx_local_data_outcome(root.path(), false, false);

    assert_eq!(outcome.class, ProbeClass::Excluded);
    assert_eq!(
        outcome.row.api,
        format!("CDC CVX/MVX local data ({})", root.path().display())
    );
    assert_eq!(outcome.row.status, HealthStatus::NotConfigured);
    assert_eq!(outcome.row.latency, "n/a");
    assert_eq!(outcome.row.affects.as_deref(), Some(CVX_LOCAL_DATA_AFFECTS));
}

#[test]
fn cvx_local_data_errors_when_default_root_is_partial() {
    let root = TempDirGuard::new("health");
    write_cvx_files(root.path(), &[crate::sources::cvx::CVX_FILE]);

    let outcome = cvx_local_data_outcome(root.path(), false, false);

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(outcome.row.status, HealthStatus::Error);
    assert_eq!(outcome.row.affects.as_deref(), Some(CVX_LOCAL_DATA_AFFECTS));
}

#[test]
fn cvx_local_data_reports_available_when_default_root_is_complete() {
    let root = TempDirGuard::new("health");
    write_cvx_files(root.path(), crate::sources::cvx::CVX_REQUIRED_FILES);

    let outcome = cvx_local_data_outcome(root.path(), false, false);

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(
        outcome.row.api,
        format!("CDC CVX/MVX local data ({})", root.path().display())
    );
    assert_eq!(outcome.row.status, HealthStatus::Available);
    assert_eq!(outcome.row.affects, None);
}

#[test]
fn cvx_local_data_reports_configured_stale_when_env_root_is_complete_but_old() {
    let root = TempDirGuard::new("health");
    write_cvx_files(root.path(), crate::sources::cvx::CVX_REQUIRED_FILES);
    set_stale_mtime_with_age(
        &root.path().join(crate::sources::cvx::MVX_FILE),
        crate::sources::cvx::CVX_STALE_AFTER + std::time::Duration::from_secs(60),
    );

    let outcome = cvx_local_data_outcome(root.path(), true, false);

    assert_eq!(outcome.class, ProbeClass::Warning);
    assert_eq!(outcome.row.status, HealthStatus::Configured);
    assert_eq!(outcome.row.affects.as_deref(), Some(CVX_LOCAL_DATA_AFFECTS));
}

#[test]
fn who_local_data_not_configured_when_default_root_is_empty() {
    let root = TempDirGuard::new("health");

    let outcome = who_local_data_outcome(root.path(), false, false);

    assert_eq!(outcome.class, ProbeClass::Excluded);
    assert_eq!(
        outcome.row.api,
        format!(
            "WHO Prequalification local data ({})",
            root.path().display()
        )
    );
    assert_eq!(outcome.row.status, HealthStatus::NotConfigured);
    assert_eq!(outcome.row.affects.as_deref(), Some(WHO_LOCAL_DATA_AFFECTS));
}

#[test]
fn who_local_data_errors_when_env_root_is_missing_file() {
    let root = TempDirGuard::new("health");

    let outcome = who_local_data_outcome(root.path(), true, false);

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(outcome.row.status, HealthStatus::Error);
    assert_eq!(outcome.row.affects.as_deref(), Some(WHO_LOCAL_DATA_AFFECTS));
}

#[test]
fn who_local_data_reports_available_when_default_root_is_complete() {
    let root = TempDirGuard::new("health");
    write_who_files(root.path(), crate::sources::who_pq::WHO_PQ_REQUIRED_FILES);

    let outcome = who_local_data_outcome(root.path(), false, false);

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(
        outcome.row.api,
        format!(
            "WHO Prequalification local data ({})",
            root.path().display()
        )
    );
    assert_eq!(outcome.row.status, HealthStatus::Available);
    assert_eq!(outcome.row.affects, None);
}

#[test]
fn who_local_data_reports_configured_when_env_root_is_complete() {
    let root = TempDirGuard::new("health");
    write_who_files(root.path(), crate::sources::who_pq::WHO_PQ_REQUIRED_FILES);

    let outcome = who_local_data_outcome(root.path(), true, false);

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(outcome.row.status, HealthStatus::Configured);
    assert_eq!(outcome.row.affects, None);
}

#[test]
fn who_local_data_reports_configured_stale_when_env_root_is_complete_but_old() {
    let root = TempDirGuard::new("health");
    write_who_files(root.path(), crate::sources::who_pq::WHO_PQ_REQUIRED_FILES);
    set_stale_mtime(
        &root
            .path()
            .join(crate::sources::who_pq::WHO_PQ_API_CSV_FILE),
    );

    let outcome = who_local_data_outcome(root.path(), true, false);

    assert_eq!(outcome.class, ProbeClass::Warning);
    assert_eq!(outcome.row.status, HealthStatus::Configured);
    assert_eq!(outcome.row.affects.as_deref(), Some(WHO_LOCAL_DATA_AFFECTS));
}

#[test]
fn who_local_data_reports_default_path_stale_when_complete_but_old() {
    let root = TempDirGuard::new("health");
    write_who_files(root.path(), crate::sources::who_pq::WHO_PQ_REQUIRED_FILES);
    set_stale_mtime(
        &root
            .path()
            .join(crate::sources::who_pq::WHO_PQ_API_CSV_FILE),
    );

    let outcome = who_local_data_outcome(root.path(), false, false);

    assert_eq!(outcome.class, ProbeClass::Warning);
    assert_eq!(outcome.row.status, HealthStatus::Available);
    assert_eq!(outcome.row.affects.as_deref(), Some(WHO_LOCAL_DATA_AFFECTS));
}

#[test]
fn who_local_data_errors_when_only_api_file_is_missing() {
    let root = TempDirGuard::new("health");
    write_who_files(
        root.path(),
        &[
            crate::sources::who_pq::WHO_PQ_CSV_FILE,
            crate::sources::who_pq::WHO_VACCINES_CSV_FILE,
        ],
    );

    let outcome = who_local_data_outcome(root.path(), true, false);

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(outcome.row.status, HealthStatus::Error);
    assert_eq!(outcome.row.affects.as_deref(), Some(WHO_LOCAL_DATA_AFFECTS));
}

#[test]
fn who_local_data_errors_when_only_vaccine_file_is_missing() {
    let root = TempDirGuard::new("health");
    write_who_files(
        root.path(),
        &[
            crate::sources::who_pq::WHO_PQ_CSV_FILE,
            crate::sources::who_pq::WHO_PQ_API_CSV_FILE,
        ],
    );

    let outcome = who_local_data_outcome(root.path(), true, false);

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(outcome.row.status, HealthStatus::Error);
    assert_eq!(outcome.row.affects.as_deref(), Some(WHO_LOCAL_DATA_AFFECTS));
}

#[test]
fn who_ivd_local_data_not_configured_when_default_root_is_empty() {
    let root = TempDirGuard::new("health");

    let outcome = who_ivd_local_data_outcome(root.path(), false, false);

    assert_eq!(outcome.class, ProbeClass::Excluded);
    assert_eq!(
        outcome.row.api,
        format!("WHO IVD local data ({})", root.path().display())
    );
    assert_eq!(outcome.row.status, HealthStatus::NotConfigured);
    assert_eq!(
        outcome.row.affects.as_deref(),
        Some(WHO_IVD_LOCAL_DATA_AFFECTS)
    );
}

#[test]
fn who_ivd_local_data_errors_when_env_root_is_missing_file() {
    let root = TempDirGuard::new("health");

    let outcome = who_ivd_local_data_outcome(root.path(), true, false);

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(outcome.row.status, HealthStatus::Error);
    assert_eq!(
        outcome.row.affects.as_deref(),
        Some(WHO_IVD_LOCAL_DATA_AFFECTS)
    );
}

#[test]
fn who_ivd_local_data_reports_available_when_default_root_is_complete() {
    let root = TempDirGuard::new("health");
    write_who_ivd_files(root.path(), crate::sources::who_ivd::WHO_IVD_REQUIRED_FILES);

    let outcome = who_ivd_local_data_outcome(root.path(), false, false);

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(
        outcome.row.api,
        format!("WHO IVD local data ({})", root.path().display())
    );
    assert_eq!(outcome.row.status, HealthStatus::Available);
    assert_eq!(outcome.row.affects, None);
}

#[test]
fn who_ivd_local_data_reports_configured_when_env_root_is_complete() {
    let root = TempDirGuard::new("health");
    write_who_ivd_files(root.path(), crate::sources::who_ivd::WHO_IVD_REQUIRED_FILES);

    let outcome = who_ivd_local_data_outcome(root.path(), true, false);

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(outcome.row.status, HealthStatus::Configured);
    assert_eq!(outcome.row.affects, None);
}

#[test]
fn who_ivd_local_data_reports_configured_stale_when_env_root_is_complete_but_old() {
    let root = TempDirGuard::new("health");
    write_who_ivd_files(root.path(), crate::sources::who_ivd::WHO_IVD_REQUIRED_FILES);
    set_stale_mtime(&root.path().join(crate::sources::who_ivd::WHO_IVD_CSV_FILE));

    let outcome = who_ivd_local_data_outcome(root.path(), true, false);

    assert_eq!(outcome.class, ProbeClass::Warning);
    assert_eq!(outcome.row.status, HealthStatus::Configured);
    assert_eq!(
        outcome.row.affects.as_deref(),
        Some(WHO_IVD_LOCAL_DATA_AFFECTS)
    );
}

#[test]
fn gtr_local_data_not_configured_when_default_root_is_empty() {
    let root = TempDirGuard::new("health");

    let outcome = gtr_local_data_outcome(root.path(), false, false);

    assert_eq!(outcome.class, ProbeClass::Excluded);
    assert_eq!(
        outcome.row.api,
        format!("GTR local data ({})", root.path().display())
    );
    assert_eq!(outcome.row.status, HealthStatus::NotConfigured);
    assert_eq!(outcome.row.affects.as_deref(), Some(GTR_LOCAL_DATA_AFFECTS));
}

#[test]
fn gtr_local_data_errors_when_default_root_is_partial() {
    let root = TempDirGuard::new("health");
    write_gtr_files(root.path(), &[crate::sources::gtr::GTR_TEST_VERSION_FILE]);

    let outcome = gtr_local_data_outcome(root.path(), false, false);

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(outcome.row.status, HealthStatus::Error);
    assert_eq!(outcome.row.affects.as_deref(), Some(GTR_LOCAL_DATA_AFFECTS));
}

#[test]
fn gtr_local_data_reports_available_when_default_root_is_complete() {
    let root = TempDirGuard::new("health");
    write_gtr_files(root.path(), &crate::sources::gtr::GTR_REQUIRED_FILES);

    let outcome = gtr_local_data_outcome(root.path(), false, false);

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(outcome.row.status, HealthStatus::Available);
    assert_eq!(outcome.row.affects, None);
}

#[test]
fn gtr_local_data_reports_configured_stale_when_env_root_is_complete_but_old() {
    let root = TempDirGuard::new("health");
    write_gtr_files(root.path(), &crate::sources::gtr::GTR_REQUIRED_FILES);
    set_stale_mtime_with_age(
        &root.path().join(crate::sources::gtr::GTR_TEST_VERSION_FILE),
        crate::sources::gtr::GTR_STALE_AFTER + std::time::Duration::from_secs(60),
    );

    let outcome = gtr_local_data_outcome(root.path(), true, false);

    assert_eq!(outcome.class, ProbeClass::Warning);
    assert_eq!(outcome.row.status, HealthStatus::Configured);
    assert_eq!(outcome.row.affects.as_deref(), Some(GTR_LOCAL_DATA_AFFECTS));
}

#[test]
fn fhir_row_reports_configured_or_not_configured_without_a_url() {
    let unset = fhir_config_outcome(false);
    assert_eq!(unset.row.status, HealthStatus::NotConfigured);
    assert_eq!(unset.class, ProbeClass::Excluded);
    assert_eq!(
        unset.row.required_env_var.as_deref(),
        Some("BIOMCP_FHIR_BASE")
    );
    let set = fhir_config_outcome(true);
    assert_eq!(set.row.status, HealthStatus::Configured);
    assert_eq!(set.class, ProbeClass::Healthy);
    let markdown = report_from_outcomes(vec![unset, set]).to_markdown();
    assert!(
        markdown.contains("| FHIR (patient record) | not configured |"),
        "{markdown}"
    );
    assert!(
        markdown.contains("| FHIR (patient record) | configured |"),
        "{markdown}"
    );
    assert!(!markdown.contains("://"), "{markdown}");
}
