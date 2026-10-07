//! The ClinVar section owner: direct NCBI ClinVar retrieval, the
//! MyVariant.info fallback copy, and the failure classification that keeps
//! why the direct source dropped instead of collapsing it to `Err(())`.

use std::time::Duration;

use crate::entities::section_outcome::SectionOutcome;
use crate::error::BioMcpError;
use crate::sources::ncbi_efetch::clinvar::{CLINVAR_RATE_LIMIT_MESSAGE, ClinvarClient};

use super::{ClinvarRecord, Variant};

const CLINVAR_ID_REQUIRED: &str =
    "Direct ClinVar retrieval requires a resolved numeric Variation ID.";

/// Why the direct NCBI ClinVar lookup dropped, kept instead of an
/// `Err(())` collapse so a degraded section names the source that failed
/// and the reason it failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ClinvarDirectFailure {
    /// The enrichment deadline fired before the direct answer arrived.
    /// The shared client's rate-limit waits (the Retry-After retry sleep
    /// and the in-process rate limiter) run inside the dropped send future,
    /// so this seam cannot tell a slow answer from a sustained rate limit;
    /// the wording admits both instead of claiming a pure timeout.
    Timeout,
    RateLimited,
    /// Any non-rate-limit provider failure: transport error, non-success
    /// HTTP status, or a response that failed decode and parse.
    ProviderError,
}

impl ClinvarDirectFailure {
    fn reason(self) -> &'static str {
        match self {
            Self::Timeout => "NCBI ClinVar timed out (slow answer or sustained rate limiting)",
            Self::RateLimited => "NCBI ClinVar rate limited the request",
            Self::ProviderError => "NCBI ClinVar request failed",
        }
    }

    fn from_error(error: &BioMcpError) -> Self {
        let mut current = error;
        while let BioMcpError::WithSourceContext { source, .. } = current {
            current = source;
        }
        match current {
            BioMcpError::Api { api, message }
                if api == "NCBI ClinVar" && message == CLINVAR_RATE_LIMIT_MESSAGE =>
            {
                Self::RateLimited
            }
            _ => Self::ProviderError,
        }
    }

    fn degraded_message(self, fallback: Option<&ClinvarRecord>) -> String {
        match fallback_evaluation_date(fallback) {
            Some(date) => format!(
                "{}; showing MyVariant.info fallback data (newest evaluation {date}).",
                self.reason()
            ),
            None => format!("{}; showing MyVariant.info fallback data.", self.reason()),
        }
    }

    fn unavailable_message(self) -> &'static str {
        match self {
            Self::Timeout => {
                "NCBI ClinVar timed out (slow answer or sustained rate limiting); \
                              no MyVariant.info fallback data is available."
            }
            Self::RateLimited => {
                "NCBI ClinVar rate limited the request; no MyVariant.info fallback data is \
                 available."
            }
            Self::ProviderError => {
                "NCBI ClinVar request failed; no MyVariant.info fallback data is available."
            }
        }
    }
}

/// Newest `last_evaluated` date in the MyVariant.info fallback copy, so a
/// degraded label can say how old the fallback data may be. Only strict
/// day-shaped dates surface; any other provider spelling is omitted.
/// ISO day shape only: four digits, a dash, two digits, a dash, two
/// digits, naming a real calendar day under the Gregorian leap rule.
/// Provider spellings like "01 Apr 2019" or reordered forms are
/// omitted rather than trusted.
fn is_day_shaped(value: &str) -> bool {
    let bytes = value.as_bytes();
    let digits_at =
        |indices: [usize; 8]| indices.iter().all(|index| bytes[*index].is_ascii_digit());
    if value.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !digits_at([0, 1, 2, 3, 5, 6, 8, 9])
    {
        return false;
    }
    match (
        value[0..4].parse::<u16>(),
        value[5..7].parse::<u8>(),
        value[8..10].parse::<u8>(),
    ) {
        // Digit-only fixed-width slices always parse; the arms below are the
        // real gate: a real month and a day that month can hold.
        (Ok(year), Ok(month), Ok(day)) => {
            (1..=12).contains(&month) && day >= 1 && day <= days_in_month(year, month)
        }
        _ => false,
    }
}

