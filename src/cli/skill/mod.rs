//! BioMCP skill CLI facade and stable public command surface.

use clap::Subcommand;

mod assets;
mod catalog;
mod install;

pub(crate) use catalog::list_use_case_refs;
pub use catalog::{list_use_cases, render_system_prompt, show_overview, show_use_case};
pub use install::install_skills;

#[derive(Subcommand, Debug)]
pub enum SkillCommand {
    /// List embedded worked examples
    List,
    /// Render the canonical agent-facing prompt
    Render,
    /// Show a specific use-case by number or name
    #[command(external_subcommand)]
    Show(Vec<String>),
    /// Install BioMCP skill guidance to an agent directory
    Install {
        /// Agent root or skills directory (e.g. ~/.claude, ~/.claude/skills, ~/.claude/skills/biomcp)
        dir: Option<String>,
        /// Replace existing installation
        #[arg(long)]
        force: bool,
    },
}

#[cfg(test)]
mod tests {
    mod catalog;
    mod install;
}
