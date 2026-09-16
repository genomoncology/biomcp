use anyhow::Context;

use crate::cli::CommandOutcome;
use crate::entities::article::{
    ArticleRankingOptions, ArticleSearchFilters, ArticleSort, ArticleSourceFilter,
};
use crate::entities::discover::{DiscoverArticleSearch, DiscoverArticleSearchRender};

#[derive(Debug, Clone)]
pub struct DiscoverArgs {
    pub query: String,
    pub limit: usize,
    pub offset: usize,
    pub full: bool,
    pub search: bool,
}

/// One bounded page, matching the suggested command's `--limit 5`.
const DISCOVER_ARTICLE_SEARCH_LIMIT: usize = 5;

pub async fn run(args: DiscoverArgs, json: bool) -> anyhow::Result<String> {
    Ok(run_outcome(args, json).await?.text)
}

pub async fn run_outcome(args: DiscoverArgs, json: bool) -> anyhow::Result<CommandOutcome> {
    let mut result = crate::entities::discover::resolve_query_with_options(
        &args.query,
        crate::entities::discover::DiscoverMode::Command,
        crate::entities::discover::DiscoverOptions {
            limit: args.limit,
            offset: args.offset,
            full: args.full,
        },
    )
    .await
    .context("discover requires OLS4")?;

    if args.search && result.concepts.is_empty() {
        let query = result.query.clone();
        result.article_search = Some(run_inline_article_search(&query).await?);
    }

    let budget = if args.full { 256 * 1024 } else { 32 * 1024 };
    let structured = loop {
        let candidate = crate::render::json::to_discover_json(&result)?;
        if candidate.len() <= budget || result.concepts.len() <= 1 {
            break candidate;
        }
        result.concepts.pop();
        result.preview_meta.pop();
        result.returned = result.concepts.len();
        result.has_more = true;
        result.next_offset = Some(result.offset.saturating_add(result.returned));
        result.budget_truncated = true;
        result.continuation_command = result.next_offset.map(|next| {
            crate::entities::discover::discover_continuation_command(
                &result.query,
                result.limit,
                next,
                result.full,
            )
        });
        crate::entities::discover::refresh_selected_guidance(&mut result);
    };
    let text = if json {
        structured.clone()
    } else {
        crate::render::markdown::render_discover(&result)?
    };
    Ok(CommandOutcome::stdout(text).with_metadata_json(structured))
}

