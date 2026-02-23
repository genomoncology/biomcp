use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn command_output(command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn write_shell_description() -> Result<(), Box<dyn std::error::Error>> {
    let list_reference = fs::read_to_string("src/cli/list_reference.md")?;
    let mut description = String::new();
    description.push_str("BioMCP: Search and retrieve genes, variants, clinical trials, articles, drugs, diseases, pathways, proteins, adverse events, and pharmacogenomic data from 15 biomedical sources.\n\n");
    description.push_str(list_reference.trim());
    description.push_str(
        "\n\nSEARCH FILTERS:\n  Use `biomcp list <entity>` for entity-specific filters and examples.\n  Trial geo filters include --lat, --lon, and --distance.\n\nAGENT GUIDANCE:\n  Use biomedical synonyms and abbreviations (for example NSCLC -> non-small cell lung cancer).\n  If zero results are returned, retry with nearby terms, aliases, or alternate spellings.\n",
    );

    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    fs::write(out_dir.join("mcp_shell_description.txt"), description)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=protos/dna_model_service.proto");
    println!("cargo:rerun-if-changed=protos/dna_model.proto");
    println!("cargo:rerun-if-changed=protos/tensor.proto");
    println!("cargo:rerun-if-changed=src/cli/list.rs");
    println!("cargo:rerun-if-changed=src/cli/list_reference.md");

    write_shell_description()?;

    let git_sha = command_output("git", &["rev-parse", "--short", "HEAD"])
        .unwrap_or_else(|| "unknown".into());
    let build_date =
        command_output("date", &["-u", "+%Y-%m-%dT%H:%M:%SZ"]).unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=BIOMCP_BUILD_GIT_SHA={git_sha}");
    println!("cargo:rustc-env=BIOMCP_BUILD_DATE={build_date}");

    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile_protos(&["protos/dna_model_service.proto"], &["protos"])?;

    Ok(())
}
