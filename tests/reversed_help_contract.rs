use std::process::Command;

#[test]
fn reversed_entity_help_names_the_exact_registered_command() {
    let registry = biomcp_cli::cli::build_cli();
    for verb in ["search", "get"] {
        let family = registry
            .find_subcommand(verb)
            .expect("entity command family");
        for entity in family.get_subcommands() {
            let entity = entity.get_name();
            if registry
                .find_subcommand(entity)
                .and_then(|command| command.find_subcommand(verb))
                .is_some()
            {
                continue; // A registered nested command already owns this help.
            }
            for help in ["--help", "-h"] {
                let canonical = Command::new(env!("CARGO_BIN_EXE_biomcp"))
                    .args([verb, entity, help])
                    .output()
                    .expect("canonical help");
                assert!(
                    canonical.status.success(),
                    "{verb} {entity} {help}: {canonical:?}"
                );
                let reversed = Command::new(env!("CARGO_BIN_EXE_biomcp"))
                    .args([entity, verb, help])
                    .output()
                    .expect("reversed help");
                let diagnostic = String::from_utf8_lossy(&reversed.stderr);
                let correct_help = reversed.status.success() && reversed.stdout == canonical.stdout;
                let exact_correction = reversed.status.code() == Some(2)
                    && diagnostic.contains(&format!("biomcp {verb} {entity} {help}"));
                assert!(
                    correct_help || exact_correction,
                    "{entity} {verb} {help} must show matching help or its exact command: {reversed:?}"
                );
            }
        }
    }
}

#[test]
fn reversed_help_does_not_reinterpret_genuine_argument_errors() {
    for args in [
        ["get", "not-an-entity", "--help"].as_slice(),
        ["article", "not-a-command", "--help"].as_slice(),
        ["get", "article", "--not-an-option"].as_slice(),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_biomcp"))
            .args(args)
            .output()
            .expect("invalid command");
        assert_eq!(output.status.code(), Some(2), "{args:?}: {output:?}");
        assert!(!String::from_utf8_lossy(&output.stderr).contains("reversed"));
    }
}

#[test]
fn reversed_get_help_preserves_json_and_registered_aliases() {
    let output = Command::new(env!("CARGO_BIN_EXE_biomcp"))
        .args(["--json", "article", "get", "--help"])
        .output()
        .expect("JSON correction");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON error");
    assert_eq!(value["error"]["code"], "invalid_argument");
    assert!(
        value["error"]["message"]
            .as_str()
            .unwrap()
            .contains("biomcp --json get article --help")
    );

    let alias = Command::new(env!("CARGO_BIN_EXE_biomcp"))
        .args(["gene", "get", "--help"])
        .output()
        .expect("registered alias help");
    assert!(alias.status.success());
    assert!(String::from_utf8_lossy(&alias.stdout).contains("biomcp gene definition"));
}