/// Days in a Gregorian calendar month; zero for a value that is not a
/// month, so callers can fold the month-range check into the day bound.
fn days_in_month(year: u16, month: u8) -> u8 {
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    }
}

fn fallback_evaluation_date(record: Option<&ClinvarRecord>) -> Option<&str> {
    record?
        .aggregates
        .iter()
        .filter_map(|row| row.evaluation_date.as_deref())
        .filter(|value| is_day_shaped(value))
        .max()
}

fn indirect_conditions(value: Option<&serde_json::Value>, preferred: Option<&str>) -> Vec<String> {
    fn collect(value: &serde_json::Value, out: &mut Vec<String>) {
        match value {
            serde_json::Value::String(value) if !value.trim().is_empty() => out.push(value.clone()),
            serde_json::Value::Array(values) => values.iter().for_each(|value| collect(value, out)),
            serde_json::Value::Object(object) => {
                for key in ["name", "preferred_name"] {
                    if let Some(value) = object.get(key) {
                        collect(value, out);
                    }
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    if let Some(value) = value {
        collect(value, &mut out);
    }
    if let Some(preferred) = preferred.map(str::trim).filter(|value| !value.is_empty()) {
        out.push(preferred.to_string());
    }
    out.sort();
    out.dedup();
    out
}

pub(super) fn indirect_clinvar_record(
    hit: &crate::sources::myvariant::MyVariantHit,
) -> Option<super::ClinvarRecord> {
    let clinvar = hit.clinvar.as_ref()?;
    let variation_id = clinvar.variant_id?;
    let aggregates = clinvar
        .rcv
        .iter()
        .filter_map(|rcv| {
            let accession = rcv.accession.as_deref()?.trim();
            (!accession.is_empty()).then(|| super::ClinvarAggregate {
                source: "MyVariant.info".into(),
                accession: accession.into(),
                version: rcv.version,
                classification_domain: "germline".into(),
                classification: rcv.clinical_significance.clone(),
                review_status: rcv.review_status.clone(),
                evaluation_date: rcv.last_evaluated.clone(),
                record_status: None,
                number_submitters: rcv.number_submitters,
                submission_count: None,
                conditions: indirect_conditions(
                    rcv.conditions.as_ref(),
                    rcv.preferred_name.as_deref(),
                ),
            })
        })
        .collect::<Vec<_>>();
    (!aggregates.is_empty()).then(|| super::ClinvarRecord {
        source: "MyVariant.info".into(),
        variation_id,
        accession: None,
        version: None,
        record_status: None,
        number_submissions: None,
        number_submitters: None,
        germline_classification: None,
        aggregates,
        submissions: Vec::new(),
    })
}

pub(super) fn apply_clinvar_result(
    variant: &mut Variant,
    fallback: Option<super::ClinvarRecord>,
    direct: Result<Option<super::ClinvarRecord>, ClinvarDirectFailure>,
) {
    match direct {
        Ok(Some(record)) if !record.aggregates.is_empty() || !record.submissions.is_empty() => {
            super::get::apply_record_level_headline(variant, &record);
            variant.clinvar = Some(record);
            variant
                .section_outcomes
                .complete("clinvar", SectionOutcome::data("NCBI ClinVar"));
        }
        Ok(_) => variant
            .section_outcomes
            .complete("clinvar", SectionOutcome::empty("NCBI ClinVar")),
        Err(failure) => {
            variant.clinvar = fallback;
            if variant.clinvar.is_some() {
                variant.section_outcomes.complete(
                    "clinvar",
                    SectionOutcome::degraded(
                        ["MyVariant.info"],
                        failure.degraded_message(variant.clinvar.as_ref()),
                    ),
                );
            } else {
                variant.section_outcomes.complete(
                    "clinvar",
                    SectionOutcome::unavailable(failure.unavailable_message()),
                );
            }
        }
    }
}

pub(super) async fn add_clinvar(
    variant: &mut Variant,
    hit: &crate::sources::myvariant::MyVariantHit,
    timeout: Duration,
) {
    let fallback = indirect_clinvar_record(hit);
    let variation_id = hit.clinvar.as_ref().and_then(|clinvar| clinvar.variant_id);
    super::get::strip_clinvar_details(variant);
    let Some(variation_id) = variation_id else {
        variant
            .section_outcomes
            .complete("clinvar", SectionOutcome::inapplicable(CLINVAR_ID_REQUIRED));
        return;
    };
    let direct = async {
        let client =
            ClinvarClient::new().map_err(|error| ClinvarDirectFailure::from_error(&error))?;
        client
            .variation(variation_id)
            .await
            .map_err(|error| ClinvarDirectFailure::from_error(&error))
    };
    let result = tokio::time::timeout(timeout, direct)
        .await
        .map_err(|_| ClinvarDirectFailure::Timeout)
        .and_then(|result| result);
    apply_clinvar_result(variant, fallback, result);
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use crate::entities::section_outcome::SectionOutcomeState;

    use super::*;

    fn hit() -> crate::sources::myvariant::MyVariantHit {
        serde_json::from_value(serde_json::json!({
            "_id": "chr5:g.118860951A>G",
            "clinvar": {"variant_id": 974782, "rcv": {
                "accession": "RCV001251043", "clinical_significance": "Likely pathogenic",
                "last_evaluated": "2019-04-11"
            }}
        }))
        .expect("fixture")
    }

    fn direct_record(with_row: bool) -> super::super::ClinvarRecord {
        super::super::ClinvarRecord {
            source: "NCBI ClinVar".into(),
            variation_id: 974782,
            accession: Some("VCV000974782".into()),
            version: Some(2),
            record_status: Some("current".into()),
            number_submissions: None,
            number_submitters: None,
            germline_classification: None,
            aggregates: if with_row {
                indirect_clinvar_record(&hit())
                    .expect("fallback")
                    .aggregates
            } else {
                Vec::new()
            },
            submissions: Vec::new(),
        }
    }

    #[test]
    fn canonical_outcomes_and_provenance_match_selected_payload_source() {
        let cases = [
            (
                Ok(Some(direct_record(true))),
                Some(indirect_clinvar_record(&hit()).expect("fallback")),
                SectionOutcomeState::Data,
                vec!["NCBI ClinVar"],
                Some("NCBI ClinVar"),
            ),
            (
                Ok(Some(direct_record(false))),
                Some(indirect_clinvar_record(&hit()).expect("fallback")),
                SectionOutcomeState::Empty,
                vec!["NCBI ClinVar"],
                None,
            ),
            (
                Err(ClinvarDirectFailure::Timeout),
                Some(indirect_clinvar_record(&hit()).expect("fallback")),
                SectionOutcomeState::Degraded,
                vec!["MyVariant.info"],
                Some("MyVariant.info"),
            ),
            (
                Err(ClinvarDirectFailure::Timeout),
                None,
                SectionOutcomeState::Unavailable,
                vec![],
                None,
            ),
        ];
        for (direct, fallback, state, sources, payload_source) in cases {
            let mut variant = crate::transform::variant::from_myvariant_hit(&hit());
            variant.clinvar = None;
            apply_clinvar_result(&mut variant, fallback, direct);
            let outcome = variant.section_outcomes.get("clinvar").expect("outcome");
            assert_eq!(outcome.outcome(), state);
            assert_eq!(outcome.sources(), sources);
            assert_eq!(
                variant
                    .clinvar
                    .as_ref()
                    .map(|record| record.source.as_str()),
                payload_source
            );
            let provenance = crate::render::provenance::variant_section_sources(&variant);
            let clinvar = provenance
                .iter()
                .find(|row| row.key == "clinvar")
                .expect("row");
            assert_eq!(clinvar.outcome, state);
            assert_eq!(clinvar.sources, sources);
        }
    }

    #[tokio::test]
    async fn missing_numeric_variation_id_is_source_free_inapplicable() {
        let hit = serde_json::from_value(serde_json::json!({
            "_id": "chr5:g.118860951A>G",
            "clinvar": {"rcv": {"accession": "RCV001251043"}}
        }))
        .expect("fixture");
        let mut variant = crate::transform::variant::from_myvariant_hit(&hit);
        add_clinvar(&mut variant, &hit, Duration::from_millis(1)).await;
        let outcome = variant.section_outcomes.get("clinvar").expect("outcome");
        assert_eq!(outcome.outcome(), SectionOutcomeState::Inapplicable);
        assert!(outcome.sources().is_empty());
        assert!(variant.clinvar.is_none());
        let provenance = crate::render::provenance::variant_section_sources(&variant);
        let row = provenance
            .iter()
            .find(|row| row.key == "clinvar")
            .expect("row");
        assert_eq!(row.outcome, SectionOutcomeState::Inapplicable);
        assert!(row.sources.is_empty());
    }

    #[test]
    fn degraded_labels_name_the_failed_source_reason_and_fallback_age() {
        let cases = [
            (
                ClinvarDirectFailure::Timeout,
                "NCBI ClinVar timed out (slow answer or sustained rate limiting); showing \
                 MyVariant.info fallback data (newest evaluation 2019-04-11).",
            ),
            (
                ClinvarDirectFailure::RateLimited,
                "NCBI ClinVar rate limited the request; showing MyVariant.info \
                 fallback data (newest evaluation 2019-04-11).",
            ),
            (
                ClinvarDirectFailure::ProviderError,
                "NCBI ClinVar request failed; showing MyVariant.info \
                 fallback data (newest evaluation 2019-04-11).",
            ),
        ];
        for (failure, expected) in cases {
            let mut variant = crate::transform::variant::from_myvariant_hit(&hit());
            variant.clinvar = None;
            apply_clinvar_result(
                &mut variant,
                Some(indirect_clinvar_record(&hit()).expect("fallback")),
                Err(failure),
            );
            let outcome = variant.section_outcomes.get("clinvar").expect("outcome");
            assert_eq!(outcome.outcome(), SectionOutcomeState::Degraded);
            assert_eq!(outcome.sources(), ["MyVariant.info"]);
            assert_eq!(outcome.message(), Some(expected));
        }
    }

    #[test]
    fn unavailable_labels_name_the_failed_source_and_reason() {
        let cases = [
            (
                ClinvarDirectFailure::Timeout,
                "NCBI ClinVar timed out (slow answer or sustained rate limiting); \
                 no MyVariant.info fallback data is available.",
            ),
            (
                ClinvarDirectFailure::RateLimited,
                "NCBI ClinVar rate limited the request; no MyVariant.info fallback \
                 data is available.",
            ),
            (
                ClinvarDirectFailure::ProviderError,
                "NCBI ClinVar request failed; no MyVariant.info fallback data is available.",
            ),
        ];
        for (failure, expected) in cases {
            let mut variant = crate::transform::variant::from_myvariant_hit(&hit());
            variant.clinvar = None;
            apply_clinvar_result(&mut variant, None, Err(failure));
            let outcome = variant.section_outcomes.get("clinvar").expect("outcome");
            assert_eq!(outcome.outcome(), SectionOutcomeState::Unavailable);
            assert_eq!(outcome.message(), Some(expected));
        }
    }

    #[test]
    fn provider_errors_classify_into_rate_limit_and_provider_buckets() {
        use crate::error::{SourceContext, SourceProvider};

        let rate_limited = BioMcpError::Api {
            api: "NCBI ClinVar".into(),
            message: CLINVAR_RATE_LIMIT_MESSAGE.into(),
        };
        assert_eq!(
            ClinvarDirectFailure::from_error(&rate_limited),
            ClinvarDirectFailure::RateLimited
        );
        let wrapped =
            rate_limited.with_source_context(SourceContext::narrow(SourceProvider::NCBI_EFETCH));
        assert_eq!(
            ClinvarDirectFailure::from_error(&wrapped),
            ClinvarDirectFailure::RateLimited
        );
        // The marker requires both the ClinVar api and the shared message, so
        // another lane's rate-limit refusal cannot read as ours.
        let other_lane = BioMcpError::Api {
            api: "pubmed-eutils".into(),
            message: CLINVAR_RATE_LIMIT_MESSAGE.into(),
        };
        assert_eq!(
            ClinvarDirectFailure::from_error(&other_lane),
            ClinvarDirectFailure::ProviderError
        );
        let other = BioMcpError::Api {
            api: "NCBI ClinVar".into(),
            message: "ClinVar record was unavailable".into(),
        };
        assert_eq!(
            ClinvarDirectFailure::from_error(&other),
            ClinvarDirectFailure::ProviderError
        );
        let transport = BioMcpError::Io(std::io::Error::other("connection reset"));
        assert_eq!(
            ClinvarDirectFailure::from_error(&transport),
            ClinvarDirectFailure::ProviderError
        );
    }

    #[test]
    fn day_shaped_gate_requires_iso_shape_and_a_real_calendar_day() {
        assert!(is_day_shaped("2019-04-01"));
        assert!(is_day_shaped("2020-02-29"));
        assert!(is_day_shaped("2000-02-29"));
        assert!(!is_day_shaped("1900-02-29"));
        assert!(!is_day_shaped("2021-02-29"));
        assert!(!is_day_shaped("2021-02-30"));
        assert!(!is_day_shaped("2021-04-31"));
        assert!(!is_day_shaped("2019-13-01"));
        assert!(!is_day_shaped("01-04-2019"));
        assert!(!is_day_shaped("9999-99-99"));
        assert!(!is_day_shaped("----------"));
        assert!(!is_day_shaped("2019-4-01"));
        assert!(!is_day_shaped("01 Apr 2019"));
    }

    #[test]
    fn fallback_age_is_omitted_without_a_strict_day_shaped_date() {
        let mut dated = hit();
        if let Some(rcv) = dated
            .clinvar
            .as_mut()
            .and_then(|clinvar| clinvar.rcv.first_mut())
        {
            rcv.last_evaluated = Some("01 Apr 2019".into());
        }
        let fallback = indirect_clinvar_record(&dated).expect("fallback");
        let mut variant = crate::transform::variant::from_myvariant_hit(&dated);
        variant.clinvar = None;
        apply_clinvar_result(
            &mut variant,
            Some(fallback),
            Err(ClinvarDirectFailure::Timeout),
        );
        let outcome = variant.section_outcomes.get("clinvar").expect("outcome");
        assert_eq!(
            outcome.message(),
            Some(
                "NCBI ClinVar timed out (slow answer or sustained rate limiting); showing \
                 MyVariant.info fallback data.",
            )
        );
    }

    #[test]
    fn fallback_age_names_the_newest_of_several_dated_rows() {
        let dated = serde_json::from_value(serde_json::json!({
            "_id": "chr5:g.118860951A>G",
            "clinvar": {"variant_id": 974782, "rcv": [
                {"accession": "RCV000000001", "last_evaluated": "2015-06-01"},
                {"accession": "RCV000000002", "last_evaluated": "01 Apr 2022"},
                {"accession": "RCV000000003", "last_evaluated": "2019-04-11"},
                {"accession": "RCV000000004", "last_evaluated": "2021-03-11"}
            ]}
        }))
        .expect("fixture");
        let fallback = indirect_clinvar_record(&dated).expect("fallback");
        // The newest dated row wins even when a newer-looking provider
        // spelling ("01 Apr 2022") is not day-shaped and older rows sit
        // earlier in iteration order.
        assert_eq!(
            fallback_evaluation_date(Some(&fallback)),
            Some("2021-03-11")
        );
        let mut variant = crate::transform::variant::from_myvariant_hit(&dated);
        variant.clinvar = None;
        apply_clinvar_result(
            &mut variant,
            Some(fallback),
            Err(ClinvarDirectFailure::Timeout),
        );
        let outcome = variant.section_outcomes.get("clinvar").expect("outcome");
        assert_eq!(
            outcome.message(),
            Some(
                "NCBI ClinVar timed out (slow answer or sustained rate limiting); showing \
                 MyVariant.info fallback data (newest evaluation 2021-03-11).",
            )
        );
    }

    struct ClinvarFixtureEnv(Vec<(&'static str, Option<std::ffi::OsString>)>);

    impl ClinvarFixtureEnv {
        fn set(&mut self, name: &'static str, value: &str) {
            self.0.push((name, std::env::var_os(name)));
            // SAFETY: this test holds the serial-test process-wide environment lock.
            unsafe { std::env::set_var(name, value) };
        }
    }

    impl Drop for ClinvarFixtureEnv {
        fn drop(&mut self) {
            for (name, prior) in self.0.drain(..).rev() {
                // SAFETY: this test holds the serial-test process-wide environment lock.
                unsafe {
                    if let Some(value) = prior {
                        std::env::set_var(name, value);
                    } else {
                        std::env::remove_var(name);
                    }
                }
            }
        }
    }

    /// How the direct ClinVar fixture server answers efetch.
    #[derive(Clone, Copy)]
    enum DirectAnswer {
        /// Never answer; hold the connection open past the caller's deadline.
        HeldOpen,
        /// Answer 429 at once, with or without a Retry-After hold.
        Fast429 { retry_after: Option<u64> },
        /// Answer 500 at once, a non-rate-limit provider failure.
        Fast500,
    }

    async fn direct_answer_server(answer: DirectAnswer) -> (String, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind direct ClinVar fixture");
        let base = format!("http://{}", listener.local_addr().expect("fixture address"));
        let task = tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let mut request = vec![0_u8; 16 * 1024];
                    if stream.read(&mut request).await.is_err() {
                        return;
                    }
                    match answer {
                        DirectAnswer::HeldOpen => {
                            tokio::time::sleep(Duration::from_secs(60)).await;
                        }
                        DirectAnswer::Fast429 { retry_after } => {
                            let headers = match retry_after {
                                Some(seconds) => format!("Retry-After: {seconds}\r\n"),
                                None => String::new(),
                            };
                            let response = format!(
                                "HTTP/1.1 429 Too Many Requests\r\n{headers}\
                                 Content-Length: 0\r\nConnection: close\r\n\r\n"
                            );
                            stream
                                .write_all(response.as_bytes())
                                .await
                                .expect("write fixture response");
                        }
                        DirectAnswer::Fast500 => {
                            stream
                                .write_all(
                                    b"HTTP/1.1 500 Internal Server Error\r\n\
                                      Content-Length: 0\r\nConnection: close\r\n\r\n",
                                )
                                .await
                                .expect("write fixture response");
                        }
                    }
                });
            }
        });
        (base, task)
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn add_clinvar_labels_a_deadline_miss_when_the_direct_answer_never_arrives() {
        let (base, server) = direct_answer_server(DirectAnswer::HeldOpen).await;
        let mut env = ClinvarFixtureEnv(Vec::new());
        env.set("BIOMCP_CLINVAR_BASE", &base);
        // A fresh empty cache root keeps shared client construction fast
        // and inside the deadline being asserted; scanning a large default
        // user cache blocks the runtime thread past any short deadline.
        let cache_dir = tempfile::tempdir().expect("temp cache dir");
        env.set(
            "BIOMCP_CACHE_DIR",
            cache_dir.path().to_str().expect("utf-8 temp path"),
        );

        let started = Instant::now();
        let mut variant = crate::transform::variant::from_myvariant_hit(&hit());
        add_clinvar(&mut variant, &hit(), Duration::from_millis(300)).await;
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "deadline must bound the wait"
        );
        server.abort();

        assert_eq!(
            variant
                .clinvar
                .as_ref()
                .map(|record| record.source.as_str()),
            Some("MyVariant.info")
        );
        let outcome = variant.section_outcomes.get("clinvar").expect("outcome");
        assert_eq!(outcome.outcome(), SectionOutcomeState::Degraded);
        assert_eq!(outcome.sources(), ["MyVariant.info"]);
        assert_eq!(
            outcome.message(),
            Some(
                "NCBI ClinVar timed out (slow answer or sustained rate limiting); showing \
                 MyVariant.info fallback data (newest evaluation 2019-04-11).",
            )
        );
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn add_clinvar_admits_rate_limiting_when_a_sustained_429_outlives_the_deadline() {
        // A 429 whose Retry-After hold outlives the caller's deadline: the
        // retry sleep runs inside the dropped send future, so the deadline
        // miss can only be labeled with the honest merged wording.
        let (base, server) = direct_answer_server(DirectAnswer::Fast429 {
            retry_after: Some(30),
        })
        .await;
        let mut env = ClinvarFixtureEnv(Vec::new());
        env.set("BIOMCP_CLINVAR_BASE", &base);
        // A fresh empty cache root keeps shared client construction fast
        // and inside the deadline being asserted; scanning a large default
        // user cache blocks the runtime thread past any short deadline.
        let cache_dir = tempfile::tempdir().expect("temp cache dir");
        env.set(
            "BIOMCP_CACHE_DIR",
            cache_dir.path().to_str().expect("utf-8 temp path"),
        );

        let started = Instant::now();
        let mut variant = crate::transform::variant::from_myvariant_hit(&hit());
        add_clinvar(&mut variant, &hit(), Duration::from_millis(300)).await;
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "deadline must bound the wait"
        );
        server.abort();

        let outcome = variant.section_outcomes.get("clinvar").expect("outcome");
        assert_eq!(outcome.outcome(), SectionOutcomeState::Degraded);
        assert_eq!(
            outcome.message(),
            Some(
                "NCBI ClinVar timed out (slow answer or sustained rate limiting); showing \
                 MyVariant.info fallback data (newest evaluation 2019-04-11).",
            )
        );
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn add_clinvar_names_a_fast_429_refusal_as_rate_limited() {
        let (base, server) =
            direct_answer_server(DirectAnswer::Fast429 { retry_after: None }).await;
        let mut env = ClinvarFixtureEnv(Vec::new());
        env.set("BIOMCP_CLINVAR_BASE", &base);
        // A fresh empty cache root keeps shared client construction fast
        // and inside the deadline being asserted; scanning a large default
        // user cache blocks the runtime thread past any short deadline.
        let cache_dir = tempfile::tempdir().expect("temp cache dir");
        env.set(
            "BIOMCP_CACHE_DIR",
            cache_dir.path().to_str().expect("utf-8 temp path"),
        );

        let mut variant = crate::transform::variant::from_myvariant_hit(&hit());
        add_clinvar(&mut variant, &hit(), Duration::from_secs(15)).await;
        server.abort();

        let outcome = variant.section_outcomes.get("clinvar").expect("outcome");
        assert_eq!(outcome.outcome(), SectionOutcomeState::Degraded);
        assert_eq!(outcome.sources(), ["MyVariant.info"]);
        assert_eq!(
            outcome.message(),
            Some(
                "NCBI ClinVar rate limited the request; showing MyVariant.info \
                 fallback data (newest evaluation 2019-04-11).",
            )
        );
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn add_clinvar_names_a_non_429_provider_failure_distinctly() {
        let (base, server) = direct_answer_server(DirectAnswer::Fast500).await;
        let mut env = ClinvarFixtureEnv(Vec::new());
        env.set("BIOMCP_CLINVAR_BASE", &base);
        // A fresh empty cache root keeps shared client construction fast
        // and inside the deadline being asserted; scanning a large default
        // user cache blocks the runtime thread past any short deadline.
        let cache_dir = tempfile::tempdir().expect("temp cache dir");
        env.set(
            "BIOMCP_CACHE_DIR",
            cache_dir.path().to_str().expect("utf-8 temp path"),
        );

        let mut variant = crate::transform::variant::from_myvariant_hit(&hit());
        add_clinvar(&mut variant, &hit(), Duration::from_secs(15)).await;
        server.abort();

        let outcome = variant.section_outcomes.get("clinvar").expect("outcome");
        assert_eq!(outcome.outcome(), SectionOutcomeState::Degraded);
        assert_eq!(outcome.sources(), ["MyVariant.info"]);
        assert_eq!(
            outcome.message(),
            Some(
                "NCBI ClinVar request failed; showing MyVariant.info \
                 fallback data (newest evaluation 2019-04-11).",
            )
        );
    }
}
