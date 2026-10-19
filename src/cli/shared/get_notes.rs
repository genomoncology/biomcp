use anyhow::Context;

use crate::cli::CommandOutcome;

pub(in crate::cli) fn append_get_json_stale_notes(
    outcome: &mut CommandOutcome,
) -> anyhow::Result<()> {
    let sentences = crate::sources::take_stale_serve_sentences();
    if sentences.is_empty() {
        return Ok(());
    }
    let mut value: serde_json::Value = serde_json::from_str(&outcome.text)?;
    let meta = value
        .as_object_mut()
        .context("get JSON response must be an object")?
        .entry("_meta")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .context("get JSON metadata must be an object")?;
    let notes = meta
        .entry("notes")
        .or_insert_with(|| serde_json::json!([]))
        .as_array_mut()
        .context("get JSON notes must be an array")?;
    for sentence in sentences {
        let note = serde_json::Value::String(sentence);
        if !notes.contains(&note) {
            notes.push(note);
        }
    }
    outcome.text = crate::render::json::to_pretty(&value)?;
    if outcome.metadata_json.is_some() {
        outcome.metadata_json = Some(outcome.text.clone());
    }
    Ok(())
}
