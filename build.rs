use std::fs;
use std::path::PathBuf;
use std::process::Command;

const MCP_SHELL_INTRO: &str = "BioMCP is a read-only biomedical MCP tool for \
search, detail retrieval, discovery, enrichment, and study analytics across \
leading public biomedical data sources.\n\n";
const BLOCKED_MCP_DESCRIPTION_TERMS: &[&str] = &[
    "`skill install`",
    "`ema sync`",
    "`who sync`",
    "`cvx sync`",
    "`gtr sync`",
    "`who-ivd sync`",
    "`update [--check]`",
    "`uninstall`",
];
const STUDY_PATTERN_LINE: &str = "- `study list|download|query|co-occurrence|cohort|survival|compare` - local cBioPortal study analytics";
const MCP_SAFE_STUDY_PATTERN_LINE: &str = "- `study list|download --list|query|filter|co-occurrence|cohort|survival|compare` - local cBioPortal study analytics";
const STUDY_DOWNLOAD_LINE: &str = "- `study download [--list] [<study_id>]`";
const MCP_SAFE_STUDY_DOWNLOAD_LINE: &str = "- `study download --list`";

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

fn is_blocked_mcp_description_line(line: &str) -> bool {
    // Cache-family commands stay CLI-only because they reveal workstation-local paths.
    line.trim_start().starts_with("- `cache ")
        || line.trim_start().starts_with("- `update ")
        || BLOCKED_MCP_DESCRIPTION_TERMS
            .iter()
            .any(|term| line.contains(term))
}

fn mcp_safe_description_line(line: &str) -> Option<String> {
    if is_blocked_mcp_description_line(line) {
        return None;
    }

    let rewritten = match line {
        STUDY_PATTERN_LINE => MCP_SAFE_STUDY_PATTERN_LINE,
        STUDY_DOWNLOAD_LINE => MCP_SAFE_STUDY_DOWNLOAD_LINE,
        _ => line,
    };
    Some(rewritten.to_string())
}

fn mcp_safe_list_reference(list_reference: &str) -> String {
    list_reference
        .lines()
        .filter_map(mcp_safe_description_line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn write_shell_description() -> Result<(), Box<dyn std::error::Error>> {
    let list_reference = mcp_safe_list_reference(&fs::read_to_string("src/cli/list_reference.md")?);
    let mut description = String::new();
    description.push_str(MCP_SHELL_INTRO);
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
    let git_tag = command_output("git", &["describe", "--tags", "--always"]);
    let build_date =
        command_output("date", &["-u", "+%Y-%m-%dT%H:%M:%SZ"]).unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=BIOMCP_BUILD_GIT_SHA={git_sha}");
    if let Some(tag) = &git_tag {
        println!("cargo:rustc-env=BIOMCP_BUILD_GIT_TAG={tag}");
    }
    println!("cargo:rustc-env=BIOMCP_BUILD_DATE={build_date}");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    let proto_out = out_dir.join("google.gdm.gdmscience.alphagenome.v1main.rs");
    let vendored = PathBuf::from("src/generated/google.gdm.gdmscience.alphagenome.v1main.rs");

    let compiled = tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile_protos(&["protos/dna_model_service.proto"], &["protos"]);

    match compiled {
        Ok(()) => {
            // Update vendored copy so it stays current.
            if proto_out.exists() {
                let _ = fs::copy(&proto_out, &vendored);
            }
        }
        Err(e) => {
            if vendored.exists() {
                eprintln!("cargo:warning=protoc unavailable ({e}), using vendored protobuf output");
                fs::copy(&vendored, &proto_out)?;
            } else {
                return Err(e.into());
            }
        }
    }

    Ok(())
}
