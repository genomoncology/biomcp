//! Provider-exact author CLI.

mod detail;
mod papers;
mod search;

use clap::Subcommand;

pub use detail::AuthorGetArgs;
pub(in crate::cli) use detail::handle_get;
#[cfg(test)]
pub(crate) use detail::render_loaded_card;
pub(in crate::cli) use papers::handle_papers;
pub use search::AuthorSearchArgs;
pub(in crate::cli) use search::handle_search;

#[derive(Subcommand, Debug)]
pub enum AuthorCommand {
    /// List papers for one exact Semantic Scholar author record (compact by default; --full adds rich source metadata)
    Papers {
        /// Provider-qualified author ID (`semanticscholar:<id>`)
        id: String,
        /// Maximum papers, 1-100 (default: 10); one bounded page, no prefetching
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Zero-based provider offset
        #[arg(long, default_value = "0")]
        offset: usize,
        /// Return the rich page with abstract, counts, open-access, fields, and byline (source-exact, one page)
        #[arg(long)]
        full: bool,
    },
}

pub(in crate::cli) async fn handle(
    command: AuthorCommand,
    json: bool,
) -> anyhow::Result<crate::cli::CommandOutcome> {
    let AuthorCommand::Papers {
        id,
        limit,
        offset,
        full,
    } = command;
    handle_papers(id, limit, offset, full, json).await
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

    #[test]
    fn author_papers_accepts_the_full_flag_with_limit_and_offset() {
        let cli = Cli::try_parse_from([
            "biomcp",
            "author",
            "papers",
            "semanticscholar:1716151",
            "--full",
            "--limit",
            "100",
            "--offset",
            "25",
        ])
        .expect("rich author papers grammar should parse");
        let crate::cli::commands::Commands::Author {
            cmd:
                crate::cli::author::AuthorCommand::Papers {
                    id,
                    limit,
                    offset,
                    full,
                },
        } = cli.command
        else {
            panic!("expected the author papers command");
        };
        assert_eq!(id, "semanticscholar:1716151");
        assert_eq!(limit, 100);
        assert_eq!(offset, 25);
        assert!(full, "--full selects the rich page");
    }
}
