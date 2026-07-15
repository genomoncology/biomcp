//! Provider-exact author CLI.

mod detail;
mod search;

pub use detail::AuthorGetArgs;
pub(in crate::cli) use detail::handle_get;
pub use search::AuthorSearchArgs;
pub(in crate::cli) use search::handle_search;

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::types::Cli;

    #[test]
    fn author_grammar_requires_named_query_and_omits_affiliation() {
        Cli::try_parse_from([
            "biomcp",
            "search",
            "author",
            "--query",
            "A. Butte",
            "--source",
            "semanticscholar",
            "--limit",
            "5",
            "--offset",
            "1",
        ])
        .expect("supported author search should parse");

        for unsupported in [
            vec!["biomcp", "search", "author", "A. Butte"],
            vec![
                "biomcp",
                "search",
                "author",
                "--query",
                "A. Butte",
                "--affiliation",
                "UCSF",
            ],
        ] {
            assert!(
                Cli::try_parse_from(unsupported).is_err(),
                "unsupported author grammar parsed"
            );
        }
    }
}
