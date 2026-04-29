//! Variant guidance response shaping.

use crate::cli::CommandOutcome;

fn variant_guidance_markdown(guidance: &crate::entities::variant::VariantGuidance) -> String {
    let err = crate::error::BioMcpError::NotFound {
        entity: "variant".into(),
        id: guidance.query.clone(),
        suggestion: crate::render::markdown::variant_guidance_suggestion(guidance),
    };
    format!("Error: {err}")
}

pub(super) fn variant_guidance_outcome(
    guidance: &crate::entities::variant::VariantGuidance,
    json_output: bool,
) -> anyhow::Result<CommandOutcome> {
    if json_output {
        return Ok(CommandOutcome::stdout_with_exit(
            crate::render::json::to_variant_guidance_json(guidance)?,
            1,
        ));
    }
    Ok(CommandOutcome::stderr_with_exit(
        variant_guidance_markdown(guidance),
        1,
    ))
}
