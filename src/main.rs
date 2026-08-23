use std::io::{IsTerminal, Write};

use tracing_subscriber::EnvFilter;

const BUILD_VERSION: &str = match option_env!("BIOMCP_BUILD_VERSION") {
    Some(value) => value,
    None => env!("CARGO_PKG_VERSION"),
};
const BUILD_GIT_SHA: &str = match option_env!("BIOMCP_BUILD_GIT_SHA") {
    Some(value) => value,
    None => "unknown",
};
const BUILD_DATE: &str = match option_env!("BIOMCP_BUILD_DATE") {
    Some(value) => value,
    None => "unknown",
};

fn human_error(error: &dyn std::fmt::Display) -> String {
    biomcp_cli::cli::sanitize_human_diagnostic(&error.to_string())
}

fn is_mime_token(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

fn is_mime_parameter(parameter: &str) -> bool {
    let Some((name, value)) = parameter.trim().split_once('=') else {
        return false;
    };
    let value = value.trim();
    is_mime_token(name.trim())
        && (is_mime_token(value)
            || (value.starts_with('"')
                && value.ends_with('"')
                && value.len() > 2
                && value[1..value.len() - 1]
                    .bytes()
                    .all(|byte| byte.is_ascii_graphic() && byte != b'"')))
}

fn is_terminal_textual_article_asset(media_type: Option<&str>) -> bool {
    let Some(media_type) = media_type else {
        return false;
    };
    let mut parts = media_type.split(';');
    let Some((kind, subtype)) = parts.next().unwrap_or_default().trim().split_once('/') else {
        return false;
    };
    if parts.any(|parameter| !is_mime_parameter(parameter))
        || subtype.contains('/')
        || !is_mime_token(kind)
        || !is_mime_token(subtype)
    {
        return false;
    }
    if kind.eq_ignore_ascii_case("text") {
        return true;
    }
    kind.eq_ignore_ascii_case("application")
        && matches!(
            subtype.to_ascii_lowercase().as_str(),
            "json" | "xml" | "yaml" | "x-yaml" | "javascript" | "ecmascript"
        )
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .with_ansi(std::io::stderr().is_terminal())
        .try_init();
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    biomcp_cli::build_identity::install(biomcp_cli::build_identity::BuildIdentity {
        version: BUILD_VERSION,
        git_revision: BUILD_GIT_SHA,
        build_date: BUILD_DATE,
    });
    init_tracing();

    let cli = biomcp_cli::cli::parse_cli_from_env();
    if cli.json
        && matches!(
            &cli.command,
            biomcp_cli::cli::Commands::Mcp(biomcp_cli::cli::McpArgs { command: None })
                | biomcp_cli::cli::Commands::Serve
                | biomcp_cli::cli::Commands::ServeHttp(_)
                | biomcp_cli::cli::Commands::ServeSse
        )
    {
        let outcome = biomcp_cli::cli::server_json_rejection();
        println!("{}", outcome.text);
        let _ = std::io::stdout().flush();
        return std::process::ExitCode::from(outcome.exit_code);
    }
    match cli.command {
        biomcp_cli::cli::Commands::Mcp(biomcp_cli::cli::McpArgs { command: None })
        | biomcp_cli::cli::Commands::Serve => match biomcp_cli::mcp::run_stdio().await {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("Error: {}", human_error(&err));
                std::process::ExitCode::from(1)
            }
        },
        biomcp_cli::cli::Commands::ServeHttp(args) => {
            let host = args.host;
            let port = args.port;
            let allowed_hosts = args.allowed_hosts;
            let unsafe_allow_any_host = args.unsafe_allow_any_host;
            match biomcp_cli::mcp::run_http(&host, port, allowed_hosts, unsafe_allow_any_host).await
            {
                Ok(()) => std::process::ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("Error: {}", human_error(&err));
                    std::process::ExitCode::from(1)
                }
            }
        }
        biomcp_cli::cli::Commands::ServeSse => match biomcp_cli::mcp::run_sse().await {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("Error: {}", human_error(&err));
                std::process::ExitCode::from(1)
            }
        },
        _ => match biomcp_cli::cli::run_outcome(cli).await {
            Ok(output) => {
                match output.stream {
                    biomcp_cli::cli::OutputStream::Stdout => {
                        if let Some(binary) = output.bytes {
                            if binary.output_path.is_some() {
                                if let Err(error) = binary.write_to_destination().await {
                                    eprintln!("Error: {}", human_error(&error));
                                    let _ = std::io::stderr().flush();
                                    std::process::exit(i32::from(error.exit_code()));
                                }
                            } else {
                                let mut stdout = std::io::stdout();
                                if binary.is_article_asset
                                    && stdout.is_terminal()
                                    && !is_terminal_textual_article_asset(
                                        binary.media_type.as_deref(),
                                    )
                                {
                                    let error = biomcp_cli::error::BioMcpError::InvalidArgument(
                                        "Article asset is binary or has an unknown media type; use --output FILE or pipe standard output."
                                            .into(),
                                    );
                                    eprintln!("Error: {}", human_error(&error));
                                    let _ = std::io::stderr().flush();
                                    std::process::exit(i32::from(error.exit_code()));
                                }
                                let _ = stdout.write_all(&binary.bytes);
                            }
                        } else {
                            println!("{}", output.text);
                        }
                        let _ = std::io::stdout().flush();
                    }
                    biomcp_cli::cli::OutputStream::Stderr => {
                        eprintln!("{}", output.text);
                        let _ = std::io::stderr().flush();
                    }
                }
                std::process::exit(i32::from(output.exit_code));
            }
            Err(err) => {
                if let Some(bio_err) = err.downcast_ref::<biomcp_cli::error::BioMcpError>() {
                    eprintln!("Error: {}", human_error(bio_err));
                    let _ = std::io::stderr().flush();
                    std::process::exit(i32::from(bio_err.exit_code()));
                } else {
                    eprintln!("Error: {}", human_error(&err));
                    let _ = std::io::stderr().flush();
                    std::process::exit(1);
                }
            }
        },
    }
}
