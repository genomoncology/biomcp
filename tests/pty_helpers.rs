// dead-code reason: Cargo also compiles this shared integration helper as a standalone target
#![allow(dead_code)]

use std::path::Path;
use std::process::Command;

use rexpect::session::spawn_command;

pub(crate) fn run_command_with_tty(command: Command) -> Result<String, Box<dyn std::error::Error>> {
    let mut session = spawn_command(command, Some(10_000))?;
    Ok(session.exp_eof()?)
}

pub(crate) fn run_biomcp_with_tty(
    args: &[&str],
    cache_home: &Path,
    config_home: &Path,
    answer: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_biomcp"));
    command.args(args);
    command.env("XDG_CACHE_HOME", cache_home);
    command.env("XDG_CONFIG_HOME", config_home);
    command.env_remove("BIOMCP_CACHE_DIR");
    command.env_remove("BIOMCP_CACHE_MAX_SIZE");
    command.env_remove("BIOMCP_CACHE_MAX_AGE");
    command.env_remove("RUST_LOG");

    let mut session = spawn_command(command, Some(10_000))?;
    session.exp_string("Continue? [y/N]: ")?;
    session.send_line(answer)?;
    Ok(session.exp_eof()?)
}