/// Run the article keyword search the no-concept note recommends, through the
/// same entity entry point the `search article` command uses.
pub(crate) async fn run_inline_article_search(
    query: &str,
) -> anyhow::Result<DiscoverArticleSearch> {
    let filters = ArticleSearchFilters {
        gene: None,
        gene_anchored: false,
        disease: None,
        drug: None,
        variant: None,
        author: None,
        keyword: Some(query.trim().to_string()),
        date_from: None,
        date_to: None,
        article_type: Some("review".to_string()),
        journal: None,
        open_access: false,
        no_preprints: false,
        exclude_retracted: true,
        max_per_source: None,
        sort: ArticleSort::Relevance,
        ranking: ArticleRankingOptions::default(),
    };
    let source_filter = ArticleSourceFilter::All;
    let limit = DISCOVER_ARTICLE_SEARCH_LIMIT;
    let offset = 0;
    let page =
        crate::entities::article::search_page(&filters, limit, offset, source_filter).await?;
    let backend_plan = crate::entities::article::plan_backends(&filters, source_filter)?;
    let semantic_scholar_enabled =
        matches!(
            backend_plan,
            crate::entities::article::BackendPlan::Both
                | crate::entities::article::BackendPlan::SemanticScholarOnly
        ) && crate::entities::article::semantic_scholar_search_enabled(&filters, source_filter);
    let pagination =
        crate::cli::shared::PaginationMeta::offset(offset, limit, page.results.len(), page.total);
    let warning = super::article::article_search_warnings(&filters, &page.results)
        .first()
        .map(|warning| warning.message.to_string());
    let note = crate::entities::article::article_type_limitation_note(&filters, source_filter);
    Ok(DiscoverArticleSearch {
        command: crate::entities::discover::review_article_fallback_command(query),
        returned: page.results.len(),
        results: page.results,
        render: DiscoverArticleSearchRender {
            query_summary: super::article::article_query_summary(
                &filters,
                source_filter,
                false,
                limit,
                offset,
            ),
            pagination_footer: crate::cli::shared::pagination_footer_offset(&pagination),
            filters,
            source_filter,
            semantic_scholar_enabled,
            warning,
            note,
            source_status: page.source_status,
            limit,
            offset,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read as _, Write as _};
    use std::sync::{Arc, Mutex, OnceLock};

    const QUERY: &str = "SCENAR therapy";
    const RESOLVED_QUERY: &str = "SCENAR therapy resolved";
    const FAILING_QUERY: &str = "SCENAR therapy failing";
    const SUGGESTED_COMMAND: &str =
        "biomcp search article -k \"SCENAR therapy\" --type review --limit 5";

    const OLS4_NO_MATCH: &str = r#"{"response":{"docs":[],"numFound":0,"start":0},"responseHeader":{"QTime":0,"status":0},"facet_counts":{"facet_fields":{}}}"#;
    const OLS4_MATCH: &str = r#"{"response":{"docs":[{"iri":"http://purl.obolibrary.org/obo/HP_6000181","ontology_name":"hp","ontology_prefix":"HP","short_form":"HP_6000181","obo_id":"HP:6000181","label":"SCENAR therapy response","exact_synonyms":["SCENAR therapy"],"type":"class","is_defining_ontology":false}],"numFound":1,"start":0},"responseHeader":{"QTime":0,"status":0},"facet_counts":{"facet_fields":{}}}"#;
    const EUROPE_PMC_ROWS: &str = r#"{"hitCount":3,"resultList":{"result":[
{"id":"1001","pmid":"1001","title":"SCENAR therapy review one","journalTitle":"Fixture Journal","firstPublicationDate":"2025-01-01","pubType":"review"},
{"id":"1002","pmid":"1002","title":"SCENAR therapy review two","journalTitle":"Fixture Journal","firstPublicationDate":"2025-02-01","pubType":"review"},
{"id":"1003","pmid":"1003","title":"SCENAR therapy review three","journalTitle":"Fixture Journal","firstPublicationDate":"2025-03-01","pubType":"review"}]}}"#;
    const EUROPE_PMC_EMPTY: &str = r#"{"hitCount":3,"resultList":{"result":[]}}"#;
    const PUBMED_NO_IDS: &str = r#"{"esearchresult":{"count":"0","idlist":[]}}"#;

    struct Fixture {
        base: String,
        requests: Arc<Mutex<Vec<String>>>,
        cache: crate::test_support::TempDirGuard,
    }

    impl Fixture {
        fn requests(&self) -> Vec<String> {
            self.requests.lock().unwrap().clone()
        }
    }

    /// The suggested command's provider requests inside a request-log slice.
    fn article_requests(requests: &[String]) -> Vec<String> {
        let mut values = requests
            .iter()
            .filter(|request| request.contains("/europepmc/") || request.contains("/pubmed/"))
            .cloned()
            .collect::<Vec<_>>();
        values.sort();
        values
    }

    fn http_response(status: &str, body: &str) -> Vec<u8> {
        let mut response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .into_bytes();
        response.extend_from_slice(body.as_bytes());
        response
    }

    fn fixture_response(target: &str) -> Vec<u8> {
        if target.starts_with("/ols4/") {
            http_response(
                "200 OK",
                if target.contains("resolved") {
                    OLS4_MATCH
                } else {
                    OLS4_NO_MATCH
                },
            )
        } else if target.starts_with("/hpo/") {
            http_response("200 OK", r#"{"terms":[]}"#)
        } else if target.starts_with("/europepmc/search") {
            if target.contains("failing") {
                http_response("500 Internal Server Error", r#"{"error":"down"}"#)
            } else if target.contains("page=2") {
                http_response("200 OK", EUROPE_PMC_EMPTY)
            } else {
                http_response("200 OK", EUROPE_PMC_ROWS)
            }
        } else if target.starts_with("/pubmed/esearch.fcgi") {
            if target.contains("failing") {
                http_response("500 Internal Server Error", r#"{"error":"down"}"#)
            } else {
                http_response("200 OK", PUBMED_NO_IDS)
            }
        } else {
            http_response("404 Not Found", r#"{"error":"not found"}"#)
        }
    }

    /// One process-wide loopback fixture. The shared HTTP client reads its
    /// unpaced origin once, so every test must target the same origin, and a
    /// plain thread keeps the fixture alive across per-test runtimes. The
    /// fixture also owns the cache root: HTTP client construction walks the
    /// cache tree, and the machine cache root is large enough that the walk
    /// can outlast the federated source deadline under any other disk load.
    fn fixture() -> &'static Fixture {
        static FIXTURE: OnceLock<Fixture> = OnceLock::new();
        FIXTURE.get_or_init(|| {
            let listener =
                std::net::TcpListener::bind("127.0.0.1:0").expect("bind discover fixture");
            let address = listener.local_addr().expect("discover fixture address");
            let requests = Arc::new(Mutex::new(Vec::new()));
            let logged = requests.clone();
            std::thread::spawn(move || {
                for stream in listener.incoming() {
                    let Ok(mut stream) = stream else {
                        continue;
                    };
                    let logged = logged.clone();
                    std::thread::spawn(move || {
                        let mut buffer = [0_u8; 16 * 1024];
                        let length = stream.read(&mut buffer).unwrap_or(0);
                        let request = String::from_utf8_lossy(&buffer[..length]).to_string();
                        let target = request
                            .lines()
                            .next()
                            .and_then(|line| line.split_whitespace().nth(1))
                            .unwrap_or_default()
                            .to_string();
                        logged.lock().unwrap().push(target.clone());
                        let _ = stream.write_all(&fixture_response(&target));
                    });
                }
            });
            Fixture {
                base: format!("http://{address}"),
                requests,
                cache: crate::test_support::TempDirGuard::new("discover-cache"),
            }
        })
    }

    struct TestEnvironment {
        previous: Vec<(&'static str, Option<std::ffi::OsString>)>,
    }

    impl TestEnvironment {
        fn for_fixture() -> Self {
            let mut environment = Self {
                previous: Vec::new(),
            };
            let base = fixture().base.clone();
            for (key, value) in [
                ("BIOMCP_OLS4_BASE", format!("{base}/ols4")),
                ("BIOMCP_HPO_BASE", format!("{base}/hpo")),
                (
                    "BIOMCP_MEDLINEPLUS_BASE",
                    format!("{base}/unused-medlineplus"),
                ),
                ("BIOMCP_EUROPEPMC_BASE", format!("{base}/europepmc")),
                ("BIOMCP_PUBMED_BASE", format!("{base}/pubmed")),
                ("BIOMCP_S2_BASE", format!("{base}/s2")),
                ("BIOMCP_TEST_UNPACED_ORIGIN", base.clone()),
                (
                    "BIOMCP_CACHE_DIR",
                    fixture().cache.path().to_string_lossy().into_owned(),
                ),
                ("BIOMCP_CACHE_MODE", "off".to_string()),
                ("UMLS_API_KEY", String::new()),
                ("NCBI_API_KEY", String::new()),
                ("S2_API_KEY", String::new()),
            ] {
                environment.set(key, value);
            }
            environment
        }

        fn set(&mut self, key: &'static str, value: impl AsRef<std::ffi::OsStr>) {
            if !self.previous.iter().any(|(existing, _)| *existing == key) {
                self.previous.push((key, std::env::var_os(key)));
            }
            // SAFETY: discover tests that mutate provider variables share the source_env serial key.
            unsafe { std::env::set_var(key, value) };
        }
    }

    impl Drop for TestEnvironment {
        fn drop(&mut self) {
            for (key, previous) in self.previous.drain(..).rev() {
                // SAFETY: discover tests that mutate provider variables share the source_env serial key.
                unsafe {
                    match previous {
                        Some(value) => std::env::set_var(key, value),
                        None => std::env::remove_var(key),
                    }
                }
            }
        }
    }

    /// The command future is large (ticket 1191). Test threads keep a smaller
    /// stack than the CLI execute thread, so run each body on a sized thread.
    fn run_with_stack<F: std::future::Future + Send + 'static>(future: F) -> F::Output
    where
        F::Output: Send + 'static,
    {
        std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("discover test runtime")
                    .block_on(future)
            })
            .expect("spawn discover test thread")
            .join()
            .unwrap_or_else(|payload| std::panic::resume_unwind(payload))
    }

    fn discover_args(query: &str, search: bool) -> DiscoverArgs {
        DiscoverArgs {
            query: query.to_string(),
            limit: 5,
            offset: 0,
            full: false,
            search,
        }
    }

    async fn run_json(query: &str, search: bool) -> (serde_json::Value, String) {
        let outcome = run_outcome(discover_args(query, search), true)
            .await
            .expect("discover should run");
        let json = serde_json::from_str(&outcome.text).expect("discover json");
        (json, outcome.text)
    }

    /// The suggested command's filters, restated the way the `search article`
    /// CLI command builds them for `-k <query> --type review --limit 5`.
    fn suggested_search_filters(query: &str) -> ArticleSearchFilters {
        ArticleSearchFilters {
            gene: None,
            gene_anchored: false,
            disease: None,
            drug: None,
            variant: None,
            author: None,
            keyword: Some(query.to_string()),
            date_from: None,
            date_to: None,
            article_type: Some("review".to_string()),
            journal: None,
            open_access: false,
            no_preprints: false,
            exclude_retracted: true,
            max_per_source: None,
            sort: ArticleSort::Relevance,
            ranking: ArticleRankingOptions::default(),
        }
    }

    #[test]
    #[serial_test::serial(source_env)]
    fn discover_search_without_concepts_runs_the_suggested_article_search() {
        run_with_stack(async {
            let _environment = TestEnvironment::for_fixture();

            let (json, _) = run_json(QUERY, true).await;
            assert_eq!(json["concepts"], serde_json::json!([]));
            let article = &json["article_search"];
            assert_eq!(article["command"], SUGGESTED_COMMAND);
            assert_eq!(article["returned"], 3);
            assert_eq!(article["results"].as_array().map(Vec::len), Some(3));

            let direct = crate::entities::article::search_page(
                &suggested_search_filters(QUERY),
                5,
                0,
                ArticleSourceFilter::All,
            )
            .await
            .expect("direct search");
            assert_eq!(direct.results.len(), 3);
            assert_eq!(
                article["results"],
                serde_json::to_value(&direct.results).expect("direct search json"),
                "the inline results must match a direct search call's results"
            );
        });
    }

    #[test]
    #[serial_test::serial(source_env)]
    fn discover_without_search_keeps_the_note_only_response() {
        run_with_stack(async {
            let _environment = TestEnvironment::for_fixture();
            let fixture = fixture();
            let logged_before = fixture.requests().len();

            let (json, _) = run_json(QUERY, false).await;
            assert!(
                json.get("article_search").is_none(),
                "article_search must appear only with --search"
            );
            assert!(
                json["notes"]
                    .as_array()
                    .expect("notes array")
                    .iter()
                    .any(|note| note
                        .as_str()
                        .is_some_and(|note| note.contains(SUGGESTED_COMMAND))),
                "the no-concept note still carries the suggested command"
            );
            assert!(
                article_requests(&fixture.requests()[logged_before..]).is_empty(),
                "discover without --search must not run the article search"
            );
        });
    }

    #[test]
    #[serial_test::serial(source_env)]
    fn discover_search_with_resolved_concepts_is_unchanged() {
        run_with_stack(async {
            let _environment = TestEnvironment::for_fixture();
            let fixture = fixture();
            let logged_before = fixture.requests().len();

            let (_, without) = run_json(RESOLVED_QUERY, false).await;
            let (json, with) = run_json(RESOLVED_QUERY, true).await;
            assert!(
                !json["concepts"]
                    .as_array()
                    .expect("concepts array")
                    .is_empty(),
                "the fixture resolves a concept for this query"
            );
            assert_eq!(without, with, "--search stays inert when concepts resolve");
            assert!(
                article_requests(&fixture.requests()[logged_before..]).is_empty(),
                "--search must not run the article search when concepts resolve"
            );
        });
    }

    #[test]
    #[serial_test::serial(source_env)]
    fn discover_inline_search_matches_the_suggested_command_requests() {
        run_with_stack(async {
            let _environment = TestEnvironment::for_fixture();
            let fixture = fixture();

            let direct_before = fixture.requests().len();
            let _ = crate::entities::article::search_page(
                &suggested_search_filters(QUERY),
                5,
                0,
                ArticleSourceFilter::All,
            )
            .await
            .expect("direct search");
            let direct_requests = article_requests(&fixture.requests()[direct_before..]);

            let inline_before = fixture.requests().len();
            let _ = run_json(QUERY, true).await;
            let inline_requests = article_requests(&fixture.requests()[inline_before..]);

            assert_eq!(
                direct_requests, inline_requests,
                "the inline search must issue the suggested command's provider requests"
            );
        });
    }

    #[test]
    #[serial_test::serial(source_env)]
    fn discover_inline_search_failure_propagates() {
        run_with_stack(async {
            let _environment = TestEnvironment::for_fixture();

            let error = run_outcome(discover_args(FAILING_QUERY, true), true)
                .await
                .expect_err("a failed inline search must surface as the command error");
            assert!(
                !error.to_string().is_empty(),
                "the propagated error keeps its message"
            );
        });
    }

    #[test]
    #[serial_test::serial(source_env)]
    fn discover_article_section_matches_the_direct_render_apart_from_the_header() {
        run_with_stack(async {
            let _environment = TestEnvironment::for_fixture();

            let article = run_inline_article_search(QUERY)
                .await
                .expect("inline article search");
            let render = article.render.clone();
            let direct = crate::render::markdown::article_search_markdown_with_footer_and_context(
                &render.query_summary,
                &article.results,
                &render.pagination_footer,
                &render.filters,
                crate::render::markdown::ArticleSearchRenderContext {
                    source_filter: render.source_filter,
                    semantic_scholar_enabled: render.semantic_scholar_enabled,
                    warning: render.warning.as_deref(),
                    note: render.note.as_deref(),
                    debug_plan: None,
                    exact_entity_commands: &[],
                    source_status: &render.source_status,
                    retry_page: Some((render.limit, render.offset)),
                    header: None,
                },
            )
            .expect("direct render");
            let (_, body) = direct.split_once('\n').expect("direct render header line");
            let expected = format!("## Article search\n{body}");

            let outcome = run_outcome(discover_args(QUERY, true), false)
                .await
                .expect("discover should run");
            assert!(
                outcome.text.contains(&expected),
                "the discover section must match the direct render with only the header substituted:\n{}",
                outcome.text
            );
            assert!(
                outcome.text.contains("|PMID 1001|"),
                "the section carries the inline search rows"
            );
        });
    }
}
