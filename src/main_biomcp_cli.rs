use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn sibling_biomcp() -> Result<PathBuf, String> {
    let current =
        env::current_exe().map_err(|error| format!("cannot locate biomcp-cli: {error}"))?;
    let directory = current
        .parent()
        .ok_or_else(|| "cannot locate the biomcp-cli installation directory".to_string())?;
    #[cfg(windows)]
    let name = "biomcp.exe";
    #[cfg(not(windows))]
    let name = "biomcp";
    let sibling = directory.join(name);
    let metadata = sibling
        .symlink_metadata()
        .map_err(|error| format!("cannot run sibling {}: {error}", sibling.display()))?;
    if !metadata.file_type().is_file() {
        return Err(format!(
            "sibling is not a regular file: {}",
            sibling.display()
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(format!("sibling is not executable: {}", sibling.display()));
        }
    }
    Ok(sibling)
}

fn main() {
    let sibling = sibling_biomcp().unwrap_or_else(|message| {
        eprintln!("biomcp-cli: {message}");
        std::process::exit(126);
    });
    let mut command = Command::new(sibling);
    command
        .args(env::args_os().skip(1))
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let error = command.exec();
        eprintln!("biomcp-cli: failed to execute sibling biomcp: {error}");
        std::process::exit(126);
    }
    #[cfg(windows)]
    match command.status() {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(error) => {
            eprintln!("biomcp-cli: failed to execute sibling biomcp: {error}");
            std::process::exit(126);
        }
    }
}
