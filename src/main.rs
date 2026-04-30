use tracing_subscriber::EnvFilter;

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .try_init();
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    init_tracing();

    let cli = biomcp_cli::cli::parse_cli_from_env();
    match cli.command {
        biomcp_cli::cli::Commands::Mcp | biomcp_cli::cli::Commands::Serve => {
            match biomcp_cli::mcp::run_stdio().await {
                Ok(()) => std::process::ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("Error: {err}");
                    std::process::ExitCode::from(1)
                }
            }
        }
        biomcp_cli::cli::Commands::ServeHttp(args) => {
            let host = args.host;
            let port = args.port;
            match biomcp_cli::mcp::run_http(&host, port).await {
                Ok(()) => std::process::ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("Error: {err}");
                    std::process::ExitCode::from(1)
                }
            }
        }
        biomcp_cli::cli::Commands::ServeSse => match biomcp_cli::mcp::run_sse().await {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("Error: {err}");
                std::process::ExitCode::from(1)
            }
        },
        _ => match biomcp_cli::cli::run_outcome(cli).await {
            Ok(output) => {
                match output.stream {
                    biomcp_cli::cli::OutputStream::Stdout => println!("{}", output.text),
                    biomcp_cli::cli::OutputStream::Stderr => eprintln!("{}", output.text),
                }
                std::process::ExitCode::from(output.exit_code)
            }
            Err(err) => {
                if let Some(bio_err) = err.downcast_ref::<biomcp_cli::error::BioMcpError>() {
                    eprintln!("Error: {bio_err}");
                    match bio_err {
                        biomcp_cli::error::BioMcpError::InvalidArgument(_) => {
                            std::process::ExitCode::from(2)
                        }
                        _ => std::process::ExitCode::from(1),
                    }
                } else {
                    eprintln!("Error: {err}");
                    std::process::ExitCode::from(1)
                }
            }
        },
    }
}
