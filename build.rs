use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

mod build_git_watch;

use build_git_watch::git_ref_watch_paths;

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

fn emit_rerun_if_changed_if_exists(path: &Path) {
    if path.exists() {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}

fn emit_git_ref_rerun_paths() {
    let Some(git_dir) =
        command_output("git", &["rev-parse", "--path-format=absolute", "--git-dir"])
    else {
        return;
    };
    let Some(git_common_dir) = command_output(
        "git",
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    ) else {
        return;
    };

    let paths = git_ref_watch_paths(
        &git_dir,
        &git_common_dir,
        &fs::read_to_string(Path::new(&git_dir).join("HEAD")).unwrap_or_default(),
    );

    emit_rerun_if_changed_if_exists(&paths.head);
    if let Some(current_ref) = &paths.current_ref {
        emit_rerun_if_changed_if_exists(current_ref);
    }
    emit_rerun_if_changed_if_exists(&paths.packed_refs);
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
        "\n\nTYPED MCP TOOLS:\n  Prefer typed `search` and `get` for structured entity lookup; their schemas enumerate valid entities, get sections, and a bounded search limit.\n  Use raw `biomcp` as an escape hatch for long-tail read-only commands not yet covered by typed tools.\n\nSEARCH FILTERS:\n  Use `biomcp list <entity>` for entity-specific filters and examples.\n  Trial geo filters include --lat, --lon, and --distance.\n\nMCP RESPONSE METADATA:\n  Default text responses append compact `Sources` and `Next commands` sections when upstream provenance is available.\n  Pass tool input `json: true` to return the CLI JSON contract with full `_meta.section_sources`, `_meta.evidence_urls`, `_meta.next_commands`, and `_meta.ladder`.\n\nAGENT GUIDANCE:\n  Use biomedical synonyms and abbreviations (for example NSCLC -> non-small cell lung cancer).\n  If zero results are returned, retry with nearby terms, aliases, or alternate spellings.\n",
    );

    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    fs::write(out_dir.join("mcp_shell_description.txt"), description)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=protos/dna_model_service.proto");
    println!("cargo:rerun-if-changed=protos/dna_model.proto");
    println!("cargo:rerun-if-changed=protos/tensor.proto");
    // `src/cli/list.rs` was decomposed into the `src/cli/list/` module dir; the
    // old path no longer exists. cargo treats a missing rerun-if-changed file as
    // permanently stale, so it re-ran this build script — and recompiled the whole
    // crate — on EVERY build. Watch the current directory instead.
    println!("cargo:rerun-if-changed=src/cli/list");
    println!("cargo:rerun-if-changed=src/cli/list_reference.md");
    // Stamp git identity below, so watch the real git metadata files that move
    // when HEAD moves. Worktrees store `.git` as a pointer file, so resolve the
    // per-worktree git dir for HEAD and the common git dir for refs/packed-refs.
    // Missing git metadata is ignored so non-git package builds keep the existing
    // `unknown` fallback instead of failing or watching permanently-missing paths.
    emit_git_ref_rerun_paths();

    write_shell_description()?;

    let git_sha = command_output("git", &["rev-parse", "--short=8", "HEAD"])
        .unwrap_or_else(|| "unknown".into());
    let git_tag = command_output("git", &["describe", "--tags", "--always"]);
    // Stamp the HEAD commit date (deterministic), not the wall-clock build time.
    // A wall-clock timestamp is a fresh value on every build-script run, i.e. a
    // changed compile input that forces a full crate recompile on every build —
    // the cache can never go warm. The commit date is stable for a given commit
    // and is the more reproducible thing to record anyway.
    let build_date =
        command_output("git", &["log", "-1", "--format=%cI"]).unwrap_or_else(|| "unknown".into());
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
            // Refresh the vendored fallback only when the generated output actually
            // changed. Copying on every build rewrites this tracked source-tree file,
            // which bumps its mtime and makes cargo treat the package as dirty —
            // forcing a full recompile on every "warm" build (cargo build, make spec,
            // make test, focused). Compare bytes and write only on a real change so
            // the package stays clean and the build cache works.
            if proto_out.exists() {
                let new_bytes = fs::read(&proto_out).ok();
                let current = fs::read(&vendored).ok();
                if let Some(new_bytes) = new_bytes
                    && current.as_deref() != Some(new_bytes.as_slice())
                {
                    fs::write(&vendored, &new_bytes)?;
                }
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
