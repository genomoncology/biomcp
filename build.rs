use std::fs;
use std::path::PathBuf;

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
        "\n\nTYPED MCP TOOLS:\n  Prefer typed `search` and `get` for structured entity lookup; their schemas enumerate valid entities, get sections, and a bounded search limit.\n  Use raw `biomcp` as an escape hatch for long-tail read-only commands not yet covered by typed tools.\n\nSEARCH FILTERS:\n  Use `biomcp list <entity>` for entity-specific filters and examples.\n  Trial geo filters include --lat, --lon, and --distance.\n\nMCP RESPONSE METADATA:\n  Default text responses append compact `Sources` and `Next commands` sections when upstream provenance is available.\n  Pass tool input `json: true` to return the CLI JSON contract with full `_meta.section_sources`, `_meta.evidence_urls`, `_meta.next_commands`, and `_meta.ladder`.\n\nAGENT GUIDANCE:\n  Use biomedical synonyms and abbreviations (for example NSCLC -> non-small cell lung cancer).\n  If zero results are returned, retry with nearby terms, aliases, or alternate spellings.\n",
    );

    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    let output = out_dir.join("mcp_shell_description.txt");
    if fs::read(&output).ok().as_deref() != Some(description.as_bytes()) {
        fs::write(output, description)?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "alphagenome")]
    {
        println!("cargo:rerun-if-changed=protos/dna_model_service.proto");
        println!("cargo:rerun-if-changed=protos/dna_model.proto");
        println!("cargo:rerun-if-changed=protos/tensor.proto");
    }
    // `src/cli/list.rs` was decomposed into the `src/cli/list/` module dir; the
    // old path no longer exists. cargo treats a missing rerun-if-changed file as
    // permanently stale, so it re-ran this build script — and recompiled the whole
    // crate — on EVERY build. Watch the current directory instead.
    println!("cargo:rerun-if-changed=src/cli/list");
    println!("cargo:rerun-if-changed=src/cli/list_reference.md");
    write_shell_description()?;
    #[cfg(feature = "alphagenome")]
    {
        let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
        let proto_out = out_dir.join("google.gdm.gdmscience.alphagenome.v1main.rs");
        let vendored = PathBuf::from("src/generated/google.gdm.gdmscience.alphagenome.v1main.rs");

        let compiled = tonic_build::configure()
            .build_client(true)
            .build_server(false)
            .compile_protos(&["protos/dna_model_service.proto"], &["protos"]);

        match compiled {
            Ok(()) => {
                // Refresh the vendored fallback only when the generated output actually
                // changed. Copying on every build rewrites this tracked source-tree file,
                // which bumps its mtime and makes cargo treat the package as dirty —
                // forcing a full recompile on every "warm" build (cargo build, make spec,
                // make test, focused). Compare bytes and write only on a real change so
                // the package stays clean and the build cache works.
                if proto_out.exists() {
                    let generated = fs::read_to_string(&proto_out)?.replace(
                    "pub mod dna_model_service_client {\n    #![allow(",
                    "pub mod dna_model_service_client {\n    // dead-code reason: generated provider client includes members unused by this runtime\n    #![allow(",
                );
                    let new_bytes = generated.as_bytes();
                    let current = fs::read(&vendored).ok();
                    if current.as_deref() != Some(new_bytes) {
                        fs::write(&vendored, new_bytes)?;
                    }
                    fs::write(&proto_out, new_bytes)?;
                }
            }
            Err(e) => {
                if vendored.exists() {
                    eprintln!(
                        "cargo:warning=protoc unavailable ({e}), using vendored protobuf output"
                    );
                    fs::copy(&vendored, &proto_out)?;
                } else {
                    return Err(e.into());
                }
            }
        }
    }

    Ok(())
}
