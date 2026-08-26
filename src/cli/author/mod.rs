//! Provider-exact author CLI.

mod detail;
mod papers;
mod search;

use clap::Subcommand;

pub use detail::AuthorGetArgs;
pub(in crate::cli) use detail::handle_get;
pub(in crate::cli) use papers::handle_papers;
pub use search::AuthorSearchArgs;
pub(in crate::cli) use search::handle_search;

#[derive(Subcommand, Debug)]
pub enum AuthorCommand {
    /// List compact papers for one exact Semantic Scholar author record
    Papers {
        /// Provider-qualified author ID (`semanticscholar:<id>`)
        id: String,
        /// Maximum papers, 1-100 (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Zero-based provider offset
        #[arg(long, default_value = "0")]
        offset: usize,
    },
}

pub(in crate::cli) async fn handle(
    command: AuthorCommand,
    json: bool,
) -> anyhow::Result<crate::cli::CommandOutcome> {
    let AuthorCommand::Papers { id, limit, offset } = command;
    handle_papers(id, limit, offset, json).await
}

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
