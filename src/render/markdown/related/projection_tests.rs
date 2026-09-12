use super::*;

#[test]
fn related_trial_uses_shared_projection_fields() {
    let trial = TrialResponse::test_ctgov(
        "NCT01234567",
        "Example completed trial",
        "COMPLETED",
        "melanoma",
        Some("dabrafenib"),
    );

    let commands = related_trial(&trial);

    assert_eq!(
        commands[0],
        "biomcp search article --drug dabrafenib -q \"NCT01234567 Example completed trial\" --limit 5"
    );
    assert_eq!(commands[1], "biomcp search disease --query melanoma");
    assert!(commands.contains(&"biomcp drug trials dabrafenib".to_string()));
}

fn trial(status: &str, title: &str, interventions: serde_json::Value) -> TrialResponse {
    let input = serde_json::json!({
        "protocolSection": {
            "identificationModule": {"nctId": "NCT01234567", "briefTitle": title},
            "statusModule": {"overallStatus": status},
            "sponsorCollaboratorsModule": {"leadSponsor": {"name": "Sponsor"}},
            "conditionsModule": {"conditions": ["melanoma"]},
            "designModule": {"studyType": "INTERVENTIONAL"},
            "armsInterventionsModule": {"interventions": interventions}
        }
    })
    .to_string();
    TrialResponse::test_ctgov_json("NCT01234567", &input)
}

#[test]
fn related_trial_prefers_the_first_available_alias() {
    let trial = trial(
        "RECRUITING",
        "Trial",
        serde_json::json!([
            {"name":"first", "otherNames":[]},
            {"name":"second", "otherNames":["second alias", "later"]},
            {"name":"third", "otherNames":["third alias"]}
        ]),
    );
    let commands = related_trial(&trial);
    assert!(commands.contains(&"biomcp search drug -q \"second alias\"".to_string()));
    assert!(!commands.iter().any(|value| value.contains("third alias")));
    assert!(
        !commands
            .iter()
            .any(|value| value.starts_with("biomcp drug trials"))
    );
}

#[test]
fn related_trial_without_aliases_uses_first_intervention() {
    let trial = trial(
        "RECRUITING",
        "Trial",
        serde_json::json!([{"name":"first drug"}, {"name":"second drug"}]),
    );
    let commands = related_trial(&trial);
    assert!(commands.contains(&"biomcp search drug -q \"first drug\"".to_string()));
    assert!(commands.contains(&"biomcp drug trials \"first drug\"".to_string()));
}

#[test]
fn related_trial_results_search_is_ordered_and_shell_safe() {
    let completed = trial(
        "TERMINATED",
        "Alpha\\path's $(touch /tmp/biomcp-trial-title-expanded) \"quoted\" $HOME; `uname` tail",
        serde_json::json!([{"name":"SAFE-357"}]),
    );
    let completed_commands = related_trial(&completed);
    assert!(completed_commands[0].starts_with("biomcp search article --drug SAFE-357"));
    assert_eq!(
        completed_commands[0],
        "biomcp search article --drug SAFE-357 -q \"NCT01234567 Alpha\\\\path's \\$(touch /tmp/biomcp-trial-title-expanded) \\\"quoted\\\" \\$HOME; \\`uname\\`\" --limit 5"
    );
    let argv = shlex::split(&completed_commands[0]).expect("valid shell syntax");
    assert_eq!(
        argv[6],
        "NCT01234567 Alpha\\path's $(touch /tmp/biomcp-trial-title-expanded) \"quoted\" $HOME; `uname`"
    );
    assert_eq!(
        completed_commands[1],
        "biomcp search disease --query melanoma"
    );

    let recruiting = trial(
        "RECRUITING",
        "Recruiting",
        serde_json::json!([{"name":"dabrafenib"}]),
    );
    let recruiting_commands = related_trial(&recruiting);
    assert_eq!(
        recruiting_commands[0],
        "biomcp search disease --query melanoma"
    );
    assert!(
        !recruiting_commands
            .iter()
            .any(|value| value.contains("search article --drug"))
    );
}

#[test]
fn related_trial_without_intervention_keeps_completed_seed_quoted() {
    let trial = trial("COMPLETED", "Title with spaces", serde_json::json!([]));
    let commands = related_trial(&trial);
    assert_eq!(
        commands[0],
        "biomcp search article -q \"NCT01234567 Title with spaces\" --limit 5"
    );
}
