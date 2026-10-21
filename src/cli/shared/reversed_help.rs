use std::ffi::OsStr;

use clap::error::ErrorKind;

use super::{HIDDEN_GLOBAL_FLAGS, RECOVERY_LIMITS, build_cli};

/// Correct help requests only. A registered command in the supplied order wins.
pub(super) fn correction<T: AsRef<OsStr>>(args: &[T]) -> Option<String> {
    if args.len() > RECOVERY_LIMITS.0 {
        return None;
    }
    let words = args
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(_, arg)| !HIDDEN_GLOBAL_FLAGS.iter().any(|flag| arg.as_ref() == *flag))
        .collect::<Vec<_>>();
    let [(entity_index, entity), (verb_index, verb), (_, help)] = words.as_slice() else {
        return None;
    };
    if !matches!(help.as_ref().to_str(), Some("--help" | "-h")) {
        return None;
    }
    let entity = entity.as_ref().to_str()?;
    let verb = verb.as_ref().to_str()?;
    if !matches!(verb, "search" | "get") {
        return None;
    }
    let registry = build_cli();
    let family = registry.find_subcommand(verb)?;
    family.find_subcommand(entity)?;
    if registry
        .find_subcommand(entity)
        .and_then(|command| command.find_subcommand(verb))
        .is_some()
    {
        return None;
    }
    let mut candidate = args
        .iter()
        .map(|arg| arg.as_ref().to_os_string())
        .collect::<Vec<_>>();
    candidate.swap(*entity_index, *verb_index);
    let error = registry.try_get_matches_from(candidate.clone()).err()?;
    if error.kind() != ErrorKind::DisplayHelp {
        return None;
    }
    let rendered = shlex::try_join(
        std::iter::once("biomcp").chain(
            candidate
                .iter()
                .skip(1)
                .map(|arg| arg.to_str())
                .collect::<Option<Vec<_>>>()?,
        ),
    )
    .ok()?;
    Some(format!(
        "reversed {verb} syntax; use {}",
        crate::render::markdown::markdown_command_code_span(&rendered)
    ))
}
