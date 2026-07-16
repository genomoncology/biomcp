use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SectionOutcomeState {
    #[default]
    NotRequested,
    Data,
    Empty,
    Degraded,
    Unavailable,
}

impl SectionOutcomeState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotRequested => "not_requested",
            Self::Data => "data",
            Self::Empty => "empty",
            Self::Degraded => "degraded",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
            sources: sources.into_iter().map(Into::into).collect(),
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
            sources: sources.into_iter().map(Into::into).collect(),
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

fn bounded_message(message: &'static str) -> String {
    message
        .chars()
        .filter(|ch| !ch.is_control())
        .take(160)
        .collect()
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
        debug_assert_eq!(current.outcome, SectionOutcomeState::NotRequested);
        *current = outcome;
    }

    pub fn get(&self, key: &str) -> Option<&SectionOutcome> {
        self.0.get(key)
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
        let mut outcomes = SectionOutcomes::with_keys(&["empty", "unavailable"]);
        outcomes.complete("empty", SectionOutcome::empty("Source"));
        outcomes.complete(
            "unavailable",
            SectionOutcome::unavailable("Source data is unavailable.\n"),
        );
        let value = serde_json::to_value(outcomes).unwrap();
        assert_eq!(value["empty"]["outcome"], "empty");
        assert_eq!(value["empty"]["sources"][0], "Source");
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
    #[should_panic]
    fn registry_rejects_second_completion_in_tests() {
        let mut outcomes = SectionOutcomes::with_keys(&["section"]);
        outcomes.complete("section", SectionOutcome::empty("Source"));
        outcomes.complete("section", SectionOutcome::data("Source"));
    }
}
