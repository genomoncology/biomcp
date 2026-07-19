use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SectionOutcomeState {
    #[default]
    NotRequested,
    Inapplicable,
    Data,
    Empty,
    Degraded,
    Unavailable,
}

impl SectionOutcomeState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotRequested => "not_requested",
            Self::Inapplicable => "inapplicable",
            Self::Data => "data",
            Self::Empty => "empty",
            Self::Degraded => "degraded",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SectionOutcome {
    outcome: SectionOutcomeState,
    sources: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

impl SectionOutcome {
    pub fn data(source: impl Into<String>) -> Self {
        Self::data_sources([source])
    }

    pub fn data_sources<I, S>(sources: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::successful(SectionOutcomeState::Data, sources)
    }

    pub fn empty(source: impl Into<String>) -> Self {
        Self::empty_sources([source])
    }

    pub fn empty_sources<I, S>(sources: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::successful(SectionOutcomeState::Empty, sources)
    }

    pub fn degraded<I, S>(sources: I, message: &'static str) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            outcome: SectionOutcomeState::Degraded,
            sources: successful_sources(sources),
            message: Some(bounded_message(message)),
        }
    }

    pub fn inapplicable(message: &'static str) -> Self {
        Self {
            outcome: SectionOutcomeState::Inapplicable,
            sources: Vec::new(),
            message: Some(bounded_message(message)),
        }
    }

    pub fn unavailable(message: &'static str) -> Self {
        Self {
            outcome: SectionOutcomeState::Unavailable,
            sources: Vec::new(),
            message: Some(bounded_message(message)),
        }
    }

    fn successful<I, S>(outcome: SectionOutcomeState, sources: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            outcome,
            sources: successful_sources(sources),
            message: None,
        }
    }

    pub fn outcome(&self) -> SectionOutcomeState {
        self.outcome
    }

    pub fn sources(&self) -> &[String] {
        &self.sources
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

fn successful_sources<I, S>(sources: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let sources = sources.into_iter().map(Into::into).collect::<Vec<_>>();
    assert!(
        !sources.is_empty() && sources.iter().all(|source| !source.trim().is_empty()),
        "successful section outcomes require non-blank sources"
    );
    sources
}

fn message_is_safe(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    message.chars().count() <= 160
        && !message.chars().any(char::is_control)
        && !lower.contains("://")
        && !lower.contains("credential")
        && !lower.contains("password")
        && !lower.contains("token=")
        && !lower.contains("parser error")
        && !lower.contains("transport error")
        && !message
            .split_whitespace()
            .any(|word| word.starts_with('/') || word.contains(":\\"))
}

fn bounded_message(message: &'static str) -> String {
    let message = message
        .chars()
        .filter(|ch| !ch.is_control())
        .take(160)
        .collect::<String>();
    assert!(
        message_is_safe(&message),
        "unsafe public section outcome message"
    );
    message
}

#[derive(Deserialize)]
struct SerializedSectionOutcome {
    outcome: SectionOutcomeState,
    #[serde(default)]
    sources: Vec<String>,
    #[serde(default)]
    message: Option<String>,
}

impl<'de> Deserialize<'de> for SectionOutcome {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;

        let value = SerializedSectionOutcome::deserialize(deserializer)?;
        let sources_are_valid = value.sources.iter().all(|source| !source.trim().is_empty());
        let message_is_valid = value.message.as_deref().is_none_or(message_is_safe);
        let shape_is_valid = match value.outcome {
            SectionOutcomeState::NotRequested => {
                value.sources.is_empty() && value.message.is_none()
            }
            SectionOutcomeState::Data | SectionOutcomeState::Empty => {
                !value.sources.is_empty() && value.message.is_none()
            }
            SectionOutcomeState::Degraded => !value.sources.is_empty() && value.message.is_some(),
            SectionOutcomeState::Inapplicable | SectionOutcomeState::Unavailable => {
                value.sources.is_empty() && value.message.is_some()
            }
        };
        if !sources_are_valid || !message_is_valid || !shape_is_valid {
            return Err(D::Error::custom(
                "invalid section outcome state/source/message combination",
            ));
        }
        Ok(Self {
            outcome: value.outcome,
            sources: value.sources,
            message: value.message,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct SectionOutcomes(BTreeMap<String, SectionOutcome>);

impl SectionOutcomes {
    pub fn with_keys(keys: &[&str]) -> Self {
        Self(
            keys.iter()
                .map(|key| {
                    (
                        (*key).to_string(),
                        SectionOutcome {
                            outcome: SectionOutcomeState::NotRequested,
                            sources: Vec::new(),
                            message: None,
                        },
                    )
                })
                .collect(),
        )
    }

    pub fn complete(&mut self, key: &str, outcome: SectionOutcome) {
        let current = self
            .0
            .get_mut(key)
            .unwrap_or_else(|| panic!("unknown section outcome key: {key}"));
        assert_eq!(
            current.outcome,
            SectionOutcomeState::NotRequested,
            "section outcome completed more than once: {key}"
        );
        *current = outcome;
    }

    pub fn get(&self, key: &str) -> Option<&SectionOutcome> {
        self.0.get(key)
    }

    pub fn validate_keys(&self, allowed: &[&str]) -> Result<(), String> {
        if let Some(key) = self.0.keys().find(|key| !allowed.contains(&key.as_str())) {
            return Err(format!("unknown section outcome key: {key}"));
        }
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &SectionOutcome)> {
        self.0.iter().map(|(key, outcome)| (key.as_str(), outcome))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_serializes_all_states_and_safe_messages() {
        let mut outcomes = SectionOutcomes::with_keys(&["empty", "inapplicable", "unavailable"]);
        outcomes.complete("empty", SectionOutcome::empty("Source"));
        outcomes.complete(
            "inapplicable",
            SectionOutcome::inapplicable("A required input is missing."),
        );
        outcomes.complete(
            "unavailable",
            SectionOutcome::unavailable("Source data is unavailable.\n"),
        );
        let value = serde_json::to_value(outcomes).unwrap();
        assert_eq!(value["empty"]["outcome"], "empty");
        assert_eq!(value["empty"]["sources"][0], "Source");
        assert_eq!(value["inapplicable"]["outcome"], "inapplicable");
        assert_eq!(value["inapplicable"]["sources"], serde_json::json!([]));
        let round_trip = serde_json::from_value::<SectionOutcomes>(value.clone()).unwrap();
        assert_eq!(
            round_trip.get("inapplicable").unwrap().message(),
            Some("A required input is missing.")
        );
        assert_eq!(value["unavailable"]["outcome"], "unavailable");
        assert_eq!(
            value["unavailable"]["message"],
            "Source data is unavailable."
        );
    }

    #[test]
    #[should_panic(expected = "unknown section outcome key")]
    fn registry_rejects_unknown_keys() {
        SectionOutcomes::with_keys(&[]).complete("unknown", SectionOutcome::empty("Source"));
    }

    #[test]
    #[should_panic(expected = "section outcome completed more than once")]
    fn registry_rejects_second_completion() {
        let mut outcomes = SectionOutcomes::with_keys(&["section"]);
        outcomes.complete("section", SectionOutcome::empty("Source"));
        outcomes.complete("section", SectionOutcome::data("Source"));
    }

    #[test]
    #[should_panic(expected = "non-blank sources")]
    fn successful_outcomes_require_a_source() {
        SectionOutcome::data_sources(Vec::<String>::new());
    }

    #[test]
    fn deserialized_registry_rejects_entity_foreign_keys() {
        let outcomes = serde_json::from_str::<SectionOutcomes>(
            r#"{"foreign":{"outcome":"empty","sources":["Source"]}}"#,
        )
        .expect("outcome shape is valid");

        assert_eq!(
            outcomes.validate_keys(&["expected"]).unwrap_err(),
            "unknown section outcome key: foreign"
        );
    }

    #[test]
    fn deserialization_rejects_illegal_shapes_and_unsafe_messages() {
        for json in [
            r#"{"outcome":"data","sources":[]}"#,
            r#"{"outcome":"inapplicable","sources":[]}"#,
            r#"{"outcome":"inapplicable","sources":["Provider"],"message":"Not applicable."}"#,
            r#"{"outcome":"unavailable","sources":["Provider"],"message":"Unavailable."}"#,
            r#"{"outcome":"degraded","sources":[],"message":"Incomplete."}"#,
            r#"{"outcome":"unavailable","sources":[],"message":"See https://example.test/raw"}"#,
            r#"{"outcome":"unavailable","sources":[],"message":"read /tmp/provider.json"}"#,
        ] {
            assert!(
                serde_json::from_str::<SectionOutcome>(json).is_err(),
                "{json}"
            );
        }
    }
}
