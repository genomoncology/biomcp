//! BioMCP skill CLI facade and stable public command surface.

use clap::Subcommand;

mod assets;
mod atomic_swap;
mod catalog;
mod install;
mod status;

pub(crate) use catalog::list_use_case_refs;
pub use catalog::{list_use_cases, render_system_prompt, show_overview, show_use_case};
pub use install::{SkillInstallResult, install_skills};
pub use status::skill_status;

pub(crate) fn render_install_result(
    result: &SkillInstallResult,
    json: bool,
) -> anyhow::Result<String> {
    if !json {
        return Ok(result.human_text());
    }
    #[derive(serde::Serialize)]
    struct Response<'a> {
        kind: &'static str,
        action: &'static str,
        #[serde(flatten)]
        result: &'a SkillInstallResult,
    }
    Ok(crate::render::json::to_pretty(&Response {
        kind: "skill",
        action: "install",
        result,
    })?)
}

#[derive(Subcommand, Debug)]
pub enum SkillCommand {
    /// List embedded worked examples
    List,
    /// Render the canonical agent-facing prompt
    Render,
    /// Report whether installed guidance matches this BioMCP binary
    Status {
        /// Agent root, skills directory, or skills/biomcp directory
        dir: Option<String>,
    },
    /// Install BioMCP skill guidance to an agent directory
    Install {
        /// Agent root or skills directory (e.g. ~/.claude, ~/.claude/skills, ~/.claude/skills/biomcp)
        dir: Option<String>,
        /// Replace existing installation
        #[arg(long)]
        force: bool,
    },
    /// Show a specific use-case by number or name
    #[command(external_subcommand)]
    Show(Vec<String>),
}

#[cfg(test)]
mod tests {
    mod catalog;
    mod install;
}
