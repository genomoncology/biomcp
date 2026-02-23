use clap::Parser;
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

    let cli = biomcp_cli::cli::Cli::parse();
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
        biomcp_cli::cli::Commands::ServeHttp { host, port } => {
            match biomcp_cli::mcp::run_http(&host, port).await {
                Ok(()) => std::process::ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("Error: {err}");
                    std::process::ExitCode::from(1)
                }
            }
        }
        _ => match biomcp_cli::cli::run(cli).await {
            Ok(output) => {
                println!("{output}");
                std::process::ExitCode::SUCCESS
            }
            Err(err) => {
                if let Some(bio_err) = err.downcast_ref::<biomcp_cli::error::BioMcpError>() {
                    eprintln!("Error: {bio_err}");
                } else {
                    eprintln!("Error: {err}");
                }
                std::process::ExitCode::from(1)
            }
        },
    }
}
